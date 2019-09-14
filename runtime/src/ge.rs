use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result, Parameter, ensure};
use sr_primitives::traits::{ Member, SimpleArithmetic, Bounded, CheckedAdd, One };
use system::ensure_signed;
use codec::{Encode, Decode};
use rstd::{result, convert::{TryInto}};
use support::traits::{WithdrawReasons, LockableCurrency, Currency};
use crate::tcx;


/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
  type GeId:  Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[cfg_attr(feature ="std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct GovernanceEntity {
		pub threshold: u64,
    // rules
    // threshold for ge, tcx, time
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Ge {

    GovernanceEntities get(governance_entity): map T::GeId => Option<GovernanceEntity>;
    GovernanceEntitiesCount get(governance_entities_count): T::GeId;

    // Stake: which, amount
    StakedAmount get(staked_amount): map (T::GeId, T::AccountId) => BalanceOf<T>;
    TotalStakedAmount get(total_staked_amount): map T::GeId => BalanceOf<T>;

    // Invest
    InvestedAmount get(invested_amount): map (T::GeId, T::AccountId) => BalanceOf<T>;
    TotalInvestedAmount get(total_invested_amount): map T::GeId => BalanceOf<T>;
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
      let temp: Option<BalanceOf<T>> = balance.try_into().ok();
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
    pub fn stake(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      let who = ensure_signed(origin)?;
      ensure!(<GovernanceEntities<T>>::exists(id), "GE does not exist");
      // TODO: actually stake real balance, below simulates
      const STAKING_ID: [u8; 8] = *b"staking ";
      T::Currency::set_lock(
        STAKING_ID,
        &who,
        amount,
        T::BlockNumber::max_value(),
        WithdrawReasons::all(),
      );
      
      Ok(())
    }

    pub fn withdraw(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      // TODO: withdraw balance
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
    Balance = BalanceOf<T>,
  {
		Created(AccountId, GeId, Balance),
	}
);

impl<T: Trait> Module<T> {

}



/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
  pub type Balance = u128;

  impl balances::Trait for Test {
    type Balance = Balance;
    /// What to do if an account's free balance gets zeroed.
    type OnFreeBalanceZero = ();
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// The ubiquitous event type.
    type Event = ();
    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ();
    type TransferFee = ();
    type CreationFee = ();
    type TransactionBaseFee = ();
    type TransactionByteFee = ();
    type WeightToFee = ();
  }

	impl Trait for Test {
		type Event = ();
    type GeId = u64;
	  type Currency = balances::Module<Test>;
	}
	type Tcx = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			// assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// // asserting that the stored value is equal to what we stored
			// assert_eq!(TemplateModule::something(), Some(42));
      assert_ok!(Tcx::create(Origin::signed(1)));
      assert_eq!(Tcx::governance_entities_count(), 1);
		});
	}
}

