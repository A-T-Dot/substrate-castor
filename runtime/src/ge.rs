use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result, Parameter, ensure};
use sr_primitives::traits::{ Member, SimpleArithmetic, Bounded, CheckedAdd, One };
use system::ensure_signed;
use codec::{Encode, Decode};
use rstd::{result, convert::{TryInto}};
use crate::tcx;


/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
  type GeId:  Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
}

#[cfg_attr(feature ="std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct GovernanceEntity {
		pub threshold: u64,
    // rules
    // threshold for ge, tcx
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Ge {

    GovernanceEntities get(governance_entity): map T::GeId => Option<GovernanceEntity>;
    GovernanceEntitiesCount get(governance_entities_count): T::GeId;

	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;
    
    // create ge with rules
    pub fn create(origin) -> Result {
      let who = ensure_signed(origin)?;
      let count = Self::governance_entities_count();
      
      // get new ge_id
      let one = T::GeId::from(1 as u32);
      let new_count = count.checked_add(&one).ok_or("exceed maximum amount of ge")?;

      // TODO: do something with balance here e.g. lock balance, reduce balance
      let balance: u128 = 12;
      let temp: Option<T::Balance> = balance.try_into().ok();
      let balance = temp.ok_or("Cannot convert to balance")?;

      let new_governance_entity = GovernanceEntity {
        threshold: 0,
      };

      <GovernanceEntities<T>>::insert(new_count, new_governance_entity);
      <GovernanceEntitiesCount<T>>::put(new_count);

      Self::deposit_event(RawEvent::Created(who, new_count, balance));

      Ok(())
    }

    // stake ge
    pub fn stake(origin, id: T::GeId, amount: T::Balance) -> Result {
      Ok(())
    }


    pub fn invest(origin) -> Result {
      
      
      Ok(())
    }

    pub fn updateRules(origin) -> Result {
      Ok(())
    }


	}
}

decl_event!(
	pub enum Event<T> 
  where 
    AccountId = <T as system::Trait>::AccountId,
    <T as Trait>::GeId,
    <T as balances::Trait>::Balance,
  {
		Created(AccountId, GeId, Balance),
	}
);

impl<T: Trait> Module<T> {

}