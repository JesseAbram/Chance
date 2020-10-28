#![cfg_attr(not(feature = "std"), no_std)]

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

use sp_std::{prelude::*};
use sp_runtime::{DispatchResult, DispatchError};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure};
use frame_support::traits::Get;
use frame_system::{self as system, ensure_signed};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MaxSettlers: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Admin {
        Settlers get (fn settlers) config(): Vec<T::AccountId>
    }
}

decl_event!(
    pub enum Event<T> where 
        AccountId = <T as system::Trait>::AccountId,
        {
            SettlerAdded(AccountId),
            SettlerRemoved(AccountId),
        }
    );

decl_error! {
    pub enum Error for Module<T: Trait> {
        AlreadySetter,
        NotSettler,
        SettlerLimit,
        LastSettler
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default; 

        const MaxSettlers: u32 = T::MaxSettlers::get();

        #[weight = 0]
        fn add_setter(origin, who: T::AccountId) {
            // nitpick: I think I would leave `ensure_signed` and `ensure_settler` separate
            Self::ensure_settler(origin)?;
            Self::try_add_settler(&who)?;
			Self::deposit_event(RawEvent::SettlerAdded(who));
        }

        #[weight = 0]
        fn remove_settler(origin, who: T::AccountId) {
            Self::ensure_settler(origin)?;
            Self::try_remove_settler(&who)?;
			Self::deposit_event(RawEvent::SettlerRemoved(who));
        }
    }
}

impl<T: Trait> Module<T> {

    pub fn is_settler(who: &T::AccountId) -> bool {
        let settlers = Self::settlers();
        settlers.binary_search(&who).is_ok()
    }
    pub fn ensure_settler(origin: T::Origin) -> Result<T::AccountId, DispatchError> {
        let caller = ensure_signed(origin)?;
        ensure!(Self::is_settler(&caller), Error::<T>::NotSettler);
        Ok(caller)
    }
    pub fn try_add_settler(who: &T::AccountId) -> DispatchResult {
        Settlers::<T>::try_mutate(|settlers| -> DispatchResult {
            if settlers.len() < T::MaxSettlers::get() as usize {
                match settlers.binary_search(&who) {
                    Ok(_) => Err(Error::<T>::AlreadySetter)?,
                    Err(pos) => settlers.insert(pos, who.clone()),
                }
                Ok(())
            } else {
                Err(Error::<T>::SettlerLimit)?
            }
        })?;
        Ok(())
    }

    fn try_remove_settler(who: &T::AccountId) -> DispatchResult {
		Settlers::<T>::try_mutate(|settlers| -> DispatchResult {
			if settlers.len() == 1 as usize {
				Err(Error::<T>::LastSettler)?
			} else {
				match settlers.binary_search(&who) {
					Ok(pos) => settlers.remove(pos),
					Err(_) => Err(Error::<T>::NotSettler)?,
				};
				Ok(())
			}
		})?;
		Ok(())
	}
}