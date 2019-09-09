//! The arboric::config module holds the structures and functions
//! for Arboric's configuration model

use crate::abac::PDP;
use hyper::Uri;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

mod builder;
pub use builder::ListenerBuilder;

/// The 'root' level configuration
#[derive(Debug)]
pub struct Configuration {
    listeners: Vec<Listener>,
}

impl Configuration {
    // Creates a new, empty [Configuration](arboric::config::Configuration)
    pub fn new() -> Configuration {
        Configuration {
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

    pub fn add_listener(&mut self, listener: Listener) {
        self.listeners.push(listener);
    }
}

/// A [KeyEncoding](arboric::config::KeyEncoding) just tells us whether the value is encoded as
/// hex or base64
#[derive(Debug)]
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
#[derive(Debug)]
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
            key: key,
            encoding: KeyEncoding::Hex,
        }
    }

    pub fn base64_from_env(key: String) -> JwtSigningKeySource {
        JwtSigningKeySource::FromEnv {
            key: key,
            encoding: KeyEncoding::Base64,
        }
    }
}

/// An [Listener](arboric::config::Listener) defines:
///
/// * an inbound endpoint, comprising:
///   * a 'bind' IP address
///   * an optional 'path' or prefix, e.g. `"/graphql"`
/// * a back-end API URL
/// * an `arboric::abac::PDP` or set of ABAC policies
#[derive(Debug)]
pub struct Listener {
    listener_address: SocketAddr,
    listener_path: Option<String>,
    api_uri: Uri,
    jwt_signing_key_source: Option<JwtSigningKeySource>,
    pdp: crate::abac::PDP,
}

impl Listener {
    /// Construct a [Listener](arboric::config::Listener) that binds to the given
    /// [IpAddr](std::net::IpAddr), port, and forwards to the API at the given [Uri](hyper::Uri)
    pub fn ip_addr_and_port(ip_addr: IpAddr, port: u16, to_uri: Uri) -> Listener {
        Listener {
            listener_address: SocketAddr::new(ip_addr, port),
            listener_path: None,
            api_uri: to_uri,
            jwt_signing_key_source: None,
            pdp: PDP::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_config_builder() {
        let mut configuration = Configuration::new();
        assert!(configuration.listeners.is_empty());

        configuration.listener(|listener| listener.localhost().port(4000));
        assert!(!configuration.listeners.is_empty());
        assert_eq!(1, configuration.listeners.iter().count());
        assert_eq!(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 4000)),
            configuration.listeners.first().unwrap().listener_address
        );
    }

}
