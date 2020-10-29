
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{    
traits::{Currency, ExistenceRequirement::AllowDeath},
Parameter, decl_module, decl_event, decl_storage, decl_error, ensure, dispatch
};
use frame_system::{self as system, ensure_signed};

use sp_runtime::{
    traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, StaticLookup, AccountIdConversion},
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
			// best practice is to use `saturating_sub` here (though this is safe because of the ensure)
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

	// this does not seem like a burn and more like a payout function ^^
	// I'm having trouble following what your invariants are and whether they are being kept
	// Why is `amount` compared to a balance but then used as a percentage?
	pub fn burn(who: T::AccountId, amount: BalanceOf<T>)  -> dispatch::DispatchResult{
		let origin_balance = <Balances<T>>::get(&who);
		ensure!(origin_balance >= amount, Error::<T>::BalanceLow);
		let total_supply = Self::total_supply();
		let balance_of_pallet = T::Currency::free_balance(&Self::account_id());
		// nitpick: the `* 100` and `/ 100` combination seems redundant
		let percent_of_total = amount * (100.into()) / total_supply;
		let payout = balance_of_pallet * percent_of_total / 100.into();
		T::Currency::transfer(&Self::account_id(), &who, payout, AllowDeath)?;
		<Balances<T>>::mutate(who, |balance| *balance -= payout);
		// o.O
		// use `saturating_sub`
		if payout > total_supply {
			<TotalSupply<T>>::mutate(|total| *total -= *total);

		} else {
			<TotalSupply<T>>::mutate(|total| *total -= payout);
		}
		Ok(())

	}

	fn account_id() -> T::AccountId{
        const PALLET_ID: ModuleId = ModuleId(*b"assethdl");
        PALLET_ID.into_account()
    }
}



