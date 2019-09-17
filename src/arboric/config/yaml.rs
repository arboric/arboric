//! YamlConfig is a reader for [arboric::Configuration](arboric::Configuration)
//! that reads the configuration from a YAML file of the form
//!
//! ```yaml
//! ---
//! arboric:
//!   # 'global' config goes here
//!   log:
//!     console:
//!       level: info
//!     file:
//!       location: /var/log/arboric.log
//!       level: debug
//! listeners:
//! - bind: localhost
//!   port: 4000
//!   proxy: http://localhost:3001/graphql
//!   jwt_signing_key:
//!     from_env:
//!       key: SECRET_KEY_BASE
//!       encoding: hex
//!   log_to:
//!     influx_db:
//!       uri: https://localhost:8086
//!       database: arboric
//! ```

use crate::Configuration;
use http::Uri;
use serde::{Deserialize, Serialize};

/// Read the Configuration from the specified YAML file
pub fn read_yaml_configuration(filename: &str) -> crate::Result<crate::Configuration> {
    let f = std::fs::File::open(filename)?;
    let yaml_config: YamlConfig = serde_yaml::from_reader(f)?;

    let mut config = Configuration::new();
    if let Some(listeners) = yaml_config.listeners {
        for listener_config in listeners.iter() {
            config.listener(|mut listener| {
                listener = if listener_config.bind == "localhost" {
                    listener.localhost()
                } else {
                    let ip_addr = listener_config.bind.parse::<std::net::IpAddr>().unwrap();
                    listener.bind_addr(ip_addr)
                };
                listener = listener
                    .port(listener_config.port)
                    .proxy(listener_config.proxy.parse::<Uri>().unwrap());
                if listener_config.jwt_signing_key.from_env.encoding == "hex" {
                    listener =
                        listener.jwt_from_env_hex(&listener_config.jwt_signing_key.from_env.key);
                }
                if let Some(ref log_to) = listener_config.log_to {
                    if let Some(ref influx_db) = log_to.influx_db {
                        listener = listener.log_to_influx_db(&influx_db.uri, &influx_db.database);
                    }
                }
                // TODO: Allow specifiying policies in YAML
                let policy = crate::abac::Policy::allow_any();
                listener.add_policy(policy)
            })
        }
    }

    Ok(config)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct YamlConfig {
    arboric: Arboric,
    listeners: Option<Vec<Listener>>,
    policies: Option<Policies>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Arboric {
    log: Log,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Log {
    console: Option<Console>,
    file: Option<File>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Console {
    level: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct File {
    level: String,
    location: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Listener {
    bind: String,
    port: u16,
    proxy: String,
    jwt_signing_key: JwtSigningKey,
    log_to: Option<LogTo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct JwtSigningKey {
    from_env: FromEnv,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FromEnv {
    key: String,
    encoding: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct LogTo {
    influx_db: Option<InfluxDbConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct InfluxDbConfig {
    uri: String,
    database: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Policies {
    policies: Option<Vec<Policy>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Policy {
    when: Vec<When>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum When {
    ClaimIsPresent(ClaimIsPresent),
    ClaimEquals(ClaimEquals),
    ClaimIncludes(ClaimIncludes),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ClaimIsPresent {
    claim_is_present: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ClaimEquals {
    claim: String,
    equals: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ClaimIncludes {
    claim: String,
    includes: String,
}

#[cfg(test)]
mod test {
    // Import names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_yaml_config_policies() {
        let s = "---
policies:
- when:
  - claim_is_present: sub
  - claim: iss
    equals: arboric.io
  - claim: roles
    includes: admin
";
        println!("{:?}", s);
        let doc: Policies = serde_yaml::from_str(s).unwrap();
        println!("{:?}", doc);
        let policies = doc.policies.unwrap();
        let first = policies.first().unwrap();
        assert_eq!(
            When::ClaimIsPresent(ClaimIsPresent {
                claim_is_present: String::from("sub")
            }),
            *first.when.get(0).unwrap()
        );
        assert_eq!(
            When::ClaimEquals(ClaimEquals {
                claim: String::from("iss"),
                equals: String::from("arboric.io")
            }),
            *first.when.get(1).unwrap()
        );
        assert_eq!(
            When::ClaimIncludes(ClaimIncludes {
                claim: String::from("roles"),
                includes: String::from("admin")
            }),
            *first.when.get(2).unwrap()
        );
    }

    static SAMPLE_CONFIG_YML: &str = "---
arboric:
  log:
    console:
      level: info
listeners:
- bind: localhost
  port: 4000
  proxy: http://localhost:3001/graphql
  jwt_signing_key:
    from_env:
      key: SECRET_KEY_BASE
      encoding: hex
  log_to:
    influx_db:
      uri: http://localhost:8086
      database: arboric
";

    #[test]
    fn test_yaml_config() {
        println!("{:?}", SAMPLE_CONFIG_YML);
        let yaml_config: YamlConfig = serde_yaml::from_str(SAMPLE_CONFIG_YML).unwrap();
        assert!(yaml_config.arboric.log.console.is_some());
        assert_eq!("info", yaml_config.arboric.log.console.unwrap().level);
        assert!(yaml_config.arboric.log.file.is_none());
        assert!(yaml_config.listeners.is_some());
        let listeners = yaml_config.listeners.unwrap();
        assert!(!listeners.is_empty());
        let first = listeners.first();
        println!("{:?}", first);
    }

}
