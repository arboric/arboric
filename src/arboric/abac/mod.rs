//! Arboric ABAC (attribute-based access control) modules and functions

use crate::graphql::Pattern;
use graphql_parser::query::Definition::Operation;
use graphql_parser::query::{Document, OperationDefinition, Selection, SelectionSet};
use log::{debug, error, info, trace, warn};

/// A pdp::Policy represents an ABAC rule, which either
/// `Allow` or `Deny` a certain `arboric::graphql::Pattern`
#[derive(PartialEq, Debug)]
pub enum Policy {
    Allow(Pattern),
    Deny(Pattern),
}

pub struct PDP {
    pub rules: Vec<Policy>,
}

impl PDP {
    fn new() -> PDP {
        PDP {
            rules: vec![Policy::Allow(Pattern::parse("query:*"))],
        }
    }
    pub fn allow(&self, document: &Document) -> bool {
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
            _ => {
                warn!("Don't know how to handle{:?}", def);
                false
            }
        })
    }

    fn allow_all(&self, selection_set: &SelectionSet) -> bool {
        selection_set.items.iter().all(|selection| match selection {
            Selection::Field(_field) => false,
            // Don't know what to do with FragmentSpread or InlineFragment
            _ => true,
        })
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_pdp_no_rules() {
        let pdp = PDP { rules: vec![] };
        let document = graphql_parser::parse_query("{__schema{queryType{name}}}").unwrap();
        assert!(!pdp.allow(&document));
    }

    #[test]
    fn test_default_pdp() {
        let pdp = PDP::new();
        let document = graphql_parser::parse_query("{__schema{queryType{name}}}").unwrap();
        assert!(pdp.allow(&document));
    }
}
