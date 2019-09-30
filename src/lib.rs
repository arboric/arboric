//! The arboric library
//!
use serde_json::Map;
use simplelog::{LevelFilter, SimpleLogger};
use std::env;

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

pub fn initialize_logging() {
    let mut config = simplelog::ConfigBuilder::new();
    let level_filter = get_env_log_level_filter();
    config.set_thread_level(level_filter);
    config.add_filter_allow_str("arboric");
    let _ = SimpleLogger::init(level_filter, config.build());
}

const DEBUG_LEVELFILTER: LevelFilter = LevelFilter::Debug;

fn get_env_log_level_filter() -> simplelog::LevelFilter {
    if let Ok(val) = env::var("ARBORIC_LOG") {
        println!("ARBORIC_LOG => \"{}\"", &val);
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
