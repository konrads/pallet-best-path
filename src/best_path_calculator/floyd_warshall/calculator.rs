use sp_std::{collections::btree_map::BTreeMap, collections::btree_set::BTreeSet, vec, vec::Vec};
use crate::types::*;
use crate::BestPathCalculator;
use super::algo;
use sp_std::convert::TryInto;
use sp_std::result::{Result, Result::*};

impl From<algo::PathCalculationError> for CalculatorError {
    fn from(err: algo::PathCalculationError) -> Self {
        match err {
            algo::PathCalculationError::NegativeCyclesError => CalculatorError::NegativeCyclesError
        } 
    }
}

const PRECISION: f64 = 1_000_000_000_000.0;

pub struct FloydWarshallCalculator {}
impl<C: Currency, A: Amount> BestPathCalculator<C, A> for FloydWarshallCalculator {
    /// Wraps Floyd-Warshall's algorithm that uses indexes from/into BestPathCalculator data structures
	fn calc_best_paths(pairs_and_prices: &[(ProviderPair<C>, A)]) -> Result<BTreeMap<Pair<C>, PricePath<C, A>>, CalculatorError> {
        // get unique and indexed currencies and providers
        let currencies = pairs_and_prices.iter().flat_map(|(ProviderPair { pair: Pair{source, target}, .. }, ..)| vec![source, target].into_iter())
            .collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let providers = pairs_and_prices.iter().map(|(ProviderPair { provider, .. }, ..)| provider)
            .collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let currencies_by_ind = currencies.iter().enumerate().map(|(i, &x)| (x, i)).collect::<BTreeMap<_, _>>();
        let providers_by_ind = providers.iter().enumerate().map(|(i, &x)| (x, i)).collect::<BTreeMap<_, _>>();

        // construct the graph for Floyd-Warshall lib
        let mut graph = Vec::new();
        for &c in currencies.iter() {
            for (pp, cost) in pairs_and_prices {
                if c == &pp.pair.source {
                    graph.push(algo::Edge {
                        pair:        algo::Pair{source: currencies_by_ind[&pp.pair.source], target: currencies_by_ind[&pp.pair.target]},
                        provider:    providers_by_ind[&pp.provider],
                        cost:        TryInto::<u128>::try_into(*cost).map_err(|_| CalculatorError::ConversionError)? as f64 / PRECISION
                    });
                }
            }
        }

        // run Floyd-Warshall for all combinations of currencies in the graph
        let res = algo::longest_paths_mult(&graph)?;
        let res_map = res.into_iter().map(|(algo::Pair{source, target}, algo::Path{total_cost, edges})| {
            let pair = Pair{source: currencies[source].clone(), target: currencies[target].clone()};
            let total_cost_u128 = (total_cost * PRECISION) as u128;
            let path = PricePath{ total_cost: total_cost_u128.try_into().ok().unwrap(), steps: edges.into_iter().map(|algo::Edge{pair: algo::Pair{source, target, ..}, provider, cost}|
                PathStep{pair: Pair{source: currencies[source].clone(), target: currencies[target].clone()}, provider: providers[provider].clone(), cost: ((cost * PRECISION) as u128).try_into().ok().unwrap()}).collect()
            };
            (pair, path)
        }).collect::<BTreeMap<_, _>>();
        Ok(res_map)
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_PROVIDER: Provider = Provider::CRYPTOCOMPARE;

    #[test]
    fn test_real_life_graph() {
        /*
        Test prices generated via:
        curl https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USDT  # {"USDT":35997.42}
        curl https://min-api.cryptocompare.com/data/price?fsym=USDT&tsyms=BTC  # {"BTC":0.00002778}
        curl https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USDT  # {"USDT":2384.99}
        curl https://min-api.cryptocompare.com/data/price?fsym=USDT&tsyms=ETH  # {"ETH":0.0004194}
        curl https://min-api.cryptocompare.com/data/price?fsym=BNB&tsyms=USDT  # {"USDT":364.19}
        curl https://min-api.cryptocompare.com/data/price?fsym=USDT&tsyms=BNB  # {"BNB":0.002746}
        curl https://min-api.cryptocompare.com/data/price?fsym=DOT&tsyms=USDT  # {"USDT":17.43}
        curl https://min-api.cryptocompare.com/data/price?fsym=USDT&tsyms=DOT  # {"DOT":0.05737}
        curl https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=ETH   # {"ETH":15.09}
        curl https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=BTC   # {"BTC":0.06627}
        curl https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=BNB   # {"BNB":6.548}
        curl https://min-api.cryptocompare.com/data/price?fsym=BNB&tsyms=ETH   # {"ETH":0.1527}
        */
        let in_graph = vec![
            ("BTC".to_owned(),  "USDT".to_owned(), MOCK_PROVIDER, 5997.42),
            ("USDT".to_owned(), "BTC".to_owned(),  MOCK_PROVIDER, 0.00002777),
            ("ETH".to_owned(),  "USDT".to_owned(), MOCK_PROVIDER, 2384.99),
            ("USDT".to_owned(), "ETH".to_owned(),  MOCK_PROVIDER, 0.0004192),
            ("BNB".to_owned(),  "USDT".to_owned(), MOCK_PROVIDER, 364.19),
            ("USDT".to_owned(), "BNB".to_owned(),  MOCK_PROVIDER, 0.002745),
            ("DOT".to_owned(),  "USDT".to_owned(), MOCK_PROVIDER, 17.43),
            ("USDT".to_owned(), "DOT".to_owned(),  MOCK_PROVIDER, 0.05737),
            ("BTC".to_owned(),  "ETH".to_owned(),  MOCK_PROVIDER, 15.09),
            ("ETH".to_owned(),  "BTC".to_owned(),  MOCK_PROVIDER, 0.06626),
            ("ETH".to_owned(),  "BNB".to_owned(),  MOCK_PROVIDER, 6.548),
            ("BNB".to_owned(),  "ETH".to_owned(),  MOCK_PROVIDER, 0.1527),
        ].into_iter().map(|(source, target, provider, cost)| (ProviderPair{pair: Pair{source: source.as_str().as_bytes().to_vec(), target: target.as_str().as_bytes().to_vec()}, provider}, (cost * PRECISION) as u128)).collect::<Vec<_>>();
        let res_out = FloydWarshallCalculator::calc_best_paths(&in_graph).unwrap().into_iter().collect::<Vec<(_, _)>>()
            .into_iter().map(|(p, pp)|(
                String::from_utf8(p.source).unwrap(),
                String::from_utf8(p.target).unwrap(),
                pp.total_cost as f64 / PRECISION,
                pp.steps.into_iter().map(|PathStep{pair: Pair{source, target}, provider, cost}| (
                    String::from_utf8(source).unwrap(),
                    String::from_utf8(target).unwrap(),
                    provider,
                    cost as f64 / PRECISION,
                )).collect::<Vec<(String, String, Provider, f64)>>())
            ).collect::<Vec<(String, String, f64, Vec<(String, String, Provider, f64)>)>>();
        assert_eq!(
            vec![
                ("BNB".to_owned(), "BNB".to_owned(), 1.0,                vec![]),
                ("BNB".to_owned(), "BTC".to_owned(), 0.010117902,        vec![("BNB".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 0.1527), ("ETH".to_owned(), "BTC".to_owned(), MOCK_PROVIDER, 0.06626)]),
                ("BNB".to_owned(), "DOT".to_owned(), 20.8935803,         vec![("BNB".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 364.19), ("USDT".to_owned(), "DOT".to_owned(), MOCK_PROVIDER, 0.05737)]),
                ("BNB".to_owned(), "ETH".to_owned(), 0.1527,             vec![("BNB".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 0.1527)]),
                ("BNB".to_owned(), "USDT".to_owned(), 364.19,            vec![("BNB".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 364.19)]),
                ("BTC".to_owned(), "BNB".to_owned(), 98.80932,           vec![("BTC".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 15.09), ("ETH".to_owned(), "BNB".to_owned(), MOCK_PROVIDER, 6.548)]),
                ("BTC".to_owned(), "BTC".to_owned(), 1.0,                vec![]),
                ("BTC".to_owned(), "DOT".to_owned(), 2064.717563366999,  vec![("BTC".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 15.09), ("ETH".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 2384.99), ("USDT".to_owned(), "DOT".to_owned(), MOCK_PROVIDER, 0.05737)]),
                ("BTC".to_owned(), "ETH".to_owned(), 15.09,              vec![("BTC".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 15.09)]),
                ("BTC".to_owned(), "USDT".to_owned(), 35989.49909999999, vec![("BTC".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 15.09), ("ETH".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 2384.99)]),
                ("DOT".to_owned(), "BNB".to_owned(), 0.04784535,         vec![("DOT".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 17.43), ("USDT".to_owned(), "BNB".to_owned(), MOCK_PROVIDER, 0.002745)]),
                ("DOT".to_owned(), "BTC".to_owned(), 0.000484139026,     vec![("DOT".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 17.43), ("USDT".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 0.0004192), ("ETH".to_owned(), "BTC".to_owned(), MOCK_PROVIDER, 0.06626)]),
                ("DOT".to_owned(), "DOT".to_owned(), 1.0,                vec![]),
                ("DOT".to_owned(), "ETH".to_owned(), 0.007306656,        vec![("DOT".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 17.43), ("USDT".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 0.0004192)]),
                ("DOT".to_owned(), "USDT".to_owned(), 17.43,             vec![("DOT".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 17.43)]),
                ("ETH".to_owned(), "BNB".to_owned(), 6.548,              vec![("ETH".to_owned(), "BNB".to_owned(), MOCK_PROVIDER, 6.548)]),
                ("ETH".to_owned(), "BTC".to_owned(), 0.06626,            vec![("ETH".to_owned(), "BTC".to_owned(), MOCK_PROVIDER, 0.06626)]),
                ("ETH".to_owned(), "DOT".to_owned(), 136.826876299999,   vec![("ETH".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 2384.99), ("USDT".to_owned(), "DOT".to_owned(), MOCK_PROVIDER, 0.05737)]),
                ("ETH".to_owned(), "ETH".to_owned(), 1.0,                vec![]),
                ("ETH".to_owned(), "USDT".to_owned(), 2384.99,           vec![("ETH".to_owned(), "USDT".to_owned(), MOCK_PROVIDER, 2384.99)]),
                ("USDT".to_owned(), "BNB".to_owned(), 0.002745,          vec![("USDT".to_owned(), "BNB".to_owned(), MOCK_PROVIDER, 0.002745)]),
                ("USDT".to_owned(), "BTC".to_owned(), 2.7776192e-5,      vec![("USDT".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 0.0004192), ("ETH".to_owned(), "BTC".to_owned(), MOCK_PROVIDER, 0.06626)]),
                ("USDT".to_owned(), "DOT".to_owned(), 0.05737,           vec![("USDT".to_owned(), "DOT".to_owned(), MOCK_PROVIDER, 0.05737)]),
                ("USDT".to_owned(), "ETH".to_owned(), 0.0004192,         vec![("USDT".to_owned(), "ETH".to_owned(), MOCK_PROVIDER, 0.0004192)]),
                ("USDT".to_owned(), "USDT".to_owned(), 1.0,              vec![])
            ],
            res_out);
    }
}