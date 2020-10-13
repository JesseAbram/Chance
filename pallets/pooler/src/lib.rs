
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{    
traits::{Currency, ExistenceRequirement::AllowDeath, StoredMap},
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
			//TODO add event
			Ok(())

		}
		#[weight = 0]
		pub fn withdraw(origin, amount: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::burn(who, amount)?;
			// TODO add event
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
		/// Some pooler were issued. \[asset_id, owner, total_supply\]
		Issued(AssetId, AccountId, Balance),
		/// Some pooler were transferred. \[asset_id, from, to, amount\]
		Transferred(AccountId, AccountId, Balance),
		/// Some pooler were destroyed. \[asset_id, owner, balance\]
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
	trait Store for Module<T: Trait> as pooler {
		/// The number of units of pooler held by any given account.
		Balances: map hasher(blake2_128_concat) T::AccountId => BalanceOf<T>;
		/// The next asset identifier up for grabs.
		///
		/// TWOX-NOTE: `AssetId` is trusted, so this is safe.
		TotalSupply get(fn total_supply): BalanceOf<T>;

		// Reserves get(fn reserves): BalanceOf<T>;
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
		// <Reserves<T>>::mutate(|total| *total += amount);
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
		// <Reserves<T>>::mutate(|total| *total -= amount);
		Ok(())

	}

	// pub fn track_reserves_increase(amount: BalanceOf<T>) {
	// 	<Reserves<T>>::mutate(|total| *total += amount);
	// }

	// pub fn track_reserves_decrease(amount: BalanceOf<T>) {
	// 	<Reserves<T>>::mutate(|total| *total -= amount);
	// }

	fn account_id() -> T::AccountId{
        const PALLET_ID: ModuleId = ModuleId(*b"assethdl");
        PALLET_ID.into_account()
    }
}



