use crate::*;
use frame_support::{assert_ok, assert_noop, impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use codec::{alloc::sync::Arc, Decode};
use parking_lot::RwLock;
use sp_core::{
	offchain::{
		testing::{self, OffchainState, PoolState},
		OffchainExt, TransactionPoolExt,
	},
	sr25519::{self, Signature},
	testing::KeyStore,
	traits::KeystoreExt,
	H256,
};
use sp_core::{Pair, Public};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, IdentityLookup, Verify, IdentifyAccount},
	Perbill,

};

use crate as ocw_demo;

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1_000_000;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

pub type Balances = pallet_balances::Module<Test>;
pub type System = frame_system::Module<Test>;

impl system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type Call = ();
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type AccountData = pallet_balances::AccountData<u64>;
}

impl pooler::Trait for Test {
    type Event = ();
    type Balance = u128;
	type AssetId = u128;
	type Currency = Balances;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Trait for Test {
    type Balance = u64;
    type Event = ();
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
}

parameter_types! {
	pub const MaxSettlers: u32  = 10; 
}


impl admin::Trait for Test {
	type Event = ();
	type MaxSettlers = MaxSettlers;
}

parameter_types! {
	pub const SystemDecimals: u128  = 100000000000;
}

impl chance::Trait for Test {
    type Event = ();
	type Currency = Balances;
	type SystemDecimals = SystemDecimals;
}

type TestExtrinsic = TestXt<Call<Test>, ()>;

parameter_types! {
	pub const UnsignedPriority: u64 = 100;
}

impl Trait for Test {
	type AuthorityId = crypto::TestAuthId;
	type Call = Call<Test>;
    type Event = ();
    type UnsignedPriority = UnsignedPriority;
}

impl<LocalCall> system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	Call<Test>: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call<Test>,
		_public: <Signature as Verify>::Signer,
		_account: <Test as system::Trait>::AccountId,
		index: <Test as system::Trait>::Index,
	) -> Option<(
		Call<Test>,
		<TestExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		Some((call, (index, ())))
	}
}

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
	Call<Test>: From<C>,
{
	type OverarchingCall = Call<Test>;
	type Extrinsic = TestExtrinsic;
}

pub type OcwDemo = Module<Test>;
pub type Chance = chance::Module<Test>;
pub type Pooler = pooler::Module<Test>;


struct ExternalityBuilder;

impl ExternalityBuilder {
	pub fn build() -> (
		TestExternalities,
		Arc<RwLock<PoolState>>,
		Arc<RwLock<OffchainState>>,
	) {
		const PHRASE: &str =
			"expire stage crawl shell boss any story swamp skull yellow bamboo copy";

		let (offchain, offchain_state) = testing::TestOffchainExt::new();
		let (pool, pool_state) = testing::TestTransactionPoolExt::new();
		let keystore = KeyStore::new();
		keystore
			.write()
			.sr25519_generate_new(KEY_TYPE, Some(&format!("{}/hunter1", PHRASE)))
            .unwrap();
            
        let acct: <Test as system::Trait>::AccountId = Default::default();

		let mut storage = system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
        pallet_balances::GenesisConfig::<Test> {
                balances: vec![(acct, 100000000000000000)],
			}.assimilate_storage(&mut storage).unwrap();
		admin::GenesisConfig::<Test> {
				settlers: vec![acct],
		}.assimilate_storage(&mut storage).unwrap();
		let mut t = TestExternalities::from(storage);
		t.register_extension(OffchainExt::new(offchain));
		t.register_extension(TransactionPoolExt::new(pool));
		t.register_extension(KeystoreExt(keystore));
		t.execute_with(|| System::set_block_number(1));
		(t, pool_state, offchain_state)
	}
}


#[test]
fn test_ocw_call_bet_won() {
  let (mut t, _, _) = ExternalityBuilder::build();
	t.execute_with(|| {
        let acct: <Test as system::Trait>::AccountId = Default::default();

		assert_ok!(Pooler::deposit(Origin::signed(acct), 100000000000000));
		assert_eq!(Pooler::balance(acct), 100000000000000);
		assert_ok!(Chance::bet(Origin::signed(acct), 1000000000000));
		println!("check on bets before {:#?}", Chance::scheduled_bet());
		let bet = [(acct,990000000000,),];
		assert_eq!(Chance::scheduled_bet(), bet);
		assert_ok!(OcwDemo::submit_signed(Origin::signed(acct), acct, 990000000000, true));
		let bet_after = [];
		assert_eq!(Chance::scheduled_bet(), bet_after);
		println!("check on bets after {:#?}", Chance::scheduled_bet());
		assert_eq!(Balances::free_balance(Chance::account_id()), 99010000000000);


	})
}

#[test]
fn test_ocw_call_bet_lost() {
  let (mut t, _, _) = ExternalityBuilder::build();
	t.execute_with(|| {
        let acct: <Test as system::Trait>::AccountId = Default::default();

		assert_ok!(Pooler::deposit(Origin::signed(acct), 100000000000000));
		assert_eq!(Pooler::balance(acct), 100000000000000);
		assert_ok!(Chance::bet(Origin::signed(acct), 1000000000000));
		println!("check on bets before {:#?}", Chance::scheduled_bet());
		let bet = [(acct,990000000000,),];
		assert_eq!(Chance::scheduled_bet(), bet);
		assert_ok!(OcwDemo::submit_signed(Origin::signed(acct), acct, 990000000000, false));
		let bet_after = [];
		assert_eq!(Chance::scheduled_bet(), bet_after);
		println!("check on bets after {:#?}", Chance::scheduled_bet());
		
		println!("account {:#?}", Balances::free_balance(Chance::account_id()));
		assert_eq!(Balances::free_balance(Chance::account_id()), 100990000000000);

	})
}

#[test]
fn test_ocw_called_by_non_settler_should_fail() {
  let (mut t, _, _) = ExternalityBuilder::build();
	t.execute_with(|| {
        let acct: <Test as system::Trait>::AccountId = Default::default();
		let non_settler = get_account_id_from_seed::<sr25519::Public>("Alice");

		assert_ok!(Pooler::deposit(Origin::signed(acct), 100000000000000));
		assert_eq!(Pooler::balance(acct), 100000000000000);
		assert_ok!(Chance::bet(Origin::signed(acct), 1000000000000));
		println!("check on bets before {:#?}", Chance::scheduled_bet());
		let bet = [(acct,990000000000,),];
		assert_eq!(Chance::scheduled_bet(), bet);
		assert_noop!(OcwDemo::submit_signed(Origin::signed(non_settler), acct, 990000000000, false), admin::Error::<Test>::NotSettler);
	})
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> <Test as system::Trait>::AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}