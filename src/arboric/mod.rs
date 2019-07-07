use graphql_parser::query::Definition::Operation;
use graphql_parser::query::{parse_query, OperationDefinition};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri};
use log::{debug, info, trace, warn};

pub fn log_post(body: &String) {
    trace!("log_post({:?})", &body);
    let n = count_top_level_fields(body);
    info!("Found {} fields/queries", n);
}

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
