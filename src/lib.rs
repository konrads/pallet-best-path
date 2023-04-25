#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

use core::fmt::Debug;
use codec::{Decode, Encode, FullCodec};
use frame_support::{pallet_prelude::*, traits::Get, traits::tokens::Balance, transactional};
use frame_system::{
	self,
	offchain::{AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer, SigningTypes},
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	offchain::{http, Duration, storage::StorageValueRef, storage_lock::{StorageLock, Time}},
	traits::IdentifyAccount,
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};
use sp_std::{
	collections::btree_set::BTreeSet,
	convert::TryInto,
	iter::Iterator,
	vec::Vec,
	vec,
	str,
	prelude::*
};
pub mod types;
use types::*;
mod utils;
use utils::*;
pub mod traits;
use traits::{BestPath as BestPathTrait};
pub use best_path;
use best_path::{BestPathCalculator, prelude::*};
pub mod heap;
pub mod price_provider;
use scale_info::{prelude::{string::String, format}, TypeInfo};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

/// Duration for getting the OCW lock, in millis
pub const OCW_LOCK_DURATION: u64 = 100;

/// Application identifier for crypto keys of this module
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"bepa");

/// Transaction tag to deduplicate OCW transactions
pub const TX_TAG: &[u8] = b"best_path";

/// OCW off-chain lookup
pub const OCW_WORKER_LOCK: &[u8] = b"best_path::ocw_lock";

/// Key for the next offchain trigger.
pub const NEXT_OFFCHAIN_TRIGGER_BLOCK: &[u8] = b"best_path::next_offchain_trigger_block";

/// Available Price Providers
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Ord, PartialOrd)]
#[allow(clippy::upper_case_acronyms)]
pub enum PriceProviderId {
    CRYPTOCOMPARE
}

/// Implementor of price fetching mechanism, per provider
pub trait PriceProviderHub<A: Amount, P: Eq> {
	/// For a given provider, source & target currency, fetch the pair price
	fn get_price<C: AsRef<[u8]>>(provider: &P, source: C, target: C) -> Result<A, PriceProviderErr>;
}

#[derive(Debug)]
pub enum PriceProviderErr {
	TransportErr(http::Error),
}

impl From<http::Error> for PriceProviderErr {
    fn from(err: http::Error) -> Self {
        PriceProviderErr::TransportErr(err)
    }
}

/// Signed payload of unsigned transaction that carries best path changes, nonce and public key.
///
/// Changes map source/target currency to an Option of a best path. If the Option is Some(), price update is requested, if None, removal.
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, RuntimeDebug)]
pub struct BestPathChangesPayload<Public, BlockNumber, C: Currency, A: Amount, P: Provider> {
	changes: Vec<(C, C, Option<PricePath<C, A, P>>)>,
	nonce: u64,
	block_number: BlockNumber,
	public: Public,
}

impl<T: SigningTypes, C: Currency + Encode, A: Amount + Encode, P: Provider + Encode> SignedPayload<T> for BestPathChangesPayload<T::Public, T::BlockNumber, C, A, P> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
    use core::convert::TryFrom;
	app_crypto!(sr25519, KEY_TYPE);

	pub struct AuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for AuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	/// DoubleMap of trading path by source & target currencies
	#[pallet::storage]
	pub(super) type BestPaths<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::Currency /* source currency */, Blake2_128Concat, T::Currency /* target currency */, PricePath<T::Currency, T::Amount, T::Provider> /* best path */>;

	/// Map to keep track of source & target currencies we wish to monitor
	#[pallet::storage]
	pub(super) type MonitoredPairs<T: Config> = StorageMap<_, Blake2_128Concat, ProviderPair<T::Currency, T::Provider>, (), OptionQuery>;  // membership in the map indicates price is to be fetched, Some(()) - existence of the latest price

	/// Map storing whitelisted accounts that are whitelisted to sign the payload of unsigned transactions.
	#[pallet::storage]
	pub(super) type WhitelistedOffchainAuthorities<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	/// Nonce used for replay protection of unsigned transactions
	#[pallet::storage]
	pub(super) type UnsignedTxNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Submission of best prices onchain
		/// \[{source_currency, target_currency, new_cost, operation}\]
		BestPricesSubmitted(Vec<(T::Currency, T::Currency, T::Amount, Operation)>),

		/// Addition of a monitored currency/provider pair.
		/// \[source_currency, target_currency, provider, operation\]
		MonitoredPairsSubmitted(Vec<(T::Currency, T::Currency, T::Provider, Operation)>),

		/// Addition of offchain authority account.
		/// \[account_id\]
		WhitelistedOffchainAuthorityAdded(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Indicates currency/provider pair not found
		PricePairNotFoundError,
		/// Indicates stale unsigned transaction, possibly due to replay attack
		StaleUnsignedTxError,
	}
	
	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The overarching dispatch call type.
		type Call: From<Call<Self>>;

		/// Currency type
		type Currency: Currency + Conversions + AsRef<[u8]> + FullCodec + TypeInfo + Debug;

		/// Currency type
		type Provider: Provider + FullCodec + TypeInfo + Debug;

		/// Type indicating amounts: price, cost, balance
		type Amount: Balance;

		/// Dynamic implementation of the best path calculator
		type BestPathCalculator: BestPathCalculator<Self::Currency, Self::Amount, Self::Provider>;

		/// Dynamic implementation of the price oracle, per provider
		type PriceProviderHub: PriceProviderHub<Self::Amount, Self::Provider>;

		/// Benchmarking weight type
		type WeightInfo: WeightInfo;

		// Configuration parameters

		/// Delay between successful submissions of OCW best prices, used for rate limiting.
		#[pallet::constant]
		type OffchainTriggerDelay: Get<Self::BlockNumber>;

		/// How long we allow unsigned transaction to remain in the pool before deemed Stale?
		#[pallet::constant]
		type MaxTxPoolStayTime: Get<Self::BlockNumber>;

		/// Priority of unsigned transactions, parametrizable for this pallet
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

		/// Tolerance of price change in best paths, expressed in 1/1,000,000, filters out insignificant price changes
		#[pallet::constant]
		type PriceChangeTolerance: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Off-chain Worker entry point.
		///
		/// First checks whether can act upon this block, if so, attempts to obtain the lock, if successful, fetches and updates the best paths.
		/// Once done, bumps the next trigger block storage by the delay amount, and lock is released.
		fn offchain_worker(block_number: T::BlockNumber) {
			if Self::should_trigger_offchain(block_number) {
				// obtain the OCW lock
				let lock_expiration = Duration::from_millis(OCW_LOCK_DURATION);
				let mut lock = StorageLock::<'static, Time>::with_deadline(OCW_WORKER_LOCK, lock_expiration);

				match lock.try_lock() {
					Ok(_guard) => {
						if let Err(e) = Self::fetch_prices_and_update_best_paths(block_number) {
							log::error!("OCW price fetching error: {}", e);
						}
						// bump the offchain trigger block
						let next_trigger = block_number + T::OffchainTriggerDelay::get();
						StorageValueRef::persistent(NEXT_OFFCHAIN_TRIGGER_BLOCK).set(&next_trigger);
					},
					Err(e) => log::warn!("OCW failed to obtain OCW lock due to {:?}", e)
				};
			}
		}
	}

	/// BestPath extrinsic API.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Submit best path prices calculated offchain.
		///
		/// Adds/removes best price paths, as per `best_path_change_payload.changes`.
		/// Dedups by provider_pair, picking last operation only.
		/// This call should only get through once its payload has been validated to be signed by a whitelisted authority.
		/// Uses nonce for replay protection, bumping it upon the success.
		/// Issues an event listing all supplied changes.
		#[pallet::weight(T::WeightInfo::submit_monitored_pairs(best_path_change_payload.changes.len()))]
		#[transactional]
		pub fn ocw_submit_best_paths_changes(
			origin: OriginFor<T>,
			best_path_change_payload: BestPathChangesPayload<T::Public, T::BlockNumber, T::Currency, T::Amount, T::Provider>,
			_signature: T::Signature,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let current_nonce = <UnsignedTxNonce<T>>::get();
			ensure!(current_nonce == best_path_change_payload.nonce, Error::<T>::StaleUnsignedTxError);

			let mut event_payload = vec![];
			for (source, target, ref mut new_path) in best_path_change_payload.changes {
				BestPaths::<T>::mutate_exists(
					source.clone(),
					target.clone(),
					|old_path| {
						match new_path.take() {
							Some(path) => {
								let total_cost = path.total_cost;
								*old_path = Some(path);
								log::info!("Onchain: adding/changing price onchain for {} -> {}: {:?}", source.to_str(), target.to_str(), total_cost);
								event_payload.push((source, target, total_cost, Operation::Add));
							}
							None => if old_path.take().is_some() {
								log::info!("Onchain: removing price onchain: {} -> {}", source.to_str(), target.to_str());
								event_payload.push((source, target, T::Amount::default(), Operation::Del));
							}
						}
					}
				);
			}

			<UnsignedTxNonce<T>>::set(current_nonce + 1);
			// only issue event if mods were made
			if !event_payload.is_empty() {
				Self::deposit_event(Event::BestPricesSubmitted(event_payload));
			}
			Ok(Pays::No.into())
	    }

		/// Submit monitored price pair adds/deletes.
		///
		/// Root operation, requires sudo.
		/// Validates that all operations are mapped to a valid provider, then each operation is added/deleted to monitored pairs map.
		/// Operations to be added are upserted, operations to be deleted are removed if exist, skipped otherwise.
		#[pallet::weight(T::WeightInfo::submit_monitored_pairs(operations.len()))]
		#[transactional]
		pub fn submit_monitored_pairs(
			origin: OriginFor<T>,
			operations: Vec<ProviderPairOperation<T::Currency, T::Provider>>) -> DispatchResult {
			ensure_root(origin)?;
			Self::do_submit_monitored_pairs(operations);
			Ok(())
		}

		// Add authorities (OCW) that are allowed to submit signed payloads of unsigned transactions
		#[pallet::weight(T::WeightInfo::add_whitelisted_offchain_authority())]
		pub fn add_whitelisted_offchain_authority(
			origin: OriginFor<T>,
			offchain_authority: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			Self::deposit_event(Event::WhitelistedOffchainAuthorityAdded(offchain_authority.clone()));
			<WhitelistedOffchainAuthorities<T>>::insert(&offchain_authority, ());
			Ok(())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned calls to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::ocw_submit_best_paths_changes {
				best_path_change_payload: payload,
				signature,
			} = call
			{
				let signature_valid = SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
				if !signature_valid {
					log::error!("OCW rejected transaction due to invalid signature");
					return InvalidTransaction::BadProof.into()
				}

				let current_block = <frame_system::Pallet<T>>::block_number();
				if payload.block_number + T::MaxTxPoolStayTime::get() < current_block {
					// transaction was in pool for too long
					return InvalidTransaction::Stale.into();
				}

				let account_id = payload.public.clone().into_account();
				if !WhitelistedOffchainAuthorities::<T>::contains_key(&account_id) {
					log::error!("OCW rejected transaction due to signer not on the offchain authority whitelist: {:?}", account_id);
					return InvalidTransaction::BadProof.into();
				}

				ValidTransaction::with_tag_prefix("BestPathWorker")
					.priority(T::UnsignedPriority::get())
					.and_provides((TX_TAG, current_block))
					.longevity(5)  // transaction is only valid for next 5 blocks. After that it's revalidated by the pool.
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_submit_monitored_pairs(operations: Vec<ProviderPairOperation<T::Currency, T::Provider>>) {
		// dedupe operations, keep latest per provider_pair, preserving order
		let mut operations2 = vec![];
		let mut uniques = BTreeSet::new();
		for op in operations.into_iter().rev() {
			if uniques.insert(op.provider_pair.clone()) {
				operations2.push(op);
			}
		}
		let operations = operations2.into_iter().rev();

		// add/delete monitored pairs
		let mut event_payload = vec![];
		for ProviderPairOperation{provider_pair, operation} in operations {
			<MonitoredPairs<T>>::mutate_exists(provider_pair.clone(), |exists_indicator| {
				let ProviderPair { pair: Pair { source, target }, provider } = provider_pair;
				match operation {
					Operation::Add => if exists_indicator.is_none() {
						*exists_indicator = Some(());
						event_payload.push((source, target, provider, operation));
					},
					Operation::Del => {
						if exists_indicator.take().is_some() {
							event_payload.push((source, target, provider, operation));
						}
					},
				}
			});
		}

		// only issue event if mods were made
		if !event_payload.is_empty() {
			Self::deposit_event(Event::MonitoredPairsSubmitted(event_payload));
		}
	}

	/// Determine if can trigger OCW based on the next offchain trigger block delay mechanism
	fn should_trigger_offchain(block_number: T::BlockNumber) -> bool {
		match StorageValueRef::persistent(NEXT_OFFCHAIN_TRIGGER_BLOCK).get::<T::BlockNumber>() {
			Ok(Some(next_offchain_trigger)) if block_number >= next_offchain_trigger => {
				log::info!("Offchain trigger block encountered!");
				true
			}
			Ok(Some(_)) => {
				log::info!("Offchain trigger attempted too soon!");
				false
			}
			Ok(None) => {
				log::info!("Offchain trigger block initiated!");
				true
			}
			_ => {
				log::info!("Offchain trigger not readable!");
				false
			}
		}
	}

	/// A helper function to fetch the price, sign payload and send an unsigned transaction
	fn fetch_prices_and_update_best_paths(block_number: T::BlockNumber) -> Result<(), String> {
		let fetched_pairs = <MonitoredPairs<T>>::iter_keys()
			.filter_map(|pp| {
				T::PriceProviderHub::get_price(&pp.provider, &pp.pair.source, &pp.pair.target).ok().map(|res| (pp, res))
			})
			.collect::<Vec<(_, _)>>();

		if fetched_pairs.is_empty() {
			log::debug!("Offchain: no price pairs to update!");
			return Ok(())
		}

		let new_best_paths = T::BestPathCalculator::calc_best_paths(&fetched_pairs).map_err(|e| format!("Failed to calculate best prices due to {:?}", e))?;

		// select the best path differences
		// - elements changed at all and outside of acceptable tolerance
		// - no longer existing elements
		// - newly added elements
		let mut changes = vec![];
		let tolerance = T::PriceChangeTolerance::get();
		for (source, target, old_price_path) in BestPaths::<T>::iter() {  // FIXME: iterating ovr *all* of BestPaths...
			let pair = Pair{ source: source.clone(), target: target.clone() };
			match new_best_paths.get(&pair) {
				Some(new_price_path) => {
					let old_total_cost: u128 = old_price_path.total_cost.try_into().map_err(|_| "failed to convert old_price_path.total_cost")?;
					let new_total_cost: u128 = new_price_path.total_cost.try_into().map_err(|_| "failed to convert new_price_path.total_cost")?;
					if breaches_tolerance(old_total_cost, new_total_cost, tolerance) {
						log::debug!("Offchain: adding price change for {:?} -> {:?} in excess of tolerance: {:?}: {:?} -> {:?}", pair.source.to_str(), pair.target.to_str(), tolerance, old_total_cost, new_total_cost);
						changes.push((source, target, Some(new_price_path.clone())));
					} else {
						log::debug!("Offchain: skipping price change for {:?} -> {:?} within tolerance of {:?}: {:?} -> {:?}", pair.source.to_str(), pair.target.to_str(), tolerance, old_total_cost, new_total_cost);
					}
				}
				None => log::debug!("Offchain: no price fetched for {:?} -> {:?}", pair.source.to_str(), pair.target.to_str()),
			}
		}
		for (Pair{source, target}, new_price_path) in new_best_paths.into_iter() {
			if ! BestPaths::<T>::contains_key(&source, &target) {
				log::debug!("Offchain: adding new price: for {:?} -> {:?}: {:?}", source.to_str(), target.to_str(), &new_price_path.total_cost);
				changes.push((source, target, Some(new_price_path.clone())))
			}
		}

		if changes.is_empty() {
			log::info!("Offchain: detected no price changes that breached tolerance level")
		} else {
			let (_, result) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| BestPathChangesPayload {changes: changes.clone(), nonce: <UnsignedTxNonce<T>>::get(), block_number, public: account.public.clone() },
					|payload, signature| Call::ocw_submit_best_paths_changes {
						best_path_change_payload: payload,
						signature,
					},
				)
				.ok_or("No local accounts accounts available")?;
			result.map_err(|()| "Unable to submit transaction")?;

			log::info!("Offchain: updated best paths!");
		}

		Ok(())
	}
}

impl<T: Config> BestPathTrait<T::Currency, T::Amount, T::Provider> for Pallet<T> {
    fn submit_monitored_pairs(operations: Vec<ProviderPairOperation<T::Currency, T::Provider>>) {
		Self::do_submit_monitored_pairs(operations);
	}
    fn get_price_path(source: T::Currency, target: T::Currency) -> Option<PricePath<T::Currency, T::Amount, T::Provider>> {
		BestPaths::<T>::get(&source, &target)
	}
}
