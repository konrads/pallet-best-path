use sp_std::{collections::btree_map::BTreeMap, collections::btree_set::BTreeSet, vec, vec::Vec};
use sp_std::cmp::{Eq, Ord, PartialEq, PartialOrd, Ordering};
use sp_std::result::{Result, Result::*};
use sp_runtime::RuntimeDebug;
#[allow(unused_imports)]
use num_traits::Float;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub (crate) struct Pair {
    pub (crate) source: usize,
    pub (crate) target: usize,
}

#[derive(Copy, Clone, Debug)]
pub (crate) struct Edge {
    pub (crate) pair: Pair,
    pub (crate) provider: usize,
    pub (crate) cost: f64,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Edge) -> bool {
        self.pair == other.pair && self.provider == other.provider && self.cost == other.cost
    }
}

impl Eq for Edge {}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pair.cmp(&other.pair)
            .then_with(|| self.provider.cmp(&other.provider))
            .then_with(|| self.cost.partial_cmp(&other.cost).unwrap())
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub (crate) struct Path {
    pub (crate) total_cost: f64,
    pub (crate) edges: Vec<Edge>,
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost == other.total_cost && self.edges == other.edges
    }
}

impl Eq for Path {}

impl Path {
    pub (crate) fn add(&mut self, edge: &Edge) {
        self.total_cost += edge.cost;
        self.edges.push(*edge);
    }
}

#[derive(RuntimeDebug)]
pub enum PathCalculationError {
    NegativeCyclesError,
}

/// Gets longest paths, as per: https://www.coursera.org/lecture/algorithms-on-graphs/currency-exchange-reduction-to-shortest-paths-cw8Tm
/// - switch weights to log2(w) to allow for shortest_paths() addition of weights
/// - negate log2(w) in order to reuse shortest_paths()
/// 
/// Formula, given: x*y = 2^(log2(x) + log2(y))
/// maximizing x*y is equivalent to maximizing log2(x) + log2(y)
/// ie. can convert weights x => log2(x), y => log2(y)
/// Negate the weight for compatibility with shortest_path()
pub (crate) fn longest_paths_mult(edges: &[Edge]) -> Result<BTreeMap<Pair, Path>, PathCalculationError> {
    let edges = unique_cheapest_edges(edges, Ordering::Greater);
    // record the original weights
    let weight_map: BTreeMap<(Pair, usize), f64> = edges.iter().map(|e|((e.pair, e.provider), e.cost)).collect();
    // map weights x => log2(x)
    let edges_with_log_weights: Vec<Edge> = edges.iter().map(|e|Edge{cost: -e.cost.log2(), ..*e}).collect() ;

    // run longest path algo
    let res = shortest_paths(&edges_with_log_weights);

    // map weights back to x, recalculate total_cost
    res.map(|res_map|{
        res_map.iter().map(|(pair, path)|{
            let edges_iter = path.edges.iter().map(|e|{
                Edge{cost: weight_map[&(e.pair, e.provider)], ..*e}
            });
            let total_cost = edges_iter.clone().fold(1.0, |acc, e| acc * e.cost);
            let path = Path{total_cost, edges: edges_iter.collect::<Vec<_>>()};
            (*pair, path)
        }).collect::<BTreeMap<Pair, Path>>()
    })
}

pub (crate) fn shortest_paths(edges: &[Edge]) -> Result<BTreeMap<Pair, Path>, PathCalculationError> {
    floyd_warshall_shortest_paths(&unique_cheapest_edges(edges, Ordering::Less))
}

// Floyd-Warshall shortest path algorithm.
// Utilizes simple data structures with range of usize
fn floyd_warshall_shortest_paths(edges: &[Edge]) -> Result<BTreeMap<Pair, Path>, PathCalculationError> {
    let mut vertices: BTreeSet<usize> = BTreeSet::new();
    let mut edges_by_pair: BTreeMap<Pair, Edge> = BTreeMap::new();
    let mut paths_by_pair: BTreeMap<Pair, Path> = BTreeMap::new();

    for e in edges.iter() {
        vertices.insert(e.pair.source);
        vertices.insert(e.pair.target);
        edges_by_pair.insert(Pair{source: e.pair.source, target: e.pair.target}, *e);
        paths_by_pair.entry(Pair{source: e.pair.source, target: e.pair.target}).or_insert(Path{total_cost: 0.0, edges: vec![]}).add(e);
    }

    let mut matrix: BTreeMap<Pair, Path> = BTreeMap::new();
    // initial setup based on edges
    for v in vertices.iter() {
        matrix.insert(Pair{source: *v, target: *v}, Path{total_cost: 0.0, edges: vec![]});
    }
    for e in edges.iter() {
        matrix.insert(Pair{source: e.pair.source, target: e.pair.target}, Path{total_cost: e.cost, edges: vec![*e]});
    }

    // recalculate the matrix as per: https://youtu.be/oNI0rf2P9gE?t=817
    // A[i,j] = min(A[i,j], A[i,k] + A[k,j])
    for k in vertices.iter() {
        for i in vertices.iter() {
            for j in vertices.iter() {
                let ij_cost = match matrix.get(&Pair{source: *i, target: *j}) {
                    Some(ij) => ij.total_cost,
                    None           => f64::MAX  // suggests infinite cost
                };
                let (ik_cost, ik_edges) = match matrix.get(&Pair{source: *i, target: *k}) {
                    Some(ik) => (ik.total_cost, ik.edges.clone()),
                    None           => (f64::MAX, vec![])  // suggests infinite cost
                };
                let (kj_cost, kj_edges) = match matrix.get(&Pair{source: *k, target: *j}) {
                    Some(kj) => (kj.total_cost, kj.edges.clone()),
                    None           => (f64::MAX, vec![])  // suggests infinite cost
                };

                if ik_cost + kj_cost != f64::MAX && ij_cost > ik_cost + kj_cost {
                    let mut new_ij_edges = ik_edges;
                    new_ij_edges.extend(kj_edges);
                    matrix.insert(Pair{source: *i, target: *j},Path{total_cost: ik_cost + kj_cost, edges: new_ij_edges});
                }
            }
        }
    }

    // check for negative cycles
    for i in vertices {
        let contains_negative_cycle = matrix.get(&Pair{source: i, target: i}).unwrap().total_cost < 0.0;
        if contains_negative_cycle {
            return Err(PathCalculationError::NegativeCyclesError)
        }
    }

    Ok(matrix)
}

fn unique_cheapest_edges(edges: &[Edge], ordering: Ordering) -> Vec<Edge> {
    let mut edges_by_pair: BTreeMap<(usize, usize), Edge> = BTreeMap::new();
    for e in edges.iter() {
        edges_by_pair.entry((e.pair.source, e.pair.target))
            .and_modify(|old_costed| if old_costed.cost.partial_cmp(&e.cost).unwrap() == ordering { *old_costed = *e })
            .or_insert_with(|| *e);
    }
    edges_by_pair.values().cloned().collect::<Vec<_>>()
}

/// Tests make use of example and algorithm presented in: https://www.youtube.com/watch?v=oNI0rf2P9gE&ab_channel=AbdulBari
///
///         8            3
///    0. *----------------* 1.
///    * *                   |
///  2 |  5 \                |
///    |       \             |
///    |          \          |
///    |             \       |
///  7 |                \    | 2
///    *                   \ *
///    3. *----------------- 2.
///         1
///
/// Expected result (as per youtube):
///      0.  1.  2.  3.
///  0.  0   3   5   6
///  1.  5   0   2   3
///  2.  3   6   0   1
///  3.  2   5   7   0
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_edges() {
        let set = unique_cheapest_edges(&vec![
            Edge{pair: Pair{source: 1, target: 2}, provider:   3, cost: 1.0},
            Edge{pair: Pair{source: 1, target: 2}, provider:  33, cost: 10.0},
            Edge{pair: Pair{source: 1, target: 2}, provider: 333, cost: 5.0},
        ], Ordering::Less);

        assert_eq!(1, set.len());
        assert_eq!(10.0, set.iter().next().unwrap().cost);
    }

    /// Graph
    ///      .5     2        .25    4
    /// 0. *----------* 1. *----------* 2.
    #[test]
    fn test_simple() {
        let edges = vec![
            Edge{pair: Pair{source: 0, target: 1}, provider: 1, cost: 2.0},
            Edge{pair: Pair{source: 1, target: 0}, provider: 1, cost: 0.5},
            Edge{pair: Pair{source: 1, target: 2}, provider: 1, cost: 4.0},
            Edge{pair: Pair{source: 2, target: 1}, provider: 1, cost: 0.25},
        ];
        let res = longest_paths_mult(&edges).unwrap().into_iter().collect::<Vec<_>>();
        assert_eq!(
            res,
            vec![
                (Pair { source: 0, target: 0 }, Path { total_cost: 1.0,   edges: vec![] }),
                (Pair { source: 0, target: 1 }, Path { total_cost: 2.0,   edges: vec![Edge { pair: Pair{source: 0, target: 1}, provider: 1, cost: 2.0 }] }),
                (Pair { source: 0, target: 2 }, Path { total_cost: 8.0,   edges: vec![Edge { pair: Pair { source: 0, target: 1 }, provider: 1, cost: 2.0 }, Edge { pair: Pair { source: 1, target: 2 }, provider: 1, cost: 4.0 }] }),
                (Pair { source: 1, target: 0 }, Path { total_cost: 0.5,   edges: vec![Edge { pair: Pair { source: 1, target: 0 }, provider: 1, cost: 0.5 }] }),
                (Pair { source: 1, target: 1 }, Path { total_cost: 1.0,   edges: vec![] }),
                (Pair { source: 1, target: 2 }, Path { total_cost: 4.0,   edges: vec![Edge { pair: Pair { source: 1, target: 2 }, provider: 1, cost: 4.0 }] }),
                (Pair { source: 2, target: 0 }, Path { total_cost: 0.125, edges: vec![Edge { pair: Pair { source: 2, target: 1 }, provider: 1, cost: 0.25 }, Edge { pair: Pair { source: 1, target: 0 }, provider: 1, cost: 0.5 }] }),
                (Pair { source: 2, target: 1 }, Path { total_cost: 0.25,  edges: vec![Edge { pair: Pair { source: 2, target: 1 }, provider: 1, cost: 0.25 }] }),
                (Pair { source: 2, target: 2 }, Path { total_cost: 1.0,   edges: vec![] })
            ],
        );
    }

    #[test]
    fn test_youtube() {
        let edges = vec![
            Edge{pair: Pair{source: 0, target: 1}, provider: 1, cost: 3.0},
            Edge{pair: Pair{source: 0, target: 3}, provider: 1, cost: 7.0},
            Edge{pair: Pair{source: 0, target: 3}, provider: 10, cost: 6.5},  // ignore!!!

            Edge{pair: Pair{source: 1, target: 0}, provider: 1, cost: 8.0},
            Edge{pair: Pair{source: 1, target: 2}, provider: 1, cost: 2.0},
            
            Edge{pair: Pair{source: 2, target: 0}, provider: 1, cost: 5.0},
            Edge{pair: Pair{source: 2, target: 3}, provider: 1, cost: 1.0},

            Edge{pair: Pair{source: 3, target: 0}, provider: 1, cost: 2.0},
        ];
        let res = shortest_paths(&edges).unwrap();
        let costs = (0_usize..=3).map(|source|
            (0_usize..=3).map(|target|
                res.get(&Pair{source, target}).map(|p|p.total_cost)
            ).collect::<Vec<_>>()
        ).collect::<Vec<_>>();
        assert_eq!(vec![
            vec![Some(0.0), Some(3.0), Some(5.0), Some(6.0)],
            vec![Some(5.0), Some(0.0), Some(2.0), Some(3.0)],
            vec![Some(3.0), Some(6.0), Some(0.0), Some(1.0)],
            vec![Some(2.0), Some(5.0), Some(7.0), Some(0.0)],
            ], costs);
        assert_eq!(vec![
            Edge { pair: Pair { source: 0, target: 1 }, provider: 1, cost: 3.0 },
            Edge { pair: Pair { source: 1, target: 2 }, provider: 1, cost: 2.0 },
            Edge { pair: Pair { source: 2, target: 3 }, provider: 1, cost: 1.0 }], res[&Pair{source: 0, target: 3}].edges);
        assert_eq!(vec![
            Edge { pair: Pair { source: 3, target: 0 }, provider: 1, cost: 2.0 },], res[&Pair{source: 3, target: 0}].edges);
        }

    #[test]
    fn test_youtube_negative_cycle() {
        let edges = vec![
            Edge{pair: Pair{source: 0, target: 1}, provider: 1, cost: 3.0},
            Edge{pair: Pair{source: 0, target: 3}, provider: 1, cost: 7.0},
            Edge{pair: Pair{source: 0, target: 3}, provider: 10, cost: 6.0},  // ignore!!!

            Edge{pair: Pair{source: 1, target: 0}, provider: 1, cost: 8.0},
            Edge{pair: Pair{source: 1, target: 2}, provider: 1, cost: 2.0},
            
            Edge{pair: Pair{source: 2, target: 0}, provider: 1, cost: -6.0},  // causes negative cycle
            Edge{pair: Pair{source: 2, target: 3}, provider: 1, cost: 1.0},

            Edge{pair: Pair{source: 3, target: 0}, provider: 1, cost: 2.0},
        ];
        assert!(shortest_paths(&edges).is_err());
    }

    #[test]
    /// Few tests to ensure comparability of floats
    fn test_f64() {
        assert!(f64::MAX  > 1.0);
        assert!(-f64::MAX < 1.0);
        assert!(! (f64::MAX > f64::MAX + 1.0));  // neither > or < than MAX
        assert!(! (f64::MAX < f64::MAX + 1.0));
        assert_eq!(f64::MAX, f64::MAX + 1.0);
        assert_eq!(f64::MAX, f64::MAX - 1.0);
        assert!(!(f64::MAX == 8.0 + 7.0));
        assert!(f64::MAX != 8.0 + 7.0);
        assert!(f64::MAX > 8.0 + 7.0);
        assert!(! (f64::MAX <= 8.0 + 7.0));

        let max_u128 = u128::MAX;
        let max_u128_as_f64 = (max_u128 as f64) * 10.0 + 10.0;
        let max_u128_as_f64_as_u128 = max_u128_as_f64 as u128;
        assert_eq!(max_u128, max_u128_as_f64_as_u128);
    }
}
