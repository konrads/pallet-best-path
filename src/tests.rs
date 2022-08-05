#![cfg(test)]

use crate::*;

use codec::Decode;
use frame_support::{assert_ok, assert_noop};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
use sp_runtime::traits::BadOrigin;
use crate::mock::{Test, BTC_CURRENCY, ETH_CURRENCY, MOCK_PROVIDER, BOGUS_CURRENCY, USDT_CURRENCY, Call, Origin, Extrinsic, Event, Fixture, new_test_ext, last_event, new_test_ext_with_keystore};

#[test]
fn test_submit_monitored_pairs_ok() {
	new_test_ext().execute_with(|| {
		// validate deletion of non existent entry - should succeed, but expect no storage change/event
		assert_ok!(Fixture::submit_monitored_pairs(Origin::root(), vec![
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: BOGUS_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, operation: Operation::Del},
		]));
		assert_eq!(last_event(), None);
		assert_eq!(0, MonitoredPairs::<Test>::iter_keys().count());

		// validate additions
		assert_ok!(Fixture::submit_monitored_pairs(Origin::root(), vec![
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, operation: Operation::Add},
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: ETH_CURRENCY.to_vec()},  provider: MOCK_PROVIDER}, operation: Operation::Del},
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: ETH_CURRENCY.to_vec()},  provider: MOCK_PROVIDER}, operation: Operation::Add},  // deduped, replaces the Del above
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: ETH_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, operation: Operation::Add},
		]));
		assert_eq!(vec![
				ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER},
				ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: ETH_CURRENCY.to_vec()},  provider: MOCK_PROVIDER},
				ProviderPair{pair: Pair{source: ETH_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER},
			].into_iter().collect::<BTreeSet<ProviderPair<Vec<u8>, PriceProviderId>>>(),
			MonitoredPairs::<Test>::iter_keys().collect::<BTreeSet<ProviderPair<Vec<u8>, PriceProviderId>>>()
		);
		assert_eq!(last_event(), Some(Event::Fixture(crate::Event::<Test>::MonitoredPairsSubmitted(vec![
			(BTC_CURRENCY.to_vec(), USDT_CURRENCY.to_vec(), MOCK_PROVIDER, Operation::Add),
			(BTC_CURRENCY.to_vec(), ETH_CURRENCY.to_vec(),  MOCK_PROVIDER, Operation::Add),
			(ETH_CURRENCY.to_vec(), USDT_CURRENCY.to_vec(), MOCK_PROVIDER, Operation::Add),
		]))));

		// validate deletions
		assert_ok!(Fixture::submit_monitored_pairs(Origin::root(), vec![
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()},  provider: MOCK_PROVIDER}, operation: Operation::Del},
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: ETH_CURRENCY.to_vec()},   provider: MOCK_PROVIDER}, operation: Operation::Add},
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: ETH_CURRENCY.to_vec()},   provider: MOCK_PROVIDER}, operation: Operation::Del},  // deduped, replaces the Add above
			// Note: ETH - USDT still remains
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: ETH_CURRENCY.to_vec(), target: BOGUS_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, operation: Operation::Del},  // expect it skipped in the event
		]));
		assert_eq!(vec![
				ProviderPair{pair: Pair{source: ETH_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER},
			],
			MonitoredPairs::<Test>::iter_keys().collect::<Vec<ProviderPair<Vec<u8>, PriceProviderId>>>()
		);
		assert_eq!(last_event(), Some(Event::Fixture(crate::Event::<Test>::MonitoredPairsSubmitted(vec![
			(BTC_CURRENCY.to_vec(), USDT_CURRENCY.to_vec(), MOCK_PROVIDER, Operation::Del),
			(BTC_CURRENCY.to_vec(), ETH_CURRENCY.to_vec(),  MOCK_PROVIDER, Operation::Del),
		]))));

		// validate mixture of additions and deletions
		assert_ok!(Fixture::submit_monitored_pairs(Origin::root(), vec![
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: ETH_CURRENCY.to_vec(),   target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, operation: Operation::Del},
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BOGUS_CURRENCY.to_vec(), target: ETH_CURRENCY.to_vec()},  provider: MOCK_PROVIDER}, operation: Operation::Del}, // expect it skipped in the event
			ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: USDT_CURRENCY.to_vec(),  target: ETH_CURRENCY.to_vec()},  provider: MOCK_PROVIDER}, operation: Operation::Add},
		]));
		assert_eq!(vec![
				ProviderPair{pair: Pair{source: USDT_CURRENCY.to_vec(), target: ETH_CURRENCY.to_vec()}, provider: MOCK_PROVIDER},
			],
			MonitoredPairs::<Test>::iter_keys().collect::<Vec<ProviderPair<Vec<u8>, PriceProviderId>>>()
		);
		assert_eq!(last_event(), Some(Event::Fixture(crate::Event::<Test>::MonitoredPairsSubmitted(vec![
			(ETH_CURRENCY.to_vec(),  USDT_CURRENCY.to_vec(), MOCK_PROVIDER, Operation::Del),
			(USDT_CURRENCY.to_vec(), ETH_CURRENCY.to_vec(),  MOCK_PROVIDER, Operation::Add),
		]))));
	});
}

#[test]
fn test_submit_monitored_pairs_errors() {
	new_test_ext().execute_with(|| {
		// validate incorrect origins
		assert_noop!(
			Fixture::submit_monitored_pairs(Origin::signed(sp_core::sr25519::Public([0_u8; 32])), vec![ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, operation: Operation::Add}]),
			BadOrigin);
		assert_eq!(0, MonitoredPairs::<Test>::iter_keys().count());

		assert_noop!(
			Fixture::submit_monitored_pairs(Origin::none(), vec![ProviderPairOperation{provider_pair: ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, operation: Operation::Add}]),
			BadOrigin);
		assert_eq!(0, MonitoredPairs::<Test>::iter_keys().count());
	});
}

#[test]
fn test_ocw_submit_best_paths_changes() {
	let (t, _, _, public_key) = &mut new_test_ext_with_keystore();
	let payload = BestPathChangesPayload {
		nonce: 0,
		block_number: 1,
		changes: vec![(BTC_CURRENCY.to_vec(), USDT_CURRENCY.to_vec(), Some(PricePath{total_cost: 50000, steps: vec![]}))],
		public: <Test as SigningTypes>::Public::from(*public_key),
	};
	let payload2 = payload.clone();

	t.execute_with(|| {
		let signature = 
			<BestPathChangesPayload<
				<Test as SigningTypes>::Public,
				<Test as frame_system::Config>::BlockNumber,
				<Test as Config>::Currency,
				<Test as Config>::Amount,
				<Test as Config>::Provider,
			> as SignedPayload<Test>>::sign::<crypto::TestAuthId>(&payload).unwrap();
		assert_ok!(Fixture::ocw_submit_best_paths_changes(Origin::none(), payload.clone(), signature.clone()));
		assert_noop!(Fixture::ocw_submit_best_paths_changes(Origin::none(), payload, signature), Error::<Test>::StaleUnsignedTxError);
	});

	// verify with bogus
	let (t, _, _, _) = &mut new_test_ext_with_keystore();
	t.execute_with(|| {
		let signature = 
			<BestPathChangesPayload<
				<Test as SigningTypes>::Public,
				<Test as frame_system::Config>::BlockNumber,
				<Test as Config>::Currency,
				<Test as Config>::Amount,
				<Test as Config>::Provider,
			> as SignedPayload<Test>>::sign::<crypto::TestAuthId>(&payload2).unwrap();
		assert_noop!(Fixture::ocw_submit_best_paths_changes(Origin::root(), payload2, signature), BadOrigin);
	});
}

#[test]
fn test_fetch_prices_and_update_best_paths() {
	let (t, _, pool_state, public_key) = &mut new_test_ext_with_keystore();
	let payload = BestPathChangesPayload {
		nonce: 0,
		block_number: 1,
		changes: vec![(BTC_CURRENCY.to_vec(), USDT_CURRENCY.to_vec(), Some(PricePath{total_cost: 50000, steps: vec![]}))],
		public: <Test as SigningTypes>::Public::from(*public_key),
	};

	// verify extrinsic was called
	t.execute_with(|| {
		MonitoredPairs::<Test>::insert(
			ProviderPair{pair: Pair{source: BTC_CURRENCY.to_vec(), target: USDT_CURRENCY.to_vec()}, provider: MOCK_PROVIDER}, 
			());

		assert!(Fixture::fetch_prices_and_update_best_paths(1).is_ok());
		let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let decoded_tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(decoded_tx.signature, None);
		if let Call::Fixture(crate::Call::ocw_submit_best_paths_changes {
			best_path_change_payload: body,
			signature,
		}) = decoded_tx.call
		{
			assert_eq!(body, payload);

			let signature_valid =
				<BestPathChangesPayload<
					<Test as SigningTypes>::Public,
					<Test as frame_system::Config>::BlockNumber,
					<Test as Config>::Currency,
					<Test as Config>::Amount,
					<Test as Config>::Provider,
				> as SignedPayload<Test>>::verify::<crypto::TestAuthId>(&payload, signature);

			assert!(signature_valid);
		}
	});
}

#[test]
fn test_should_trigger_offchain() {
	let (t, _, _, _) = &mut new_test_ext_with_keystore();
	t.execute_with(|| {
		StorageValueRef::persistent(NEXT_OFFCHAIN_TRIGGER_BLOCK).set(&10_u64);
		assert!(! Fixture::should_trigger_offchain(9));
		assert!(Fixture::should_trigger_offchain(10));
		assert!(Fixture::should_trigger_offchain(11));
	});
}
