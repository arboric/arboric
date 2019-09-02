use graphql_parser::query::Definition::Operation;
use graphql_parser::query::{parse_query, OperationDefinition};
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use serde_json::Result;
use std::collections::HashMap;

pub mod proxy_service;
pub use proxy_service::ProxyService;

pub fn log_post(content_type: Option<mime::Mime>, body: &String) {
    use influx_db_client::{Client, Point, Points, Precision, Value};

    let application_graphql: mime::Mime = "application/graphql".parse().unwrap();
    trace!("log_post({:?}, {:?})", &content_type, &body);
    let n = match content_type {
        Some(ref mime_type) if &application_graphql == mime_type => count_top_level_fields(body),
        Some(ref mime_type) if mime_type == &mime::APPLICATION_JSON => {
            match count_json_query(body) {
                Ok(count) => count,
                Err(err) => {
                    warn!("{:?}", err);
                    0
                }
            }
        }
        Some(mime_type) => {
            warn!("Don't know how to handle {}!", &mime_type);
            0
        }
        None => {
            warn!("No content-type specified, will try to parse as application/graphql");
            count_top_level_fields(body)
        }
    };
    info!("Found {} fields/queries", n);

    let client = Client::new("http://localhost:8086", "arboric");

    let point = Point::new("queries")
        .add_field("n", Value::Integer(n as i64))
        .to_owned();

    let points = points!(point);

    // if Precision is None, the default is second
    // Multiple write
    // let _ = client
    //     .write_points(points, Some(Precision::Milliseconds), None)
    //     .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLJSONQuery {
    query: String,
    operation_name: Option<String>,
    variables: Option<HashMap<String, Value>>,
}

fn count_json_query(body: &str) -> Result<usize> {
    trace!("count_json_query({})", &body);
    let q: GraphQLJSONQuery = serde_json::from_str(body)?;
    trace!("{:?}", &q);
    trace!("{}", &q.query);
    Ok(count_top_level_fields(q.query.as_str()))
}

/// Counts the top level fields in the given GraphQL query string
fn count_top_level_fields(query: &str) -> usize {
    trace!("count_top_level_fields({:?})", &query);
    let mut n: usize = 0;
    if let Ok(document) = parse_query(&query) {
        trace!("document => {:?}", &document);
        for def in document.definitions.iter() {
            match def {
                Operation(OperationDefinition::Query(query)) => {
                    if let Some(query_name) = &query.name {
                        debug!("query.name => {}", query_name);
                    }
                    let count = query.selection_set.items.iter().count();
                    n = n + count;
                }
                Operation(OperationDefinition::SelectionSet(selection_set)) => {
                    let count = selection_set.items.iter().count();
                    n = n + count;
                }
                _ => warn!("{:?}", def),
            }
        }
    };
    return n;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use influxdb::client::InfluxDbClient;
    use influxdb::query::{InfluxDbQuery, Timestamp};

    #[test]
    fn test_count_top_level_fields() {
        assert_eq!(count_top_level_fields("{}"), 0);
        assert_eq!(count_top_level_fields("{foo{id}}"), 1);
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
        assert_eq!(count_top_level_fields(&q), 2);
    }
}
