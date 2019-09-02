//! The arboric module. Functions and structs in this file are available
//! in the `arboric::` namespace

use crate::ArboricError;
use graphql_parser::query::Definition::Operation;
use graphql_parser::query::{parse_query, OperationDefinition, SelectionSet};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::collections::HashMap;

pub mod proxy_service;
pub use proxy_service::ProxyService;

pub fn log_post(content_type: Option<mime::Mime>, body: &String) {
    use influx_db_client::{Client, Point, Points, Precision, Value};

    let application_graphql: mime::Mime = "application/graphql".parse().unwrap();
    trace!("log_post({:?}, {:?})", &content_type, &body);
    let results = match content_type {
        Some(ref mime_type) if &application_graphql == mime_type => count_top_level_fields(body),
        Some(ref mime_type) if mime_type == &mime::APPLICATION_JSON => {
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
            Ok(HashMap::new())
        }
        None => {
            warn!("No content-type specified, will try to parse as application/graphql");
            count_top_level_fields(body)
        }
    };
    if let Ok(map) = results {
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
                .add_tag("field", Value::String(field))
                .add_field("n", Value::Integer(n as i64))
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

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLJSONQuery {
    query: String,
    operation_name: Option<String>,
    variables: Option<HashMap<String, Value>>,
}

type QueryCounts = HashMap<String, usize>;
type QueryCountsResult = Result<QueryCounts, ArboricError>;

fn count_json_query(body: &str) -> QueryCountsResult {
    trace!("count_json_query({})", &body);
    let q: GraphQLJSONQuery = serde_json::from_str(body)?;
    trace!("{:?}", &q);
    trace!("{}", &q.query);
    count_top_level_fields(q.query.as_str())
}

/// Counts the top level fields in the given GraphQL query string
fn count_top_level_fields(query: &str) -> QueryCountsResult {
    trace!("count_top_level_fields({:?})", &query);
    let mut results: HashMap<String, usize> = HashMap::new();
    match parse_query(&query) {
        Ok(document) => {
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
        }
        Err(e) => {
            error!("graphql_parser::query::ParseError({})", e);
        }
    }
    return Ok(results);
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
        assert_eq!(count_top_level_fields("{}").unwrap(), expected);
        expected.insert("foo".into(), 1);
        assert_eq!(count_top_level_fields("{foo{id}}").unwrap(), expected);
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
        assert_eq!(count_top_level_fields(&q).unwrap(), expected);
    }
}
