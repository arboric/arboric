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
    use influx_db_client::{Client, Point, Points, Precision, Value};

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

    #[test]
    fn test_influxdb() {
        // default with "http://127.0.0.1:8086", db with "test"
        let client = Client::default().set_authentication("root", "root");

        let mut point = point!("test1");
        point
            .add_field("foo", Value::String("bar".to_string()))
            .add_field("integer", Value::Integer(11))
            .add_field("float", Value::Float(22.3))
            .add_field("'boolean'", Value::Boolean(false));

        let point1 = Point::new("test1")
            .add_tag("tags", Value::String(String::from("\\\"fda")))
            .add_tag("number", Value::Integer(12))
            .add_tag("float", Value::Float(12.6))
            .add_field("fd", Value::String("'3'".to_string()))
            .add_field("quto", Value::String("\\\"fda".to_string()))
            .add_field("quto1", Value::String("\"fda".to_string()))
            .to_owned();

        let points = points!(point1, point);

        // if Precision is None, the default is second
        // Multiple write
        let _ = client
            .write_points(points, Some(Precision::Seconds), None)
            .unwrap();

        // query, it's type is Option<Vec<Node>>
        let res = client.query("select * from test1", None).unwrap();
        println!("{:?}", res.unwrap()[0].series)
    }
}
