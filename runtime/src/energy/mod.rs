//! # Energy Module
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as EnergyStorage {
		// TODO
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// fn deposit_event() = default;
	}
}

decl_event!(
	pub enum Event<T> where
    AccountId = <T as system::Trait>::AccountId
  {
		SomethingStored(u32, AccountId),
	}
);
