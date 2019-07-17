//! The arboric library

use futures::future;
use hyper::rt::Future;
use hyper::service::NewService;
use hyper::{Body, Server};
use log::{info, trace};

pub mod arboric;

/// The main Proxy
#[derive(Debug)]
pub struct Proxy {
    api_uri: String,
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
        }))
    }
}

impl Proxy {
    pub fn new<S>(api_uri: S) -> Proxy
    where
        S: Into<String>,
    {
        Proxy {
            api_uri: api_uri.into(),
        }
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
