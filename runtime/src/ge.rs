use codec::{Encode, Decode};
use rstd::{
  convert::{TryInto}
};
use support::{
  decl_module, decl_storage, decl_event, ensure, dispatch::Result,
  traits::{
    Currency, ReservableCurrency,
    OnUnbalanced, Get, // WithdrawReasons, 
  },
  StorageValue, StorageMap, Parameter,
};
use sr_primitives::traits::{
  Member, SimpleArithmetic, Bounded, CheckedAdd, CheckedSub,
};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait {
  /// GovernanceEntity ID
  type GeId: Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  /// Content hash
  type ContentHash: Parameter + Member + Default + Copy;

	/// Currency type for this module.
	type Currency: ReservableCurrency<Self::AccountId>;

  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// Handler for the unbalanced reduction when slashing a staker.
	type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

	/// Handler for the unbalanced increment when rewarding a staker.
	type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;
  /// Cost of new GE creation.
  type GeCreationFee: Get<BalanceOf<Self>>;
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
    pub fn create(origin, content_hash: T::ContentHash, amount: BalanceOf<T>) -> Result {
      let who = ensure_signed(origin)?;
      Self::do_create(who.clone(), content_hash, amount)
    }

    // stake ge
    pub fn stake(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      let who = ensure_signed(origin)?;
      ensure!(<GovernanceEntities<T>>::exists(id), "GE does not exist");
      
      // actually stake real balance, below simulates
      // const STAKING_ID: [u8; 8] = *b"staking ";
      // T::Currency::set_lock(
      //   STAKING_ID,
      //   &who,
      //   amount,
      //   T::BlockNumber::max_value(),
      //   WithdrawReasons::all(),
      // );
      Self::do_stake(who.clone(), id, amount)
    }

    pub fn withdraw(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      // withdraw balance
      let who = ensure_signed(origin)?;
      ensure!(<GovernanceEntities<T>>::exists(id), "GE does not exist");
      // actually stake real balance, below simulates
      // const STAKING_ID: [u8; 8] = *b"staking ";
      // T::Currency::remove_lock(
      //   STAKING_ID,
      //   &who,
      // );
      Self::do_withdraw(who.clone(), id, amount)
    }


    pub fn invest(origin, id: T::GeId, amount: BalanceOf<T>) -> Result {
      let who = ensure_signed(origin)?;
      ensure!(<GovernanceEntities<T>>::exists(id), "GE does not exist");
      Self::do_invest(who.clone(), id, amount)
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
    Invested(AccountId, GeId, Balance),
    // FIXME: uncomment when front-end is ready
    // (Who, GeId, Amount, Remained)
    // Withdrawed(AccountId, GeId, Balance, Balance),
  }
);

impl<T: Trait> Module<T> {
  pub fn is_member_of_ge(ge_id: T::GeId, who: T::AccountId) -> bool {
    let invested = Self::invested_amount((ge_id, who.clone()));
    let staked = Self::staked_amount((ge_id, who.clone()));
    (invested != <BalanceOf<T>>::from(0)) || (staked != <BalanceOf<T>>::from(0))
  }

  pub fn do_create(who: T::AccountId, content_hash: T::ContentHash, balance: BalanceOf<T>) -> Result {
    let count = Self::governance_entities_count();
      
    // get new ge_id
    let one = T::GeId::from(1 as u32);
    let new_count = count.checked_add(&one).ok_or("exceed maximum amount of ge")?;

    // do something with balance here e.g. lock balance, reduce balance
    // let balance: u128 = 12;
    // let temp: Option<BalanceOf<T>> = balance.try_into().ok();
    // let balance = temp.ok_or("Cannot convert to balance")?;
    // reserve some balance
    ensure!(balance >= T::GeCreationFee::get(), "deposit should more than Ge creation fee.");
    ensure!(T::Currency::can_reserve(&who, balance), "Balance not enough for creating new GE.");
    T::Currency::reserve(&who, balance)?;

    let new_governance_entity = GovernanceEntity::<BalanceOf<T>, T::Moment, T::ContentHash> {
      threshold: 0,
      min_deposit: BalanceOf::<T>::from(3000),
      apply_stage_len: T::Moment::from(60000),
      commit_stage_len: T::Moment::from(60000),
      content_hash: content_hash,
    };

    <InvestedAmount<T>>::insert((new_count, who.clone()), balance);
    <TotalInvestedAmount<T>>::insert(new_count, balance);
    <GovernanceEntities<T>>::insert(new_count, new_governance_entity);
    <GovernanceEntitiesCount<T>>::put(new_count);

    Self::deposit_event(RawEvent::Created(who, new_count, balance, content_hash));

    Ok(())
  }

  pub fn do_stake(who: T::AccountId, id: T::GeId, amount: BalanceOf<T>) -> Result {
    // check if enough balance
    ensure!(T::Currency::can_reserve(&who, amount), "Balance not enough.");
    // check if overflow
    let staked_amount = Self::staked_amount((id, who.clone()));
    let total_staked_amount = Self::total_staked_amount(id);
    let new_staked_amount = staked_amount.checked_add(&amount).ok_or("Overflow stake amount")?;
    let new_total_staked_amount = total_staked_amount.checked_add(&amount).ok_or("Overflow total stake amount")?;
    // actual stake
    T::Currency::reserve(&who, amount)?;

    <StakedAmount<T>>::insert((id, who.clone()), new_staked_amount);
    <TotalStakedAmount<T>>::insert(id, new_total_staked_amount);

    Self::deposit_event(RawEvent::Staked(who, id, amount));

    Ok(())
  }

  pub fn do_withdraw(who: T::AccountId, id: T::GeId, amount: BalanceOf<T>) -> Result {
    // check if reserved balance is enough
    // staked_amount is the balance staked(reserved) to this GeId
    let staked_amount = Self::staked_amount((id, who.clone()));
    ensure!(staked_amount >= amount, "Can not withdraw more than you have staked.");
    // balance_reserved is the total reserved balance of this account,
    // it consists of staked and invested balance not only to this GeId,
    // but to other GeIds as well
    let balance_reserved = T::Currency::reserved_balance(&who);
    ensure!(balance_reserved >= staked_amount, "Total reserved balance should more than staked balance.");
    // deduct staked amount in Ge
    let new_staked_amount = staked_amount.checked_sub(&amount).ok_or("Underflow staked amount.")?;
    let total_staked_amount = Self::total_staked_amount(id);
    let new_total_staked_amount = total_staked_amount.checked_sub(&amount).ok_or("Underflow total staked amount.")?;
    // actual unreserve balance
    T::Currency::unreserve(&who, amount);

    <StakedAmount<T>>::insert((id, who.clone()), new_staked_amount);
    <TotalStakedAmount<T>>::insert(id, new_total_staked_amount);
    // FIXME: uncomment when front-end is ready
    // let remained_amount = staked_amount - amount;
    // Self::deposit_event(RawEvent::Withdrawed(who, id, amount, remained_amount));

    Ok(())
  }

  pub fn do_invest(who: T::AccountId, id: T::GeId, amount: BalanceOf<T>) -> Result {
    // invest, check if enough balance
    ensure!(T::Currency::can_reserve(&who, amount), "Balance not enough.");
    // check if overflow
    let invested_amount = Self::invested_amount((id, who.clone()));
    let total_invested_amount = Self::total_invested_amount(id);
    let new_invested_amount = invested_amount.checked_add(&amount).ok_or("Overflow stake amount")?;
    let new_total_invested_amount = total_invested_amount.checked_add(&amount).ok_or("Overflow total stake amount")?;

    // actual invest
    T::Currency::reserve(&who, amount)?;

    <InvestedAmount<T>>::insert((id, who.clone()), new_invested_amount);
    <TotalInvestedAmount<T>>::insert(id, new_total_invested_amount);

    Self::deposit_event(RawEvent::Invested(who, id, amount));
    
    Ok(())
  }
}
