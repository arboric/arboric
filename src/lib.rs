//! The arboric library

use futures::future;
use hyper::rt::Future;
use hyper::service::NewService;
use hyper::{Body, Server};
use log::{info, trace};
use std::env;

pub mod arboric;

/// The main Proxy
#[derive(Debug)]
pub struct Proxy {
    api_uri: String,
    secret_key_bytes: Option<Vec<u8>>,
}

impl NewService for Proxy {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type InitError = hyper::Error;
    type Future = Box<Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type Service = arboric::ProxyService;
    fn new_service(&self) -> Self::Future {
        trace!("new_service(&Proxy)");
        Box::new(future::ok(arboric::ProxyService {
            api_uri: self.api_uri.clone(),
            secret_key_bytes: self.secret_key_bytes.clone(),
        }))
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

    fn unsafe_get_secret_key_bytes() -> Result<Vec<u8>, Box<std::error::Error>> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
