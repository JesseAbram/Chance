use crate::{Error, mock::*};
use crate::*;
use frame_support::{assert_ok, assert_noop};



#[test]
fn issuing_asset_units_to_issuer_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Pooler::balance(1), 0);
	});
}

#[test]
fn minting_pooler_multiple_times() {
	new_test_ext().execute_with(|| {				  
		assert_ok!(Pooler::deposit(Origin::signed(1), 9000000000000));
		assert_eq!(Pooler::balance(1), 9000000000000);
		assert_ok!(Pooler::deposit(Origin::signed(2), 1000000000000));
		assert_eq!(Pooler::balance(2), 1000000000000);
		assert_eq!(Pooler::total_supply(), 10000000000000);
		assert_ok!(Pooler::transfer(Origin::signed(1), 6, Pooler::balance(1)));
		assert_ok!(Pooler::withdraw(Origin::signed(6), Pooler::balance(6)));
		assert_eq!(Test_Balances::free_balance(6), 9000000000000);

	});
}

#[test]
fn minting_burning_pooler_multiple_times_fee_accumilation() {
	new_test_ext().execute_with(|| {				  
		assert_ok!(Pooler::deposit(Origin::signed(1), 9000000000000));
		assert_ok!(Test_Balances::transfer(Origin::signed(3), Pooler::account_id(), 9000000000000));
		assert_eq!(Pooler::balance(1), 9000000000000);
		assert_ok!(Pooler::deposit(Origin::signed(2), 1000000000000));
		assert_eq!(Pooler::balance(2), 500000000000);
		assert_eq!(Pooler::total_supply(), 9500000000000);

		assert_ok!(Pooler::transfer(Origin::signed(1), 6, Pooler::balance(1)));
		assert_ok!(Pooler::withdraw(Origin::signed(6), Pooler::balance(6)));
		assert_eq!(Test_Balances::free_balance(6), 18000000000000);

		assert_eq!(Pooler::total_supply(), 500000000000);

		assert_ok!(Pooler::withdraw(Origin::signed(2), Pooler::balance(2)));
		assert_eq!(Test_Balances::free_balance(2), 1000000000000);

	});
}


#[test]
fn querying_total_supply_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Pooler::deposit(Origin::signed(1), 100));
		assert_eq!(Pooler::balance(1), 100);
		assert_ok!(Pooler::transfer(Origin::signed(1), 2, 50));
		assert_eq!(Pooler::balance(1), 50);
		assert_eq!(Pooler::balance(2), 50);
		assert_ok!(Pooler::transfer(Origin::signed(2), 3, 31));
		assert_eq!(Pooler::balance(1), 50);
		assert_eq!(Pooler::balance(2), 19);
		assert_eq!(Pooler::balance(3), 31);
		assert_eq!(Pooler::total_supply(), 100);
	});
}

#[test]
fn transferring_amount_above_available_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Pooler::deposit(Origin::signed(1), 100));
		assert_eq!(Pooler::balance(1), 100);
		assert_ok!(Pooler::transfer(Origin::signed(1), 2, 50));
		assert_eq!(Pooler::balance(1), 50);
		assert_eq!(Pooler::balance(2), 50);
	});
}

#[test]
fn transferring_amount_more_than_available_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Pooler::deposit(Origin::signed(1), 100));
		assert_eq!(Pooler::balance(1), 100);
		assert_ok!(Pooler::transfer(Origin::signed(1), 2, 50));
		assert_eq!(Pooler::balance(1), 50);
		assert_eq!(Pooler::balance(2), 50);
		assert_ok!(Pooler::withdraw(Origin::signed(1), 50));
		assert_eq!(Pooler::balance(1), 0);
		assert_noop!(Pooler::transfer(Origin::signed(1), 1, 50), Error::<Test>::BalanceLow);
	});
}

#[test]
fn transferring_less_than_one_unit_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Pooler::deposit(Origin::signed(1), 100));
		assert_eq!(Pooler::balance(1), 100);
		assert_noop!(Pooler::transfer(Origin::signed(1), 2, 0), Error::<Test>::AmountZero);
	});
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Pooler::deposit(Origin::signed(1), 100));
		assert_eq!(Pooler::balance(1), 100);
		assert_noop!(Pooler::transfer(Origin::signed(1), 2, 101), Error::<Test>::BalanceLow);
	});
}

