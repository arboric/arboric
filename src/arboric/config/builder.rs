//! An arboric::config::Builder allows for a fluent interface for
//! building arboric::Configuration

use hyper::Uri;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct ListenerBuilder {
    pub bind_address: IpAddr,
    pub port: u16,
}

impl ListenerBuilder {
    pub fn new() -> ListenerBuilder {
        ListenerBuilder {
            bind_address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 0,
        }
    }

    pub fn bind(&mut self, a: u8, b: u8, c: u8, d: u8) -> &mut ListenerBuilder {
        self.bind_address = IpAddr::V4(Ipv4Addr::new(a, b, c, d));
        self
    }

    pub fn port(&mut self, port: u16) -> &mut ListenerBuilder {
        self.port = port;
        self
    }

    pub fn build(&self) -> crate::config::Listener {
        let uri: Uri = "http://localhost:3001".parse().unwrap();
        crate::config::Listener::ip_addr_and_port(self.bind_address, self.port, uri)
    }
}
