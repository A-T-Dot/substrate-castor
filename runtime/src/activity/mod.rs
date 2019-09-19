//! # Activity Module
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use codec::{Codec, Encode, Decode};
use support::{
	StorageValue, StorageMap, Parameter, decl_event, decl_storage, decl_module,
	traits::{
		Currency, LockableCurrency, ReservableCurrency,
		UpdateBalanceOutcome, OnFreeBalanceZero, OnUnbalanced,
		WithdrawReason, WithdrawReasons, LockIdentifier, ExistenceRequirement,
		Imbalance, SignedImbalance, Get, Time,
	},
	dispatch::Result,
};
use sr_primitives::{
	transaction_validity::{
		TransactionPriority, ValidTransaction, InvalidTransaction, TransactionValidityError,
		TransactionValidity,
	},
	traits::{
		Zero, SimpleArithmetic, StaticLookup, Member, CheckedAdd, CheckedSub, MaybeSerializeDebug,
		Saturating, Bounded, SignedExtension, SaturatedConversion, Convert,
	},
	weights::{DispatchInfo, SimpleDispatchInfo, Weight},
};
use system::{IsDeadAccount, OnNewAccount, ensure_signed, ensure_root};

use crate::non_transfer_asset;

/// The module's configuration trait.
pub trait Trait: system::Trait + non_transfer_asset::Trait {
	/// Time used for computing
	type Time: Time;

	/// Currency type for this module.
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Energy type for this module
	type Energy: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

	/// Action point type for this module
	type ActionPoint: Currency<Self::AccountId>;

	/// Gives a chance to clean up resources associated with the given account.
	type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

	/// Handler for when a new account is created.
	type OnNewAccount: OnNewAccount<Self::AccountId>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
type MomentOf<T>= <<T as Trait>::Time as Time>::Moment;

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as EnergyStorage {
		// TODO
	}
}

decl_event!(
	pub enum Event<T> where
    AccountId = <T as system::Trait>::AccountId
  {
		SomethingStored(u32, AccountId),
	}
);

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// fn deposit_event() = default;
	}
}

// The module's main implement
impl<T: Trait> Module<T> {
	// PUBLIC IMMUTABLES
	// TODO

	// PRIVATE MUTABLES
	// TODO

}

impl<T: Trait> OnNewAccount<T::AccountId> for Module<T> {
	// Implementation of the config type managing the creation of new accounts.
	fn on_new_account(who: &T::AccountId) {
		// TODO
	}
}
