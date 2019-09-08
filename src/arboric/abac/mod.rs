//! Arboric ABAC (attribute-based access control) modules and functions

use crate::graphql::Pattern;
use crate::Request;
use graphql_parser::query::Definition::Operation;
use graphql_parser::query::{Document, Field, OperationDefinition, Selection, SelectionSet};
use log::{debug, trace, warn};
use std::borrow::Borrow;

/// A abac::Policy comprises:
///
/// * a list of `MatchAttribute`s, and
/// * a list of `Rule`s
#[derive(Debug, PartialEq)]
pub struct Policy {
    attributes: Vec<MatchAttribute>,
    rules: Vec<Rule>,
}

pub trait RequestMatcher {
    fn matches<R: Borrow<Request>>(&self, request: R) -> bool;
}

/// A pdp:MatchAttribute is a rule that can be used to match
/// an incoming Request to see if the associated ACLs apply to it
#[derive(Debug, PartialEq)]
pub enum MatchAttribute {
    ClaimPresent { claim: String },
    ClaimEquals { claim: String, value: String },
    ClaimIncludes { claim: String, element: String },
}

impl MatchAttribute {
    // Creates a MatchAttribute::ClaimPresent
    pub fn claim_present(claim: &str) -> MatchAttribute {
        MatchAttribute::ClaimPresent {
            claim: claim.to_owned(),
        }
    }

    // Creates a MatchAttribute::ClaimEquals
    pub fn claim_equals(claim: &str, value: &str) -> MatchAttribute {
        MatchAttribute::ClaimEquals {
            claim: claim.to_owned(),
            value: value.to_owned(),
        }
    }

    // Creates a MatchAttribute::ClaimIncludes
    pub fn claim_includes(claim: &str, element: &str) -> MatchAttribute {
        MatchAttribute::ClaimIncludes {
            claim: claim.to_owned(),
            element: element.to_owned(),
        }
    }
}

impl RequestMatcher for MatchAttribute {
    fn matches<R: Borrow<Request>>(&self, request: R) -> bool {
        let req = request.borrow();
        let claims = &req.claims;
        match self {
            MatchAttribute::ClaimPresent { claim } => {
                trace!("request.claims => {:?}", &req.claims);
                trace!("claim => {:?}", &claim);
                claims.contains_key(claim)
            }
            MatchAttribute::ClaimEquals { claim, value } => {
                claims.contains_key(claim)
                    && match claims.get(claim) {
                        Some(v) => value == v,
                        _ => false,
                    }
            }
            MatchAttribute::ClaimIncludes { claim, element } => {
                claims.contains_key(claim)
                    && match claims.get(claim) {
                        Some(v) => v
                            .as_str()
                            .unwrap()
                            .split(",")
                            .collect::<Vec<&str>>()
                            .contains(&element.as_ref()),
                        _ => false,
                    }
            }
        }
    }
}

/// A abac::Rule will either `Allow` or `Deny` a certain `arboric::graphql::Pattern`
#[derive(Debug, PartialEq)]
pub enum Rule {
    Allow(Pattern),
    Deny(Pattern),
}

impl Rule {
    pub fn allow(&self, field: &Field) -> bool {
        trace!("allow({:?}, {:?}", &self, &field);
        match &self {
            Rule::Allow(pattern) => pattern.matches(field),
            Rule::Deny(pattern) => !pattern.matches(field),
        }
    }
}

pub struct PDP {
    pub rules: Vec<Rule>,
}

impl PDP {
    pub fn new() -> PDP {
        PDP {
            rules: vec![Rule::Allow(Pattern::Any)],
        }
    }

    pub fn allow(&self, document: &Document) -> bool {
        trace!("allow({:?})", &document);
        if self.rules.is_empty() {
            return false;
        }
        document.definitions.iter().all(|def| match def {
            Operation(OperationDefinition::Query(query)) => {
                if let Some(query_name) = &query.name {
                    debug!("query.name => {}", query_name);
                }
                self.allow_all(&query.selection_set)
            }
            Operation(OperationDefinition::SelectionSet(ref selection_set)) => {
                self.allow_all(&selection_set)
            }
            Operation(OperationDefinition::Mutation(mutation)) => {
                if let Some(mutation_name) = &mutation.name {
                    debug!("mutation.name => {}", mutation_name);
                }
                self.allow_all(&mutation.selection_set)
            }
            _ => {
                warn!("Don't know how to handle {:?}", def);
                false
            }
        })
    }

    fn allow_all(&self, selection_set: &SelectionSet) -> bool {
        selection_set.items.iter().all(|selection| {
            trace!("selection => {:?}", &selection);
            match selection {
                Selection::Field(field) => self.rules.iter().all(|policy| policy.allow(field)),
                // Don't know what to do with FragmentSpread or InlineFragment
                _ => true,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;
    use frank_jwt::{decode, Algorithm};
    use serde_json::json;

    #[test]
    fn test_frank_jwt() {
        let secret_key_hex = "fb9f0a56c2195aa7294f7b076d145bb1a701decd06e8e32cbfdc2f3146a11b3637c5b77d2f98ffb5081af31ae180b69bf2b127ff2496f3c252fcaa20c89d1b019a4639fd26056b6136dd327d118c7d833b357d673d4ba79f1997c4d1d47b74549e0b0e827444fe36dcd7411c0a1384140121e099343d074b6a34c9179ed4687d";
        let secret_key = hex::decode(&secret_key_hex).unwrap();

        let s = "eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJsb2NhbGhvc3QiLCJzdWIiOiIxIiwicm9sZXMiOiJhZG1pbiJ9.OWRGbi-54ERS5stXrvJaofZL23HVbGEzyGmz-YCXbOE";
        let (header, payload) = decode(&s, &secret_key, Algorithm::HS256).unwrap();
        println!("header => {:?}", &header);
        println!("payload => {:?}", &payload);
        assert_eq!("HS256", header.as_object().unwrap().get("alg").unwrap());
        assert_eq!("1", payload.as_object().unwrap().get("sub").unwrap());
    }

    #[test]
    fn test_abac_match_attributes_claim_present() {
        let json = json!({"sub": "1"});
        let claims = json.as_object().unwrap();
        let request = Request {
            claims: claims.to_owned(),
        };
        assert!(MatchAttribute::claim_present("sub").matches(&request));
        assert!(!MatchAttribute::claim_present("roles").matches(&request));
    }

    #[test]
    fn test_abac_match_attributes_claim_equals() {
        let json = json!({"sub": "1"});
        let claims = json.as_object().unwrap();
        let request = Request {
            claims: claims.to_owned(),
        };
        assert!(MatchAttribute::claim_equals("sub", "1").matches(&request));
        assert!(!MatchAttribute::claim_equals("sub", "2").matches(&request));
    }

    #[test]
    fn test_abac_match_attributes_claim_includes() {
        let json = json!({"roles": "user,admin"});
        let claims = json.as_object().unwrap();
        let request = Request {
            claims: claims.to_owned(),
        };
        assert!(MatchAttribute::claim_includes("roles", "user").matches(&request));
        assert!(MatchAttribute::claim_includes("roles", "admin").matches(&request));
        assert!(!MatchAttribute::claim_includes("roles", "guest").matches(&request));
    }

    #[test]
    fn test_abac_no_rules() {
        crate::initialize_logging();
        let pdp = PDP { rules: vec![] };
        let document = graphql_parser::parse_query("{__schema{queryType{name}}}").unwrap();
        assert!(!pdp.allow(&document));
    }

    #[test]
    fn test_abac_default() {
        crate::initialize_logging();
        let pdp = PDP::new();
        assert!(pdp.allow(&graphql_parser::parse_query("{__schema{queryType{name}}}").unwrap()));
        assert!(pdp.allow(
            &graphql_parser::parse_query("mutation Foo {foo(name:\"test\") { id }}").unwrap()
        ));
    }
}
