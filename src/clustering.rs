use std::cmp::max;
use std::hash::Hash;

use crate::{HashMap, HashSet};
use itertools::Itertools;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};

use crate::NodeId;

pub fn get_significant_core(
    module: &HashSet<NodeId>,
    modules: &[&HashSet<NodeId>],
    conf: f32,
    seed: u64,
) -> HashSet<NodeId> {
    let (mut core, candidates) = {
        // Count the number of modules that each node is in
        let mut counts = module
            .iter()
            .map(|&node| (node, modules.iter().filter(|m| m.contains(&node)).count()))
            .collect::<HashMap<_, _>>();

        // Add all nodes that are present in all partitions
        let core = counts
            .iter()
            .filter_map(|(&node, &count)| {
                let in_all = count == modules.len();
                in_all.then_some(node)
            })
            .collect::<HashSet<_>>();

        // Remove all nodes that are present in all partitions
        counts.retain(|_, &mut count| 0 < count && count < modules.len());

        // Special case: if the counts are empty, all nodes are in the core
        if counts.is_empty() {
            return core;
        }

        // Nodes that are not present in all partitions
        let candidates = counts.keys().copied().collect::<Vec<_>>();

        (core, candidates)
    };

    let mut rng = StdRng::seed_from_u64(seed);

    // Randomize start
    for &node in candidates.iter() {
        if rng.gen::<bool>() {
            core.insert(node);
        }
    }

    let penalty_weight = 10 * module.len() as i64;

    let scorer = Scorer::new(penalty_weight, get_num_to_exclude(modules.len(), conf));

    let (mut score, mut penalty) = scorer.score(&core, modules);

    const MAX_OUTER_LOOPS: usize = 1000;
    const MAX_INNER_LOOPS: usize = 1000;
    let num_iterations: usize = max(module.len(), 100);

    let mut best_score = None;

    fn flip(set: &mut HashSet<NodeId>, node_id: NodeId, remove: bool) {
        if remove {
            set.remove(&node_id);
        } else {
            set.insert(node_id);
        }
    }

    for _ in 0..MAX_OUTER_LOOPS {
        let mut temperature = 1.0;

        for _ in 0..MAX_INNER_LOOPS {
            let mut switches = 0;

            for _ in 0..num_iterations {
                // Select random node
                let node_id = *candidates.iter().choose(&mut rng).unwrap();
                let remove = core.contains(&node_id);

                // Remove or add the node
                flip(&mut core, node_id, remove);

                let (new_score, new_penalty) = scorer.score(&core, modules);

                let delta_s = {
                    let s = score - penalty_weight * penalty;
                    let s_new = new_score - penalty_weight * new_penalty;
                    (s_new - s) as f64
                };

                // Always accept if delta_s is positive
                // Accept with some probability if negative
                if (delta_s / temperature).exp() > rng.gen::<f64>() {
                    score = new_score;
                    penalty = new_penalty;
                    switches += 1;
                } else {
                    // Revert the change
                    flip(&mut core, node_id, !remove);
                }

                if penalty == 0 && Some(score) > best_score {
                    best_score = Some(score)
                }
            }

            temperature *= 0.99;

            if switches == 0 {
                break;
            }
        }

        if best_score.is_some() {
            break;
        }
    }

    core
}

trait IntersectionDifference {
    fn intersection_difference_count(&self, other: &Self) -> (usize, usize);
}

impl<T> IntersectionDifference for HashSet<T>
where
    T: Eq + Hash,
{
    fn intersection_difference_count(&self, other: &Self) -> (usize, usize) {
        self.iter()
            .fold((0, 0), |(intersection, difference), item| {
                if other.contains(item) {
                    (intersection + 1, difference)
                } else {
                    (intersection, difference + 1)
                }
            })
    }
}

struct Scorer {
    penalty_weight: i64,
    num_partitions_to_exclude: usize,
}

impl Scorer {
    fn new(penalty_weight: i64, num_partitions_to_exclude: usize) -> Self {
        Self {
            penalty_weight,
            num_partitions_to_exclude,
        }
    }

    fn score(&self, module: &HashSet<NodeId>, modules: &[&HashSet<NodeId>]) -> (i64, i64) {
        // Sum the best scores and penalties
        modules
            .iter()
            .map(|module2| {
                let (score, penalty) = module.intersection_difference_count(module2);
                let (score, penalty) = (score as i64, penalty as i64);
                (score - self.penalty_weight * penalty, score, penalty)
            })
            .sorted_by_key(|(module_score, ..)| *module_score)
            .skip(self.num_partitions_to_exclude)
            .fold((0, 0), |(s, p), (_, score, penalty)| {
                (s + score, p + penalty)
            })
    }
}

fn get_num_to_exclude(num_partitions: usize, conf: f32) -> usize {
    ((1.0 - conf) * num_partitions as f32 + 0.5) as usize
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use super::*;

    #[test]
    fn test_num_to_exclude() {
        for i in 1..=10 {
            assert_eq!(get_num_to_exclude(i, 1.0), 0);
        }

        for i in 1..=9 {
            assert_eq!(get_num_to_exclude(i, 0.95), 0);
        }
        assert_eq!(get_num_to_exclude(10, 0.95), 1);

        for i in 1..=4 {
            assert_eq!(get_num_to_exclude(i, 0.9), 0);
        }
        for i in 5..=10 {
            assert_eq!(get_num_to_exclude(i, 0.9), 1);
        }

        for i in 1..=3 {
            assert_eq!(get_num_to_exclude(i, 0.85), 0);
        }
        for i in 4..=10 {
            assert_eq!(get_num_to_exclude(i, 0.85), 1);
        }

        for i in 1..=2 {
            assert_eq!(get_num_to_exclude(i, 0.8), 0);
        }
        for i in 3..=7 {
            assert_eq!(get_num_to_exclude(i, 0.8), 1);
        }
        for i in 8..=10 {
            assert_eq!(get_num_to_exclude(i, 0.8), 2);
        }
    }

    fn setup() -> (HashSet<NodeId>, Vec<HashSet<NodeId>>) {
        let module = (0..10).collect::<HashSet<_>>();

        let modules = vec![
            (0..10).collect::<HashSet<_>>(),
            (0..10).collect::<HashSet<_>>(),
            (1..11).collect::<HashSet<_>>(),
            (0..10).collect::<HashSet<_>>(),
            (0..10).collect::<HashSet<_>>(),
        ];

        (module, modules)
    }

    #[test]
    fn test_get_significant_core() {
        let (module, modules) = setup();

        assert_eq!(
            get_significant_core(&module, &modules.iter().collect::<Vec<_>>(), 0.95, 123),
            (1..10).collect::<HashSet<_>>()
        );
    }

    #[bench]
    fn bench_get_significant_core(b: &mut Bencher) {
        let (module, modules) = setup();

        b.iter(|| {
            get_significant_core(&module, &modules.iter().collect::<Vec<_>>(), 0.95, 123);
        });
    }

    #[test]
    fn test_calc_score() {
        let (module, modules) = setup();

        let (score, penalty) = Scorer::new(1, 0).score(&module, &[&module]);
        assert_eq!(score, 10);
        assert_eq!(penalty, 0);

        let (score, penalty) =
            Scorer::new(1, 1).score(&module, &modules.iter().collect::<Vec<_>>());
        assert_eq!(score, 40);
        assert_eq!(penalty, 0);

        let (score, penalty) =
            Scorer::new(1, 0).score(&module, &modules.iter().collect::<Vec<_>>());
        assert_eq!(score, 49);
        assert_eq!(penalty, 1);
    }

    #[bench]
    fn bench_score(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        const NUM_NODES: u32 = 1_000;
        const NUM_PARTITIONS: usize = 100;

        let module = (0..NUM_NODES).collect::<HashSet<_>>();
        let mut modules = Vec::with_capacity(NUM_PARTITIONS);

        for _ in 0..NUM_PARTITIONS {
            let mut module = module.clone();

            let num_remove: u32 = rng.gen_range(0..((NUM_NODES / 10) as u32));

            for _ in 0..num_remove {
                let node = *module.iter().choose(&mut rng).unwrap();
                module.remove(&node);
            }

            modules.push(module);
        }

        let num_partitions_to_exclude = ((1.0 - 0.95) * modules.len() as f32) as usize;
        let penalty_weight = 10 * module.len() as i64;
        let scorer = Scorer::new(penalty_weight, modules.len() - num_partitions_to_exclude);

        b.iter(|| scorer.score(&module, &modules.iter().collect::<Vec<_>>()));
    }
}
