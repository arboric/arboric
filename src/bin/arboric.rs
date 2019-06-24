extern crate hyper;

use hyper::rt::{self, Future};

use hyper::service::service_fn_ok;
use hyper::{Body, Client, Request, Response, Server};
use log::{info, trace, warn};
use simplelog::{LevelFilter, SimpleLogger};
use std::error::Error;

const PHRASE: &str = "Hello, World!";

fn hello_world(_req: Request<Body>) -> Response<Body> {
    let uri = "http://localhost:4000/graphql?query={tweets{id}}"
        .parse()
        .unwrap();

    let client = Client::new();

    let res = client
        .get(uri)
        .map(|res| {
            let s = format!("*** Response: {}", res.status());
            println!("{}", s);
            s
        })
        .map_err(|err| {
            let s = format!("*** Error: {}", err);
            println!("{}", s);
            err
        })
        .wait().unwrap();

    trace!("{}", res);

    Response::new(Body::from(PHRASE))
}

fn main() -> Result<(), Box<Error>> {
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    rt::run(rt::lazy(|| {
        let uri = "http://localhost:4000/graphql?query={tweets{id}}"
            .parse()
            .unwrap();

        let client = Client::new();

        client
            .get(uri)
            .map(|res| {
                let s = format!("*** Response: {}", res.status());
                println!("{}", s);
            })
            .map_err(|err| {
                let s = format!("*** Error: {}", err);
                println!("{}", s);
            })
    }));

    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`
        service_fn_ok(hello_world)
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    // Run this server for... forever!
    hyper::rt::run(server);

    println!("Ok");
    Ok(())
}
