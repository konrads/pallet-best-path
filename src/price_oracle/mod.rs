pub mod crypto_compare;
mod crypto_compare_tests;
use crate::{types::*, PriceOracle, PriceOracleErr};
use sp_std::convert::AsRef;


const SCALE: u32 = 12;

/// Dynamic implementation of the best path calculator.
/// 
/// Re-implement as new providers are added!
pub struct PriceOracleImpl {}
impl PriceOracle<u128> for PriceOracleImpl {
	fn get_price<C: AsRef<[u8]>>(provider: &Provider, source: C, target: C) -> Result<u128, PriceOracleErr> {
		match provider {
			Provider::CRYPTOCOMPARE => crypto_compare::get_price(source.as_ref(), target.as_ref(), SCALE),
		}
	}
}
