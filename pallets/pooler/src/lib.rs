#![cfg_attr(not(feature = "std"), no_std)]


use frame_support::{
	traits::{Currency, ExistenceRequirement::AllowDeath, Randomness},
	decl_module, decl_storage, decl_event, decl_error, dispatch, Parameter, ensure
};
use frame_system::{self as system, ensure_signed};
use pallet_assets as assets;
use sp_runtime::{
    traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, One, StaticLookup, AccountIdConversion},
    ModuleId
};
use codec::Encode;
use std::ops::{Mul, Div};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

pub trait Trait: frame_system::Trait + assets::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type PalletAssetId: Parameter + AtLeast32Bit + Default + Copy;
	type Currency: Currency<Self::AccountId>;
	type RandomnessSource: Randomness<u128>;
}

type AccountIdOf<T> = <T as system::Trait>::AccountId;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<AccountIdOf<T>>>::Balance;

decl_storage! {
	trait Store for Module<T: Trait> as Pooler {
		PalletAssetId get(fn pallet_asset_id): T::PalletAssetId;
		Nonce get(fn nonce): u32;
	}
}


decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error if module is not initiated.
       NotEnoughLiquidity,

	}
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = 0]
		pub fn bet(origin, amount: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::ensure_liquidity(&amount)?;
			let fee_multiplier = 99;
			let converted_amount = amount; //TryInto::<u128>::try_into(amount).unwrap_or(u128::max_value());
			let bet = converted_amount.mul(fee_multiplier.into()).div(100.into());
			// TODO add slippage
			let subject = Self::encode_and_update_nonce();

			let random_result = T::RandomnessSource::random(&subject);
			let win;
			match random_result % 2 == 0 {
				false => win = false,
				true => win = true
			};
			if win {
				<T as Trait>::Currency::transfer(&Self::account_id(), &who, bet.into(), AllowDeath)?;
			} else {
				<T as Trait>::Currency::transfer(&who, &Self::account_id(), bet.into(), AllowDeath)?;
			}
			Ok(())
		}

		
	}
}

impl<T: Trait> Module<T> {

	fn account_id() -> T::AccountId{
        const PALLET_ID: ModuleId = ModuleId(*b"assethdl");
        PALLET_ID.into_account()
	}
	
	fn ensure_liquidity(amount: &BalanceOf<T>) -> dispatch::DispatchResult {
		let current_balance = <T as Trait>::Currency::free_balance(&Self::account_id());
		ensure!(amount <= &current_balance, Error::<T>::NotEnoughLiquidity);
		Ok(())
	}

	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = Nonce::get();
		Nonce::put(nonce.wrapping_add(1));
		nonce.encode()
	}
    
}