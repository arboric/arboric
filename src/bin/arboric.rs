extern crate hyper;

use log::debug;
use simplelog::{LevelFilter, SimpleLogger};
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    let proxy = arboric::Proxy{};
    debug!("{:?}", proxy);

    proxy.run();

    println!("Ok");
    Ok(())
}
