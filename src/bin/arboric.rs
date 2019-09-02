extern crate hyper;

use log::debug;
use simplelog::{LevelFilter, SimpleLogger};

use std::env;
use std::error::Error;

const API_URI: &str = "http://localhost:3001/graphql";

/// The `arboric` CLI entrypoint
fn main() -> Result<(), Box<dyn Error>> {
    arboric::initialize_logging();

    let proxy = arboric::Proxy::new(API_URI);
    debug!("{:?}", proxy);

    proxy.run();

    println!("Ok");
    Ok(())
}
