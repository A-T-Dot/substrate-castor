use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, 
dispatch::Result, Parameter, ensure, traits::{Currency, ReservableCurrency,}};
use sr_primitives::traits::{ Member, SimpleArithmetic, Bounded, CheckedAdd, CheckedMul};
use system::ensure_signed;
use codec::{Encode, Decode};
// use crate::ge;
use crate::node;
/// The module's configuration trait.
pub trait Trait: system::Trait + node::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
  // type InteractType: Parameter + Member + Default + Copy;
  // type InteractId: Parameter + Member + Default + Copy;
  type LikeId: Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  type AdmireId: Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  type GrantId: Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  type ReportId: Parameter + Member + Default + Bounded + SimpleArithmetic + Copy;
  /// Currency type for this module.
	type Currency: ReservableCurrency<Self::AccountId>;
	/// Action point type for this module
	type ActivityCurrency: Currency<Self::AccountId>;
	/// Reputation point type for this module
	type ReputationCurrency: Currency<Self::AccountId>;
}
// Balance zone
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
// Action zone
pub type ActionPointOf<T> = <<T as Trait>::ActivityCurrency as Currency<<T as system::Trait>::AccountId>>::Balance;
// Reputation zone
pub type ReputationOf<T> = <<T as Trait>::ReputationCurrency as Currency<<T as system::Trait>::AccountId>>::Balance;

// #[cfg_attr(feature ="std", derive(Debug))]
// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// pub struct Interact<ContentHash, InteractType, InteractId> {
//   pub interact_type: InteractType, // like, admire, grant, report
//   pub interact_id: InteractId,
//   pub target: ContentHash,
// }

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Like<AccountId, ContentHash> {
  pub from: AccountId,
  pub to: ContentHash,
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Admire<AccountId, ContentHash> {
  pub from: AccountId,
  pub to: ContentHash,
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Grant<AccountId, ContentHash, Balance> {
  pub from: AccountId,
  pub to: ContentHash,
  pub amount: Balance,
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Report<ContentHash, AccountId> {
  pub from: AccountId,
  pub target: ContentHash,
  pub reason: ContentHash,
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as Interaction {
    pub AllLikes get(get_like): map T::LikeId => Option<Like<T::AccountId, <T as node::Trait>::ContentHash>>;
    pub AllLikesCount get(all_likes_count): T::LikeId;

    pub AllAdmires get(get_admire): map T::AdmireId => Option<Admire<T::AccountId, <T as node::Trait>::ContentHash>>;
    pub AllAdmiresCount get(all_admires_count): T::AdmireId;

    pub AllGrants get(get_grant): map T::GrantId => Option<Grant<T::AccountId, <T as node::Trait>::ContentHash, BalanceOf<T>>>;
    pub AllGrantsCount get(all_grants_count): T::GrantId;

    pub AllReports get(get_report): map T::ReportId => Option<Report<<T as node::Trait>::ContentHash, T::AccountId>>;
    pub AllReportsCount get(all_reports_count): T::ReportId;
    // how many people like this node
    pub NodeLikedCount get(node_liked_count): map <T as node::Trait>::ContentHash => u64;
    // how many people admire this node
    pub NodeAdmiredCount get(node_admired_count): map <T as node::Trait>::ContentHash => u64;
    // how much money this node received
    pub NodeGrantedBalance get(node_granted_balance): map <T as node::Trait>::ContentHash => BalanceOf<T>;
    // does this node be reported
    pub NodeReportedBy get(node_reported_by): map <T as node::Trait>::ContentHash => T::AccountId;

    pub LikedNode get(liked_node): map (T::AccountId, <T as node::Trait>::ContentHash) => T::LikeId;
    pub AdmiredNode get(admired_node): map (T::AccountId, <T as node::Trait>::ContentHash) => T::AdmireId;
    pub GrantedNode get(granted_node): map (T::AccountId, <T as node::Trait>::ContentHash) => T::GrantId;
    pub ReportedNode get(reported_node): map (T::AccountId, <T as node::Trait>::ContentHash) => T::ReportId;
  }
} 

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;

    pub fn like(origin, to: <T as node::Trait>::ContentHash) -> Result {
      let sender = ensure_signed(origin)?;
      <node::Module<T>>::owner_of(to).ok_or("Content Node does not exist")?;
      // ensure!(<LikedNode<T>>::exists((sender.clone(), to)), "Already liked this Node.");
      let like_id = Self::liked_node((sender.clone(), to));
      // let zero = T::LikeId::from(0);
      if like_id != T::LikeId::from(0) {
        return Err("Already liked this Node.")
      }
      Self::do_like(sender.clone(), to)
    }

    pub fn admire(origin, to: <T as node::Trait>::ContentHash) -> Result {
      let sender = ensure_signed(origin)?;
      let node_owner = match <node::Module<T>>::owner_of(to) {
        Some(owner) => owner,
        None => return Err("Content Node does not exist"),
      };
      ensure!(sender != node_owner, "Can not admire yourself.");

      let admire_id = Self::admired_node((sender.clone(), to));
      if admire_id != T::AdmireId::from(0) {
        return Err("Already admired this Node.")
      }
      Self::do_admire(sender.clone(), node_owner, to)
    }

    pub fn grant(origin, to: <T as node::Trait>::ContentHash, amount: BalanceOf<T>) -> Result {
      let sender = ensure_signed(origin)?;
      let node_owner = match <node::Module<T>>::owner_of(to) {
        Some(owner) => owner,
        None => return Err("Content Node does not exist"),
      };
      ensure!(sender != node_owner, "Can not grant yourself.");
      let grant_id = Self::granted_node((sender.clone(), to));
      if grant_id != T::GrantId::from(0) {
        return Err("Already granted this Node.")
      }
      ensure!(amount > BalanceOf::<T>::from(0), "grant should more than 0.");
      Self::do_grant(sender.clone(), node_owner, to, amount)
    }

    pub fn report(origin, target: <T as node::Trait>::ContentHash, reason: <T as node::Trait>::ContentHash) -> Result {
      let sender = ensure_signed(origin)?;
      // <node::Module<T>>::owner_of(target).ok_or("Content Node does not exist")?;
      let node_owner = match <node::Module<T>>::owner_of(target) {
        Some(owner) => owner,
        None => return Err("Content Node does not exist"),
      };
      ensure!(sender != node_owner, "Can not report yourself.");
      let report_id = Self::reported_node((sender.clone(), target));
      if report_id != T::ReportId::from(0) {
        return Err("Already reported this Node.")
      }

      Self::do_report(sender.clone(), target, reason)
    }
  }
}

decl_event!(
  pub enum Event<T> 
  where 
    AccountId = <T as system::Trait>::AccountId,
    ContentHash = <T as node::Trait>::ContentHash,
    Amount = BalanceOf<T>,
    LikeId = <T as Trait>::LikeId,
    AdmireId = <T as Trait>::AdmireId,
    GrantId = <T as Trait>::GrantId,
    ReportId = <T as Trait>::ReportId,
    ReasonHash = <T as node::Trait>::ContentHash,
  {
    Liked(ContentHash, AccountId, LikeId, u64),
    /// (ContentHash, sender, receiver, AdmireId, admired_count)
    Admired(ContentHash, AccountId, AccountId, AdmireId, u64),
    /// (ContentHash, sender, receiver, GrantedAmount, GrantId, NodeGrantedBalance)
    Granted(ContentHash, AccountId, AccountId, Amount, GrantId, Amount),
    Reported(ContentHash, AccountId, ReasonHash, ReportId),
  }
);

impl<T: Trait> Module<T> {
  pub fn do_like(sender: T::AccountId, to: <T as node::Trait>::ContentHash) -> Result {
    let new_like = Like {
        from: sender.clone(),
        to: to,
      };
      // get new like id
      let one = T::LikeId::from(1);
      let all_likes_count = Self::all_likes_count();
      let new_like_id = all_likes_count.checked_add(&one)
          .ok_or("Exceed total max likes count")?;
      let node_liked_count = Self::node_liked_count(to);
      let new_node_liked_count = node_liked_count.checked_add(1)
          .ok_or("Exceed node max likes count")?;
      
      <AllLikes<T>>::insert(new_like_id, new_like);
      <AllLikesCount<T>>::put(new_like_id);
      <NodeLikedCount<T>>::insert(to, new_node_liked_count);
      <LikedNode<T>>::insert((sender.clone(), to), new_like_id);

      Self::deposit_event(RawEvent::Liked(to, sender, new_like_id, node_liked_count));

      Ok(())
  }
  pub fn do_admire(sender: T::AccountId, node_owner: T::AccountId, 
  to: <T as node::Trait>::ContentHash) -> Result {
    // new admire
    let new_admire = Admire {
      from: sender.clone(),
      to: to,
    };

    let one = T::AdmireId::from(1);
    let all_admires_count = Self::all_admires_count();
    let new_admire_id = all_admires_count.checked_add(&one)
        .ok_or("Exceed total max admires count")?;
    let node_admired_count = Self::node_admired_count(to);
    let new_node_admired_count = node_admired_count.checked_add(1)
        .ok_or("Exceed node max admires count")?;
    
    <AllAdmires<T>>::insert(new_admire_id, new_admire);
    <AllAdmiresCount<T>>::put(new_admire_id);
    <NodeAdmiredCount<T>>::insert(to, new_node_admired_count);
    <AdmiredNode<T>>::insert((sender.clone(), to), new_admire_id);
    // TODO: consume some CAP, content owner earns some CRP
    

    Self::deposit_event(RawEvent::Admired(to, sender, node_owner, new_admire_id, node_admired_count));

    Ok(())
  }

  pub fn do_grant(sender: T::AccountId, node_owner: T::AccountId, 
  to: <T as node::Trait>::ContentHash, amount: BalanceOf<T>) -> Result {
    // new grant
    let new_grant = Grant {
      from: sender.clone(),
      to: to,
      amount: amount,
    };
    let one = T::GrantId::from(1);
    let all_grants_count = Self::all_grants_count();
    let new_grant_id = all_grants_count.checked_add(&one)
      .ok_or("Exceed total max grants count")?;
    let node_granted_balance = Self::node_granted_balance(to);
    let new_node_granted_balance = node_granted_balance.checked_add(&amount)
      .ok_or("Exceed node max grants count")?;

    <AllGrants<T>>::insert(new_grant_id, new_grant);
    <AllGrantsCount<T>>::put(new_grant_id);
    <NodeGrantedBalance<T>>::insert(to, new_node_granted_balance);
    <GrantedNode<T>>::insert((sender.clone(), to), new_grant_id);
    // TODO: transfer CCT to node owner
    let free_balance = T::Currency::free_balance(&sender);
    ensure!(free_balance >= amount, "Currency not enough for Grant.");
    T::Currency::transfer(&sender, &node_owner, amount)?;

    Self::deposit_event(RawEvent::Granted(to, sender, node_owner, amount, new_grant_id, new_node_granted_balance));
    Ok(())
  }

  pub fn do_report(sender: T::AccountId, target: <T as node::Trait>::ContentHash, reason: <T as node::Trait>::ContentHash) -> Result {
    // new report
    let new_report = Report {
      from: sender.clone(),
      target,
      reason,
    };

    let one = T::ReportId::from(1);
    let all_reports_count = Self::all_reports_count();
    let new_report_id = all_reports_count.checked_add(&one)
      .ok_or("Exceed total max reports count")?;

    // TODO: lock 10 CAP for the reporter
    
    <AllReports<T>>::insert(new_report_id, new_report);
    <AllReportsCount<T>>::put(new_report_id);
    <NodeReportedBy<T>>::insert(target, sender.clone());
    <ReportedNode<T>>::insert((sender.clone(), target), new_report_id);

    Self::deposit_event(RawEvent::Reported(target, sender, reason, new_report_id));

    Ok(())
  }
}