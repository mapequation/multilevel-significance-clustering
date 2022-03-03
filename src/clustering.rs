use std::collections::HashSet;

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
    let mut rng = StdRng::seed_from_u64(seed);
    let mut core = HashSet::new();
    let node_ids = module.iter().collect::<Vec<_>>();

    // Randomize start
    for &node in module.iter() {
        if rng.gen::<bool>() {
            core.insert(node);
        }
    }

    let num_partitions_to_exclude = ((1.0 - conf) * modules.len() as f32) as usize;
    let penalty_weight = 10 * module.len() as i64;

    let (mut score, mut penalty) =
        calc_score_penalty(module, modules, penalty_weight, num_partitions_to_exclude);

    const NUM_ITERATIONS: usize = 10;
    let mut best_score = -1;

    let mut search = true;
    while search {
        let mut temperature = 1.0;

        loop {
            let mut switches = 0;

            for _ in 0..NUM_ITERATIONS {
                // Select random node
                let node_id = *node_ids.iter().choose(&mut rng).unwrap();
                let in_conf_set = core.contains(node_id);

                // Remove or add the node
                if in_conf_set {
                    core.remove(node_id);
                } else {
                    core.insert(*node_id);
                }

                let was_in_conf_set = in_conf_set;

                let (new_score, new_penalty) =
                    calc_score_penalty(&core, modules, penalty_weight, num_partitions_to_exclude);

                let s = score - penalty_weight * penalty;
                let s_new = new_score - penalty_weight * new_penalty;
                let delta_s = (s_new - s) as f64;

                if (delta_s / temperature).exp() > rng.gen::<f64>() {
                    score = new_score;
                    penalty = new_penalty;
                    switches += 1;
                } else {
                    // Revert the change
                    if was_in_conf_set {
                        core.insert(*node_id);
                    } else {
                        core.remove(node_id);
                    }
                }

                if penalty == 0 && score > best_score {
                    best_score = score;
                }
            }

            temperature *= 0.99;

            if switches == 0 {
                break;
            }
        }

        if best_score > 0 {
            search = false;
        }
    }

    core
}

fn calc_score_penalty(
    module: &HashSet<NodeId>,
    modules: &[&HashSet<NodeId>],
    penalty_weight: i64,
    num_partitions_to_exclude: usize,
) -> (i64, i64) {
    let mut score = 0;
    let mut penalty = 0;
    let mut worst_scores = Vec::new();
    let num_partitions_to_keep = modules.len() - num_partitions_to_exclude;

    // Calculate penalty without worst results
    for &module2 in modules.iter() {
        let intersection = module.intersection(module2).count();
        let difference = module.difference(module2).count();

        let module_score = intersection as i64 - penalty_weight * difference as i64;

        worst_scores.push(module_score);
        worst_scores.sort_unstable();

        let worst_score = if worst_scores.len() > num_partitions_to_keep {
            let best = worst_scores.pop().unwrap();
            let worst = *worst_scores.first().unwrap_or(&best);
            worst
        } else {
            i64::MIN
        };

        if module_score > worst_score {
            score += intersection as i64;
            penalty += difference as i64;
        }
    }

    (score, penalty)
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;

    #[test]
    fn test_calc_score_penalty() {
        let module = (0..10).collect::<HashSet<_>>();

        let (score, penalty) = calc_score_penalty(&module, &[&module], 1, 0);
        assert_eq!(score, 10);
        assert_eq!(penalty, 0);

        let modules = vec![
            (0..10).collect::<HashSet<_>>(),
            (0..10).collect::<HashSet<_>>(),
            (0..10).collect::<HashSet<_>>(),
            (0..10).collect::<HashSet<_>>(),
            (1..11).collect::<HashSet<_>>(),
        ];

        let (score, penalty) =
            calc_score_penalty(&module, &modules.iter().collect::<Vec<_>>(), 1, 1);
        assert_eq!(score, 40);
        assert_eq!(penalty, 0);

        let (score, penalty) =
            calc_score_penalty(&module, &modules.iter().collect::<Vec<_>>(), 1, 0);
        assert_eq!(score, 49);
        assert_eq!(penalty, 1);
    }

    #[bench]
    fn bench_calc_score_penalty(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        const NUM_NODES: u32 = 1000;
        const NUM_PARTITIONS: u32 = 100;

        let module = (0..NUM_NODES).collect::<HashSet<_>>();
        let mut modules = Vec::new();

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

        b.iter(|| {
            calc_score_penalty(
                &module,
                &modules.iter().collect::<Vec<_>>(),
                penalty_weight,
                num_partitions_to_exclude,
            );
        });
    }
}
