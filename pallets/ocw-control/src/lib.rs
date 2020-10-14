#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	debug, decl_error, decl_event, decl_module, dispatch::DispatchResult, traits::Get, weights::Pays,
};

use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	offchain as rt_offchain,
	offchain::storage::StorageValueRef,
	transaction_validity::TransactionPriority,
};
use sp_std::prelude::*;
use sp_std::str;
use chance::BalanceOf;

// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
//   with serde(features `std`) and alt_serde(features `no_std`).
// use alt_serde::{Deserialize, Deserializer};

pub const HTTP_REMOTE_REQUEST_STRING: &str = "http://localhost:3000/random";
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwc");


pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	// implemented for ocw-runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// This is the pallet's configuration trait
pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> + chance::Trait + admin::Trait {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The type to sign and send transactions.
	type UnsignedPriority: Get<TransactionPriority>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	/// Events generated by the module.
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
	{
		BetWon(AccountId, Balance),
		BetLost(AccountId, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		// Error returned when making remote http fetching
		HttpFetchingError,
		// Error returned when Weather-info has already been fetched
		AlreadyFetched,
		ConvertionError,
		SubmitError,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = (0, Pays::No)]
		pub fn submit_signed(origin, better: T::AccountId, wager: BalanceOf<T>, did_win: bool) -> DispatchResult {
			<admin::Module<T>>::ensure_settler(origin.clone())?;
			debug::info!("Entering submit_signed. {:#?}, {:#?}, {:#?}", better, wager, did_win);

			<chance::Module<T>>::scheduled_bet_callback(origin, better.clone(), wager, did_win)?;
			if did_win {
				Self::deposit_event(RawEvent::BetWon(better, wager));
			} else {
				Self::deposit_event(RawEvent::BetLost(better, wager));
			}

			Ok(())
		}

		fn offchain_worker(block_number: T::BlockNumber) {
            let pending_bets = <chance::Module<T>>::scheduled_bet();
            debug::info!("Entering offchain worker");
			if pending_bets.len() > 0 {
				debug::info!("Entering action");
				for bet in pending_bets {
                    let better = bet.0.clone();
                    let wager = bet.1.clone();
                    debug::info!("better {:#?}", better);
                    debug::info!("bet {:#?}", wager);
					let _ = Self::fetch_if_needed(bet);
				}
			}
		}
	}
}

impl<T: Trait> Module<T> {
	
	/// Check if we have fetched Weather info before. If yes, we use the cached version that is
	///   stored in off-chain worker storage `storage`. If no, we fetch the remote info and then
	///   write the info into the storage for future retrieval.
	fn fetch_if_needed(tx: (T::AccountId, BalanceOf<T>)) -> Result<(), Error<T>> {
		debug::info!("Tx: {:#?}, {:#?}", &tx.0, &tx.1);
		// Start off by creating a reference to Local Storage value.
		// Since the local storage is common for all offchain workers, it's a good practice
		// to prepend our entry with the pallet name.
		let s_lock = StorageValueRef::persistent(b"ocw-postgres::lock");

		// The local storage is persisted and shared between runs of the offchain workers,
		// and offchain workers may run concurrently. We can use the `mutate` function, to
		// write a storage entry in an atomic fashion.
		//
		// It has a similar API as `StorageValue` that offer `get`, `set`, `mutate`.
		// If we are using a get-check-set access pattern, we likely want to use `mutate` to access
		// the storage in one go.
		//

		// We are implementing a mutex lock here with `s_lock`
		let res: Result<Result<bool, bool>, Error<T>> = s_lock.mutate(|s: Option<Option<bool>>| {
			match s {
				// `s` can be one of the following:
				//   `None`: the lock has never been set. Treated as the lock is free
				//   `Some(None)`: unexpected case, treated it as AlreadyFetch
				//   `Some(Some(false))`: the lock is free
				//   `Some(Some(true))`: the lock is held

				// If the lock has never been set or is free (false), return true to execute `fetch_n_parse`
				None | Some(Some(false)) => Ok(true),

				// Otherwise, someone already hold the lock (true), we want to skip `fetch_n_parse`.
				// Covering cases: `Some(None)` and `Some(Some(true))`
				_ => Err(<Error<T>>::AlreadyFetched),
			}
		});

		// Cases of `res` returned result:
		//   `Err(<Error<T>>)` - lock is held, so we want to skip `fetch_n_parse` function.
		//   `Ok(Err(true))` - Another ocw is writing to the storage while we set it,
		//                     we also skip `fetch_n_parse` in this case.
		//   `Ok(Ok(true))` - successfully acquire the lock, so we run `fetch_n_parse`
		if let Ok(Ok(true)) = res {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if !signer.can_sign() {
				debug::error!("No local account available");
				s_lock.set(&false);
				return Err(<Error<T>>::SubmitError);
			}
			match Self::fetch_n_parse() {
				Ok(true) => {
					debug::info!("Fetch and parse returned true.");
					s_lock.set(&false);
					let _ = signer.send_signed_transaction(|_acct| {
						Call::submit_signed(tx.0.clone(), tx.1.clone(), true)
					});
				},
				Ok(false) => {
					debug::info!("Fetch and parse returned false.");
					s_lock.set(&false);
					let _ = signer.send_signed_transaction(|_acct| {
						Call::submit_signed(tx.0.clone(), tx.1.clone(), false)
					});
				},
				Err(err) => {
					debug::info!("Fetch and parse returned error.");
					s_lock.set(&false);
					return Err(err);
				}
			}
		}
		Ok(())
	}

	/// Fetch from remote and deserialize the JSON to a struct
	fn fetch_n_parse() -> Result<bool, Error<T>> {
		let resp_bytes = Self::fetch_from_remote().map_err(|e| {
			debug::error!("fetch_from_remote error: {:?}", e);
			<Error<T>>::HttpFetchingError
		})?;

		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
		debug::info!("Fetch and parse fetched: {:#?}.", resp_str);
		if resp_str == "1" {
			return Ok(true);
		}

		// let s = str::replace(&resp_str.replace("]", ""), "[", "");
		//debug::info!("Fetched String: {:#?}", &s);

		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
		// let denied_account: DeniedAccount =
		// 			serde_json::from_str::<DeniedAccount>(&s).map_err(|_| <Error<T>>::HttpFetchingError)?;
		// Print out our fetched JSON string
		// debug::info!("{:#?}", denied_account);

		Ok(false)
	}

	/// This function uses the `offchain::http` API to query the remote Weather information,
	///   and returns the JSON response as vector of bytes.
	fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
		let url_with_address = HTTP_REMOTE_REQUEST_STRING;
		//debug::info!("URL: {}", url_with_address);
		let remote_url_bytes = url_with_address.as_bytes().to_vec();
		let remote_url =
			str::from_utf8(&remote_url_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;

		debug::info!("sending request to: {}", remote_url);

		// Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`.
		let request = rt_offchain::http::Request::get(remote_url);

		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
		let timeout = sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(3000));

		let pending = request
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		// By default, the http request is async from the runtime perspective. So we are asking the
		//   runtime to wait here.
		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
		//   ref: https://substrate.dev/rustdocs/v2.0.0-rc3/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError);
		}

		// Next we fully read the response body and collect it to a vector of bytes.
		Ok(response.body().collect::<Vec<u8>>())
	}
}