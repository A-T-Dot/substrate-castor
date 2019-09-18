
//! # Non-transfer Asset Module
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};

use sr_primitives::traits::{
	CheckedAdd, CheckedSub, MaybeSerializeDebug, Member, One, Saturating, SimpleArithmetic, Zero, Bounded
};

use rstd::prelude::*;
use rstd::{cmp, result};
use support::dispatch::Result;
use support::{
	decl_event, decl_module, decl_storage, ensure,
	traits::{
		Currency, ExistenceRequirement, Imbalance, LockIdentifier, LockableCurrency, ReservableCurrency,
		SignedImbalance, UpdateBalanceOutcome, WithdrawReason, WithdrawReasons,
	},
	Parameter, StorageDoubleMap, StorageMap, StorageValue,
};
use system::{ensure_signed, ensure_root};

// test mod
// mod mock;
// mod tests;

pub use self::imbalances::{NegativeImbalance, PositiveImbalance};

pub trait Trait: system::Trait {
	type Balance: Parameter
		+ Member
		+ SimpleArithmetic
		+ Default
		+ Copy
		+ MaybeSerializeDebug
		+ From<Self::BlockNumber>;
	type AssetId: Parameter + Member + SimpleArithmetic + Default + Copy;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub trait Subtrait: system::Trait {
	type Balance: Parameter
		+ Member
		+ SimpleArithmetic
		+ Default
		+ Copy
		+ MaybeSerializeDebug
		+ From<Self::BlockNumber>;
	type AssetId: Parameter + Member + SimpleArithmetic + Default + Copy;
}

impl<T: Trait> Subtrait for T {
	type Balance = T::Balance;
	type AssetId = T::AssetId;
}

/// Asset creation options.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq, Eq)]
pub struct AssetOptions<Balance: HasCompact> {
	/// Initial issuance of this asset. All deposit to the creater of the asset.
	#[codec(compact)]
	pub initial_issuance: Balance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceLock<Balance, BlockNumber> {
	pub id: LockIdentifier,
	pub amount: Balance,
	pub until: BlockNumber,
	pub reasons: WithdrawReasons,
}

decl_storage! {
	trait Store for Module<T: Trait> as GenericAsset {
		/// Total issuance of a given asset.
		pub TotalIssuance get(total_issuance): map T::AssetId => T::Balance;

		/// The free balance of a given asset under an account.
		pub FreeBalance: double_map T::AssetId, twox_128(T::AccountId) => T::Balance;

		/// The reserved balance of a given asset under an account.
		pub ReservedBalance: double_map T::AssetId, twox_128(T::AccountId) => T::Balance;

		/// Next available ID for user-created asset.
		pub NextAssetId get(next_asset_id) config(): T::AssetId;

		/// Any liquidity locks on some account balances.
		pub Locks get(locks): map T::AccountId => Vec<BalanceLock<T::Balance, T::BlockNumber>>;

		/// The identity of the asset which is the one that is designated for the chain's staking system.
		pub StakingAssetId get(staking_asset_id) config(): T::AssetId;

		/// The identity of the asset which is the one that is designated for the chain's encouraging system.
		pub ClaimingAssetId get(claiming_asset_id) config(): T::AssetId;

		/// The identity of the asset which is the one that is designated for paying the chain's transaction fee.
		pub SpendingAssetId get(spending_asset_id) config(): T::AssetId;
	}
	add_extra_genesis {
		config(assets): Vec<T::AssetId>;

		build(|config: &GenesisConfig<T>| {
			config.assets.iter().for_each(|asset_id| {
				<TotalIssuance<T>>::insert(asset_id, &Zero::zero());
				<FreeBalance<T>>::insert(asset_id, &<<T as system::Trait>::AccountId as Default>::default(), &Zero::zero());
			});
		});
	}
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AssetId,
		AssetOptions = AssetOptions<<T as Trait>::Balance>
	{
		/// Asset created (asset_id, creator, asset_options).
		Created(AssetId, AccountId, AssetOptions),
		/// New asset minted (asset_id, account, amount).
		Minted(AssetId, AccountId, Balance),
		/// Asset burned (asset_id, account, amount).
		Burned(AssetId, AccountId, Balance),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {
	// PUBLIC IMMUTABLES

	/// Creates an asset.
	///
	/// # Arguments
	/// * `asset_id`: An ID of a reserved asset.
	/// If not provided, a user-generated asset will be created with the next available ID.
	/// * `from_account`: The initiator account of this call
	/// * `asset_options`: Asset creation options.
	///
	pub fn create_asset(
		asset_id: Option<T::AssetId>,
		from_account: Option<T::AccountId>,
		options: AssetOptions<T::Balance>,
	) -> Result {
		let asset_id = if let Some(asset_id) = asset_id {
			ensure!(!<TotalIssuance<T>>::exists(&asset_id), "Asset id already taken.");
			ensure!(asset_id < Self::next_asset_id(), "Asset id not available.");
			asset_id
		} else {
			let asset_id = Self::next_asset_id();
			let next_id = asset_id
				.checked_add(&One::one())
				.ok_or_else(|| "No new user asset id available.")?;
			<NextAssetId<T>>::put(next_id);
			asset_id
		};

		let account_id = from_account.unwrap_or_default();

		<TotalIssuance<T>>::insert(asset_id, &options.initial_issuance);
		<FreeBalance<T>>::insert(&asset_id, &account_id, &options.initial_issuance);

		Self::deposit_event(RawEvent::Created(asset_id, account_id, options));

		Ok(())
	}

	/// Mint account's amount
	///
	/// # Arguments
	/// * `asset_id`: An ID of a reserved asset.
	/// * `to`: The initiator account of this call
	/// * `amount`: Balance
	///
	pub fn make_mint(asset_id: &T::AssetId, to: &T::AccountId, amount: T::Balance) -> Result {
		Self::check_mint(asset_id, to, amount)?;
		Self::do_mint(asset_id, to, amount);

		Ok(())
	}
	fn check_mint(asset_id: &T::AssetId, to: &T::AccountId, amount: T::Balance) -> Result {
		Self::free_balance(asset_id, to).checked_add(&amount)
			.ok_or("free balance got overflow after minting.")?;

		<TotalIssuance<T>>::get(asset_id).checked_add(&amount)
			.ok_or("total_issuance got overflow after minting.")?;

		Ok(())
	}
	/// check outside
	fn do_mint(asset_id: &T::AssetId, to: &T::AccountId, amount: T::Balance) {
		<TotalIssuance<T>>::mutate(asset_id, |issued| *issued += amount);
		<FreeBalance<T>>::mutate(asset_id, to, |balance| *balance += amount);

		// send event
		Self::deposit_event(RawEvent::Minted(*asset_id, to.clone(), amount));
	}

	/// Burn account's amount
	///
	/// # Arguments
	/// * `asset_id`: An ID of a reserved asset.
	/// * `to`: The initiator account of this call
	/// * `amount`: burn amount
	///
	pub fn make_burn(asset_id: &T::AssetId, to: &T::AccountId, amount: T::Balance) -> Result {
		Self::check_burn(asset_id, to, amount)?;
		Self::do_burn(asset_id, to, amount);

		Ok(())
	}
	fn check_burn(asset_id: &T::AssetId, to: &T::AccountId, amount: T::Balance) -> Result {
		Self::free_balance(&asset_id, &to).checked_sub(&amount)
			.ok_or("free_balance got underflow after burning")?;

		<TotalIssuance<T>>::get(asset_id).checked_sub(&amount)
			.ok_or("total_issuance got underflow after burning")?;

		Ok(())
	}
	fn do_burn(asset_id: &T::AssetId, to: &T::AccountId, amount: T::Balance) {
		<TotalIssuance<T>>::mutate(asset_id, |issued| *issued -= amount);
		<FreeBalance<T>>::mutate(asset_id, to, |balance| *balance -= amount);
		// send event
		Self::deposit_event(RawEvent::Burned(*asset_id, to.clone(), amount));
	}

	/// Convert account's amount
	///
	/// # Arguments
	/// * `owner`: The initiator account of this call
	/// * `src_asset_id`: An ID of a reserved asset
	/// * `dst_asset_id`: An ID of a reserved asset.
	/// * `src_burn_amount`: burn amount.
	/// * `dst_mint_amount`: mint amount.
	///
	pub fn make_convert(
		owner: &T::AccountId,
		src_asset_id: &T::AssetId,
		dst_asset_id: &T::AssetId,
		src_burn_amount: T::Balance,
		dst_mint_amount: T::Balance
	) -> Result {
		// ensure withdraw
		let new_src_balance = Self::free_balance(src_asset_id, owner)
			.checked_sub(&src_burn_amount)
			.ok_or("balance too low to src amount")?;
		Self::ensure_can_withdraw(src_asset_id, owner, src_burn_amount, WithdrawReason::TransactionPayment, new_src_balance)?;

		// ensure
		Self::check_burn(src_asset_id, owner, src_burn_amount)?;
		Self::check_mint(dst_asset_id, owner, dst_mint_amount)?;

		// do save
		Self::do_burn(src_asset_id, owner, src_burn_amount);
		Self::do_mint(dst_asset_id, owner, dst_mint_amount);

		Ok(())
	}

	/// Get an account's total balance of an asset kind.
	pub fn total_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
		Self::free_balance(asset_id, who) + Self::reserved_balance(asset_id, who)
	}

	/// Get an account's free balance of an asset kind.
	pub fn free_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
		<FreeBalance<T>>::get(asset_id, who)
	}

	/// Get an account's reserved balance of an asset kind.
	pub fn reserved_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
		<ReservedBalance<T>>::get(asset_id, who)
	}

	/// Move `amount` from free balance to reserved balance.
	///
	/// If the free balance is lower than `amount`, then no funds will be moved and an `Err` will
	/// be returned. This is different behavior than `unreserve`.
	pub fn reserve(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> Result {
		// Do we need to consider that this is an atomic transaction?
		let original_reserve_balance = Self::reserved_balance(asset_id, who);
		let original_free_balance = Self::free_balance(asset_id, who);
		if original_free_balance < amount {
			return Err("not enough free funds");
		}
		let new_reserve_balance = original_reserve_balance + amount;
		Self::set_reserved_balance(asset_id, who, new_reserve_balance);
		let new_free_balance = original_free_balance - amount;
		Self::set_free_balance(asset_id, who, new_free_balance);
		Ok(())
	}

	/// Moves up to `amount` from reserved balance to free balance. This function cannot fail.
	///
	/// As many assets up to `amount` will be moved as possible. If the reserve balance of `who`
	/// is less than `amount`, then the remaining amount will be returned.
	/// NOTE: This is different behavior than `reserve`.
	pub fn unreserve(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> T::Balance {
		let b = Self::reserved_balance(asset_id, who);
		let actual = rstd::cmp::min(b, amount);
		let original_free_balance = Self::free_balance(asset_id, who);
		let new_free_balance = original_free_balance + actual;
		Self::set_free_balance(asset_id, who, new_free_balance);
		Self::set_reserved_balance(asset_id, who, b - actual);
		amount - actual
	}

	/// Deduct up to `amount` from the combined balance of `who`, preferring to deduct from the
	/// free balance. This function cannot fail.
	///
	/// As much funds up to `amount` will be deducted as possible. If this is less than `amount`
	/// then `Some(remaining)` will be returned. Full completion is given by `None`.
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn slash(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> Option<T::Balance> {
		let free_balance = Self::free_balance(asset_id, who);
		let free_slash = rstd::cmp::min(free_balance, amount);
		let new_free_balance = free_balance - free_slash;
		Self::set_free_balance(asset_id, who, new_free_balance);
		if free_slash < amount {
			Self::slash_reserved(asset_id, who, amount - free_slash)
		} else {
			None
		}
	}

	/// Deducts up to `amount` from reserved balance of `who`. This function cannot fail.
	///
	/// As much funds up to `amount` will be deducted as possible. If the reserve balance of `who`
	/// is less than `amount`, then a non-zero second item will be returned.
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn slash_reserved(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> Option<T::Balance> {
		let original_reserve_balance = Self::reserved_balance(asset_id, who);
		let slash = rstd::cmp::min(original_reserve_balance, amount);
		let new_reserve_balance = original_reserve_balance - slash;
		Self::set_reserved_balance(asset_id, who, new_reserve_balance);
		if amount == slash {
			None
		} else {
			Some(amount - slash)
		}
	}

	/// Move up to `amount` from reserved balance of account `who` to free balance of account
	/// `beneficiary`.
	///
	/// As much funds up to `amount` will be moved as possible. If this is less than `amount`, then
	/// the `remaining` would be returned, else `Zero::zero()`.
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn repatriate_reserved(
		asset_id: &T::AssetId,
		who: &T::AccountId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
	) -> T::Balance {
		let b = Self::reserved_balance(asset_id, who);
		let slash = rstd::cmp::min(b, amount);

		let original_free_balance = Self::free_balance(asset_id, beneficiary);
		let new_free_balance = original_free_balance + slash;
		Self::set_free_balance(asset_id, beneficiary, new_free_balance);

		let new_reserve_balance = b - slash;
		Self::set_reserved_balance(asset_id, who, new_reserve_balance);
		amount - slash
	}

	/// Return `Ok` iff the account is able to make a withdrawal of the given amount
	/// for the given reason.
	///
	/// `Err(...)` with the reason why not otherwise.
	pub fn ensure_can_withdraw(
		asset_id: &T::AssetId,
		who: &T::AccountId,
		_amount: T::Balance,
		reason: WithdrawReason,
		new_balance: T::Balance,
	) -> Result {
		if asset_id != &Self::staking_asset_id() {
			return Ok(());
		}

		let locks = Self::locks(who);
		if locks.is_empty() {
			return Ok(());
		}
		let now = <system::Module<T>>::block_number();
		if Self::locks(who)
			.into_iter()
			.all(|l| now >= l.until || new_balance >= l.amount || !l.reasons.contains(reason))
		{
			Ok(())
		} else {
			Err("account liquidity restrictions prevent withdrawal")
		}
	}

	// PRIVATE MUTABLES

	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_reserved_balance(asset_id: &T::AssetId, who: &T::AccountId, balance: T::Balance) {
		<ReservedBalance<T>>::insert(asset_id, who, &balance);
	}

	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_free_balance(asset_id: &T::AssetId, who: &T::AccountId, balance: T::Balance) {
		<FreeBalance<T>>::insert(asset_id, who, &balance);
	}

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		until: T::BlockNumber,
		reasons: WithdrawReasons,
	) {
		let now = <system::Module<T>>::block_number();
		let mut new_lock = Some(BalanceLock {
			id,
			amount,
			until,
			reasons,
		});
		let mut locks = <Module<T>>::locks(who)
			.into_iter()
			.filter_map(|l| {
				if l.id == id {
					new_lock.take()
				} else if l.until > now {
					Some(l)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		<Locks<T>>::insert(who, locks);
	}

	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		until: T::BlockNumber,
		reasons: WithdrawReasons,
	) {
		let now = <system::Module<T>>::block_number();
		let mut new_lock = Some(BalanceLock {
			id,
			amount,
			until,
			reasons,
		});
		let mut locks = <Module<T>>::locks(who)
			.into_iter()
			.filter_map(|l| {
				if l.id == id {
					new_lock.take().map(|nl| BalanceLock {
						id: l.id,
						amount: l.amount.max(nl.amount),
						until: l.until.max(nl.until),
						reasons: l.reasons | nl.reasons,
					})
				} else if l.until > now {
					Some(l)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		<Locks<T>>::insert(who, locks);
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		let now = <system::Module<T>>::block_number();
		let locks = <Module<T>>::locks(who)
			.into_iter()
			.filter_map(|l| if l.until > now && l.id != id { Some(l) } else { None })
			.collect::<Vec<_>>();
		<Locks<T>>::insert(who, locks);
	}
}

pub trait AssetIdProvider {
	type AssetId;
	fn asset_id() -> Self::AssetId;
}

// wrapping these imbalanes in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
	use super::{result, AssetIdProvider, Imbalance, Saturating, StorageMap, Subtrait, Zero};
	use rstd::mem;

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been created without any equal and opposite accounting.
	#[must_use]
	pub struct PositiveImbalance<T: Subtrait, U: AssetIdProvider<AssetId = T::AssetId>>(
		T::Balance,
		rstd::marker::PhantomData<U>,
	);
	impl<T, U> PositiveImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		pub fn new(amount: T::Balance) -> Self {
			PositiveImbalance(amount, Default::default())
		}
	}

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been destroyed without any equal and opposite accounting.
	#[must_use]
	pub struct NegativeImbalance<T: Subtrait, U: AssetIdProvider<AssetId = T::AssetId>>(
		T::Balance,
		rstd::marker::PhantomData<U>,
	);
	impl<T, U> NegativeImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		pub fn new(amount: T::Balance) -> Self {
			NegativeImbalance(amount, Default::default())
		}
	}

	impl<T, U> Imbalance<T::Balance> for PositiveImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		type Opposite = NegativeImbalance<T, U>;

		fn zero() -> Self {
			Self::new(Zero::zero())
		}
		fn drop_zero(self) -> result::Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self::new(first), Self::new(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self::new(a - b))
			} else {
				Err(NegativeImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T, U> Imbalance<T::Balance> for NegativeImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		type Opposite = PositiveImbalance<T, U>;

		fn zero() -> Self {
			Self::new(Zero::zero())
		}
		fn drop_zero(self) -> result::Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self::new(first), Self::new(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self::new(a - b))
			} else {
				Err(PositiveImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T, U> Drop for PositiveImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<super::ElevatedTrait<T>>>::mutate(&U::asset_id(), |v| *v = v.saturating_add(self.0));
		}
	}

	impl<T, U> Drop for NegativeImbalance<T, U>
	where
		T: Subtrait,
		U: AssetIdProvider<AssetId = T::AssetId>,
	{
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<super::ElevatedTrait<T>>>::mutate(&U::asset_id(), |v| *v = v.saturating_sub(self.0));
		}
	}
}

// TODO: #2052
// Somewhat ugly hack in order to gain access to module's `increase_total_issuance_by`
// using only the Subtrait (which defines only the types that are not dependent
// on Positive/NegativeImbalance). Subtrait must be used otherwise we end up with a
// circular dependency with Trait having some types be dependent on PositiveImbalance<Trait>
// and PositiveImbalance itself depending back on Trait for its Drop impl (and thus
// its type declaration).
// This works as long as `increase_total_issuance_by` doesn't use the Imbalance
// types (basically for charging fees).
// This should eventually be refactored so that the three type items that do
// depend on the Imbalance type (TransactionPayment, TransferPayment, DustRemoval)
// are placed in their own SRML module.
struct ElevatedTrait<T: Subtrait>(T);
impl<T: Subtrait> Clone for ElevatedTrait<T> {
	fn clone(&self) -> Self {
		unimplemented!()
	}
}
impl<T: Subtrait> PartialEq for ElevatedTrait<T> {
	fn eq(&self, _: &Self) -> bool {
		unimplemented!()
	}
}
impl<T: Subtrait> Eq for ElevatedTrait<T> {}
impl<T: Subtrait> system::Trait for ElevatedTrait<T> {
	type Origin = T::Origin;
	type Call = T::Call;
	type Index = T::Index;
	type BlockNumber = T::BlockNumber;
	type Hash = T::Hash;
	type Hashing = T::Hashing;
	type AccountId = T::AccountId;
	type Lookup = T::Lookup;
	type Header = T::Header;
	type Event = ();
	type MaximumBlockWeight = T::MaximumBlockWeight;
	type MaximumBlockLength = T::MaximumBlockLength;
	type AvailableBlockRatio = T::AvailableBlockRatio;
	type WeightMultiplierUpdate = ();
	type BlockHashCount = T::BlockHashCount;
	type Version = T::Version;
}
impl<T: Subtrait> Trait for ElevatedTrait<T> {
	type Balance = T::Balance;
	type AssetId = T::AssetId;
	type Event = ();
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AssetCurrency<T, U>(rstd::marker::PhantomData<T>, rstd::marker::PhantomData<U>);

impl<T, U> Currency<T::AccountId> for AssetCurrency<T, U>
where
	T: Trait,
	U: AssetIdProvider<AssetId = T::AssetId>,
{
	type Balance = T::Balance;
	type PositiveImbalance = PositiveImbalance<T, U>;
	type NegativeImbalance = NegativeImbalance<T, U>;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		Self::free_balance(&who) + Self::reserved_balance(&who)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		<Module<T>>::free_balance(&U::asset_id(), &who)
	}

	/// Returns the total staking asset issuance
	fn total_issuance() -> Self::Balance {
		<Module<T>>::total_issuance(U::asset_id())
	}

	fn minimum_balance() -> Self::Balance {
		Zero::zero()
	}

	fn transfer(transactor: &T::AccountId, dest: &T::AccountId, value: Self::Balance) -> Result {
		ensure!(transactor == dest, "transfer is not supported.");
		Ok(())
	}

	fn ensure_can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
		reason: WithdrawReason,
		new_balance: Self::Balance,
	) -> Result {
		<Module<T>>::ensure_can_withdraw(&U::asset_id(), who, amount, reason, new_balance)
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		reason: WithdrawReason,
		_: ExistenceRequirement, // no existential deposit policy for generic asset
	) -> result::Result<Self::NegativeImbalance, &'static str> {
		let new_balance = Self::free_balance(who)
			.checked_sub(&value)
			.ok_or_else(|| "account has too few funds")?;
		Self::ensure_can_withdraw(who, value, reason, new_balance)?;
		<Module<T>>::set_free_balance(&U::asset_id(), who, new_balance);
		Ok(NegativeImbalance::new(value))
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> result::Result<Self::PositiveImbalance, &'static str> {
		// No existential deposit rule and creation fee in GA. `deposit_into_existing` is same with `deposit_creating`.
		Ok(Self::deposit_creating(who, value))
	}

	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		let (imbalance, _) = Self::make_free_balance_be(who, Self::free_balance(who) + value);
		if let SignedImbalance::Positive(p) = imbalance {
			p
		} else {
			// Impossible, but be defensive.
			Self::PositiveImbalance::zero()
		}
	}

	fn make_free_balance_be(
		who: &T::AccountId,
		balance: Self::Balance,
	) -> (
		SignedImbalance<Self::Balance, Self::PositiveImbalance>,
		UpdateBalanceOutcome,
	) {
		let original = <Module<T>>::free_balance(&U::asset_id(), who);
		let imbalance = if original <= balance {
			SignedImbalance::Positive(PositiveImbalance::new(balance - original))
		} else {
			SignedImbalance::Negative(NegativeImbalance::new(original - balance))
		};
		<Module<T>>::set_free_balance(&U::asset_id(), who, balance);
		(imbalance, UpdateBalanceOutcome::Updated)
	}

	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		<Module<T>>::free_balance(&U::asset_id(), &who) >= value
	}

	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		let remaining = <Module<T>>::slash(&U::asset_id(), who, value);
		if let Some(r) = remaining {
			(NegativeImbalance::new(value - r), r)
		} else {
			(NegativeImbalance::new(value), Zero::zero())
		}
	}

	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		<TotalIssuance<T>>::mutate(&U::asset_id(), |issued|
			issued.checked_sub(&amount).unwrap_or_else(|| {
				amount = *issued;
				Zero::zero()
			})
		);
		PositiveImbalance::new(amount)
	}

	fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
		<TotalIssuance<T>>::mutate(&U::asset_id(), |issued|
			*issued = issued.checked_add(&amount).unwrap_or_else(|| {
				amount = Self::Balance::max_value() - *issued;
				Self::Balance::max_value()
			})
		);
		NegativeImbalance::new(amount)
	}
}

impl<T, U> ReservableCurrency<T::AccountId> for AssetCurrency<T, U>
where
	T: Trait,
	U: AssetIdProvider<AssetId = T::AssetId>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		Self::free_balance(who)
			.checked_sub(&value)
			.map_or(false, |new_balance|
				<Module<T>>::ensure_can_withdraw(&U::asset_id(), who, value, WithdrawReason::Reserve, new_balance).is_ok()
			)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		<Module<T>>::reserved_balance(&U::asset_id(), &who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> result::Result<(), &'static str> {
		<Module<T>>::reserve(&U::asset_id(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Module<T>>::unreserve(&U::asset_id(), who, value)
	}

	fn slash_reserved(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		let b = Self::reserved_balance(&who.clone());
		let slash = cmp::min(b, value);

		<Module<T>>::set_reserved_balance(&U::asset_id(), who, b - slash);
		(NegativeImbalance::new(slash), value - slash)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
	) -> result::Result<Self::Balance, &'static str> {
		Ok(<Module<T>>::repatriate_reserved(&U::asset_id(), slashed, beneficiary, value))
	}
}

pub struct StakingAssetIdProvider<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> AssetIdProvider for StakingAssetIdProvider<T> {
	type AssetId = T::AssetId;
	fn asset_id() -> Self::AssetId {
		<Module<T>>::staking_asset_id()
	}
}

impl<T> LockableCurrency<T::AccountId> for AssetCurrency<T, StakingAssetIdProvider<T>>
where
	T: Trait,
	T::Balance: MaybeSerializeDebug,
{
	type Moment = T::BlockNumber;

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		until: T::BlockNumber,
		reasons: WithdrawReasons,
	) {
		<Module<T>>::set_lock(id, who, amount, until, reasons)
	}

	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		until: T::BlockNumber,
		reasons: WithdrawReasons,
	) {
		<Module<T>>::extend_lock(id, who, amount, until, reasons)
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		<Module<T>>::remove_lock(id, who)
	}
}

pub struct SpendingAssetIdProvider<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> AssetIdProvider for SpendingAssetIdProvider<T> {
	type AssetId = T::AssetId;
	fn asset_id() -> Self::AssetId {
		<Module<T>>::spending_asset_id()
	}
}

pub struct ClaimingAssetIdProvider<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> AssetIdProvider for ClaimingAssetIdProvider<T> {
	type AssetId = T::AssetId;
	fn asset_id() -> Self::AssetId {
		<Module<T>>::claiming_asset_id()
	}
}

pub type StakingAssetCurrency<T> = AssetCurrency<T, StakingAssetIdProvider<T>>;
pub type SpendingAssetCurrency<T> = AssetCurrency<T, SpendingAssetIdProvider<T>>;
pub type ClaimingAssetCurrency<T> = AssetCurrency<T, ClaimingAssetIdProvider<T>>;
