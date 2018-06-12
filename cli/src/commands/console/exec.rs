use carina_core;
use carina_core::{Config, CarinaConfigBuilder, Events};
use clap::ArgMatches;
use commands::console::Ping;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

pub fn execute(args: &ArgMatches) {
    let mut file = File::open(args.value_of("CONFIG").unwrap().to_string()).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config: Config = match Config::from_str(&content) {
        Ok(val) => val,
        Err(e)  => panic!("Error reading config file {:?}", e)
    };

    let carina_config_builder = CarinaConfigBuilder::new()
        .add_event(Events::Ping, Arc::new(Ping{}))
        .set_config(config);
    let (thread, _, _) = carina_core::init(carina_config_builder);

    thread.join().unwrap();
}