use hashbrown::{HashMap, HashSet};
use std::collections::BTreeMap;

use crate::{Network, NodeId};

pub fn get_most_similar_modules(
    first: &Network,
    rest: &BTreeMap<usize, Network>,
) -> HashMap<String, BTreeMap<usize, String>> {
    let mut most_similar_modules = HashMap::new();

    for module1 in first.modules.values() {
        let most_similar_to_module = most_similar_modules
            .entry(module1.module_id.clone())
            .or_insert_with(BTreeMap::new);

        rest.iter().for_each(|(network_id, network)| {
            let mut distances = network
                .modules
                .values()
                .map(|module2| {
                    (
                        &module2.module_id,
                        jaccard_distance(&module1.nodes, &module2.nodes),
                    )
                })
                .collect::<Vec<_>>();

            distances.sort_unstable_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

            if let Some(distance) = distances.first() {
                most_similar_to_module.insert(*network_id, distance.0.clone());
            }
        });
    }

    most_similar_modules
}

fn jaccard_distance(m1: &HashSet<NodeId>, m2: &HashSet<NodeId>) -> f32 {
    let jaccard_index = match m1.union(m2).count() {
        0 => 0.0,
        union => {
            let intersection = m1.intersection(m2).count() as f32;
            intersection / (union as f32)
        }
    };

    1.0 - jaccard_index
}
