//! The arboric library
//!
use serde_json::Map;
use simplelog::SimpleLogger;
// use std::env;

mod arboric;

pub use crate::arboric::abac;
pub use crate::arboric::config;
pub use crate::arboric::graphql;
pub use crate::arboric::Listener;

pub use crate::arboric::ArboricError;
pub use config::Configuration;

/// Represents a list of JWT Claims (really just a JSON object)
pub type Claims = Map<String, serde_json::Value>;

/// An arboric::Request is used to process an incoming GraphQL HTTP API request
/// for ABAC and logging
#[derive(Debug, PartialEq)]
pub struct Request {
    pub claims: Claims,
    pub document: graphql_parser::query::Document,
}

pub type Result<T> = std::result::Result<T, ArboricError>;

static ARBORIC: &str = "arboric";

pub fn initialize_logging(configuration: &Configuration) {
    let loggers: Vec<Box<dyn simplelog::SharedLogger>> = configuration
        .arboric
        .loggers
        .iter()
        .map(|logger_conf| {
            println!("logger_conf => {:?}", &logger_conf);
            match logger_conf {
                arboric::config::Logger::Console(level) => init_console_logger(&level),
                arboric::config::Logger::File { location, level } => {
                    init_file_logger(location, level)
                }
            }
        })
        .collect();
    let _ = simplelog::CombinedLogger::init(loggers);
}

fn make_config(level: &log::Level) -> simplelog::Config {
    let mut config = simplelog::ConfigBuilder::new();
    config.set_thread_level(level.to_level_filter());
    config.add_filter_allow_str(&ARBORIC);
    config.build()
}

fn init_console_logger(level: &log::Level) -> Box<dyn simplelog::SharedLogger> {
    println!("init_console_logger({})", &level);
    let config = make_config(&level);
    SimpleLogger::new(level.to_level_filter(), config)
}

fn init_file_logger(location: &String, level: &log::Level) -> Box<dyn simplelog::SharedLogger> {
    println!("init_file_logger({}, {})", &location, &level);
    let config = make_config(&level);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(location)
        .unwrap_or_else(|_| panic!(r#"Unable to create log file "{}""#, &location));
    simplelog::WriteLogger::new(level.to_level_filter(), config, file)
}

#[cfg(test)]
pub fn initialize_test_logging() {
    if let Some(level) = get_env_log_level() {
        let config = make_config(&level);
        let _ = SimpleLogger::new(level.to_level_filter(), config);
    }
}

#[cfg(test)]
fn get_env_log_level() -> Option<log::Level> {
    use std::env;
    use std::str::FromStr;

    if let Ok(val) = env::var("ARBORIC_LOG") {
        println!("ARBORIC_LOG => \"{}\"", &val);
        match log::Level::from_str(val.to_lowercase().as_str()) {
            Ok(level) => Some(level),
            Err(_) => {
                eprintln!(r#"Unrecognized ARBORIC_LOG value "{}""#, &val);
                None
            }
        }
    } else {
        None
    }
}
