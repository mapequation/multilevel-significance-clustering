#![feature(test)]
#![feature(map_first_last)]
#![feature(bool_to_option)]

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::AtomicUsize;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Map};
#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use hashbrown::{HashMap, HashSet};

pub use config::Config;

pub mod clustering;
pub mod config;
pub mod io;
pub mod similarity;

pub type NodeId = u32;

#[derive(Debug, Clone)]
pub struct Module {
    pub module_id: String,
    pub module: u32,
    pub level: usize,
    pub nodes: HashSet<NodeId>,
}

impl Module {
    fn new(id: &str) -> Module {
        let path = id.split(':').collect::<Vec<&str>>();
        Module {
            module_id: id.to_string(),
            module: path.last().unwrap().parse().unwrap_or_default(),
            level: path.len(),
            nodes: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Network {
    pub modules: HashMap<String, Module>,
}

impl Network {
    fn new() -> Network {
        Network {
            modules: HashMap::new(),
        }
    }

    fn add_node(&mut self, module_id: &str, node_id: NodeId) {
        self.modules
            .entry(module_id.to_string())
            .or_insert_with(|| Module::new(module_id))
            .nodes
            .insert(node_id);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run(contents: &str, conf: f32, seed: u32) -> Map {
    let mut networks = io::read_input(contents).unwrap();
    let first = networks.remove(&0).unwrap();
    let rest = networks;

    let most_similar_modules = similarity::get_most_similar_modules(&first, &rest);

    let significant_cores = most_similar_modules
        .iter()
        .map(|(module_id1, networks)| {
            let module = &first.modules[module_id1].nodes;

            let modules = networks
                .iter()
                .map(|(network_id, module_id)| &rest[network_id].modules[module_id].nodes)
                .collect::<Vec<_>>();

            let core = clustering::get_significant_core(module, &modules, conf, seed as u64);

            (module_id1.to_string(), core)
        })
        .collect::<HashMap<String, HashSet<NodeId>>>();

    let mut nodes = HashMap::new();

    for (module_id, core) in significant_cores.iter() {
        let module = &first.modules[module_id];

        for &node in module.nodes.iter() {
            let significant = core.contains(&node);
            nodes
                .entry(node)
                .or_insert_with(BTreeMap::new)
                .insert(module.level, (module.module, significant));
        }
    }

    let result = Map::new();

    for (node, modules) in nodes.iter() {
        let array = Array::new();

        for &(module, significant) in modules.values() {
            let entry = Array::new();
            entry.push(&JsValue::from(module));
            entry.push(&JsValue::from(significant));
            array.push(&entry);
        }

        result.set(&JsValue::from(*node), &array);
    }

    result
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Multi-level significance clustering");
    println!("Running with:");
    println!("\t- conf: {}", config.conf);
    println!("\t- seed: {}", config.seed);
    println!("\t- output: {}", config.out_file);

    print!("\nReading input file... ");
    let mut networks = io::read_input(&config.in_file)?;

    let first = networks.remove(&0).unwrap();
    let num_nodes = first.modules.values().fold(0, |acc, m| acc + m.nodes.len());
    println!(
        "done ({} nodes in {} modules)",
        num_nodes,
        first.modules.len()
    );

    let rest = networks;

    print!("Computing similarities... ");
    let start = Instant::now();
    let most_similar_modules = similarity::get_most_similar_modules(&first, &rest);
    println!("done ({} ms)", start.elapsed().as_millis());

    let num_modules = most_similar_modules.len();
    print!("Clustering... 0/{} done", num_modules);
    std::io::stdout().flush().unwrap();
    let start = Instant::now();
    let current_count = Arc::new(AtomicUsize::new(0));

    let significant_cores = most_similar_modules
        .par_iter()
        .map(|(module_id1, networks)| {
            let module = &first.modules[module_id1].nodes;

            let modules = networks
                .iter()
                .map(|(network_id, module_id)| &rest[network_id].modules[module_id].nodes)
                .collect::<Vec<_>>();

            let core = clustering::get_significant_core(module, &modules, config.conf, config.seed);

            let count = current_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            print!("\rClustering... {}/{} done", count, num_modules);
            std::io::stdout().flush().unwrap();

            (module_id1.to_string(), core)
        })
        .collect::<HashMap<String, HashSet<NodeId>>>();

    println!(
        "\rClustering... {}/{} done ({} ms)",
        num_modules,
        num_modules,
        start.elapsed().as_millis()
    );

    print!("Writing output file... ");
    io::write_result(&first.modules, &significant_cores, &config.out_file)?;
    println!("done");

    Ok(())
}
