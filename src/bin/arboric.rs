//! The arboric command line
use log::{debug, trace};

use clap::{Arg, Command, crate_version};

/// The `arboric` CLI entrypoint
#[tokio::main]
async fn main() -> arboric::Result<()> {
    let matches = Command::new("Arboric")
        .version(crate_version!())
        .about("GraphQL API Gateway")
        .arg(
            Arg::new("config")
                .short('f')
                .long("config")
                .value_name("FILE")
                .help("Specify the configuration file to use")
                .default_value("/var/arboric/config.yml"),
        )
        .subcommand(Command::new("start").about("start the arboric server"))
        .get_matches();

    let config_file = matches
        .get_one::<String>("config")
        .map(String::as_str)
        .unwrap_or("/var/arboric/config.yml");
    debug!(r#"Loading configuration from: "{}""#, config_file);

    let config = arboric::config::yaml::read_yaml_configuration(config_file)?;

    run(config).await?;
    Ok(())
}

/// Run the Arboric proxy server according to the given configuration
pub async fn run(config: arboric::Configuration) -> std::io::Result<()> {
    arboric::initialize_logging(&config);

    if let Some(listener_config) = config.listeners.first() {
        let proxy = arboric::Listener::new(listener_config.clone());
        trace!("{:?}", proxy);

        proxy.run().await
    } else {
        panic!("No listeners configured! See arboric::Configuration::listener()")
    }
}
