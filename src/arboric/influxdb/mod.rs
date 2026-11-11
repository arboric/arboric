//! The InfluxDB backend interface and configuration

use influx_db_client::{Client, Point, Points, Precision, Url, Value};
use log::{trace, warn};
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
            database,
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
    pub async fn write_points(&self, map: &HashMap<String, usize>) {
        let host: Url = match self.config.influx_db_uri.parse() {
            Ok(host) => host,
            Err(err) => {
                warn!(
                    "Invalid InfluxDB URI {:?}: {}",
                    self.config.influx_db_uri, err
                );
                return;
            }
        };
        let client = Client::new(host, self.config.database.clone());

        let mut points: Vec<Point> = Vec::new();
        for (field, n) in map {
            trace!("{}: {}", field, n);
            let point = Point::new("queries")
                .add_tag("field", Value::String(field.clone().into()))
                .add_field("n", Value::Integer((*n) as i64))
                .to_owned();
            points.push(point);
        }

        // if Precision is None, the default is second
        // Multiple write
        if let Err(err) = client
            .write_points(
                Points::create_new(points),
                Some(Precision::Milliseconds),
                None,
            )
            .await
        {
            warn!("Failed to write points to InfluxDB: {}", err);
        }
    }
}
