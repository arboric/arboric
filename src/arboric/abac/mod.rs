//! Arboric ABAC (attribute-based access control) modules and functions

use crate::graphql::Pattern;
use crate::Request;
use graphql_parser::query::Definition::Operation;
use graphql_parser::query::OperationDefinition;
use log::{trace, warn};

pub trait RequestMatcher {
    fn matches(&self, request: &Request) -> bool;
}

/// A abac::Policy comprises:
///
/// * a list of `MatchAttribute`s, and
/// * a list of `Rule`s
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    attributes: Vec<MatchAttribute>,
    rules: Vec<Rule>,
}

impl Policy {
    /// Constructs an 'empty' Policy
    pub fn new() -> Self {
        Policy {
            attributes: Vec::new(),
            rules: Vec::new(),
        }
    }

    /// Constructs a Policy that matches and allows any request
    pub fn allow_any() -> Self {
        Policy {
            attributes: vec![MatchAttribute::Any],
            rules: vec![Rule::Allow(Pattern::Any)],
        }
    }

    pub fn add_match_attribute(&mut self, match_attribute: MatchAttribute) {
        self.attributes.push(match_attribute);
    }

    pub fn allows(&self, request: &Request) -> bool {
        if self
            .attributes
            .iter()
            .all(|attribute| attribute.matches(request))
        {
            let all = request.document.definitions.iter().all(|def| match def {
                Operation(operation_definition) => self.rules.iter().all(|rule| {
                    if rule.matches(operation_definition) {
                        trace!("Rule {:?} matches {:?}", &rule, &operation_definition);
                        if let Some(b) = rule.allows(operation_definition) {
                            b
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                }),
                _ => {
                    warn!("Don't know how to handle {:?}", def);
                    false
                }
            });
            trace!("all? {}", all);
            all
        } else {
            false
        }
    }
}

impl RequestMatcher for Policy {
    fn matches(&self, request: &Request) -> bool {
        self.attributes
            .iter()
            .all(|attribute| attribute.matches(request))
    }
}

/// A abac:MatchAttribute is a rule that can be used to match
/// an incoming Request to see if the associated ACLs apply to it
#[derive(Debug, Clone, PartialEq)]
pub enum MatchAttribute {
    Any,
    ClaimPresent { claim: String },
    ClaimEquals { claim: String, value: String },
    ClaimIncludes { claim: String, element: String },
}

impl MatchAttribute {
    // Creates a MatchAttribute::ClaimPresent
    pub fn claim_present<S>(claim: S) -> MatchAttribute
    where
        S: Into<String>,
    {
        MatchAttribute::ClaimPresent {
            claim: claim.into(),
        }
    }

    // Creates a MatchAttribute::ClaimEquals
    pub fn claim_equals<S, V>(claim: S, value: V) -> MatchAttribute
    where
        S: Into<String>,
        V: Into<String>,
    {
        MatchAttribute::ClaimEquals {
            claim: claim.into(),
            value: value.into(),
        }
    }

    // Creates a MatchAttribute::ClaimIncludes
    pub fn claim_includes<S, V>(claim: S, element: V) -> MatchAttribute
    where
        S: Into<String>,
        V: Into<String>,
    {
        MatchAttribute::ClaimIncludes {
            claim: claim.into(),
            element: element.into(),
        }
    }
}

impl RequestMatcher for MatchAttribute {
    fn matches(&self, request: &Request) -> bool {
        let claims = &request.claims;
        match self {
            MatchAttribute::ClaimPresent { claim } => {
                trace!("request.claims => {:?}", &request.claims);
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
            MatchAttribute::Any => true,
        }
    }
}

/// A abac::Rule will either `Allow` or `Deny` a certain `arboric::graphql::Pattern`
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    Allow(Pattern),
    Deny(Pattern),
}

impl Rule {
    pub fn allow<S>(s: S) -> Rule
    where
        S: Into<String> + PartialEq,
    {
        Rule::Allow(Pattern::parse(s))
    }

    pub fn deny<S>(s: S) -> Rule
    where
        S: Into<String> + PartialEq,
    {
        Rule::Deny(Pattern::parse(s))
    }

    pub fn matches(&self, operation_definition: &OperationDefinition) -> bool {
        trace!("matches({:?}, {:?})", &self, &operation_definition);
        match &self {
            Rule::Allow(pattern) => pattern.matches(operation_definition),
            Rule::Deny(pattern) => pattern.matches(operation_definition),
        }
    }

    pub fn allows(&self, operation_definition: &OperationDefinition) -> Option<bool> {
        trace!("allows({:?}, {:?}", &self, &operation_definition);
        match &self {
            Rule::Allow(pattern) => {
                if pattern.matches(operation_definition) {
                    trace!("returning Some(true)");
                    Some(true)
                } else {
                    None
                }
            }
            Rule::Deny(pattern) => {
                if pattern.matches(operation_definition) {
                    trace!("returning Some(false)");
                    Some(false)
                } else {
                    None
                }
            }
        }
    }
}

/// The abac::PDP or Policy Decision Point is responsible for holding
/// the list of `Policy`s. It evaluates incoming requests and
/// returns a Permit / Deny decision.
#[derive(Debug, Clone)]
pub struct PDP {
    policies: Vec<Policy>,
}

impl PDP {
    /// Constructs a PDP with no policies
    pub fn new() -> PDP {
        PDP {
            policies: Vec::new(),
        }
    }

    pub fn with_policies(policies: Vec<Policy>) -> PDP {
        PDP { policies: policies }
    }

    /// Constructs a default PDP with a single "allow any" Policy.
    pub fn default() -> PDP {
        PDP {
            policies: vec![Policy::allow_any()],
        }
    }

    pub fn allows(&self, request: &Request) -> bool {
        trace!("allow({:?})", &request);
        if self.policies.is_empty() {
            return false;
        }
        self.policies
            .iter()
            .filter(|policy| policy.matches(request))
            .any(|policy| policy.allows(request))
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;
    use frank_jwt::{decode, Algorithm};
    use serde_json::json;

    use std::borrow::Borrow;

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

    /// Constructs a test Request using the given claims (assumes a JSON Value::Object
    /// since I don't know how to write this as a macro) and query string
    fn request<C: Borrow<serde_json::Value>>(claims: C, query: &str) -> Request {
        Request {
            claims: claims.borrow().as_object().unwrap().to_owned(),
            document: graphql_parser::parse_query(query).unwrap(),
        }
    }

    #[test]
    fn test_abac_match_attributes_claim_present() {
        let request = request(json!({"sub": "1"}), "{foo{bar}}");
        assert!(MatchAttribute::claim_present("sub").matches(&request));
        assert!(!MatchAttribute::claim_present("roles").matches(&request));
    }

    #[test]
    fn test_abac_match_attributes_claim_equals() {
        let request = request(json!({"sub": "1"}), "{foo{bar}}");
        assert!(MatchAttribute::claim_equals("sub", "1").matches(&request));
        assert!(!MatchAttribute::claim_equals("sub", "2").matches(&request));
    }

    #[test]
    fn test_abac_match_attributes_claim_includes() {
        let request = request(json!({"roles": "user,admin"}), "{foo{bar}}");
        assert!(MatchAttribute::claim_includes("roles", "user").matches(&request));
        assert!(MatchAttribute::claim_includes("roles", "admin").matches(&request));
        assert!(!MatchAttribute::claim_includes("roles", "guest").matches(&request));
    }

    #[test]
    fn test_abac_rule_matches() {
        crate::initialize_logging();
        let doc = graphql_parser::parse_query("{foo{bar}}").unwrap();
        let op = doc.definitions.first().unwrap();
        if let Operation(od) = op {
            let allow_any = Rule::Allow(Pattern::Any);
            assert!(allow_any.matches(&od));
            assert!(allow_any.allows(&od).unwrap());

            let allow_foo = Rule::allow("foo");
            assert!(allow_foo.matches(&od));
            assert!(allow_foo.allows(&od).unwrap());

            let allow_query_foo = Rule::allow("query:foo");
            assert!(allow_query_foo.matches(&od));
            assert!(allow_query_foo.allows(&od).unwrap());

            let allow_mutation_foo = Rule::allow("mutation:foo");
            assert!(!allow_mutation_foo.matches(&od));
            assert!(allow_mutation_foo.allows(&od).is_none());

            let deny_all = Rule::Deny(Pattern::Any);
            assert!(deny_all.matches(&od));
            assert!(!deny_all.allows(&od).unwrap());

            let deny_foo = Rule::deny("foo");
            assert!(deny_foo.matches(&od));
            assert!(!deny_foo.allows(&od).unwrap());

            let deny_query_foo = Rule::deny("query:foo");
            assert!(deny_query_foo.matches(&od));
            assert!(!deny_query_foo.allows(&od).unwrap());

            let deny_mutation_foo = Rule::deny("mutation:foo");
            assert!(!deny_mutation_foo.matches(&od));
            assert!(deny_mutation_foo.allows(&od).is_none());
        } else {
            panic!(
                "Expected Definition::Operation(OperationDefintion), got {:?}!",
                &op
            );
        }
    }

    #[test]
    fn test_pdp_no_rules() {
        crate::initialize_logging();
        let pdp = PDP::new();
        let request = request(json!({"sub": "1"}), "{__schema{queryType{name}}}");
        assert!(!pdp.allows(&request));
    }

    #[test]
    fn test_pdp_allow_any() {
        crate::initialize_logging();
        let pdp = PDP::default();
        let request = request(json!({}), "{__schema{queryType{name}}}");
        assert!(pdp.allows(&request));
    }

    #[test]
    fn test_pdp_complex_example() {
        crate::initialize_logging();
        let user_policy = Policy {
            attributes: vec![MatchAttribute::claim_present("sub")],
            rules: vec![
                Rule::Allow(Pattern::query("*")),
                Rule::Deny(Pattern::mutation("*")),
                Rule::Deny(Pattern::query("__schema")),
            ],
        };
        let admin_policy = Policy {
            attributes: vec![MatchAttribute::claim_includes("roles", "admin")],
            rules: vec![
                Rule::Allow(Pattern::mutation("*")),
                Rule::Allow(Pattern::query("__schema")),
            ],
        };
        let pdp = PDP {
            policies: vec![user_policy, admin_policy],
        };

        assert!(!pdp.allows(&request(json!({}), "{foo{name}}")));
        let user_claims = json!({"sub": "1"});
        assert!(pdp.allows(&request(&user_claims, "{foo{name}}")));
        assert!(pdp.allows(&request(&user_claims, "query foo {foo{name}}")));
        assert!(!pdp.allows(&request(&user_claims, "{__schema{queryType{name}}}")));
        assert!(!pdp.allows(&request(
            user_claims,
            "mutation Createfoo {createfoo(name:\"Shazam!\") {foo{id}}}"
        )));
        let admin_claims = json!({"sub": "2", "roles": "user,admin"});
        assert!(pdp.allows(&request(&admin_claims, "{foo{name}}")));
        assert!(pdp.allows(&request(&admin_claims, "query foo {foo{name}}")));
        assert!(pdp.allows(&request(&admin_claims, "{__schema{queryType{name}}}")));
        assert!(pdp.allows(&request(
            admin_claims,
            "mutation Createfoo {createfoo(name:\"Shazam!\") {foo{id}}}"
        )));
    }

}
