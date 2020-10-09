// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Assets Module
//!
//! A simple, secure module for dealing with fungible assets.
//!
//! ## Overview
//!
//! The Assets module provides functionality for asset management of fungible asset classes
//! with a fixed supply, including:
//!
//! * Asset Issuance
//! * Asset Transfer
//! * Asset Destruction
//!
//! To use it in your runtime, you need to implement the assets [`Trait`](./trait.Trait.html).
//!
//! The supported dispatchable functions are documented in the [`Call`](./enum.Call.html) enum.
//!
//! ### Terminology
//!
//! * **Asset issuance:** The creation of a new asset, whose total supply will belong to the
//!   account that issues the asset.
//! * **Asset transfer:** The action of transferring assets from one account to another.
//! * **Asset destruction:** The process of an account removing its entire holding of an asset.
//! * **Fungible asset:** An asset whose units are interchangeable.
//! * **Non-fungible asset:** An asset for which each unit has unique characteristics.
//!
//! ### Goals
//!
//! The assets system in Substrate is designed to make the following possible:
//!
//! * Issue a unique asset to its creator's account.
//! * Move assets between accounts.
//! * Remove an account's balance of an asset when requested by that account's owner and update
//!   the asset's total supply.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `issue` - Issues the total supply of a new fungible asset to the account of the caller of the function.
//! * `transfer` - Transfers an `amount` of units of fungible asset `id` from the balance of
//! the function caller's account (`origin`) to a `target` account.
//! * `destroy` - Destroys the entire holding of a fungible asset `id` associated with the account
//! that called the function.
//!
//! Please refer to the [`Call`](./enum.Call.html) enum and its associated variants for documentation on each function.
//!
//! ### Public Functions
//! <!-- Original author of descriptions: @gavofyork -->
//!
//! * `balance` - Get the asset `id` balance of `who`.
//! * `total_supply` - Get the total supply of an asset `id`.
//!
//! Please refer to the [`Module`](./struct.Module.html) struct for details on publicly available functions.
//!
//! ## Usage
//!
//! The following example shows how to use the Assets module in your runtime by exposing public functions to:
//!
//! * Issue a new fungible asset for a token distribution event (airdrop).
//! * Query the fungible asset holding balance of an account.
//! * Query the total supply of a fungible asset that has been issued.
//!
//! ### Prerequisites
//!
//! Import the Assets module and types and derive your runtime's configuration traits from the Assets module trait.
//!
//! ### Simple Code Snippet
//!
//! ```rust,ignore
//! use pallet_assets as assets;
//! use frame_support::{decl_module, dispatch, ensure};
//! use frame_system::ensure_signed;
//!
//! pub trait Trait: assets::Trait { }
//!
//! decl_module! {
//! 	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//! 		pub fn issue_token_airdrop(origin) -> dispatch::DispatchResult {
//! 			let sender = ensure_signed(origin).map_err(|e| e.as_str())?;
//!
//! 			const ACCOUNT_ALICE: u64 = 1;
//! 			const ACCOUNT_BOB: u64 = 2;
//! 			const COUNT_AIRDROP_RECIPIENTS: u64 = 2;
//! 			const TOKENS_FIXED_SUPPLY: u64 = 100;
//!
//! 			ensure!(!COUNT_AIRDROP_RECIPIENTS.is_zero(), "Divide by zero error.");
//!
//! 			let asset_id = Self::next_asset_id();
//!
//! 			<NextAssetId<T>>::mutate(|asset_id| *asset_id += 1);
//! 			<Balances<T>>::insert((asset_id, &ACCOUNT_ALICE), TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
//! 			<Balances<T>>::insert((asset_id, &ACCOUNT_BOB), TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
//! 			<TotalSupply<T>>::insert(asset_id, TOKENS_FIXED_SUPPLY);
//!
//! 			Self::deposit_event(RawEvent::Issued(asset_id, sender, TOKENS_FIXED_SUPPLY));
//! 			Ok(())
//! 		}
//! 	}
//! }
//! ```
//!
//! ## Assumptions
//!
//! Below are assumptions that must be held when using this module.  If any of
//! them are violated, the behavior of this module is undefined.
//!
//! * The total count of assets should be less than
//!   `Trait::AssetId::max_value()`.
//!
//! ## Related Modules
//!
//! * [`System`](../frame_system/index.html)
//! * [`Support`](../frame_support/index.html)

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{    
traits::{Currency, ExistenceRequirement::AllowDeath},
Parameter, decl_module, decl_event, decl_storage, decl_error, ensure, dispatch
};
use frame_system::{self as system, ensure_signed};

use sp_runtime::{
    traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, One, StaticLookup, AccountIdConversion},
    ModuleId
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module configuration trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The units in which we record balances.
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	/// The arithmetic type of asset identifier.
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;

    type Currency: Currency<Self::AccountId>;

}

type AccountIdOf<T> = <T as system::Trait>::AccountId;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<AccountIdOf<T>>>::Balance;

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		
		/// Move some assets from one holder to another.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 static lookup
		/// - 2 storage mutations (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn transfer(origin,
			target: <T::Lookup as StaticLookup>::Source,
			#[compact] amount: BalanceOf<T>
		) {
			let who = ensure_signed(origin)?;
			let origin_balance = <Balances<T>>::get(&who);
			let target = T::Lookup::lookup(target)?;
			ensure!(!amount.is_zero(), Error::<T>::AmountZero);
			ensure!(origin_balance >= amount, Error::<T>::BalanceLow);

			// Self::deposit_event(RawEvent::Transferred(&who, &target, amount));
			<Balances<T>>::insert(who, origin_balance - amount);
			<Balances<T>>::mutate(target, |balance| *balance += amount);
		}

		#[weight = 0]
		pub fn deposit(origin, amount: BalanceOf<T>) -> dispatch::DispatchResult { 
			let who = ensure_signed(origin)?;
			Self::mint(who, amount)?;
			//add event
			Ok(())

		}
		#[weight = 0]
		pub fn withdraw(origin, amount: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::burn(who, amount)?;
			// take in token send out funds
			// check array of known bets make sure there is liquidity available
			Ok(())

		}
}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AssetId,
	{
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(AssetId, AccountId, Balance),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		Transferred(AccountId, AccountId, Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Destroyed(AssetId, AccountId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Transfer amount should be non-zero
		AmountZero,
		/// Account balance must be greater than or equal to the transfer amount
		BalanceLow,
		/// Balance should be non-zero
		BalanceZero,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Assets {
		/// The number of units of assets held by any given account.
		Balances: map hasher(blake2_128_concat) T::AccountId => BalanceOf<T>;
		/// The next asset identifier up for grabs.
		///
		/// TWOX-NOTE: `AssetId` is trusted, so this is safe.
		TotalSupply get(fn total_supply): BalanceOf<T>;
	}
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {

	/// Get the asset `id` balance of `who`.
	pub fn balance(who: T::AccountId) -> BalanceOf<T> {
		<Balances<T>>::get(who)
	}

	pub fn mint(who: T::AccountId, amount: BalanceOf<T>) -> dispatch::DispatchResult{
		T::Currency::transfer(&who, &Self::account_id(), amount, AllowDeath)?;
		let payout;
		let total_supply = Self::total_supply();
		if total_supply == 0.into() {
			payout = amount;
		} else {
			//TODO handle putting in less then 1% of value and handle greater than 100%
			let after_payout = total_supply + amount;
			let amount_to_payout = amount * (100.into()) / after_payout;
			payout = amount_to_payout;
		}
		<Balances<T>>::mutate(who, |balance| *balance += payout);
		<TotalSupply<T>>::mutate(|total| *total += payout);
		Ok(())
	}

	pub fn burn(who: T::AccountId, amount: BalanceOf<T>)  -> dispatch::DispatchResult{
		let origin_balance = <Balances<T>>::get(&who);
		ensure!(origin_balance >= amount, Error::<T>::BalanceLow);
		let total_supply = Self::total_supply();
		let payout = amount * (100.into()) / total_supply;
		T::Currency::transfer(&Self::account_id(), &who, amount, AllowDeath)?;
		<Balances<T>>::mutate(who, |balance| *balance -= payout);
		<TotalSupply<T>>::mutate(|total| *total -= payout);
		Ok(())

	}

	fn account_id() -> T::AccountId{
        const PALLET_ID: ModuleId = ModuleId(*b"Poolerrr");
        PALLET_ID.into_account()
    }
}



