use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result, Parameter, ensure};
use sr_primitives::traits::{ Member };
use system::ensure_signed;
use codec::{Encode, Decode};

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
  type ContentHash: Parameter + Member + Default + Copy;
}

#[cfg_attr(feature ="std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct Node<ContentHash> {
		pub id: ContentHash,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Node {
    Nodes get(node): map T::ContentHash => Option<Node<T::ContentHash>>;
    NodeOwner get(owner_of): map T::ContentHash => Option<T::AccountId>;

    AllNodesArray get(node_by_index): map u64 => T::ContentHash;
    AllNodesCount get(all_nodes_count): u64;
    AllNodesIndex: map T::ContentHash => u64;

    OwnedNodesArray get(node_of_owner_by_index): map (T::AccountId, u64) => T::ContentHash;
    OwnedNodesCount get(owned_nodes_count): map T::AccountId => u64;
		OwnedNodesIndex: map T::ContentHash => u64;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn create(origin, content_hash: T::ContentHash) -> Result {
			let sender = ensure_signed(origin)?;

      ensure!(!<NodeOwner<T>>::exists(content_hash), "Content Node already exists");

			let new_node = Node {
					id: content_hash,
			};

      let owned_nodes_count = Self::owned_nodes_count(sender.clone());
			let new_owned_nodes_count = owned_nodes_count.checked_add(1)
					.ok_or("Exceed max node count per account")?;

      let all_nodes_count = Self::all_nodes_count();
			let new_all_nodes_count = all_nodes_count.checked_add(1)
					.ok_or("Exceed total max node count")?;

      <Nodes<T>>::insert(content_hash, new_node);
      <NodeOwner<T>>::insert(content_hash, sender.clone());

      <AllNodesArray<T>>::insert(new_all_nodes_count, content_hash);
      AllNodesCount::put(new_all_nodes_count);
      <AllNodesIndex<T>>::insert(content_hash, new_all_nodes_count);

      <OwnedNodesArray<T>>::insert((sender.clone(), new_owned_nodes_count), content_hash);
      <OwnedNodesCount<T>>::insert(sender.clone(), new_owned_nodes_count);
      <OwnedNodesIndex<T>>::insert(content_hash, new_owned_nodes_count);

      Self::deposit_event(RawEvent::Created(sender, content_hash));

			Ok(())
		}

		pub fn transfer(origin, to: T::AccountId, content_hash: T::ContentHash) -> Result {
			let sender = ensure_signed(origin)?;
			
			let owner = Self::owner_of(content_hash).ok_or("No node owner")?;

      ensure!(owner == sender.clone(), "Sender does not own the node");
      
			let owned_nodes_count_from = Self::owned_nodes_count(sender.clone());
      let owned_nodes_count_to = Self::owned_nodes_count(to.clone());

			let new_owned_nodes_count_to = owned_nodes_count_to.checked_add(1)
			    .ok_or("Transfer causes overflow for node receiver")?;

			let new_owned_nodes_count_from = owned_nodes_count_from.checked_sub(1)
					.ok_or("Transfer causes underflow for node sender")?;

			let owned_node_index = <OwnedNodesIndex<T>>::get(content_hash);
			if owned_node_index != new_owned_nodes_count_from {
				let last_owned_node_id = <OwnedNodesArray<T>>::get((sender.clone(), new_owned_nodes_count_from));
				<OwnedNodesArray<T>>::insert((sender.clone(), owned_node_index), last_owned_node_id);
				<OwnedNodesIndex<T>>::insert(last_owned_node_id, owned_node_index);
			}

      <NodeOwner<T>>::insert(content_hash, to.clone());
      <OwnedNodesIndex<T>>::insert(content_hash, owned_nodes_count_to);

      <OwnedNodesArray<T>>::remove((sender.clone(), new_owned_nodes_count_from));
      <OwnedNodesArray<T>>::insert((to.clone(), owned_nodes_count_to), content_hash);

      <OwnedNodesCount<T>>::insert(sender.clone(), new_owned_nodes_count_from);
      <OwnedNodesCount<T>>::insert(to.clone(), new_owned_nodes_count_to);
			
      Self::deposit_event(RawEvent::Transferred(sender, to, content_hash));

			Ok(())
		}

	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
		ContentHash = <T as Trait>::ContentHash
	{
		Created(AccountId, ContentHash),
		Transferred(AccountId, AccountId, ContentHash),
	}
);
