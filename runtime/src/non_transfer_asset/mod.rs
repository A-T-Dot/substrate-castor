
//! # Non-transfer Asset Module
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use codec::{Codec, Encode, Decode};
use support::{
	decl_module, decl_storage, decl_event,
	StorageValue, StorageMap, Parameter,
	traits::{
		Currency, LockableCurrency, ReservableCurrency,
		Time,
	// 	UpdateBalanceOutcome, OnFreeBalanceZero, OnUnbalanced,
	// 	WithdrawReason, WithdrawReasons, LockIdentifier, ExistenceRequirement,
	// 	Imbalance, SignedImbalance
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
	ensure_signed
};

// test mod
// mod mock;
// mod tests;

/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait {
	type Currency: Currency<Self::AccountId>;
	// type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
	// type Currency: ReservableCurrency<Self::AccountId>;

	/// Time use for computing energy increase
	type Time: Time;

	/// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type BalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
// type PositiveImbalanceOf<T> =
// 	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
// type NegativeImbalanceOf<T> =
// 	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
// type MomentOf<T>= <<T as Trait>::Time as Time>::Moment;

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as CurrencyStorage {
		// TODO
	}
}

// This modules' event
decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
	{
		EnergyUpdated(AccountId, Balance),
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

