//! The arboric command line
extern crate hyper;

#[macro_use]
extern crate clap;

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

    arboric::initialize_logging();

    let proxy = arboric::Proxy::new(API_URI);
    trace!("{:?}", proxy);

    proxy.run();

    println!("Ok");
    Ok(())
}
