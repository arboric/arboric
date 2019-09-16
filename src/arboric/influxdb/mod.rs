//! The InfluxDB backend interface and configuration

use influx_db_client::{Client, Point, Points, Precision, Value};
use log::trace;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub influx_db_uri: String,
    pub database: String,
    pub precision: Precision,
}

impl Config {
    pub fn new(uri: String, database: String) -> Config {
        Config {
            influx_db_uri: uri,
            database: database,
            precision: Precision::Milliseconds,
        }
    }
}

/// The arboric::influxdb::Backend does the actual work of writing
/// a data point to InfluxDB
#[derive(Debug, Clone)]
pub struct Backend {
    pub config: Config,
}

impl Backend {
    pub fn write_points(&self, map: &HashMap<String, usize>) {
        let client = Client::new(
            self.config.influx_db_uri.clone(),
            self.config.database.clone(),
        );

        let mut points: Vec<Point> = Vec::new();
        for (field, n) in map {
            trace!("{}: {}", &field, &n);
            let point = Point::new("queries")
                .add_tag("field", Value::String(field.clone()))
                .add_field("n", Value::Integer((*n) as i64))
                .to_owned();
            points.push(point);
        }

        // if Precision is None, the default is second
        // Multiple write
        let _ = client
            .write_points(
                Points::create_new(points),
                Some(Precision::Milliseconds),
                None,
            )
            .unwrap();
    }
}
