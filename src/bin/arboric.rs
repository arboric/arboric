//! The arboric command line
extern crate hyper;

#[macro_use]
extern crate clap;

use failure::Error;
use log::{debug, trace};

use clap::{App, Arg, SubCommand};

/// The `arboric` CLI entrypoint
fn main() -> Result<(), Error> {
    let matches = App::new("Arboric")
        .version(crate_version!())
        .about("GraphQL API Gateway")
        .arg(
            Arg::with_name("config")
                .short("f")
                .long("config")
                .value_name("FILE")
                .help("Specify the configuration file to use")
                .default_value("/var/arboric/config.yml")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("start").about("start the arboric server"))
        .get_matches();

    let config_file = matches
        .value_of("config")
        .unwrap_or("/var/arboric/config.yml");
    debug!(r#"Loading configuration from: "{}""#, config_file);

    let config = arboric::config::yaml::read_yaml_configuration(config_file)?;

    run(config);
    Ok(())
}

/// Run the Arboric proxy server according to the given configuration
pub fn run(config: arboric::Configuration) {
    arboric::initialize_logging(&config);

    if let Some(listener_config) = config.listeners.first() {
        let proxy = arboric::Listener::new(listener_config.clone());
        trace!("{:?}", proxy);

        proxy.run();
    } else {
        panic!("No listeners configured! See arboric::Configuration::listener()")
    }
}
