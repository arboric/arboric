//! The arboric module. Functions and structs in this file are available
//! in the `arboric::` namespace

use graphql_parser::query::Definition::Operation;
use graphql_parser::query::{parse_query, Document, OperationDefinition, SelectionSet};
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::collections::HashMap;

pub mod abac;
pub mod config;
pub mod graphql;
pub mod influxdb;

mod error;
mod proxy;
mod proxy_service;

// arboric::ArboricError;
pub use error::ArboricError;
// arboric::Proxy
pub use proxy::Proxy;
// arboric::ProxyService
pub use proxy_service::ProxyService;

type QueryCounts = HashMap<String, usize>;
type ParsePostResult = crate::Result<Option<(Document, QueryCounts)>>;

pub fn parse_post(content_type: Option<mime::Mime>, body: &String) -> ParsePostResult {
    trace!("parse_post({:?}, {:?})", &content_type, &body);
    let application_graphql: mime::Mime = "application/graphql".parse().unwrap();
    match content_type {
        Some(ref mime_type) if &application_graphql == mime_type => count_top_level_fields(body),
        Some(ref mime_type) if &mime::APPLICATION_JSON == mime_type => {
            match count_json_query(body) {
                Ok(results) => Ok(results),
                Err(err) => {
                    warn!("{:?}", err);
                    Err(err)
                }
            }
        }
        Some(mime_type) => {
            warn!("Don't know how to handle {}!", &mime_type);
            Ok(None)
        }
        None => {
            warn!("No content-type specified, will try to parse as application/graphql");
            count_top_level_fields(body)
        }
    }
}

pub fn log_counts(influx_db_backend: &influxdb::Backend, map: &QueryCounts) {
    use influx_db_client::{Client, Point, Points, Precision, Value};
    let total: usize = map.values().sum();
    info!(
        "Found {} ({} unique) fields/queries",
        total,
        map.keys().count()
    );

    let client = Client::new("http://localhost:8086", "arboric");

    let mut points: Vec<Point> = Vec::new();
    for (field, n) in map {
        debug!("{}: {}", &field, &n);
        let point = Point::new("queries")
            .add_tag("field", Value::String(field.clone()))
            .add_field("n", Value::Integer(*n as i64))
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

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLJSONQuery {
    query: String,
    operation_name: Option<String>,
    variables: Option<HashMap<String, Value>>,
}

fn count_json_query(body: &str) -> ParsePostResult {
    trace!("count_json_query({})", &body);
    let q: GraphQLJSONQuery = serde_json::from_str(body)?;
    trace!("{:?}", &q);
    trace!("{}", &q.query);
    count_top_level_fields(q.query.as_str())
}

/// Counts the top level fields in the given GraphQL query string
fn count_top_level_fields(query: &str) -> ParsePostResult {
    trace!("count_top_level_fields({:?})", &query);
    let mut results: HashMap<String, usize> = HashMap::new();
    let document = parse_query(&query)?;

    trace!("document => {:?}", &document);
    for def in document.definitions.iter() {
        match def {
            Operation(OperationDefinition::Query(query)) => {
                if let Some(query_name) = &query.name {
                    debug!("query.name => {}", query_name);
                }
                update_results(&mut results, &query.selection_set);
            }
            Operation(OperationDefinition::SelectionSet(selection_set)) => {
                update_results(&mut results, &selection_set);
            }
            _ => warn!("{:?}", def),
        }
    }

    return Ok(Some((document, results)));
}

fn update_results(results: &mut HashMap<String, usize>, selection_set: &SelectionSet) {
    for selection in selection_set.items.iter() {
        match selection {
            graphql_parser::query::Selection::Field(field) => {
                trace!("field.name => {}", &field.name);
                let n = results.entry(field.name.clone()).or_insert(0);
                *n += 1;
            }
            _ => {
                trace!("{:?}", selection);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_count_top_level_fields() {
        crate::initialize_logging();
        let mut expected: QueryCounts = HashMap::new();
        expected.insert("foo".into(), 1);
        let (_, counts) = count_top_level_fields("{foo{id}}").unwrap().unwrap();
        assert_eq!(counts, expected);
        let q = "
        {
            foo(id: 1) {
                name
            }
            bar {
                name
            }
        }
        ";
        expected.insert("bar".into(), 1);
        let (_, counts2) = count_top_level_fields(&q).unwrap().unwrap();
        assert_eq!(counts2, expected);
    }
}
