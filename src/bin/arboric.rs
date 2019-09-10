//! The arboric command line
extern crate hyper;

use http::Uri;
use log::trace;
use std::error::Error;

const API_URI: &str = "http://localhost:3001/graphql";

/// The `arboric` CLI entrypoint
fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Move to arboric::Configuration
    arboric::initialize_logging();

    let mut config = arboric::Configuration::new();
    config.listener(|listener| {
        listener
            .localhost()
            .port(4000)
            .proxy(API_URI.parse::<Uri>().unwrap())
    });

    run(config);
    Ok(())
}

/// Run the Arboric proxy server according to the given configuration
pub fn run(config: arboric::Configuration) {
    let proxy = arboric::Proxy::new(config);
    trace!("{:?}", proxy);

    proxy.run();
}
