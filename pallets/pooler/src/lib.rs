#![cfg_attr(not(feature = "std"), no_std)]


use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get, Parameter, ensure};
use frame_system::ensure_signed;
use pallet_assets as assets;
use sp_runtime::traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, One, StaticLookup};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

pub trait Trait: frame_system::Trait + assets::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type PalletAssetId: Parameter + AtLeast32Bit + Default + Copy;

}

decl_storage! {
	trait Store for Module<T: Trait> as Pooler {
		PalletAssetId get(fn pallet_asset_id): T::PalletAssetId;
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
        NotInitiated,
        
        AlreadyInitiated,
		
	}
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn initiate(origin) -> dispatch::DispatchResult {
            ensure_signed(origin)?;
			ensure!(Self::pallet_asset_id() == Zero::zero(), Error::<T>::AlreadyInitiated);
			// initiate asset in the asset module
			// let assetId = <assets::Module<T>>::issue(origin, 100)?;
			let assetId = One::one();
			<PalletAssetId<T>>::set(assetId);
			Ok(())
		}
		#[weight = 0]
		pub fn bet(origin) -> dispatch::DispatchResult {
            ensure_signed(origin)?;
			//make sure there is liquiidty available 
			//pay fee
			// take bet push onto array of bets 
			// have OCW settle
			Ok(())
		}

		
	}
}

impl<T: Trait> Module<T> {
    
}