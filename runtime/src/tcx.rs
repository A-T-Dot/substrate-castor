use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result, Parameter, ensure};
use sr_primitives::traits::{ Member, SimpleArithmetic, Bounded, CheckedAdd };
use system::ensure_signed;
use codec::{Encode, Decode};
use rstd::result;
use crate::ge;

/// The module's configuration trait.
pub trait Trait: system::Trait + ge::Trait + timestamp::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type TcxId:  Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
	type TcxType: Parameter + Member + Default + Copy;
	type ActionId: Parameter + Member + Default + Copy;
}


#[cfg_attr(feature ="std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct Tcx<TcxType> {
  pub tcx_type: TcxType,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Tcx {
    TcxsArray get(tcx): map T::TcxId => Option<Tcx<T::TcxType>>;
		TcxsCount get(all_tcxs_count): T::TcxId;

    TcxOwner get(owner_of): map T::TcxId => Option<T::GeId>;

    OwnedTcxsArray get(tcx_of_owner_by_index): map (T::GeId, T::TcxId) => T::TcxId;
    OwnedTcxsCount get(owned_tcxs_count): map T::GeId => T::TcxId;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn propose(origin, tcx_id: T::TcxId, action_id: T::ActionId) -> Result {
			// TODO: Propose to create new tcx, add node
			let who = ensure_signed(origin)?;

			Ok(())
		}

    pub fn challenge(origin) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;
      
			Ok(())
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
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
	}
);

impl<T: Trait> Module<T> {
	pub fn create(ge_id: T::GeId, tcx_type: T::TcxType) -> rstd::result::Result<T::TcxId, &'static str> {
		let one = T::TcxId::from(1 as u32);

		// check global tcx count
		let tcxs_count = <TcxsCount<T>>::get();
		let new_tcxs_count = tcxs_count.checked_add(&one).ok_or("Exceed maximum tcx count")?;

		// check owner tcx count
		let owned_tcxs_count = <OwnedTcxsCount<T>>::get(ge_id);
		let new_owned_tcxs_count = owned_tcxs_count.checked_add(&one).ok_or("Exceed maximum tcx count for ge")?;

		let tcx  =  Tcx {
			tcx_type: tcx_type,
		};
		<TcxsArray<T>>::insert(new_tcxs_count, tcx);
		<TcxsCount<T>>::put(new_tcxs_count);

		<TcxOwner<T>>::insert(new_tcxs_count, ge_id);

		<OwnedTcxsArray<T>>::insert((ge_id, new_owned_tcxs_count), new_tcxs_count);
		<OwnedTcxsCount<T>>::insert(ge_id, new_owned_tcxs_count);

		Ok(new_tcxs_count)
	}
}