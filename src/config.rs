use std::fs;

pub(crate) struct Config {
    pub seed: u64,
    pub conf: f32,
    pub in_file: String,
    pub out_file: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let in_file = match args.next() {
            Some(arg) => fs::read_to_string(arg).expect("Failed to read input file"),
            None => return Err("Didn't get input file"),
        };

        let out_file = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get output file"),
        };

        Ok(Config {
            seed: 123,
            conf: 0.95,
            in_file,
            out_file,
        })
    }
}
