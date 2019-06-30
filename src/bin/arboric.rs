extern crate hyper;

use log::debug;
use simplelog::{LevelFilter, SimpleLogger};
use std::error::Error;

const API_URI: &str = "http://localhost:3000/graphql";

fn main() -> Result<(), Box<Error>> {
    let mut log_config = simplelog::Config::default();
    log_config.thread = Some(log::Level::Debug);
    let _ = SimpleLogger::init(LevelFilter::Debug, log_config);

    let proxy = arboric::Proxy::new(API_URI);
    debug!("{:?}", proxy);

    proxy.run();

    println!("Ok");
    Ok(())
}
