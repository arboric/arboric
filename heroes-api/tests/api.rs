use heroes_api::build_rocket;
use rocket::{
    http::{ContentType, Status},
    local::blocking::Client,
};

fn post(client: &Client, content_type: ContentType, body: &str) -> (Status, String) {
    let response = client
        .post("/graphql")
        .header(content_type)
        .body(body)
        .dispatch();
    let status = response.status();
    (status, response.into_string().unwrap_or_default())
}

fn client() -> Client {
    Client::tracked(build_rocket()).expect("valid rocket")
}

#[test]
fn health() {
    let client = client();
    let response = client.get("/health").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn hero_with_friends_as_json() {
    let body = r#"{"query": "{hero{id name friends{id name}}}"}"#;
    let (status, text) = post(&client(), ContentType::JSON, body);
    assert_eq!(status, Status::Ok);
    assert!(text.contains(r#""name":"R2-D2""#), "{text}");
    assert!(text.contains(r#""name":"Luke Skywalker""#), "{text}");
}

#[test]
fn droid_by_id() {
    let body = r#"{"query": "{droid(id:\"3\"){name friends{name}}}"}"#;
    let (status, text) = post(&client(), ContentType::JSON, body);
    assert_eq!(status, Status::Ok);
    assert!(text.contains("R2-D2"), "{text}");
}

#[test]
fn unknown_droid_is_null() {
    let body = r#"{"query": "{droid(id:\"99\"){name}}"}"#;
    let (status, text) = post(&client(), ContentType::JSON, body);
    assert_eq!(status, Status::Ok);
    assert!(text.contains(r#""droid":null"#), "{text}");
}

#[test]
fn application_graphql_content_type() {
    let query = "{ hero { id name } }";
    let (status, text) = post(&client(), ContentType::new("application", "graphql"), query);
    assert_eq!(status, Status::Ok);
    assert!(text.contains("R2-D2"), "{text}");
}

#[test]
fn introspection() {
    let body = r#"{"query": "{__schema{queryType{name}}}"}"#;
    let (status, text) = post(&client(), ContentType::JSON, body);
    assert_eq!(status, Status::Ok);
    assert!(text.contains(r#""name":"Query""#), "{text}");
}

#[test]
fn empty_query_is_an_error() {
    let (status, text) = post(&client(), ContentType::JSON, r#"{"query": ""}"#);
    assert_eq!(status, Status::BadRequest);
    assert!(text.contains("errors"), "{text}");
}

#[test]
fn mutation_ping() {
    let body = r#"{"query": "mutation { ping }"}"#;
    let (status, text) = post(&client(), ContentType::JSON, body);
    assert_eq!(status, Status::Ok);
    assert!(text.contains(r#""ping":true"#), "{text}");
}
