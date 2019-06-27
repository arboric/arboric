/// The arboric library

use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri};
use log::{debug, warn};

/// The main Proxy
#[derive(Debug)]
pub struct Proxy {}

impl Proxy {
    pub fn new() -> Proxy {
        Proxy {}
    }

    pub fn run(&self) {
        // This is our socket address...
        let addr = ([127, 0, 0, 1], 4000).into();
        let server = Server::bind(&addr)
            .serve(|| service_fn(proxy_service))
            .map_err(|e| eprintln!("server error: {}", e));

        // Run this server for... forever!
        hyper::rt::run(server);
    }
}

const API_URI: &str = "http://localhost:3000/graphql";

// Just a simple type alias
type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn do_get(req: Request<Body>) -> BoxFut {
    let req_uri = req.uri();
    debug!("req_uri => {}", req_uri);

    let params = req.uri().query().unwrap();
    let pandq = format!("/graphql?{}", params);

    let uri = Uri::builder()
        .scheme("http")
        .authority("localhost:4000")
        .path_and_query(&pandq[..])
        .build()
        .unwrap();

    debug!("uri => {}", uri);

    let client = Client::new();
    let fut = client
        .get(uri)
        .and_then(|res| {
            debug!("GET /localhost:4000 => {}", res.status());
            future::ok(res)
        })
        .map_err(|err| {
            warn!("{}", err);
            err
        });

    Box::new(fut)
}

fn do_post(req: Request<Body>) -> BoxFut {
    let req_uri = req.uri();
    debug!("req_uri => {}", req_uri);

    let uri: hyper::Uri = API_URI.parse().unwrap();
    debug!("uri => {}", uri);

    debug!("{:?}", req.body());

    let mut request = Request::post(uri)
        .header("Content-Type", "application/graphql")
        .body(Body::empty())
        .unwrap();

    *request.body_mut() = req.into_body();

    let client = Client::new();
    Box::new(client.request(request))
}

fn proxy_service(req: Request<Body>) -> BoxFut {
    match req.method() {
        &Method::GET => do_get(req),
        &Method::POST => do_post(req),
        _ => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_FOUND;
            Box::new(future::ok(response))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
