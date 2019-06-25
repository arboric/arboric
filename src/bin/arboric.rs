extern crate hyper;

use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri};
use log::{debug, warn};
use simplelog::{LevelFilter, SimpleLogger};
use std::error::Error;

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

    let uri: hyper::Uri = "http://localhost:4000/graphql".parse().unwrap();
    debug!("uri => {}", uri);

    let request = Request::post(uri)
        .header("Content-Type", "application/graphql")
        .body(Body::from("{tweets{id}}"))
        .unwrap();

    let client = Client::new();
    Box::new(client.request(request))
}

fn proxy(req: Request<Body>) -> BoxFut {
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

fn main() -> Result<(), Box<Error>> {
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(proxy))
        .map_err(|e| eprintln!("server error: {}", e));

    // Run this server for... forever!
    hyper::rt::run(server);

    println!("Ok");
    Ok(())
}
