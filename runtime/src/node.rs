/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result, Parameter};
use sr_primitives::traits::{SimpleArithmetic, One, Member};
use system::ensure_signed;
use codec::{Encode, Decode};

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

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
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		// Something get(something): Option<u32>;
    Nodes get(node): map T::Hash => Option<Node<T::ContentHash>>;
    NodeOwner get(owner_of): map T::ContentHash => Option<T::AccountId>;

    AllNodesArray get(node_by_index): map u64 => T::ContentHash;
    AllNodesCount get(all_nodes_count): u64;
    AllNodesIndex: map T::ContentHash => u64;

    OwnedNodesArray get(node_of_owner_by_index): map (T::AccountId, u64) => T::ContentHash;
    OwnedNodesCount get(owned_nodes_count): map T::AccountId => u64;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

    

		// Just a dummy entry point.
		// function that can be called by the external world as an extrinsics call
		// takes a parameter of the type `AccountId`, stores it and emits an event
		// pub fn do_something(origin, something: u32) -> Result {
		// 	// TODO: You only need this if you want to check it was signed.
		// 	let who = ensure_signed(origin)?;

		// 	// TODO: Code to execute when something calls this.
		// 	// For example: the following line stores the passed in u32 in the storage
		// 	Something::put(something);

		// 	// here we are raising the Something event
		// 	Self::deposit_event(RawEvent::SomethingStored(something, who));
		// 	Ok(())
		// }
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
