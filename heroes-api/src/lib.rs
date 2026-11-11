//! Star Wars "heroes" GraphQL backend, built with Juniper and Rocket.

pub mod data;
pub mod schema;

use juniper_rocket::{GraphQLRequest, GraphQLResponse};
use rocket::{Build, Rocket, State, get, post, routes};

use crate::{data::Database, schema::Schema};

#[post("/graphql", data = "<request>")]
async fn post_graphql(
    request: GraphQLRequest,
    schema: &State<Schema>,
    db: &State<Database>,
) -> GraphQLResponse {
    request.execute(schema, db).await
}

#[get("/graphiql")]
fn graphiql() -> rocket::response::content::RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[get("/health")]
fn health() -> &'static str {
    "ok"
}

/// Builds the Rocket instance, so tests can start it in-process.
pub fn build_rocket() -> Rocket<Build> {
    rocket::build()
        .manage(Database::new())
        .manage(schema::schema())
        .mount("/", routes![post_graphql, graphiql, health])
}
