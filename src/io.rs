use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use crate::{Module, Network, NodeId};

pub fn read_input(in_file: &str) -> Result<BTreeMap<usize, Network>, Box<dyn std::error::Error>> {
    let mut networks = BTreeMap::new();

    for line in in_file.lines() {
        if line.starts_with('#') {
            continue;
        }

        let cols = line.split(' ').collect::<Vec<_>>();

        // first column is the node id
        let node_id = cols[0].parse()?;

        // all other columns are partitions
        for (network_id, col) in cols[1..].iter().enumerate() {
            let network = networks.entry(network_id).or_insert_with(Network::new);

            let path = col.trim().split(':').collect::<Vec<_>>();

            // 1:2:3 -> [1, 1:2, 1:2:3]
            for level in 1..path.len() + 1 {
                let module_id = path[0..level].join(":");
                network.add_node(&module_id, node_id);
            }
        }
    }

    Ok(networks)
}

pub fn write_result(
    modules: &BTreeMap<String, Module>,
    significant_cores: &HashMap<&String, HashSet<NodeId>>,
    out_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut nodes = BTreeMap::new();

    for (&module_id, core) in significant_cores.iter() {
        let module = &modules[module_id];

        for node in module.nodes.iter() {
            let significant = core.contains(node);
            nodes
                .entry(node)
                .or_insert_with(BTreeMap::new)
                .insert(module.level, (module.module, significant));
        }
    }

    let mut f = BufWriter::new(File::create(out_file)?);

    for (node, entries) in nodes.iter() {
        let mut line = String::new();

        for &(module, significant) in entries.values() {
            let separator = if significant { ':' } else { ';' };
            line.push_str(&format!("{}{}", module, separator));
        }

        if &line[line.len() - 1..] == ":" {
            line.pop();
        }

        line.push_str(&format!(" {}", node));

        writeln!(f, "{}", line)?;
    }

    Ok(())
}
