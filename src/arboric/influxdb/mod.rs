//! The InfluxDB backend interface and configuration

use influxdb3_client::{Client, ClientConfig, Point, Precision};
use log::{trace, warn};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::OnceCell;

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
            precision: Precision::Millisecond,
        }
    }
}

/// The arboric::influxdb::Backend does the actual work of writing
/// a data point to InfluxDB
#[derive(Clone)]
pub struct Backend {
    pub config: Config,
    /// The client is created on first use and shared between clones
    client: Arc<OnceCell<Client>>,
}

impl fmt::Debug for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Backend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Backend {
    pub fn new(config: Config) -> Backend {
        Backend {
            config,
            client: Arc::new(OnceCell::new()),
        }
    }

    async fn client(&self) -> influxdb3_client::Result<&Client> {
        self.client
            .get_or_try_init(|| async {
                Client::new(
                    ClientConfig::builder()
                        .host(self.config.influx_db_uri.as_str())
                        .database(self.config.database.as_str())
                        .build()?,
                )
                .await
            })
            .await
    }

    pub async fn write_points(&self, map: &HashMap<String, usize>) {
        if map.is_empty() {
            return;
        }

        let points: Vec<Point> = map
            .iter()
            .map(|(field, n)| {
                trace!("{}: {}", field, n);
                Point::new("queries")
                    .tag("field", field.as_str())
                    .field("n", *n as i64)
            })
            .collect();

        let client = match self.client().await {
            Ok(client) => client,
            Err(err) => {
                warn!("Failed to create InfluxDB client: {}", err);
                return;
            }
        };

        if let Err(err) = client.write(points).precision(self.config.precision).await {
            warn!("Failed to write points to InfluxDB: {}", err);
        }
    }
}
