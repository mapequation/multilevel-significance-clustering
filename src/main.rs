#![feature(test)]
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::time::Instant;

mod clustering;
mod config;
mod io;
mod similarity;

type NodeId = u32;

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
    pub modules: BTreeMap<String, Module>,
}

impl Network {
    fn new() -> Network {
        Network {
            modules: BTreeMap::new(),
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

fn run(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Multi-level significance clustering");
    println!("Running with:");
    println!("\t- conf: {}", config.conf);
    println!("\t- seed: {}", config.seed);
    println!("\t- output: {}", config.out_file);

    print!("\nReading input file... ");
    let mut networks = io::read_input(&config.in_file)?;
    println!("done");

    let first = networks.remove(&0).unwrap();
    let rest = networks;

    print!("Computing similarities... ");
    let start = Instant::now();
    let most_similar_modules = similarity::get_most_similar_modules(&first, &rest);
    println!("done ({} ms)", start.elapsed().as_millis());

    let mut significant_cores = HashMap::new();
    let num_modules = most_similar_modules.len();

    print!("Clustering...");
    std::io::stdout().flush().unwrap();
    let start = Instant::now();
    for (i, (module_id1, networks)) in most_similar_modules.iter().enumerate() {
        let iteration_start = Instant::now();
        let module = &first.modules[module_id1].nodes;

        let modules = networks
            .iter()
            .map(|(network_id, (module_id, _))| &rest[network_id].modules[module_id].nodes)
            .collect::<Vec<_>>();

        let core =
            clustering::get_significant_core(module, modules.as_slice(), config.conf, config.seed);

        significant_cores.insert(module_id1, core);

        print!(
            "\rClustering... {}/{} ({} ms)",
            i + 1,
            num_modules,
            iteration_start.elapsed().as_millis()
        );
        std::io::stdout().flush().unwrap();
    }
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

fn main() {
    let config = config::Config::new(std::env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });

    if let Err(e) = run(config) {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
