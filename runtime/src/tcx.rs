use sr_primitives::traits::{
  Member, SimpleArithmetic, Bounded, CheckedAdd, CheckedMul, Convert, SaturatedConversion,
};
use support::{
  decl_module, decl_storage, decl_event, ensure,
  StorageValue, StorageMap, Parameter,
  traits::{
    Currency, LockableCurrency, WithdrawReasons, LockIdentifier, ReservableCurrency,
  },
  dispatch::Result,
};
use system::ensure_signed;
use codec::{Encode, Decode};
use rstd::{cmp, result, convert::{TryInto}};
use crate::ge;


/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait + ge::Trait {
  type TcxId:  Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  type TcxType: Parameter + Member + Default + Copy;
  type ActionId: Parameter + Member + Default + Copy;
  type ListingId:  Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  type ChallengeId: Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;

  /// Currency of this module
  type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber> + ReservableCurrency<Self::AccountId>;

  /// Convert Balance
  type ConvertBalance: Convert<ge::BalanceOf<Self>, BalanceOf<Self>>;

  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// Balance zone
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[cfg_attr(feature ="std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct Tcx<TcxType, ContentHash> {
  pub tcx_type: TcxType,
  pub content_hash: ContentHash,
}


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Listing<ListingId, ContentHash, Balance, Moment, 
ChallengeId, AccountId> {
  id: ListingId,
  node_id: ContentHash,
  amount: Balance,
  quota: Balance,
  application_expiry: Moment,
  whitelisted: bool,
  challenge_id: ChallengeId,
  owner: AccountId,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Challenge<Balance, Moment, AccountId, TcxId> {
  amount: Balance,
  quota: Balance,
  voting_ends: Moment,
  resolved: bool,
  reward_pool: Balance,
  total_tokens: Balance,
  owner: AccountId,
  tcx_id: TcxId, // to check if voter is a member of ge
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Vote<Balance> {
  value: bool,
  amount: Balance,
  quota: Balance,
  claimed: bool,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Poll<Balance> {
  votes_for: Balance,
  quota_for: Balance,
  votes_against: Balance,
  quota_against: Balance,
  passed: bool,
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as Tcx {
    AllTcxsArray get(tcx): map T::TcxId => Option<Tcx<T::TcxType, <T as ge::Trait>::ContentHash>>;
    AllTcxsCount get(all_tcxs_count): T::TcxId;

    TcxOwner get(owner_of): map T::TcxId => Option<T::GeId>;

    OwnedTcxsArray get(tcx_of_owner_by_index): map (T::GeId, T::TcxId) => T::TcxId;
    OwnedTcxsCount get(owned_tcxs_count): map T::GeId => T::TcxId;

    // actual tcx
    TcxListings get(listing_of_tcr_by_node_id): map (T::TcxId, <T as ge::Trait>::ContentHash) => Listing<T::ListingId, <T as ge::Trait>::ContentHash, BalanceOf<T>, T::Moment, T::ChallengeId, T::AccountId>;
    TcxListingsCount get(listing_count_of_tcx): map T::TcxId => T::ListingId;
    TcxListingsIndexHash get(node_id_of_listing): map (T::TcxId, T::ListingId) => <T as ge::Trait>::ContentHash;

    Challenges get(challenges): map T::ChallengeId => Challenge<BalanceOf<T>, T::Moment, T::AccountId, T::TcxId>;
    Votes get(votes): map (T::ChallengeId, T::AccountId) => Vote<BalanceOf<T>>;
    Polls get(polls): map T::ChallengeId => Poll<BalanceOf<T>>;

    ChallengeNonce get(challenge_nonce): T::ChallengeId;
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;


    // TODO: check if node exists
    pub fn propose(origin, tcx_id: T::TcxId, node_id: <T as ge::Trait>::ContentHash, amount: BalanceOf<T>, 
    action_id: T::ActionId) -> Result {
      
      let who = ensure_signed(origin)?;

      // only member of ge can propose
      let ge_id = Self::owner_of(tcx_id).ok_or("TCX does not exist / TCX owner does not exist")?;
      let governance_entity = <ge::Module<T>>::governance_entity(ge_id).ok_or("GE does not exist")?;
      ensure!(<ge::Module<T>>::is_member_of_ge(ge_id, who.clone()), "only member of ge can propose");
      
      // TODO: deduction balace for application
      // <token::Module<T>>::lock(sender.clone(), deposit, hashed.clone())?;
      // ensure!(<T as self::Trait>::Currency::can_reserve(&who, amount), "not enough balances to propose");
      <T as self::Trait>::Currency::reserve(&who, amount)
        .map_err(|_| "proposer's balance too low")?;
        
      // more than min deposit
      let min_deposit = T::ConvertBalance::convert(governance_entity.min_deposit);
      // TODO: quota instead of amount
      ensure!(amount >= min_deposit, "deposit should be more than min_deposit");

      let now = <timestamp::Module<T>>::get();
      let apply_stage_len = governance_entity.apply_stage_len;
      let app_exp = now.checked_add(&apply_stage_len).ok_or("Overflow when setting application expiry.")?;

      let listing_id = Self::listing_count_of_tcx(tcx_id);
      let new_listing_count = listing_id.checked_add(&T::ListingId::from(1)).ok_or("Exceed max listing count")?;

      // TODO: check action_id

      ensure!(!<TcxListings<T>>::exists((tcx_id,node_id)), "Listing already exists");
      
      // calculate propose quota
      let quota = match Self::calculate_quota(who.clone(), ge_id, amount) {
        Ok(quota) => quota,
        Err(e) => return Err(e),
      };

      // create a new listing instance
      let new_listing = Listing {
        id: new_listing_count,
        node_id: node_id,
        amount: amount,
        quota: quota,
        whitelisted: false,
        challenge_id: T::ChallengeId::from(0),
        application_expiry: app_exp,
        owner: who.clone(),
      };

      <TcxListings<T>>::insert((tcx_id, node_id), new_listing);
      <TcxListingsCount<T>>::insert(tcx_id, new_listing_count);
      <TcxListingsIndexHash<T>>::insert((tcx_id, new_listing_count), node_id);

      Self::deposit_event(RawEvent::Proposed(who, tcx_id, node_id, app_exp));

      Ok(())
    }

    // TODO: node_id or listing_id; prevent multiple challenge
    pub fn challenge(origin, tcx_id: T::TcxId, node_id: <T as ge::Trait>::ContentHash, amount: BalanceOf<T>) -> Result {
      let who = ensure_signed(origin)?;

      let ge_id = Self::owner_of(tcx_id).ok_or("TCX does not exist / TCX owner does not exist")?;
      let governance_entity = <ge::Module<T>>::governance_entity(ge_id).ok_or("GE does not exist")?;
      
      ensure!(<ge::Module<T>>::is_member_of_ge(ge_id, who.clone()), "only member of ge can challenge");

      ensure!(<TcxListings<T>>::exists((tcx_id,node_id)), "Listing not found");
      
      let listing = Self::listing_of_tcr_by_node_id((tcx_id,node_id));
      
      let quota = match Self::calculate_quota(who.clone(), ge_id, amount) {
        Ok(quota) => quota,
        Err(e) => return Err(e),
      };
      // check if challengable
      ensure!(listing.challenge_id == T::ChallengeId::from(0), "Listing is already challenged.");
      ensure!(listing.owner != who.clone(), "You cannot challenge your own listing.");
      ensure!(quota >= listing.quota, "Quota not enough to challenge");

      let now = <timestamp::Module<T>>::get();
      // check if passed apply stage
      ensure!(listing.application_expiry > now, "Apply stage length has passed.");
      
      let commit_stage_len = governance_entity.commit_stage_len;
      let voting_exp = now.checked_add(&commit_stage_len).ok_or("Overflow when setting voting expiry.")?;

      let new_challenge = Challenge {
        amount,
        quota: quota,
        voting_ends: voting_exp,
        resolved: false,
        reward_pool: BalanceOf::<T>::from(0),
        total_tokens: BalanceOf::<T>::from(0),
        owner: who.clone(),
        tcx_id
      };

      let new_poll = Poll {
        votes_for: listing.amount,
        quota_for: listing.quota,
        votes_against: amount,
        quota_against: quota,
        passed: false,
      };

      // check enough balance, lock it
      // TODO: <token::Module<T>>::lock(sender.clone(), deposit, listing_hash)?;
      // ensure!(<T as self::Trait>::Currency::can_reserve(&who, amount), "not enough balances to challenge");
      <T as self::Trait>::Currency::reserve(&who, amount)
        .map_err(|_| "challenger's balance too low")?;

      let challenge_nonce = <ChallengeNonce<T>>::get();
      let new_challenge_nonce = challenge_nonce.checked_add(&T::ChallengeId::from(1)).ok_or("Exceed maximum challenge count")?;
      
      // add a new challenge and the corresponding poll
      <Challenges<T>>::insert(new_challenge_nonce, new_challenge);
      <Polls<T>>::insert(new_challenge_nonce, new_poll);

      // update listing with challenge id
      <TcxListings<T>>::mutate((tcx_id, node_id), |listing| {
        listing.challenge_id = new_challenge_nonce;
      });

      <ChallengeNonce<T>>::put(new_challenge_nonce);

      Self::deposit_event(RawEvent::Challenged(who, new_challenge_nonce, tcx_id, node_id, voting_exp));

      Ok(())
    }

      // TODO: prevent double votes, cannot vote on your own challenge?
    pub fn vote(origin, challenge_id: T::ChallengeId, amount: BalanceOf<T>, value: bool) -> Result {
      let who = ensure_signed(origin)?;

      // check if listing is challenged
      ensure!(<Challenges<T>>::exists(challenge_id), "Challenge does not exist.");
      let challenge = Self::challenges(challenge_id);
      ensure!(challenge.resolved == false, "Challenge is already resolved.");

      // check commit stage length not passed
      let now = <timestamp::Module<T>>::get();
      ensure!(challenge.voting_ends > now, "Commit stage length has passed.");

      // deduct the deposit for vote
      // TODO: <token::Module<T>>::lock(sender.clone(), deposit, challenge.listing_hash)?;
      let mut tmp: [u8; 8] = [0; 8];
      let mutarr = &mut tmp[..];
      let mut moment_num: u64 = challenge.tcx_id.saturated_into::<u64>();
      for i in (0..8).rev() { 
        mutarr[i] = (moment_num % 0xff).try_into().unwrap();
        moment_num >>= 8;
      }
      let staking_id = LockIdentifier::from(tmp);

      <T as self::Trait>::Currency::set_lock(
        staking_id,
        &who,
        amount,
        T::BlockNumber::max_value(),
        WithdrawReasons::all(),
      );

      // calculate propose quota
      let ge_id = Self::owner_of(challenge.tcx_id).ok_or("Cannot find ge of tcx")?;
      ensure!(<ge::Module<T>>::is_member_of_ge(ge_id, who.clone()), "only member of ge can vote");

      let quota = match Self::calculate_quota(who.clone(), ge_id, amount) {
        Ok(quota) => quota,
        Err(e) => return Err(e),
      };

      let mut poll_instance = Self::polls(challenge_id);
      // based on vote value, increase the count of votes (for or against)
      match value {
        true => {
          poll_instance.votes_for += amount;
          poll_instance.quota_for += quota;
        },
        false => {
          poll_instance.votes_against += amount;
          poll_instance.quota_against += quota;
        },
      }

      // create a new vote instance with the input params
      let vote_instance = Vote {
        value,
        amount,
        quota,
        claimed: false,
      };

      // mutate polls collection to update the poll instance
      <Polls<T>>::mutate(challenge_id, |poll| *poll = poll_instance);

      <Votes<T>>::insert((challenge_id, who.clone()), vote_instance);

      Self::deposit_event(RawEvent::Voted(who, challenge_id, value));
      Ok(())
    }

    pub fn resolve(origin, tcx_id: T::TcxId, node_id: <T as ge::Trait>::ContentHash) -> Result {
      let who = ensure_signed(origin)?;
      ensure!(<TcxListings<T>>::exists((tcx_id,node_id)), "Listing not found");

      let listing = Self::listing_of_tcr_by_node_id((tcx_id,node_id));

      let now = <timestamp::Module<T>>::get();

      // check if listing was challenged
      if listing.challenge_id == T::ChallengeId::from(0) {
        // no challenge
        // check if apply stage length has passed
        ensure!(listing.application_expiry < now, "Apply stage length has not passed.");

        // update listing status
        <TcxListings<T>>::mutate((tcx_id, node_id), |listing| {
          listing.whitelisted = true;
        });

        Self::deposit_event(RawEvent::Accepted(tcx_id, node_id));
        return Ok(());
      } 

      // listing was challenged
      let	challenge = Self::challenges(listing.challenge_id);
      let	poll = Self::polls(listing.challenge_id);
      
      // check commit stage length has passed
      ensure!(challenge.voting_ends < now, "Commit stage length has not passed.");
    
      let mut whitelisted = false;

      // update the poll instance
      <Polls<T>>::mutate(listing.challenge_id, |poll| {
        if poll.quota_for >= poll.quota_against {
            poll.passed = true;
            whitelisted = true;
        } else {
            poll.passed = false;
        }
      });

      // update listing status
      <TcxListings<T>>::mutate((tcx_id, node_id), |listing| {
        listing.whitelisted = whitelisted;
        listing.challenge_id = T::ChallengeId::from(0);
      });

      // update challenge
      <Challenges<T>>::mutate(listing.challenge_id, |challenge| {
        challenge.resolved = true;
        if whitelisted == true {
          challenge.total_tokens = poll.votes_for;
          challenge.reward_pool = challenge.amount + poll.votes_against;
        } else {
          challenge.total_tokens = poll.votes_against;
          challenge.reward_pool = listing.amount + poll.votes_for;
        }
      });

      // raise appropriate event as per whitelisting status
      if whitelisted == true {
        Self::deposit_event(RawEvent::Accepted(tcx_id, node_id));
      } else {
        // if rejected, give challenge deposit back to the challenger
        // TODO: <token::Module<T>>::unlock(challenge.owner, challenge.deposit, listing_hash)?;
				// <T as self::Trait>::Currency::remove_lock(STAKING_ID, &who);
        let amount = <T as self::Trait>::Currency::reserved_balance(&who);
        <T as self::Trait>::Currency::unreserve(&who, amount);
        Self::deposit_event(RawEvent::Rejected(tcx_id, node_id));
      }

      Self::deposit_event(RawEvent::Resolved(listing.challenge_id));
      Ok(())
    }

    pub fn claim(origin, challenge_id: T::ChallengeId) -> Result {
      let who = ensure_signed(origin)?;

      ensure!(<Challenges<T>>::exists(challenge_id), "Challenge not found.");
      let challenge = Self::challenges(challenge_id);
      ensure!(challenge.resolved == true, "Challenge is not resolved.");

      // reward depends on poll passed status and vote value
      let poll = Self::polls(challenge_id);
      let vote = Self::votes((challenge_id, who.clone()));

      // ensure vote reward is not already claimed
      ensure!(vote.claimed == false, "Vote reward has already been claimed.");

      // if winning party, calculate reward and transfer
      if poll.passed == vote.value {
        // TODO: claim reward
        // let reward_ratio = challenge.reward_pool.checked_div(&challenge.total_tokens).ok_or("overflow in calculating reward")?;
        // let reward = reward_ratio.checked_mul(&vote.deposit).ok_or("overflow in calculating reward")?;
        // let total = reward.checked_add(&vote.deposit).ok_or("overflow in calculating reward")?;
        // <token::Module<T>>::unlock(sender.clone(), total, challenge.listing_hash)?;

        let mut tmp: [u8; 8] = [0; 8];
        let mutarr = &mut tmp[..];
        let mut moment_num: u64 = challenge.tcx_id.saturated_into::<u64>();
        for i in (0..8).rev() { 
          mutarr[i] = (moment_num % 0xff).try_into().unwrap();
          moment_num >>= 8;
        }
        let staking_id = LockIdentifier::from(tmp);
        <T as self::Trait>::Currency::remove_lock(staking_id, &who);

        Self::deposit_event(RawEvent::Claimed(who.clone(), challenge_id));
      }

      // update vote reward claimed status
      <Votes<T>>::mutate((challenge_id, who), |vote| vote.claimed = true);

      Ok(())
    }

    // create tcr: for testing purposes only
    pub fn propose_tcx_creation(origin, ge_id: T::GeId, tcx_type: T::TcxType, content_hash: <T as ge::Trait>::ContentHash) -> Result {
      // TODO: check if ge agrees
      let governance_entity = <ge::Module<T>>::governance_entity(ge_id).ok_or("GE does not exist")?;

      let tcx_id = Self::create(ge_id, tcx_type, content_hash)?;

      Ok(())
    }
  }
}

decl_event!(
  pub enum Event<T> 
  where 
    AccountId = <T as system::Trait>::AccountId,
    // Balance = BalanceOf<T>,
    ContentHash = <T as ge::Trait>::ContentHash,
    TcxId = <T as Trait>::TcxId,
    TcxType = <T as Trait>::TcxType,
    ChallengeId = <T as Trait>::ChallengeId,
    GeId = <T as ge::Trait>::GeId,
    // Quota = BalanceOf<T>,
    Moment = <T as timestamp::Trait>::Moment
  {
    /// (AccountId, TcxId, ContentHash, Balance, Quota, ActionId)
    Proposed(AccountId, TcxId, ContentHash, Moment),
    /// (AccountId, TcxId, ContentHash, Balance, Quota)
    Challenged(AccountId, ChallengeId, TcxId, ContentHash, Moment),
    /// (AccountId, ChallengeId, Balance, Quota, passed)
    Voted(AccountId, ChallengeId, bool),
    Resolved(ChallengeId),
    Accepted(TcxId, ContentHash),
    Rejected(TcxId, ContentHash),
    Claimed(AccountId, ChallengeId),
    Created(GeId, TcxId, TcxType, ContentHash),
  }
);

impl<T: Trait> Module<T> {
  pub fn create(ge_id: T::GeId, tcx_type: T::TcxType, content_hash: <T as ge::Trait>::ContentHash) -> rstd::result::Result<T::TcxId, &'static str> {
    let one = T::TcxId::from(1 as u32);

    // check global tcx count
    let all_tcxs_count = <AllTcxsCount<T>>::get();
    let new_all_tcxs_count = all_tcxs_count.checked_add(&one).ok_or("Exceed maximum tcx count")?;

    // check owner tcx count
    let owned_tcxs_count = <OwnedTcxsCount<T>>::get(ge_id);
    let new_owned_tcxs_count = owned_tcxs_count.checked_add(&one).ok_or("Exceed maximum tcx count for ge")?;

    let tcx  =  Tcx {
      tcx_type: tcx_type,
      content_hash: content_hash,
    };
    <AllTcxsArray<T>>::insert(new_all_tcxs_count, tcx);
    <AllTcxsCount<T>>::put(new_all_tcxs_count);

    <TcxOwner<T>>::insert(new_all_tcxs_count, ge_id);

    <OwnedTcxsArray<T>>::insert((ge_id, new_owned_tcxs_count), new_all_tcxs_count);
    <OwnedTcxsCount<T>>::insert(ge_id, new_owned_tcxs_count);
    
    Self::deposit_event(RawEvent::Created(ge_id, new_all_tcxs_count, tcx_type, content_hash));
    // return new tcx_id
    Ok(new_all_tcxs_count)
  }

  pub fn calculate_quota(who: T::AccountId, ge_id: T::GeId, amount: BalanceOf<T>) -> result::Result<BalanceOf<T>, &'static str> {
    // calculate propose quota
    let invested = T::ConvertBalance::convert(<ge::Module<T>>::invested_amount((ge_id, who.clone())));
    let min = cmp::min(amount, invested);
    let factor = BalanceOf::<T>::from(20);
    let quota = min.checked_mul(&factor).ok_or("Overflow calculating A shares.")?;
    let staked = T::ConvertBalance::convert(<ge::Module<T>>::staked_amount((ge_id, who.clone())));
    // let max = cmp::max(BalanceOf::<T>::from(0), amount-invested);
    let max = if amount > invested {
      amount - invested
    } else {
      BalanceOf::<T>::from(0)
    };
    let max = cmp::max(max, staked);
    let quota = quota.checked_add(&max).ok_or("Overflow calculating B shares.")?;
    Ok(quota)
  }
}