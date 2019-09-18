//! # Currency Module
//!
//! The Currency module provides functionality for handling accounts and balances.
//! 
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//! 
//! The Balances module provides functions for:
//!
//! - Getting and setting free balances.
//! - Retrieving total, reserved and unreserved balances.
//! - Repatriating a reserved balance to a beneficiary account that exists.
//! - Transferring a balance between accounts (when not reserved).
//! - Slashing an account balance.
//! - Account creation and removal.
//! - Managing total issuance.
//! - Setting and managing locks.

use rstd::prelude::*;
use codec::{Codec, Encode, Decode};
use support::{
	decl_module, decl_storage, decl_event,
	StorageValue, StorageMap, Parameter,
	traits::{
		Currency, LockableCurrency, ReservableCurrency,
	// 	UpdateBalanceOutcome, OnFreeBalanceZero, OnUnbalanced,
	// 	WithdrawReason, WithdrawReasons, LockIdentifier, ExistenceRequirement,
	// 	Imbalance, SignedImbalance, ReservableCurrency, Get,
	},
	dispatch::Result,
};
use sr_primitives::{
	traits::{
		Zero, SimpleArithmetic, StaticLookup, Member, CheckedAdd, CheckedSub, MaybeSerializeDebug,
		Saturating, Bounded, SignedExtension, SaturatedConversion, Convert,
	},
};
use system::{
	ensure_signed, ensure_root,
	// IsDeadAccount, OnNewAccount,
};

// test mod
// mod mock;
// mod tests;

/// The module's configuration trait.
pub trait Trait: balances::Trait + timestamp::Trait {
	type Currency: Currency<Self::AccountId>;

	/// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as CurrencyModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(something): Option<u32>;
	}
}

// This modules' event
decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		<T as balances::Trait>::Balance,
	{
		/// A new account was created.
		NewAccount(AccountId, Balance),
		/// An account was reaped.
		ReapedAccount(AccountId),
		/// Transfer succeeded (from, to, value, fees).
		Transfer(AccountId, AccountId, Balance, Balance),
	}
);

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// Transfer proxy
		pub fn transfer_proxy(origin, to: T::AccountId, value: BalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;

			T::Currency::transfer(&sender, &to, value)?;

			Ok(())
		}
	}
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
	// Public immutables
	// TODO
}

