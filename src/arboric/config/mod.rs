//! The arboric::config module holds the structures and functions
//! for Arboric's configuration model

use hyper::Uri;
use std::net::SocketAddr;

/// The 'root' level arboric::Configuration
#[derive(Debug, PartialEq)]
pub struct Configuration {
    listeners: Vec<Listener>
}

/// An arboric::config::Listener defines:
///
/// * an inbound endpoint, comprising:
///   * a 'bind' IP address
///   * an optional 'path' or prefix, e.g. `"/graphql"`
/// * a back-end API URL
/// * an `arboric::abac::PDP` or set of ABAC policies
#[derive(Debug, PartialEq)]
pub struct Listener {
    listener_address: SocketAddr,
    listener_path: Option<String>,
    api_uri: Uri,
    pdp: arboric::abac::PDP
}
