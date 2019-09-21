//! # Activity Module
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use codec::{Encode, Decode};
use support::{
	StorageValue, StorageMap, Parameter, decl_event, decl_storage, decl_module,
	traits::{
		Currency, LockableCurrency, ReservableCurrency,
		UpdateBalanceOutcome, OnFreeBalanceZero, OnUnbalanced,
		WithdrawReason, WithdrawReasons, LockIdentifier, ExistenceRequirement,
		Imbalance, SignedImbalance, Get, Time,
	},
	dispatch::Result,
};
use sr_primitives::{
	transaction_validity::{
		TransactionPriority, ValidTransaction, InvalidTransaction, TransactionValidityError,
		TransactionValidity,
	},
	traits::{
		Zero, SimpleArithmetic, StaticLookup, Member, CheckedAdd, CheckedSub, MaybeSerializeDebug,
		Saturating, Bounded, SignedExtension, SaturatedConversion, Convert,
	},
	weights::{DispatchInfo, SimpleDispatchInfo, Weight},
};
use system::{IsDeadAccount, OnNewAccount, ensure_signed, ensure_root};

use crate::non_transfer_asset::SustainableCurrency;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// Currency type for this module.
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Energy type for this module
	type EnergyCurrency: SustainableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

	/// Action point type for this module
	type ActivityCurrency: Currency<Self::AccountId>;

	/// Reputation point type for this module
	type ReputationCurrency: Currency<Self::AccountId>;

	/// Handler for the unbalanced reduction when taking transaction fees.
	type TransactionPayment: OnUnbalanced<NegativeImbalanceOf<Self>>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The fee to be paid for making a transaction; the base.
	type TransactionBaseFee: Get<BalanceOf<Self>>;

	/// The fee to be paid for making a transaction; the per-byte portion.
	type TransactionByteFee: Get<BalanceOf<Self>>;

	/// The base Energy amount of activated account
	type EnergyBaseAmount: Get<EnergyOf<Self>>;

	/// Convert a weight value into a deductible fee based on the currency type.
	type WeightToFee: Convert<Weight, BalanceOf<Self>>;

	/// Convert a fee value to energy point	
	type FeeToEnergy: Convert<BalanceOf<Self>, EnergyOf<Self>>;

	/// Convert a charging value to energy point	
	type ChargingToEnergy: Convert<BalanceOf<Self>, EnergyOf<Self>>;

	/// Convert an energy point to fee value
	type EnergyToFee: Convert<EnergyOf<Self>, BalanceOf<Self>>;

	/// Convert an energy point to locking block number
	type EnergyToLocking: Convert<EnergyOf<Self>, <Self as system::Trait>::BlockNumber>;
}

// Balance zone
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
// Energy zone
pub type EnergyOf<T> = <<T as Trait>::EnergyCurrency as Currency<<T as system::Trait>::AccountId>>::Balance;
// Action zone
pub type ActionPointOf<T> = <<T as Trait>::ActivityCurrency as Currency<<T as system::Trait>::AccountId>>::Balance;
// Reputation zone
pub type ReputationOf<T> = <<T as Trait>::ReputationCurrency as Currency<<T as system::Trait>::AccountId>>::Balance;

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Activities {
		/// Map from all extend
		pub Charged get(charged): map T::AccountId => BalanceOf<T>;
	}
}

decl_event!(
	pub enum Event<T> where
    AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
		Energy = EnergyOf<T>,
		ActionPoint = ActionPointOf<T>,
		Reputation = ReputationOf<T>
  {
		// Fee payment
		FeePayed(AccountId, Energy, Balance),
		// Energy and Actiono part
		EnergyActivated(AccountId),
		EnergyDeactivated(AccountId),
		EnergyRecovered(AccountId, Energy),
		ActionPointReward(AccountId, ActionPoint),
		// Reputation part
		ReputationReward(AccountId, Reputation),
		ReputationSlash(AccountId, Reputation),
	}
);

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event() = default;
		
		/// Bond to increase Energy
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		pub fn charge(
			origin,
			#[compact] value: BalanceOf<T>
		) {
			let who = ensure_signed(origin)?;
			Self::charge_for_energy(&who, value)?;
		}
		
		/// UnBond to decrease Energy
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		pub fn discharge(
			origin,
			#[compact] value: BalanceOf<T>
		) {
			let who = ensure_signed(origin)?;
			Self::discharge_for_energy(&who, value)?;
		}
	}
}

// The module's main implement
impl<T: Trait> Module<T> {
	// PUBLIC IMMUTABLES
	pub fn available_energy(who: &T::AccountId) -> EnergyOf<T> {
		T::EnergyCurrency::available_free_balance(who)
	}

	// PRIVATE MUTABLES
	fn charge_for_energy(who: &T::AccountId, value: BalanceOf<T>) -> Result {
		// ensure reserve
		if !T::Currency::can_reserve(who, value) {
			return Err("not enough free funds");
		}
		// check current_charged
		let current_charged = <Charged<T>>::get(who);
		let new_charged = current_charged.checked_add(&value).ok_or("account has charged overflow")?;

		let energy_to_charge = T::ChargingToEnergy::convert(value);
		let current_energy = T::EnergyCurrency::free_balance(who);
		current_energy.checked_add(&energy_to_charge).ok_or("Overflow energy amount")?;

		// MUTABLES
		T::Currency::reserve(who, value)?;
		T::EnergyCurrency::deposit_into_existing(who, energy_to_charge)?;
		<Charged<T>>::insert(who, new_charged);
		Ok(())
	}

	fn discharge_for_energy(who: &T::AccountId, value: BalanceOf<T>) -> Result {
		// check current_charged
		let current_charged = <Charged<T>>::get(who);
		let new_charged = current_charged.checked_sub(&value).ok_or("account has too few charged funds")?;
		
		let energy_to_discharge = T::ChargingToEnergy::convert(value);
		let current_energy = T::EnergyCurrency::free_balance(who);
		current_energy.checked_sub(&energy_to_discharge).ok_or("account has too few energy")?;

		// MUTABLES
		T::EnergyCurrency::withdraw(who, energy_to_discharge, WithdrawReason::Fee, ExistenceRequirement::KeepAlive)?;
		T::Currency::unreserve(who, value);
		<Charged<T>>::insert(who, new_charged);
		Ok(())
	}
}

impl<T: Trait> OnNewAccount<T::AccountId> for Module<T> {
	// Implementation of the config type managing the creation of new accounts.
	fn on_new_account(who: &T::AccountId) {
		T::EnergyCurrency::deposit_creating(who, T::EnergyBaseAmount::get());
		Self::deposit_event(RawEvent::EnergyActivated(who.clone()));
	}
}

impl<T: Trait> OnFreeBalanceZero<T::AccountId> for Module<T> {
	fn on_free_balance_zero(who: &T::AccountId) {
		let dust = <Charged<T>>::take(who);
		if !dust.is_zero() {
			T::Currency::unreserve(who, dust);
		}
		T::EnergyCurrency::slash(who, T::EnergyCurrency::total_balance(who));
		Self::deposit_event(RawEvent::EnergyDeactivated(who.clone()));
	}
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct TakeFees<T: Trait>(#[codec(compact)] BalanceOf<T>);

impl<T: Trait> TakeFees<T> {
	/// utility constructor. Used only in client/factory code.
	pub fn from(fee: BalanceOf<T>) -> Self {
		Self(fee)
	}

	/// Compute the final fee value for a particular transaction.
	///
	/// The final fee is composed of:
	///   - _length-fee_: This is the amount paid merely to pay for size of the transaction.
	///   - _weight-fee_: This amount is computed based on the weight of the transaction. Unlike
	///      size-fee, this is not input dependent and reflects the _complexity_ of the execution
	///      and the time it consumes.
	///   - (optional) _tip_: if included in the transaction, it will be added on top. Only signed
	///      transactions can have a tip.
	fn compute_fee(len: usize, info: DispatchInfo, tip: BalanceOf<T>) -> BalanceOf<T> {
		let len_fee = if info.pay_length_fee() {
			let len = <BalanceOf<T> as From<u32>>::from(len as u32);
			let base = T::TransactionBaseFee::get();
			let per_byte = T::TransactionByteFee::get();
			base.saturating_add(per_byte.saturating_mul(len))
		} else {
			Zero::zero()
		};

		let weight_fee = {
			// cap the weight to the maximum defined in runtime, otherwise it will be the `Bounded`
			// maximum of its data type, which is not desired.
			let capped_weight = info.weight.min(<T as system::Trait>::MaximumBlockWeight::get());
			let weight_update = <system::Module<T>>::next_weight_multiplier();
			let adjusted_weight = weight_update.apply_to(capped_weight);
			T::WeightToFee::convert(adjusted_weight)
		};

		len_fee.saturating_add(weight_fee).saturating_add(tip)
	}
}

#[cfg(feature = "std")]
impl<T: Trait> rstd::fmt::Debug for TakeFees<T> {
	fn fmt(&self, f: &mut rstd::fmt::Formatter) -> rstd::fmt::Result {
		self.0.fmt(f)
	}
}

impl<T: Trait> SignedExtension for TakeFees<T> where
	BalanceOf<T>: core::marker::Send + core::marker::Sync
{
	type AccountId = <T as system::Trait>::AccountId;
	type Call = T::Call;
	type AdditionalSigned = ();
	type Pre = ();
	fn additional_signed(&self) -> rstd::result::Result<(), TransactionValidityError> { Ok(()) }

	fn validate(
		&self,
		who: &Self::AccountId,
		_call: &Self::Call,
		info: DispatchInfo,
		len: usize,
	) -> TransactionValidity {
		let fee = Self::compute_fee(len, info, self.0);
		// pay fees.
		// first use energy, second use balance
		let required_energy = T::FeeToEnergy::convert(fee);
		let available_energy = T::EnergyCurrency::available_free_balance(who);
		let using_energy = required_energy.min(available_energy);
		let mut using_fee = BalanceOf::<T>::zero();
		if using_energy < required_energy {
			using_fee = T::EnergyToFee::convert(required_energy - using_energy);
		}
		let now = <system::Module<T>>::block_number();
		let locking_block = T::EnergyToLocking::convert(using_energy);

		// lock energy and get unlocked energy
		let unlocked_energy = match T::EnergyCurrency::use_and_lock_free_balance(who, using_energy.clone(), now + locking_block) {
			Ok(result) => result,
			Err(_) => return InvalidTransaction::Payment.into(),
		};
		// dispatch EnergyRecovered
		if !unlocked_energy.is_zero() {
			<Module<T>>::deposit_event(RawEvent::EnergyRecovered(who.clone(), unlocked_energy));
		}

		let imbalance = match T::Currency::withdraw(
			who,
			using_fee.clone(),
			WithdrawReason::TransactionPayment,
			ExistenceRequirement::KeepAlive,
		) {
			Ok(imbalance) => imbalance,
			Err(_) => return InvalidTransaction::Payment.into(),
		};
		T::TransactionPayment::on_unbalanced(imbalance);

		// Send event
		<Module<T>>::deposit_event(RawEvent::FeePayed(who.clone(), using_energy, using_fee));
		
		let mut r = ValidTransaction::default();
		// NOTE: we probably want to maximize the _fee (of any type) per weight unit_ here, which
		// will be a bit more than setting the priority to tip. For now, this is enough.
		r.priority = fee.saturated_into::<TransactionPriority>();
		Ok(r)
	}
}
