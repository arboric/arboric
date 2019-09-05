//! The arboric library
//!
use failure::Fail;
use simplelog::{LevelFilter, SimpleLogger};
use std::env;

mod arboric;

pub use crate::arboric::graphql;
pub use crate::arboric::Proxy;

/// Arboric error type to 'wrap' other, underlying error causes
#[derive(Debug, Fail)]
pub enum ArboricError {
    #[fail(display = "{}", message)]
    GeneralError { message: String },

    #[fail(display = "{}", message)]
    JsonError {
        message: String,
        #[cause]
        cause: serde_json::Error,
    },

    #[fail(display = "{}", message)]
    GraphqlParserError {
        message: String,
        #[cause]
        cause: graphql_parser::query::ParseError,
    },
}

impl From<serde_json::Error> for ArboricError {
    fn from(json_error: serde_json::Error) -> Self {
        ArboricError::JsonError {
            message: format!("{:?}", json_error),
            cause: json_error,
        }
    }
}

impl From<graphql_parser::query::ParseError> for ArboricError {
    fn from(parser_error: graphql_parser::query::ParseError) -> Self {
        ArboricError::GraphqlParserError {
            message: format!("{:?}", parser_error),
            cause: parser_error,
        }
    }
}

pub fn initialize_logging() {
    let mut log_config = simplelog::Config::default();
    let level_filter = get_env_log_level_filter();
    log_config.thread = Some(level_filter.to_level().unwrap());
    log_config.filter_allow = Some(&["arboric"]);
    let _ = SimpleLogger::init(level_filter, log_config);
}

const DEBUG_LEVELFILTER: LevelFilter = LevelFilter::Trace;

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
