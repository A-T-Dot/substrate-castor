use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result, Parameter, ensure};
use sr_primitives::traits::{ Member, SimpleArithmetic, Bounded, CheckedAdd };
use system::ensure_signed;
use codec::{Encode, Decode};
use rstd::result;
use crate::ge;
use crate::node;
use support::traits::{Currency};


/// The module's configuration trait.
pub trait Trait: system::Trait + ge::Trait + timestamp::Trait + node::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type TcxId:  Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
	type TcxType: Parameter + Member + Default + Copy;
	type ActionId: Parameter + Member + Default + Copy;
	type ListingId:  Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
}

type BalanceOf<T> = <<T as ge::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;


#[cfg_attr(feature ="std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct Tcx<TcxType> {
  pub tcx_type: TcxType,
}


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Listing<ContentHash, Balance, Moment> {
  node_id: ContentHash,
  amount: Balance,
  application_expiry: Moment,
  whitelisted: bool,
  challenge_id: u32,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Challenge<ListingId, Balance, Moment> {
  listing_id: ListingId,
  amount: Balance,
  voting_ends: Moment,
  resolved: bool,
  reward_pool: Balance,
  total_tokens: Balance
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Vote<U> {
  value: bool,
  deposit: U,
  claimed: bool,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Poll<T, U> {
  listing_hash: T,
  votes_for: U,
  votes_against: U,
  passed: bool,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Tcx {
    AllTcxsArray get(tcx): map T::TcxId => Option<Tcx<T::TcxType>>;
		AllTcxsCount get(all_tcxs_count): T::TcxId;

    TcxOwner get(owner_of): map T::TcxId => Option<T::GeId>;

    OwnedTcxsArray get(tcx_of_owner_by_index): map (T::GeId, T::TcxId) => T::TcxId;
    OwnedTcxsCount get(owned_tcxs_count): map T::GeId => T::TcxId;

		// actual tcx
    TcxListings get(listings_of_tcr_by_node_id): map (T::TcxId, T::ContentHash) => Listing<T::ContentHash, BalanceOf<T>, T::Moment>;
		TcxListingsCount get(listing_count_of_tcr): map T::TcxId => T::ListingId;
    TcxListingsIndexHash get(node_id_of_listing): map (T::TcxId, T::ListingId) => T::ContentHash;

	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn propose(origin, tcx_id: T::TcxId, node_id: T::ContentHash, amount: BalanceOf<T>, action_id: T::ActionId) -> Result {
			let who = ensure_signed(origin)?;
			
			// deduction balace for application
			// <token::Module<T>>::lock(sender.clone(), deposit, hashed.clone())?;
			
			// more than min deposit
			let ge_id = Self::owner_of(tcx_id).ok_or("TCX does not exist / TCX owner does not exist")?;
			let governance_entity = <ge::Module<T>>::governance_entity(ge_id).ok_or("GE does not exist")?;
			let min_deposit = governance_entity.min_deposit;
			ensure!(amount >= min_deposit, "deposit should be more than min_deposit");

			let now = <timestamp::Module<T>>::get();
			let apply_stage_len = governance_entity.apply_stage_len;
			let app_exp = now.checked_add(&apply_stage_len).ok_or("Overflow when setting application expiry.")?;

			let listing_id = Self::listing_count_of_tcr(tcx_id);
			let new_listing_count = listing_id.checked_add(&T::ListingId::from(1)).ok_or("Exceed max listing count")?;

			ensure!(!<TcxListings<T>>::exists((tcx_id,node_id)), "Listing already exists");

			// create a new listing instance
			let new_listing = Listing {
				node_id: node_id,
				amount: amount,
				whitelisted: false,
				challenge_id: 0,
				application_expiry: app_exp,
			};

			<TcxListings<T>>::insert((tcx_id, node_id), new_listing);
			<TcxListingsCount<T>>::insert(tcx_id, new_listing_count);
			<TcxListingsIndexHash<T>>::insert((tcx_id, new_listing_count), node_id);

			Self::deposit_event(RawEvent::Proposed(who, tcx_id, node_id, amount, action_id));

			Ok(())
		}

    pub fn challenge(origin) -> Result {
			let sender = ensure_signed(origin)?;


		}

    pub fn vote(origin) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;
      
			Ok(())
		}

    pub fn resolve(origin) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;
      
			Ok(())
		}

    pub fn claim(origin) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;
      
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
		ContentHash = <T as node::Trait>::ContentHash,
		TcxId = <T as Trait>::TcxId,
		ActionId = <T as Trait>::ActionId,
		Balance = <<T as ge::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance,
	{
		Proposed(AccountId, TcxId, ContentHash, Balance, ActionId),
		Challenged(AccountId, ContentHash, u32, Balance),
		Voted(AccountId, u32, Balance),
		Resolved(ContentHash, u32),
		Accepted(ContentHash),
		Rejected(ContentHash),
		Claimed(AccountId, u32),
	}
);

impl<T: Trait> Module<T> {
	pub fn create(ge_id: T::GeId, tcx_type: T::TcxType) -> rstd::result::Result<T::TcxId, &'static str> {
		let one = T::TcxId::from(1 as u32);

		// check global tcx count
		let all_tcxs_count = <AllTcxsCount<T>>::get();
		let new_all_tcxs_count = all_tcxs_count.checked_add(&one).ok_or("Exceed maximum tcx count")?;

		// check owner tcx count
		let owned_tcxs_count = <OwnedTcxsCount<T>>::get(ge_id);
		let new_owned_tcxs_count = owned_tcxs_count.checked_add(&one).ok_or("Exceed maximum tcx count for ge")?;

		let tcx  =  Tcx {
			tcx_type: tcx_type,
		};
		<AllTcxsArray<T>>::insert(new_all_tcxs_count, tcx);
		<AllTcxsCount<T>>::put(new_all_tcxs_count);

		<TcxOwner<T>>::insert(new_all_tcxs_count, ge_id);

		<OwnedTcxsArray<T>>::insert((ge_id, new_owned_tcxs_count), new_all_tcxs_count);
		<OwnedTcxsCount<T>>::insert(ge_id, new_owned_tcxs_count);

		// return new tcx_id
		Ok(new_all_tcxs_count)
	}
}