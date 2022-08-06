use best_path::prelude::{Currency, Provider, Amount, PricePath};
use crate::types::ProviderPairOperation;

/// Trait representing basic, non whitelisted operations, such as submissions of monitored pairs and fetching of price path.
pub trait BestPath<C: Currency, A: Amount, P: Provider> {
    fn submit_monitored_pairs(operations: Vec<ProviderPairOperation<C, P>>);
    fn get_price_path(source: C, target: C) -> Option<PricePath<C, A, P>>;
}