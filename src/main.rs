use significance_clustering::Config;

fn main() {
    let config = Config::new(std::env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });

    if let Err(e) = significance_clustering::run(config) {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
