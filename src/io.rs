use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::num::ParseIntError;

use itertools::Itertools;

use crate::{HashMap, HashSet, Module, Network, NetworkId, NodeId};

pub fn read_input(in_file: &str) -> Result<BTreeMap<NetworkId, Network>, ParseIntError> {
    let mut networks = BTreeMap::new();

    for line in in_file.lines() {
        if line.starts_with('#') {
            continue;
        }

        let cols = line.split_whitespace().collect::<Vec<_>>();

        if cols.len() < 2 {
            continue;
        }

        // first column is the node id
        let node_id = cols.first().unwrap().parse()?;

        // all other columns are partitions
        for (network_id, col) in cols.into_iter().skip(1).enumerate() {
            let network = networks.entry(network_id).or_insert_with(Network::new);

            let path = col.split(':');
            let len = path.clone().count();

            // 1:2:3 -> [1, 1:2, 1:2:3]
            for level in 1..=len {
                let module_id = path.clone().take(level).join(":");
                network.add_node(&module_id, node_id);
            }
        }
    }

    Ok(networks)
}

pub fn write_result(
    modules: &HashMap<String, Module>,
    significant_cores: &HashMap<String, HashSet<NodeId>>,
    out_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut nodes = BTreeMap::new();

    for (module_id, core) in significant_cores.iter() {
        let module = &modules[module_id];

        for node in module.nodes.iter() {
            let significant = core.contains(node);
            nodes
                .entry(node)
                .or_insert_with(BTreeMap::new)
                .insert(module.level, (module.module, significant));
        }
    }

    let mut nodes = nodes.into_iter().collect::<Vec<_>>();

    // Sort by top module id
    nodes.sort_unstable_by_key(|(_, entries)| (*entries.first_key_value().unwrap().1).0);

    let mut f = BufWriter::new(File::create(out_file)?);

    for (node, entries) in nodes.iter() {
        let mut path = String::with_capacity(2 * entries.values().len());

        for &(module, significant) in entries.values() {
            let separator = if significant { ':' } else { ';' };
            path.push_str(&format!("{}{}", module, separator));
        }

        if path.ends_with(':') {
            path.pop();
        }

        writeln!(f, "{} {}", path, node)?;
    }

    Ok(())
}
