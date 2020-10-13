use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn test_bet_small_bet() {
	new_test_ext().execute_with(|| {
		assert_ok!(Pooler::deposit(Origin::signed(1), 100000000000000));
		assert_ok!(Chance::bet(Origin::signed(2), 1000000000000));
		let bet = [(2,990000000000,),];
		println!("small bet{:#?}", Chance::scheduled_bet());
		assert_eq!(Chance::scheduled_bet(), bet);
	}
)}

#[test]
fn test_bet_whole_pool() {
	new_test_ext().execute_with(|| {
		assert_ok!(Pooler::deposit(Origin::signed(1), 10000000000000));
		assert_ok!(Chance::bet(Origin::signed(2), 10000000000000));
		let bet = [(2,9000000000000,),];
		println!("large bet{:#?}", Chance::scheduled_bet());
		assert_eq!(Chance::scheduled_bet(), bet);
	}
)}
