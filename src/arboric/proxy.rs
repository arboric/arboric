//! The main proxy that implements hyper::NewService
//!
use crate::config::Listener;
use crate::Configuration;
use futures::future;
use hyper::rt::Future;
use hyper::service::NewService;
use hyper::{Body, Server};
use log::{info, trace};

/// The main Proxy
#[derive(Debug)]
pub struct Proxy {
    listener: Listener,
    secret_key_bytes: Option<Vec<u8>>,
}

impl NewService for Proxy {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type InitError = hyper::Error;
    type Future = Box<dyn Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type Service = super::ProxyService;
    fn new_service(&self) -> Self::Future {
        trace!("new_service(&Proxy)");
        Box::new(future::ok(super::ProxyService::new(
            &self.listener.api_uri,
            &self.secret_key_bytes,
            &self.listener.pdp,
            &self.listener.influx_db_backend,
        )))
    }
}

impl Proxy {
    /// Constructs a new Proxy with the given backend API URI
    pub fn new(config: Configuration) -> Proxy {
        if let Some(listener) = config.listeners.first() {
            let secret_key_bytes = Self::get_secret_key_bytes(&listener);
            Proxy {
                listener: (*listener).clone(),
                secret_key_bytes: secret_key_bytes,
            }
        } else {
            panic!("No listeners configured! See arboric::Configuration::listener()")
        }
    }

    fn get_secret_key_bytes(listener: &Listener) -> Option<Vec<u8>> {
        if let Some(key_source) = &listener.jwt_signing_key_source {
            if let Ok(vec) = key_source.get_secret_key_bytes() {
                trace!("vec => {:?}", vec);
                return Some(vec);
            }
        }
        None
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
