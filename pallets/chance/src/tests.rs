use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn initate() {
	new_test_ext().execute_with(|| {
		assert_eq!(Chance::pallet_asset_id(), 0);
		println!("the nom nom {:?}", Chance::pallet_asset_id());
	}

	)}