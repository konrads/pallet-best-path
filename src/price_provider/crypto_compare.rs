use sp_std::{vec::Vec, str};
use crate::utils::{concat4, parse_price};
use crate::PriceProviderErr;
use sp_runtime::offchain::{http, Duration};

pub fn get_price(source: &[u8], target: &[u8], scale: u32) -> Result<u128, PriceProviderErr> {
	let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));  // expiry = 2s
	let url_bin = concat4(b"https://min-api.cryptocompare.com/data/price?fsym=", source, b"&tsyms=", target);
	let url = str::from_utf8(&url_bin).map_err(|err| {
		log::error!("url utf8 parsing error: {:?}", err);
		http::Error::Unknown
	})?;
	let request = http::Request::get(url);
	let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
	let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
	if response.code != 200 {
		log::warn!("Unexpected status code: {}", response.code);
		return Err(http::Error::Unknown.into())
	}
	let body = response.body().collect::<Vec<u8>>();
	let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
		log::warn!("No UTF8 body");
		http::Error::Unknown
	})?;

	let price = match parse_price(body_str, target, scale) {
		Some(price) => Ok(price),
		None => {
			log::warn!("Unable to extract price from the response: {:?}", body_str);
			Err(http::Error::Unknown)
		},
	}?;

	log::info!("Got price: {}", price as f64 / scale as f64);

	Ok(price)
}