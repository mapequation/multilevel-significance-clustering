use hashbrown::{HashMap, HashSet};
use std::collections::BTreeMap;

use crate::{Network, NodeId};

pub fn get_most_similar_modules(
    first: &Network,
    rest: &BTreeMap<usize, Network>,
) -> HashMap<String, BTreeMap<usize, String>> {
    first
        .modules
        .values()
        .map(|module1| {
            let most_similar = rest
                .iter()
                .filter_map(|(&network_id, network)| {
                    network
                        .modules
                        .values()
                        .map(|module2| {
                            (
                                &module2.module_id,
                                jaccard_distance(&module1.nodes, &module2.nodes),
                            )
                        })
                        .min_by(|(_, distance1), (_, distance2)| {
                            distance1.partial_cmp(distance2).unwrap()
                        })
                        .and_then(|(module_id, ..)| Some((network_id, module_id.clone())))
                })
                .collect();

            (module1.module_id.clone(), most_similar)
        })
        .collect()
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
