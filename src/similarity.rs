use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashSet};

use crate::{Network, NodeId};

type Distance = (String, f32);

pub fn get_most_similar_modules(
    first: &Network,
    rest: &BTreeMap<usize, Network>,
) -> BTreeMap<String, BTreeMap<usize, Distance>> {
    let mut most_similar_modules = BTreeMap::new();

    for module1 in first.modules.values() {
        // println!("{:?}", module1);
        let most_similar_to_module = most_similar_modules
            .entry(module1.module_id.clone())
            .or_insert(BTreeMap::new());

        for (network_id, network) in rest.iter() {
            // println!("\tNetwork {}:", network_id);
            for module2 in network.modules.values() {
                let module_dist = (
                    module2.module_id.clone(),
                    jaccard_distance(&module1.nodes, &module2.nodes),
                );

                match most_similar_to_module.entry(*network_id) {
                    Entry::Vacant(entry) => {
                        entry.insert(module_dist);
                    }
                    Entry::Occupied(mut entry) => {
                        let old_d = entry.get_mut();
                        if module_dist.1 < old_d.1 {
                            *old_d = module_dist;
                        }
                    }
                };
            }
        }
    }

    most_similar_modules
}

fn jaccard_distance(m1: &HashSet<NodeId>, m2: &HashSet<NodeId>) -> f32 {
    let union = m1.union(m2).count();

    let jaccard_index = if union == 0 {
        0.0
    } else {
        let intersection = m1.intersection(m2).count() as f32;
        intersection / (union as f32)
    };

    1.0 - jaccard_index
}
