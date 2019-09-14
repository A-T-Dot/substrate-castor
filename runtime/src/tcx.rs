use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Tcx {
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn propose(origin, proposal: u32) -> Result {
			// TODO: You only need this if you want to check it was signed.
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
