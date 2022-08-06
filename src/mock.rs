#![cfg(test)]

use crate as best_path;
use crate::*;

use frame_support::parameter_types;
use parking_lot::RwLock;
use sp_core::{
    offchain::{testing, testing::PoolState, OffchainDbExt, OffchainWorkerExt, TransactionPoolExt},
    sr25519::Signature,
    H256,
};
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
    RuntimeAppPublic,
};
use core::convert::TryFrom;
use sp_std::vec::Vec;
use std::sync::Arc;

pub(crate) const MOCK_PROVIDER: PriceProviderId = PriceProviderId::CRYPTOCOMPARE;
pub(crate) const BTC_CURRENCY: &[u8] = b"BTC";
pub(crate) const ETH_CURRENCY: &[u8] = b"ETH";
pub(crate) const USDT_CURRENCY: &[u8] = b"USDT";
pub(crate) const BOGUS_CURRENCY: &[u8] = b"__BOGUS_CURRENCY__";
pub struct MockProvider {}
impl PriceProvider<u64, PriceProviderId> for MockProvider {
    fn get_price<C: AsRef<[u8]>>(
        _provider: &PriceProviderId,
        _source: C,
        _target: C,
    ) -> Result<u64, PriceProviderErr> {
        Ok(50_000)
    }
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// For testing the module, we construct a mock runtime.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Fixture: best_path::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sp_core::sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub(crate) type Extrinsic = TestXt<Call, ()>;
pub(crate) type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

parameter_types! {
    pub const OffchainTriggerDelay: u64 = 1;
    pub const MaxTxPoolStayTime: u64 = 1;
    pub const UnsignedPriority: u64 = 1 << 20;
    pub const PriceChangeTolerance: u32 = 1;
}

impl Config for Test {
    type Event = Event;
    type AuthorityId = crypto::TestAuthId;
    type Call = Call;
    // BestPath specific
    type OffchainTriggerDelay = OffchainTriggerDelay;
    type MaxTxPoolStayTime = MaxTxPoolStayTime;
    type UnsignedPriority = UnsignedPriority;
    type PriceChangeTolerance = PriceChangeTolerance;
    type BestPathCalculator = best_path::prelude::noop_calculator::NoBestPathCalculator;
    type PriceProvider = MockProvider;
    type Currency = Vec<u8>;
    type Provider = PriceProviderId;
    type Amount = u64;
    type WeightInfo = ();
}

/// Return text externalities after first block, as events only get issued after the first block
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = sp_io::TestExternalities::default();
    t.execute_with(|| System::set_block_number(1));
    t
}

pub fn new_test_ext_with_keystore() -> (
    sp_io::TestExternalities,
    testing::TestOffchainExt,
    Arc<RwLock<PoolState>>,
    sp_core::sr25519::Public,
) {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, _offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = KeyStore::new();
    SyncCryptoStore::sr25519_generate_new(
        &keystore,
        crate::crypto::Public::ID,
        Some(&format!("{}/hunter1", PHRASE)),
    )
    .unwrap();
    let public_key = SyncCryptoStore::sr25519_public_keys(&keystore, crate::crypto::Public::ID)
        .get(0)
        .unwrap()
        .clone();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain.clone()));
    t.register_extension(OffchainDbExt::new(offchain.clone()));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));
    (t, offchain, pool_state, public_key)
}

pub(crate) fn last_event() -> Option<Event> {
    System::events().last().map(|e| e.event.clone())
}
