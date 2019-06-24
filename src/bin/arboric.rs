extern crate hyper;

use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Body, Client, Request, Response, Server};
use log::debug;
use simplelog::{LevelFilter, SimpleLogger};
use std::error::Error;

// Just a simple type alias
type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn proxy(_req: Request<Body>) -> BoxFut {
    let uri = "http://localhost:4000/graphql?query={tweets{id}}"
        .parse()
        .unwrap();

    let client = Client::new();

    Box::new(client.get(uri))
}

fn do_get() {
    rt::run(rt::lazy(|| {
        let uri = "http://localhost:4000/graphql?query={tweets{id}}"
            .parse()
            .unwrap();

        let client = Client::new();

        client
            .get(uri)
            .map(|res| {
                let s = format!("*** Response: {}", res.status());
                debug!("{}", s);
            })
            .map_err(|err| {
                let s = format!("*** Error: {}", err);
                debug!("{}", s);
            })
    }));
}

fn main() -> Result<(), Box<Error>> {
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    do_get();

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
