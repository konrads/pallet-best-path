use lite_json::json::JsonValue;
use sp_std::{vec::Vec, str};
use sp_runtime::SaturatedConversion;

/// Parse the price from the given JSON string using `lite-json`.
///
/// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
pub fn parse_price(price_str: &str, target_currency: &[u8], scale: u32) -> Option<u128> {
	let val = lite_json::parse_json(price_str);
	let price = match val.ok()? {
		JsonValue::Object(obj) => {
			let (_, v) = obj.into_iter().find(|(k, _)| k.iter().copied().eq(target_currency.iter().map(|b| *b as char)))?;
			match v {
				JsonValue::Number(number) => number,
				_ => return None,
			}
		},
		_ => return None,
	};

	let exp = price.fraction_length.saturating_sub(2);
	Some(price.integer as u128 * 10_u128.pow(scale) + (price.fraction as u128 * 10_u128.pow(scale-2) / 10_u128.pow(exp)) as u128)
}

/// Concat 4 &[u8]'s together
/// 
/// FIXME: add magic for varargs/method signature overwrite (1 concat function to take many params)
pub fn concat4(bin1: &[u8], bin2: &[u8], bin3: &[u8], bin4: &[u8]) -> Vec<u8> {
	bin1.iter().chain(bin2).chain(bin3).chain(bin4).copied().collect()
}

/// Check if tolerance breaches the diff
pub fn breaches_tolerance(old: u128, new: u128, tolerance: u32) -> bool {
	let delta = if old > new {
		1_000_000_u128.saturating_mul((old - new).saturated_into::<u128>()) / old.saturated_into::<u128>()
	} else {
        1_000_000_u128.saturating_mul((new - old).saturated_into::<u128>()) / old.saturated_into::<u128>()
	};
    // log::info!("##### breaches_tolerance: old: {}, new: {}, delta: {}, tolerance: {}", old, new, delta, tolerance);
	delta > tolerance as u128
}
#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    use super::*;

    #[test]
    fn test_concat4() {
        assert_eq!(b"".to_vec(),     concat4(b"", b"", b"", b""));
        assert_eq!(b"1234".to_vec(), concat4(b"1", b"2", b"3", b"4"));
        assert_eq!(b"14".to_vec(),   concat4(b"1", b"", b"", b"4"));
    }

    #[test]
    fn test_parse_price() {
        let payload = r#"{"BTC": 45, "USDT": 12.789, "ETH": 89.000001, "SHIBZELDA": 0.00000007978}"#;
        assert_eq!(Some(12_789_000_000_000_u128), parse_price(payload, b"USDT", 12));  // FIXME: should round up to 1208?
        assert_eq!(Some(89_000_001_000_000_u128), parse_price(payload, b"ETH", 12));
        assert_eq!(Some(45_000_000_000_000_u128), parse_price(payload, b"BTC", 12));
        assert_eq!(Some(            79_780_u128), parse_price(payload, b"SHIBZELDA", 12));
        assert_eq!(None,                          parse_price(r#"{"USDT": abc}"#, b"USDT", 12));
        assert_eq!(None,                          parse_price(r#""USDT": 12"#, b"USDT", 12));
    }

    #[test]
    fn test_breaches_tolerance() {
        assert!(! breaches_tolerance(1_000_000, 1_000_001, 1));
        assert!(! breaches_tolerance(1_000_005, 1_000_001, 4));
        assert!(breaches_tolerance(1_002, 1_000, 1_000));
        assert!(breaches_tolerance(1_002, 1_008, 5_000));
    }
}
