use super::*;

#[allow(unused)]
use crate::{Pallet as BestPath, PriceProviderId};
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

const MOCK_PROVIDER_ID: PriceProviderId = PriceProviderId::CRYPTOCOMPARE;

benchmarks! {
	add_price_pair_nonexisting {
		let source = T::Currency::from_vecu8(b"BTC".to_vec());
		let target = T::Currency::from_vecu8(b"ETH".to_vec());
	}: add_price_pair(RawOrigin::Root, source.clone(), target.clone(), MOCK_PROVIDER_ID) 
	verify {
		assert!(MonitoredPairs::<T>::contains_key(ProviderPair{ pair: Pair{ source: source.clone(), target: target.clone() }, provider: MOCK_PROVIDER_ID}));
	}

	add_price_pair_existing {
		let source = T::Currency::from_vecu8(b"ACA".to_vec());
		let target = T::Currency::from_vecu8(b"KAR".to_vec());
		MonitoredPairs::<T>::insert(ProviderPair{ pair: Pair{ source: source.clone(), target: target.clone() }, provider: MOCK_PROVIDER_ID }, ());
	}: add_price_pair(RawOrigin::Root, source.clone(), target.clone(), MOCK_PROVIDER_ID)
	verify {
		assert!(MonitoredPairs::<T>::contains_key(ProviderPair{ pair: Pair{ source: source.clone(), target: target.clone() }, provider: MOCK_PROVIDER_ID}));
	}

	delete_price_pair {
		let source = T::Currency::from_vecu8(b"ACA".to_vec());
		let target = T::Currency::from_vecu8(b"KAR".to_vec());
		MonitoredPairs::<T>::insert(ProviderPair{ pair: Pair{ source: source.clone(), target: target.clone() }, provider: MOCK_PROVIDER_ID }, ());
	}: _(RawOrigin::Root, source.clone(), target.clone(), MOCK_PROVIDER_ID)
	verify {
		assert!(! MonitoredPairs::<T>::contains_key(ProviderPair{ pair: Pair{ source: source.clone(), target: target.clone() }, provider: MOCK_PROVIDER_ID}));
	}

	submit_price_pairs {
		let i in 0 .. 250;
		let mut pairs = vec![];
		for j in 0..i {
			let k = j as u8;
			let op = if k % 2 == 0 { Operation::Add } else { Operation::Del };
			let source = T::Currency::from_vecu8(vec![k % 255_u8,     (k+1) % 255_u8, (k+2) % 255_u8]);
			let target = T::Currency::from_vecu8(vec![(k+1) % 255_u8, (k+2) % 255_u8, (k+3) % 255_u8]);
			pairs.push((
				source.clone(),
				target.clone(),
				MOCK_PROVIDER_ID,
				op.clone(),
			));
			if op == Operation::Del {
				MonitoredPairs::<T>::insert(ProviderPair{ pair: Pair{ source, target }, provider: MOCK_PROVIDER_ID }, ());
			}
		}
	}: submit_price_pairs(RawOrigin::Root, pairs)

	ocw_submit_best_paths_changes {
		// let i in 0 .. 250;
		// let mut changes = vec![];
		// let payload = BestPathChangesPayload{ changes: changes, nonce: 1, public: public } {
		// 	changes: Vec<(C, C, Option<PricePath<C, A>>)>,
		// 	nonce: u64,
		// 	public: Public,
		// }
		// let signature = ();
		// FIXME... complete!
	}: add_price_pair(RawOrigin::Root, T::Currency::from_vecu8(b"ACA".to_vec()), T::Currency::from_vecu8(b"KAR".to_vec()), MOCK_PROVIDER_ID)

	add_offchain_authority {
	}: add_price_pair(RawOrigin::Root, T::Currency::from_vecu8(b"ACA".to_vec()), T::Currency::from_vecu8(b"KAR".to_vec()), MOCK_PROVIDER_ID)

	impl_benchmark_test_suite!(BestPath, crate::mock::new_test_ext(), crate::mock::Test);
}