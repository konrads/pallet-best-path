pub mod crypto_compare;
mod crypto_compare_tests;
use crate::{PriceProviderId, PriceProviderHub, PriceProviderErr};
use sp_std::convert::AsRef;

const SCALE: u32 = 12;

/// Default implementation of price provider, aggregates functionality of fetching per different providers.
pub struct DefaultPriceProviderHub {}
impl PriceProviderHub<u128, PriceProviderId> for DefaultPriceProviderHub {
	fn get_price<C: AsRef<[u8]>>(oracle_id: &PriceProviderId, source: C, target: C) -> Result<u128, PriceProviderErr> {
		match oracle_id {
			PriceProviderId::CRYPTOCOMPARE => crypto_compare::get_price(source.as_ref(), target.as_ref(), SCALE),
		}
	}
}
