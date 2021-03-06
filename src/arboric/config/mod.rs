//! The arboric::config module holds the structures and functions
//! for Arboric's configuration model

use crate::abac::PDP;
use http::Uri;
use std::env;
use std::net::{IpAddr, SocketAddr};

mod listener_builder;
pub use listener_builder::ListenerBuilder;

pub mod yaml;

/// The 'root' level configuration
#[derive(Debug)]
pub struct Configuration {
    pub arboric: ArboricConfiguration,
    pub listeners: Vec<ListenerConfig>,
}

impl Configuration {
    // Creates a new, empty [Configuration](arboric::config::Configuration)
    pub fn new() -> Configuration {
        Configuration {
            arboric: ArboricConfiguration {
                loggers: Vec::new(),
            },
            listeners: Vec::new(),
        }
    }

    pub fn listener<F>(&mut self, f: F)
    where
        F: FnOnce(ListenerBuilder) -> ListenerBuilder,
    {
        let listener_builder = f(ListenerBuilder::new());
        self.listeners.push(listener_builder.build());
    }

    pub fn add_listener(&mut self, listener_config: ListenerConfig) {
        self.listeners.push(listener_config);
    }
}

#[derive(Debug)]
pub struct ArboricConfiguration {
    pub loggers: Vec<Logger>,
}

/// A Logger configuration. May be `Console` or `File`
#[derive(Debug)]
pub enum Logger {
    Console(log::Level),
    File { location: String, level: log::Level },
}

/// An [ListenerConfig](arboric::config::ListenerConfig) defines:
///
/// * an inbound endpoint, comprising:
///   * a 'bind' IP address
///   * an optional 'path' or prefix, e.g. `"/graphql"`
/// * a back-end API URL
/// * an optional InfluxDB backend configuration
/// * an `arboric::abac::PDP` or set of ABAC policies
#[derive(Debug, Clone)]
pub struct ListenerConfig {
    pub listener_address: SocketAddr,
    pub listener_path: Option<String>,
    pub api_uri: Uri,
    pub jwt_signing_key_source: Option<JwtSigningKeySource>,
    pub pdp: crate::abac::PDP,
    pub influx_db_backend: Option<super::influxdb::Backend>,
}

impl ListenerConfig {
    /// Construct a [Listener](arboric::config::Listener) that binds to the given
    /// [IpAddr](std::net::IpAddr), port, and forwards to the API at the given [Uri](hyper::Uri)
    pub fn ip_addr_and_port(ip_addr: IpAddr, port: u16, api_uri: &Uri) -> Self {
        ListenerConfig {
            listener_address: SocketAddr::new(ip_addr, port),
            listener_path: None,
            api_uri: api_uri.clone(),
            jwt_signing_key_source: None,
            pdp: PDP::default(),
            influx_db_backend: None,
        }
    }
}

/// A [KeyEncoding](arboric::config::KeyEncoding) just tells us whether the value is encoded as
/// hex or base64
#[derive(Debug, Clone)]
pub enum KeyEncoding {
    Bytes,
    Hex,
    Base64,
}

/// A [JwtSigningKeySource](arboric::config::JwtSigningKeySource) defines
/// where and how to retrieve the signing key used to validate JWT bearer tokens.
/// It can be one of
///
/// * a hard-coded `Value`,
/// * an environment variable, or
/// * a file
///
/// And in any of the above cases, the value can be either be:
///
/// * the string value or file contents taken as 'raw' bytes,
/// * a hex encoded value, or
/// * a base64 encoded value
#[derive(Debug, Clone)]
pub enum JwtSigningKeySource {
    Value(String, KeyEncoding),
    FromEnv {
        key: String,
        encoding: KeyEncoding,
    },
    FromFile {
        filename: String,
        encoding: KeyEncoding,
    },
}

impl JwtSigningKeySource {
    pub fn hex(s: String) -> JwtSigningKeySource {
        JwtSigningKeySource::Value(s, KeyEncoding::Hex)
    }

    pub fn base64(s: String) -> JwtSigningKeySource {
        JwtSigningKeySource::Value(s, KeyEncoding::Base64)
    }

    pub fn hex_from_env(key: String) -> JwtSigningKeySource {
        JwtSigningKeySource::FromEnv {
            key,
            encoding: KeyEncoding::Hex,
        }
    }

    pub fn base64_from_env(key: String) -> JwtSigningKeySource {
        JwtSigningKeySource::FromEnv {
            key,
            encoding: KeyEncoding::Base64,
        }
    }

    pub fn from_file(filename: String) -> JwtSigningKeySource {
        JwtSigningKeySource::FromFile {
            filename,
            encoding: KeyEncoding::Bytes,
        }
    }

    pub fn get_secret_key_bytes(&self) -> crate::Result<Vec<u8>> {
        match self {
            JwtSigningKeySource::Value(secret, encoding) => match encoding {
                KeyEncoding::Hex => Ok(hex::decode(&secret)?),
                KeyEncoding::Base64 => Ok(base64::decode(&secret)?),
                x => Err(crate::ArboricError::general(format!(
                    "Not yet implemented: {:?}!",
                    x
                ))),
            },
            JwtSigningKeySource::FromEnv { key, encoding } => match env::var(key) {
                Ok(secret) => match encoding {
                    KeyEncoding::Hex => Ok(hex::decode(&secret)?),
                    KeyEncoding::Base64 => Ok(base64::decode(&secret)?),
                    x => Err(crate::ArboricError::general(format!(
                        "Not yet implemented: {:?}!",
                        x
                    ))),
                },
                Err(e) => Err(crate::ArboricError::EnvVarError {
                    message: key.into(),
                    cause: e,
                }),
            },
            JwtSigningKeySource::FromFile { filename, encoding } => match encoding {
                KeyEncoding::Bytes => Ok(std::fs::read(filename)?),
                KeyEncoding::Hex => read_file_as_hex(&filename),
                KeyEncoding::Base64 => read_file_as_base64(&filename),
            },
        }
    }
}

fn read_file_as_hex(filename: &String) -> crate::Result<Vec<u8>> {
    let s = std::fs::read_to_string(filename)?;
    Ok(hex::decode(&s)?)
}

fn read_file_as_base64(filename: &String) -> crate::Result<Vec<u8>> {
    let s = std::fs::read_to_string(filename)?;
    Ok(base64::decode(&s)?)
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_config_builder() {
        let mut configuration = Configuration::new();
        assert!(configuration.listeners.is_empty());

        configuration.listener(|listener| {
            listener
                .localhost()
                .port(4000)
                .proxy("http://localhost:3000/graphql".parse::<Uri>().unwrap())
        });
        assert!(!configuration.listeners.is_empty());
        assert_eq!(1, configuration.listeners.iter().count());
        assert_eq!(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 4000)),
            configuration.listeners.first().unwrap().listener_address
        );
    }
}
