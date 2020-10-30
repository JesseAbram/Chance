#![cfg_attr(not(feature = "std"), no_std)]


use frame_support::{
	traits::{Currency, Vec, ExistenceRequirement::{KeepAlive, AllowDeath}, Get},
	decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, debug
};

use frame_system::{self as system, ensure_signed};
use pallet_pooler as pooler;
use pallet_admin as admin;
use sp_runtime::{
    traits::{AccountIdConversion},
    ModuleId
};
use core::ops::{Mul, Div};
use core::convert::TryInto;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait + pooler::Trait + admin::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
	type SystemDecimals: Get<u128>;

}

type AccountIdOf<T> = <T as system::Trait>::AccountId;
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<AccountIdOf<T>>>::Balance;

decl_storage! {
	trait Store for Module<T: Trait> as Chance {
		ScheduledBet get(fn scheduled_bet): Vec<(T::AccountId, BalanceOf<T>)>;
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
	   Other,
	   Conversion

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
			let total_locked = <T as Trait>::Currency::free_balance(&Self::account_id());
			// fee is proportional to size of bet
			let system_decimals: u128 = T::SystemDecimals::get();
			let converted_amount = TryInto::<u128>::try_into(amount).unwrap_or(u128::max_value());
			let converted_total_locked = TryInto::<u128>::try_into(total_locked).unwrap_or(u128::max_value());
			let fee_multiplier = 10;
			let fee = converted_amount.mul(system_decimals).mul(fee_multiplier).div(converted_total_locked);
			
			let bet = converted_amount.saturating_sub(fee);

			<T as Trait>::Currency::transfer(&who, &Self::account_id(), amount, KeepAlive)?;

			//prep bet for offchain worker
			ScheduledBet::<T>::try_mutate(|sch| -> dispatch::DispatchResult {
				sch.push((who, Self::u128_to_balance(bet)));
				Ok(())
			})?;

			Ok(())
		}

		
	}
}

impl<T: Trait> Module<T> {

	pub fn scheduled_bet_callback(origin: T::Origin, better: T::AccountId, bet: BalanceOf<T>, did_win: bool) -> dispatch::DispatchResult {
		<admin::Module<T>>::ensure_settler(origin.clone())?;
		debug::info!("Entering callback. {}, {:#?}", did_win, bet);
		if did_win {
			let winnings = bet.mul(2.into());
			<T as Trait>::Currency::transfer(&Self::account_id(), &better, winnings.into(), AllowDeath)?;
			// <pooler::Module<T>>::track_reserves_decrease(winnings);			
		}
		ScheduledBet::<T>::try_mutate(|sch| ->  dispatch::DispatchResult {
			match sch.binary_search(&(better, bet)) {
				Ok(pos) => {
					debug::info!("Found pending tx; removing.");
					sch.remove(pos);
				},
				Err(_) => Err(Error::<T>::Other)?,
			};
			Ok(())
		})?;
		Ok(())

	}

	pub fn account_id() -> T::AccountId{
        const PALLET_ID: ModuleId = ModuleId(*b"assethdl");
        PALLET_ID.into_account()
	}
	
	fn ensure_liquidity(amount: &BalanceOf<T>) -> dispatch::DispatchResult {
		let current_balance = <T as Trait>::Currency::free_balance(&Self::account_id());
		ensure!(amount <= &current_balance, Error::<T>::NotEnoughLiquidity);
		Ok(())
	}

	fn u128_to_balance(input: u128) -> BalanceOf<T> {
		input.try_into().unwrap_or(0.into())
	}
}