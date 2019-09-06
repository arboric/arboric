//! Arboric ABAC (attribute-based access control) modules and functions

use crate::graphql::Pattern;
use graphql_parser::query::Definition::Operation;
use graphql_parser::query::{Document, Field, OperationDefinition, Selection, SelectionSet};
use log::{debug, error, info, trace, warn};

/// A pdp::Rule represents an ABAC rule, which either
/// `Allow` or `Deny` a certain `arboric::graphql::Pattern`
#[derive(PartialEq, Debug)]
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

    #[test]
    fn test_pdp_parse() {
        crate::initialize_logging();
        graphql_parser::parse_query("mutation Foo {foo(name:\"test\") { id }}").unwrap();
    }

    #[test]
    fn test_pdp_no_rules() {
        crate::initialize_logging();
        let pdp = PDP { rules: vec![] };
        let document = graphql_parser::parse_query("{__schema{queryType{name}}}").unwrap();
        assert!(!pdp.allow(&document));
    }

    #[test]
    fn test_pdp_default() {
        crate::initialize_logging();
        let pdp = PDP::new();
        assert!(pdp.allow(&graphql_parser::parse_query("{__schema{queryType{name}}}").unwrap()));
        assert!(pdp.allow(
            &graphql_parser::parse_query("mutation Foo {foo(name:\"test\") { id }}").unwrap()
        ));
    }
}
