//! The arboric library
//!
use failure::Fail;
use futures::future;
use hyper::rt::Future;
use hyper::service::NewService;
use hyper::{Body, Server};
use log::{info, trace};
use simplelog::{LevelFilter, SimpleLogger};
use std::env;

pub mod arboric;

/// The main Proxy
#[derive(Debug)]
pub struct Proxy {
    api_uri: String,
    secret_key_bytes: Option<Vec<u8>>,
}

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

impl NewService for Proxy {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type InitError = hyper::Error;
    type Future = Box<dyn Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type Service = arboric::ProxyService;
    fn new_service(&self) -> Self::Future {
        trace!("new_service(&Proxy)");
        Box::new(future::ok(arboric::ProxyService::new(
            &self.api_uri,
            &self.secret_key_bytes,
        )))
    }
}

impl Proxy {
    /// Constructs a new Proxy with the given backend API URI
    pub fn new<S>(api_uri: S) -> Proxy
    where
        S: Into<String>,
    {
        let secret_key_bytes = Self::get_secret_key_bytes();
        Proxy {
            api_uri: api_uri.into(),
            secret_key_bytes: secret_key_bytes,
        }
    }

    fn get_secret_key_bytes() -> Option<Vec<u8>> {
        if let Ok(vec) = Self::unsafe_get_secret_key_bytes() {
            trace!("vec => {:?}", vec);
            Some(vec)
        } else {
            None
        }
    }

    fn unsafe_get_secret_key_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let secret = env::var("SECRET_KEY_BASE")?;
        Ok(hex::decode(&secret)?)
    }

    pub fn run(self) {
        // This is our socket address...
        let addr = ([127, 0, 0, 1], 4000).into();

        let bound = Server::bind(&addr);
        info!("Proxy listening on {}", &addr);
        let server = bound
            .serve(self)
            .map_err(|e| eprintln!("server error: {}", e));

        // Run this server for... forever!
        hyper::rt::run(server);
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
