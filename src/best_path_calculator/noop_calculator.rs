use sp_std::{collections::btree_map::BTreeMap, vec};
use crate::types::*;
use crate::BestPathCalculator;

pub struct NoBestPathCalculator {}
impl<C: Currency, A: Amount> BestPathCalculator<C, A> for NoBestPathCalculator {
	fn calc_best_paths(pairs_and_prices: &[(ProviderPair<C>, A)]) -> Result<BTreeMap<Pair<C>, PricePath<C, A>>, CalculatorError> {
		Ok(pairs_and_prices.iter().cloned().map(|(pp, price)| (Pair{source: pp.pair.source, target: pp.pair.target}, PricePath{total_cost: price, steps: vec![]})).collect())
	}
}
