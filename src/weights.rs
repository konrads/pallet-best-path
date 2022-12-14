//! Autogenerated weights for pallet_best_path
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-02-08, STEPS: `1`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: None, WASM-EXECUTION: Interpreted, CHAIN: None, DB CACHE: 128

// Executed Command:
// target/debug/node-template
// benchmark
// --extrinsic
// *
// --pallet
// pallet_best_path
// --wasm-execution
// interpreted-i-know-what-i-do
// --output
// ./pallets/best_path/src/weights.rs
// --template=frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_best_path.
pub trait WeightInfo {
	fn add_price_pair_nonexisting() -> Weight;
	fn add_price_pair_existing() -> Weight;
	fn remove_price_pair() -> Weight;
	fn submit_monitored_pairs(_i: usize, ) -> Weight;
	fn ocw_submit_best_paths_changes(_i: usize, ) -> Weight;
	fn add_whitelisted_offchain_authority() -> Weight;
}

/// Weights for pallet_best_path using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn add_price_pair_nonexisting() -> Weight {
		(155_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn add_price_pair_existing() -> Weight {
		(51_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_price_pair() -> Weight {
		(156_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn submit_monitored_pairs(_i: usize, ) -> Weight {
		(20_827_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(250 as Weight))			.saturating_add(T::DbWeight::get().writes(250 as Weight))
	}
	fn ocw_submit_best_paths_changes(_i: usize, ) -> Weight {
		(143_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn add_whitelisted_offchain_authority() -> Weight {
		(138_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn add_price_pair_nonexisting() -> Weight {
		(155_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn add_price_pair_existing() -> Weight {
		(51_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn remove_price_pair() -> Weight {
		(156_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn submit_monitored_pairs(_i: usize, ) -> Weight {
		(20_827_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(250 as Weight))			.saturating_add(RocksDbWeight::get().writes(250 as Weight))
	}
	fn ocw_submit_best_paths_changes(_i: usize, ) -> Weight {
		(143_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn add_whitelisted_offchain_authority() -> Weight {
		(138_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}

