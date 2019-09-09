//! The arboric command line
extern crate hyper;

use log::trace;
use std::error::Error;

const API_URI: &str = "http://localhost:3001/graphql";

/// The `arboric` CLI entrypoint
fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Move to arboric::Configuration
    arboric::initialize_logging();

    let mut config = arboric::Configuration::new();
    config.listener(|listener| listener.localhost().port(4000));

    let proxy = arboric::Proxy::new(API_URI);
    trace!("{:?}", proxy);

    proxy.run();

    println!("Ok");
    Ok(())
}
