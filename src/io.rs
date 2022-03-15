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

        let cols = line.split(' ').collect::<Vec<_>>();

        if cols.len() < 2 {
            continue;
        }

        // first column is the node id
        let node_id = cols.first().unwrap().parse()?;

        // all other columns are partitions
        for (network_id, col) in cols.iter().skip(1).enumerate() {
            let network = networks.entry(network_id).or_insert_with(Network::new);

            let path = col.trim().split(':').collect::<Vec<_>>();

            // 1:2:3 -> [1, 1:2, 1:2:3]
            for level in 1..=path.len() {
                let module_id = path.iter().take(level).join(":");
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
        let mut line = String::with_capacity(2 * entries.values().len() + 2);

        for &(module, significant) in entries.values() {
            let separator = if significant { ':' } else { ';' };
            line.push_str(&format!("{}{}", module, separator));
        }

        if line.ends_with(':') {
            line.pop();
        }

        line.push_str(&format!(" {}", node));

        writeln!(f, "{}", line)?;
    }

    Ok(())
}
