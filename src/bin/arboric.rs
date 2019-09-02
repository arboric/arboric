extern crate hyper;

use log::debug;
use simplelog::{LevelFilter, SimpleLogger};

use std::env;
use std::error::Error;

const API_URI: &str = "http://localhost:3001/graphql";

const DEBUG_LEVELFILTER: LevelFilter = LevelFilter::Trace;

fn main() -> Result<(), Box<dyn Error>> {
    initialize_logging();

    let proxy = arboric::Proxy::new(API_URI);
    debug!("{:?}", proxy);

    proxy.run();

    println!("Ok");
    Ok(())
}

fn initialize_logging() {
    let mut log_config = simplelog::Config::default();
    let level_filter = get_env_log_level_filter();
    log_config.thread = Some(level_filter.to_level().unwrap());
    log_config.filter_allow = Some(&["arboric"]);
    let _ = SimpleLogger::init(level_filter, log_config);
}

fn get_env_log_level_filter() -> simplelog::LevelFilter {
    if let Ok(val) = env::var("ARBORIC_LOG") {
        println!("Using {} log level", &val);
        match val.to_lowercase().as_str() {
            "info" => LevelFilter::Info,
            "trace" => LevelFilter::Trace,
            "warn" => LevelFilter::Warn,
            _ => DEBUG_LEVELFILTER,
        }
    } else {
        DEBUG_LEVELFILTER
    }
}
