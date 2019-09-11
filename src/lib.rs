//! The arboric library
//!
use failure::Fail;
use serde_json::Map;
use simplelog::{LevelFilter, SimpleLogger};
use std::env;

mod arboric;

pub use crate::arboric::abac;
pub use crate::arboric::config;
pub use crate::arboric::graphql;
pub use crate::arboric::Proxy;

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

/// Arboric error type to 'wrap' other, underlying error causes
#[derive(Debug, Fail)]
pub enum ArboricError {
    #[fail(display = "{}", message)]
    GeneralError { message: String },

    #[fail(display = "{}", message)]
    EnvVarError {
        message: String,
        #[cause]
        cause: std::env::VarError,
    },

    #[fail(display = "{}", message)]
    HexDecodeError {
        message: String,
        #[cause]
        cause: hex::FromHexError,
    },

    #[fail(display = "{}", message)]
    Base64DecodeError {
        message: String,
        #[cause]
        cause: base64::DecodeError,
    },

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

impl ArboricError {
    pub fn general<S: Into<String>>(message: S) -> ArboricError {
        ArboricError::GeneralError {
            message: message.into(),
        }
    }
}

// macro_rules! impl_from {
//     ($($type:ty),+) => {
//         $(
//             impl From<$type> for ArboricError {
//                 fn from(error: $type) -> Self {
//                     ArboricError::GeneralError { message: format!("{:?}", error) }
//                 }
//             }
//         )*
//     };
// }
// impl_from!(hex::FromHexError);

impl From<std::env::VarError> for ArboricError {
    fn from(error: std::env::VarError) -> Self {
        ArboricError::EnvVarError {
            message: format!("{:?}", error),
            cause: error,
        }
    }
}

impl From<hex::FromHexError> for ArboricError {
    fn from(error: hex::FromHexError) -> Self {
        ArboricError::HexDecodeError {
            message: format!("{:?}", error),
            cause: error,
        }
    }
}

impl From<base64::DecodeError> for ArboricError {
    fn from(error: base64::DecodeError) -> Self {
        ArboricError::Base64DecodeError {
            message: format!("{:?}", error),
            cause: error,
        }
    }
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
