pub mod crypto_compare;
mod crypto_compare_tests;
use crate::{PriceProviderId, PriceProvider, PriceProviderErr};
use sp_std::convert::AsRef;

const SCALE: u32 = 12;

/// Dynamic implementation of the best path calculator.
/// 
/// Re-implement as new providers are added!
pub struct PriceOracleImpl {}
impl PriceProvider<u128, PriceProviderId> for PriceOracleImpl {
	fn get_price<C: AsRef<[u8]>>(oracle_id: &PriceProviderId, source: C, target: C) -> Result<u128, PriceProviderErr> {
		match oracle_id {
			PriceProviderId::CRYPTOCOMPARE => crypto_compare::get_price(source.as_ref(), target.as_ref(), SCALE),
		}
	}
}
