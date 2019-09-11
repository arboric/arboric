//! The arboric command line
extern crate hyper;

#[macro_use]
extern crate clap;

use http::Uri;
use log::trace;
use std::error::Error;

use clap::{App, SubCommand};

const API_URI: &str = "http://localhost:3001/graphql";

/// The `arboric` CLI entrypoint
fn main() -> Result<(), Box<dyn Error>> {
    let _matches = App::new("Arboric")
        .version(crate_version!())
        .about("GraphQL API Gateway")
        .subcommand(SubCommand::with_name("start").about("start the arboric server"))
        .get_matches();

    // TODO: Move to arboric::Configuration
    arboric::initialize_logging();

    let mut config = arboric::Configuration::new();
    config.listener(|listener| {
        let policy = arboric::abac::Policy::allow_any();
        listener
            .localhost()
            .port(4000)
            .proxy(API_URI.parse::<Uri>().unwrap())
            .jwt_from_env_hex("SECRET_KEY_BASE")
            .add_policy(policy)
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
