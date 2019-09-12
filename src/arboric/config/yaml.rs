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
//! ```

use crate::Configuration;
use serde::{Deserialize, Serialize};

/// Read the Configuration from the specified YAML file
pub fn read_yaml_configuraiton(filename: &str) -> crate::Result<crate::Configuration> {
    let f = std::fs::File::open(filename)?;
    let yaml_config: YamlConfig = serde_yaml::from_reader(f)?;
    let config = Configuration::new();
    Ok(config)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct YamlConfig {
    pub arboric: Arboric,
    pub listeners: Option<Vec<Listener>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Arboric {
    pub log: Log,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Log {
    pub console: Option<Console>,
    pub file: Option<File>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Console {
    pub level: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub level: String,
    pub location: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Listener {
    pub bind: String,
    pub port: u32,
    pub proxy: String,
    pub jwt_signing_key: JwtSigningKey,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JwtSigningKey {
    pub from_env: FromEnv,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FromEnv {
    pub key: String,
    pub encoding: String,
}

#[cfg(test)]
mod test {
    // Import names from outer (for mod tests) scope.
    use super::*;

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
