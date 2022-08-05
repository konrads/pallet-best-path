#![cfg(test)]

use crate::price_provider::crypto_compare::get_price;
use sp_core::offchain::{testing, OffchainWorkerExt};

#[test]
fn test_get_price() {
    let (offchain, state) = testing::TestOffchainExt::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));

    {
        let mut state = state.write();
        state.expect_request(testing::PendingRequest {
            method: "GET".into(),
            uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
            response: Some(br#"{"USD": 50000}"#.to_vec()),
            sent: true,
            ..Default::default()
        });

        state.expect_request(testing::PendingRequest {
            method: "GET".into(),
            uri: "https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD".into(),
            response: Some(br#"{"USD": 4000}"#.to_vec()),
            sent: true,
            ..Default::default()
        });

        state.expect_request(testing::PendingRequest {
            method: "GET".into(),
            uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=ETH".into(),
            response: Some(br#"{"ETH": 12500}"#.to_vec()),
            sent: true,
            ..Default::default()
        });
    }

    t.execute_with(|| {
        assert_eq!(get_price(b"BTC", b"USD", 12).unwrap(), 50_000_000000000000);
        assert_eq!(get_price(b"ETH", b"USD", 12).unwrap(),  4_000_000000000000);
        assert_eq!(get_price(b"BTC", b"ETH", 12).unwrap(), 12_500_000000000000);
    })
}
