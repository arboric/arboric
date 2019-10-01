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

use crate::abac;
use crate::arboric::graphql;
use crate::arboric::ArboricError;
use crate::Configuration;
use http::Uri;
use log::trace;
use serde::{Deserialize, Serialize};

/// Read the Configuration from the specified YAML file
pub fn read_yaml_configuration(filename: &str) -> crate::Result<crate::Configuration> {
    use std::io::ErrorKind;

    match std::fs::File::open(filename) {
        Ok(f) => read_yaml_config(f),
        Err(cause) => {
            trace!("cause.kind() => {:?}", cause.kind());
            let message = match cause.kind() {
                ErrorKind::NotFound => format!("File not found: {}!", filename),
                _ => cause.to_string(),
            };
            Err(ArboricError::IoError { message, cause })
        }
    }
}

fn read_yaml_config(f: std::fs::File) -> crate::Result<crate::Configuration> {
    use abac::MatchAttribute;

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

                match listener_config.jwt_signing_key {
                    JwtSigningKey::FromEnv { ref from_env } => match &from_env.encoding {
                        Some(encoding) => {
                            if encoding == "hex" {
                                listener.jwt_from_env_hex(&from_env.key);
                            } else {
                                panic!(r#"Unsupported encoding "{}" "#, encoding);
                            }
                        }
                        None => (),
                    },
                    JwtSigningKey::FromFile { ref from_file } => {
                        trace!("from_file => {:?}", &from_file);
                    }
                }
                if let Some(ref log_to) = listener_config.log_to {
                    if let Some(ref influx_db) = log_to.influx_db {
                        listener.log_to_influx_db(&influx_db.uri, &influx_db.database);
                    }
                }
                if let Some(policies) = listener_config.policies.as_ref() {
                    for policy_def in policies.iter() {
                        let mut policy = abac::Policy::new();
                        match &policy_def.when {
                            Some(ref vec) => {
                                for when in vec.iter() {
                                    let match_attribute: MatchAttribute = match when {
                                        When::ClaimIsPresent(w) => {
                                            MatchAttribute::claim_present(&w.claim_is_present)
                                        }
                                        When::ClaimEquals(w) => {
                                            MatchAttribute::claim_equals(&w.claim, &w.equals)
                                        }
                                        When::ClaimIncludes(w) => {
                                            MatchAttribute::claim_includes(&w.claim, &w.includes)
                                        }
                                    };
                                    policy.add_match_attribute(match_attribute);
                                }
                            }
                            None => {
                                policy.add_match_attribute(MatchAttribute::Any);
                            }
                        }

                        if let Some(ref allows) = policy_def.allow {
                            for pattern in allows.iter().map(&pattern_def_to_graphql_pattern) {
                                trace!("allow: {:?}", pattern);
                                policy.allow(pattern);
                            }
                        }

                        if let Some(ref denies) = policy_def.deny {
                            for pattern in denies.iter().map(&pattern_def_to_graphql_pattern) {
                                trace!("deny: {:?}", pattern);
                                policy.deny(pattern);
                            }
                        }
                        listener.add_policy(policy);
                    }
                }
                listener
            })
        }
    }

    Ok(config)
}

fn pattern_def_to_graphql_pattern(pattern: &Pattern) -> graphql::Pattern {
    match pattern {
        Pattern::Query(def) => graphql::Pattern::query(&def.query),
        Pattern::Mutation(def) => graphql::Pattern::mutation(&def.mutation),
        Pattern::SomeString(ref s) => graphql::Pattern::parse(s),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct YamlConfig {
    arboric: Arboric,
    listeners: Option<Vec<Listener>>,
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
    policies: Option<Vec<Policy>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum JwtSigningKey {
    FromEnv { from_env: FromEnv },
    FromFile { from_file: FromFile },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FromEnv {
    key: String,
    encoding: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FromFile {
    name: String,
    encoding: Option<String>,
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
struct Policy {
    when: Option<Vec<When>>,
    allow: Option<Vec<Pattern>>,
    deny: Option<Vec<Pattern>>,
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
#[serde(untagged)]
enum Pattern {
    Query(QueryDef),
    Mutation(MutationDef),
    SomeString(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct QueryDef {
    query: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MutationDef {
    mutation: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ClaimIncludes {
    claim: String,
    includes: String,
}

#[cfg(test)]
impl When {
    fn claim_is_present(claim: &str) -> Self {
        When::ClaimIsPresent(ClaimIsPresent {
            claim_is_present: String::from(claim),
        })
    }

    fn claim_equals(claim: &str, equals: &str) -> Self {
        When::ClaimEquals(ClaimEquals {
            claim: String::from(claim),
            equals: String::from(equals),
        })
    }

    fn claim_includes(claim: &str, includes: &str) -> Self {
        When::ClaimIncludes(ClaimIncludes {
            claim: String::from(claim),
            includes: String::from(includes),
        })
    }
}

#[cfg(test)]
mod test {
    // Import names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_yaml_config_policy_allow() {
        let s = r#"---
when:
- claim_is_present: sub
allow:
- query: hero
- mutation: createHero
- "*"
"#;
        println!("{:?}", s);
        let policy: Policy = serde_yaml::from_str(s).unwrap();
        println!("{:?}", policy);
        let allow = policy.allow.unwrap();
        assert_eq!(
            Pattern::Query(QueryDef {
                query: String::from("hero")
            }),
            *allow.get(0).unwrap()
        );
        assert_eq!(
            Pattern::Mutation(MutationDef {
                mutation: String::from("createHero")
            }),
            *allow.get(1).unwrap()
        );
        assert_eq!(
            Pattern::SomeString(String::from("*")),
            *allow.get(2).unwrap()
        );
    }

    #[test]
    fn test_yaml_config_policies() {
        let s = r#"---
- when:
  - claim_is_present: sub
  - claim: iss
    equals: arboric.io
  - claim: roles
    includes: admin
- when:
  - claim_is_present: sub
  allow:
  - query: "*"
"#;
        println!("{:?}", s);
        let policies: Vec<Policy> = serde_yaml::from_str(s).unwrap();
        let first = policies.first().unwrap();
        let when = &first.when.as_ref().unwrap();
        assert_eq!(When::claim_is_present("sub"), *when.get(0).unwrap());
        assert_eq!(
            When::claim_equals("iss", "arboric.io"),
            *when.get(1).unwrap()
        );
        assert_eq!(
            When::claim_includes("roles", "admin"),
            *when.get(2).unwrap()
        );
    }

    #[test]
    fn test_yaml_config_listener() {
        let s = r#"---
bind: localhost
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
policies:
- when:
  - claim_is_present: sub
  - claim: iss
    equals: arboric.io
  - claim: roles
    includes: admin
  allow:
  - query: "*"
"#;
        let listener: Listener = serde_yaml::from_str(s).unwrap();
        println!("{:?}", listener);
        let policies = listener.policies.unwrap();
        let first = policies.first().unwrap();
        let when = &first.when.as_ref().unwrap();
        assert_eq!(When::claim_is_present("sub"), *when.get(0).unwrap());
        assert_eq!(
            When::claim_equals("iss", "arboric.io"),
            *when.get(1).unwrap()
        );
        assert_eq!(
            When::claim_includes("roles", "admin"),
            *when.get(2).unwrap()
        );
        let allow = first.allow.as_ref().unwrap();
        println!("{:?}", allow);
        assert_eq!(
            allow.get(0).unwrap(),
            &Pattern::Query(QueryDef {
                query: String::from("*")
            })
        )
    }

    static YAML: &str = r#"---
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
  policies:
  - when:
    - claim_is_present: sub
    - claim: iss
      equals: arboric.io
    allow:
    - query: "*"
"#;

    #[test]
    fn test_yaml_config_from_string() {
        println!("YAML: {:?}", &YAML);
        let yaml_config: YamlConfig = serde_yaml::from_str(YAML).unwrap();
        assert!(yaml_config.arboric.log.console.is_some());
        assert_eq!("info", yaml_config.arboric.log.console.unwrap().level);
        assert!(yaml_config.arboric.log.file.is_none());
        assert!(yaml_config.listeners.is_some());
        let listeners = yaml_config.listeners.unwrap();
        assert!(!listeners.is_empty());
        let listener = listeners.first().unwrap();
        let policies = listener.policies.as_ref().unwrap();
        println!("{:?}", policies);
        let policy = policies.first().unwrap();
        let when = &policy.when.as_ref().unwrap();
        assert_eq!(When::claim_is_present("sub"), *when.get(0).unwrap());
        assert_eq!(
            When::claim_equals("iss", "arboric.io"),
            *when.get(1).unwrap()
        );
    }

    static JWT_FROM_FILE_YAML: &str = r#"
arboric:
  log:
    console:
      level: info
listeners:
- bind: localhost
  port: 4000
  proxy: http://localhost:3001/graphql
  jwt_signing_key:
    from_file:
      name: "etc/arboric/secret_key_bytes"
  policies:
  - allow:
    - "*"
"#;

    #[test]
    fn test_yaml_config_jwt_from_file() {
        println!("JWT_FROM_FILE_YAML: {:?}", &JWT_FROM_FILE_YAML);
        let yaml_config: YamlConfig = serde_yaml::from_str(JWT_FROM_FILE_YAML).unwrap();
        let listeners = yaml_config.listeners.unwrap();
        assert!(!listeners.is_empty());
        let listener = listeners.first().unwrap();
        assert_eq!(
            listener.jwt_signing_key,
            JwtSigningKey::FromFile {
                from_file: FromFile {
                    name: String::from("etc/arboric/secret_key_bytes"),
                    encoding: None
                }
            }
        )
    }

    #[test]
    fn test_yaml_config_from_file() {
        let path = std::path::PathBuf::from("etc/arboric/config.yml");
        let filename = path.canonicalize().unwrap();
        println!(r#"filename: "{}""#, filename.to_str().unwrap());
        let file = std::fs::File::open(filename.as_path()).unwrap();
        let yaml_config: YamlConfig = serde_yaml::from_reader(file).unwrap();
        assert!(yaml_config.arboric.log.console.is_some());
        assert_eq!("info", yaml_config.arboric.log.console.unwrap().level);
        assert!(yaml_config.arboric.log.file.is_none());
        assert!(yaml_config.listeners.is_some());
        let listeners = yaml_config.listeners.unwrap();
        assert!(!listeners.is_empty());
        let first = listeners.first().unwrap();
        println!("{:?}", first);
        assert_eq!("localhost", first.bind);
        assert_eq!(4000, first.port);
    }
}
