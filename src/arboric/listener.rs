//! The main proxy that implements hyper::NewService
//!
use crate::config::ListenerConfig;
use futures::future;
use futures::Future;
use http::Uri;
use hyper::service::NewService;
use hyper::{Body, Server};
use log::{info, trace};
use std::net::SocketAddr;
use std::sync::Arc;

/// The main Proxy/Listener
#[derive(Debug)]
pub struct Listener {
    context: Arc<ListenerContext>,
}

#[derive(Debug)]
pub struct ListenerContext {
    pub listener_address: SocketAddr,
    pub listener_path: Option<String>,
    pub api_uri: Uri,
    pub pdp: crate::abac::PDP,
    pub influx_db_backend: Option<super::influxdb::Backend>,
    pub secret_key_bytes: Option<Vec<u8>>,
}

impl Listener {
    /// Constructs a new Listener with the given backend API URI
    pub fn new(listener_config: ListenerConfig) -> Self {
        let secret_key_bytes;
        if let Some(key_source) = &listener_config.jwt_signing_key_source {
            match key_source.get_secret_key_bytes() {
                Ok(bytes) => {
                    trace!("secret_key_bytes => {:?}", bytes);
                    secret_key_bytes = Some(bytes);
                }
                Err(err) => panic!("Enable to get secret key bytes: {}!", err),
            }
        } else {
            secret_key_bytes = None;
        }
        let context = ListenerContext {
            listener_address: listener_config.listener_address,
            listener_path: listener_config.listener_path,
            api_uri: listener_config.api_uri,
            pdp: listener_config.pdp,
            influx_db_backend: listener_config.influx_db_backend,
            secret_key_bytes,
        };
        Listener {
            context: Arc::new(context),
        }
    }

    pub fn run(self) -> ! {
        // This is our socket address...
        let addr = ([127, 0, 0, 1], 4000).into();

        let bound = Server::bind(&addr);
        info!("Proxy listening on {}", &addr);
        let server = bound
            .serve(self)
            .map_err(|e| eprintln!("server error: {}", e));

        // Run this server for... forever!
        hyper::rt::run(server);
        std::process::exit(0);
    }
}

impl NewService for Listener {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type InitError = hyper::Error;
    type Future = Box<dyn Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type Service = super::ProxyService;

    /// This creates a ProxyService
    fn new_service(&self) -> Self::Future {
        trace!("new_service(&Proxy)");
        Box::new(future::ok(super::ProxyService::new(self.context.clone())))
    }
}
