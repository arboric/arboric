//! An arboric::config::Builder allows for a fluent interface for
//! building arboric::Configuration

use super::{JwtSigningKeySource, Listener};
use crate::abac::Policy;
use hyper::Uri;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// A ListenerBuilder implements the fluent-syntax builder for
/// [arboric::Configuration](arboric::Configuration)
pub struct ListenerBuilder {
    bind_address: IpAddr,
    port: u16,
    proxy_uri: Option<Uri>,
    jwt_signing_key_source: Option<JwtSigningKeySource>,
    policies: Vec<Policy>,
}

impl ListenerBuilder {
    pub fn new() -> ListenerBuilder {
        ListenerBuilder {
            bind_address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 0,
            proxy_uri: None,
            jwt_signing_key_source: None,
            policies: Vec::new(),
        }
    }

    pub fn bind_addr(mut self, addr: IpAddr) -> ListenerBuilder {
        self.bind_address = addr;
        self
    }

    pub fn bind_addr_v4(mut self, addr_v4: Ipv4Addr) -> ListenerBuilder {
        self.bind_address = IpAddr::V4(addr_v4);
        self
    }

    pub fn localhost(self) -> ListenerBuilder {
        self.bind_addr_v4(Ipv4Addr::LOCALHOST)
    }

    pub fn bind(mut self, a: u8, b: u8, c: u8, d: u8) -> ListenerBuilder {
        self.bind_address = IpAddr::V4(Ipv4Addr::new(a, b, c, d));
        self
    }

    pub fn port(mut self, port: u16) -> ListenerBuilder {
        self.port = port;
        self
    }

    pub fn proxy<I>(mut self, i: I) -> ListenerBuilder
    where
        I: Into<Uri>,
    {
        self.proxy_uri = Some(i.into());
        self
    }

    pub fn jwt_from_env_hex<S: Into<String>>(mut self, jwt_env_key: S) -> ListenerBuilder {
        self.jwt_signing_key_source = Some(JwtSigningKeySource::hex_from_env(jwt_env_key.into()));
        self
    }

    pub fn add_policy(mut self, policy: Policy) -> ListenerBuilder {
        self.policies.push(policy);
        self
    }

    pub fn build(self) -> Listener {
        Listener {
            listener_address: SocketAddr::new(self.bind_address, self.port),
            listener_path: None,
            api_uri: self.proxy_uri.unwrap(),
            jwt_signing_key_source: self.jwt_signing_key_source,
            pdp: crate::abac::PDP::with_policies(self.policies),
            influx_db_backend: None,
        }
    }
}
