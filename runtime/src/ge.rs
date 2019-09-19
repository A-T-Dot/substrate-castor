use codec::{Encode, Decode};
use rstd::{
  convert::{TryInto}
};
use support::{
  decl_module, decl_storage, decl_event, ensure, dispatch::Result,
  traits::{
    Currency, ReservableCurrency, LockableCurrency,
    OnUnbalanced, // WithdrawReasons, 
  },
  StorageValue, StorageMap, Parameter,
};
use sr_primitives::traits::{
  Member, SimpleArithmetic, Bounded, CheckedAdd
};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait {
  /// GovernanceEntity ID
  type GeId: Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  /// Content hash
  type ContentHash: Parameter + Member + Default + Copy;

	/// Currency type for this module.
	type Currency: ReservableCurrency<Self::AccountId>
		+ LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// Handler for the unbalanced reduction when slashing a staker.
	type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

	/// Handler for the unbalanced increment when rewarding a staker.
	type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;
}

// Balance zone
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

#[cfg_attr(feature ="std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct GovernanceEntity<Balance, Moment, ContentHash> {
  threshold: u64,
  pub min_deposit: Balance,
  pub apply_stage_len: Moment,
  pub commit_stage_len: Moment,
  pub content_hash: ContentHash,
  // rules
  // threshold for ge, tcx, time
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as Ge {

    GovernanceEntities get(governance_entity): map T::GeId => Option<GovernanceEntity<BalanceOf<T>, T::Moment, T::ContentHash>>;
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
    pub fn create(origin, content_hash: T::ContentHash) -> Result {
      let who = ensure_signed(origin)?;
      let count = Self::governance_entities_count();
      
      // get new ge_id
      let one = T::GeId::from(1 as u32);
      let new_count = count.checked_add(&one).ok_or("exceed maximum amount of ge")?;

      // TODO: do something with balance here e.g. lock balance, reduce balance
      let balance: u128 = 12;
      let temp: Option<BalanceOf<T>> = balance.try_into().ok();
      let balance = temp.ok_or("Cannot convert to balance")?;

      let new_governance_entity = GovernanceEntity::<BalanceOf<T>, T::Moment, T::ContentHash> {
        threshold: 0,
        min_deposit: BalanceOf::<T>::from(3000),
        apply_stage_len: T::Moment::from(60000),
        commit_stage_len: T::Moment::from(60000),
        content_hash: content_hash,
      };

      <GovernanceEntities<T>>::insert(new_count, new_governance_entity);
      <GovernanceEntitiesCount<T>>::put(new_count);

      Self::deposit_event(RawEvent::Created(who, new_count, balance, content_hash));

      Ok(())
    }

    // stake ge
    pub fn stake(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      let who = ensure_signed(origin)?;
      ensure!(<GovernanceEntities<T>>::exists(id), "GE does not exist");
      
      // TODO: actually stake real balance, below simulates
      // const STAKING_ID: [u8; 8] = *b"staking ";
      // T::Currency::set_lock(
      //   STAKING_ID,
      //   &who,
      //   amount,
      //   T::BlockNumber::max_value(),
      //   WithdrawReasons::all(),
      // );
      // check if enough balance

      // check if overflow
      let staked_amount = Self::staked_amount((id, who.clone()));
      let total_staked_amount = Self::total_staked_amount(id);
      let new_staked_amount = staked_amount.checked_add(&amount).ok_or("Overflow stake amount")?;
      let new_total_staked_amount = total_staked_amount.checked_add(&amount).ok_or("Overflow total stake amount")?;

      <StakedAmount<T>>::insert((id, who.clone()), new_staked_amount);
      <TotalStakedAmount<T>>::insert(id, new_total_staked_amount);


      Self::deposit_event(RawEvent::Staked(who, id, amount));

      Ok(())
    }

    pub fn withdraw(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      // TODO: withdraw balance
      let who = ensure_signed(origin)?;
      ensure!(<GovernanceEntities<T>>::exists(id), "GE does not exist");
      // TODO: actually stake real balance, below simulates
      // const STAKING_ID: [u8; 8] = *b"staking ";
      // T::Currency::remove_lock(
      //   STAKING_ID,
      //   &who,
      // );

      Ok(())
    }


    pub fn invest(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      let who = ensure_signed(origin)?;
      ensure!(<GovernanceEntities<T>>::exists(id), "GE does not exist");
      // TODO: invest, check if enough balance

      // check if overflow
      let invested_amount = Self::invested_amount((id, who.clone()));
      let total_invested_amount = Self::total_invested_amount(id);
      let new_invested_amount = invested_amount.checked_add(&amount).ok_or("Overflow stake amount")?;
      let new_total_invested_amount = total_invested_amount.checked_add(&amount).ok_or("Overflow total stake amount")?;

      <InvestedAmount<T>>::insert((id, who.clone()), new_invested_amount);
      <TotalInvestedAmount<T>>::insert(id, new_total_invested_amount);

      <InvestedAmount<T>>::insert((id, who.clone()), amount);
      Self::deposit_event(RawEvent::Invested(who, id, amount));
      
      Ok(())
    }

    pub fn update_rules(origin) -> Result {
      Ok(())
    }

  }
}

decl_event!(
  pub enum Event<T> 
  where 
    AccountId = <T as system::Trait>::AccountId,
    Balance = BalanceOf<T>,
    <T as Trait>::GeId,
    ContentHash = <T as Trait>::ContentHash,
  {
    Created(AccountId, GeId, Balance, ContentHash),
    Staked(AccountId, GeId, Balance),
    Invested(AccountId,GeId, Balance),
  }
);

impl<T: Trait> Module<T> {
  pub fn is_member_of_ge(ge_id: T::GeId, who: T::AccountId) -> bool {
    let invested = Self::invested_amount((ge_id, who.clone()));
    let staked = Self::staked_amount((ge_id, who.clone()));
    (invested != <BalanceOf<T>>::from(0)) || (staked != <BalanceOf<T>>::from(0))
  }
}
