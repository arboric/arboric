use std::error::Error;
use simplelog::{SimpleLogger,LevelFilter};

fn main() -> Result<(), Box<Error>> {
    let _ = SimpleLogger::init(LevelFilter::Trace, simplelog::Config::default());

    println!("Ok");
    Ok(())
}
