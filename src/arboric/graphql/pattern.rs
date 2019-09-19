//! Represents a pattern that can be used to match incoming
//! GraphQL requests (queries or mutations) by field, type, etc.
//! Used for ABAC/ACLs, and selective logging.

use graphql_parser::query::{Field, OperationDefinition, Selection};
use log::trace;
use regex::Regex;
use std::borrow::Borrow;
use std::fmt;

/// A graphql::Pattern can be one of:
///   * `Any` - or `*` will match anything
///   * `Query` - or `query:...` will match a query
///   * `Mutation` - or `mutation:...` will match a mutation
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Any,
    Query(FieldPattern),
    Mutation(FieldPattern),
}

impl Pattern {
    /// Parses the given pattern string and returns a new `graphql::Pattern`
    ///
    /// # Examples
    ///
    /// ```
    /// use arboric::graphql::Pattern;
    ///
    /// assert_eq!(Pattern::parse("*"), Pattern::Any);
    /// assert_eq!(Pattern::parse("query:*"), Pattern::query("*"));
    /// assert_eq!(Pattern::parse("foo"), Pattern::query("foo"));
    /// assert_eq!(Pattern::parse("query:foo"), Pattern::query("foo"));
    /// assert_eq!(Pattern::parse("mutation:bar"), Pattern::mutation("bar"));
    /// ```
    pub fn parse<S>(s: S) -> Pattern
    where
        S: Into<String> + PartialEq,
    {
        let pattern: String = s.into();
        if pattern == "*" {
            Pattern::Any
        } else {
            if pattern.starts_with("mutation:") {
                Pattern::mutation(&pattern.as_str()[9..])
            } else if pattern.starts_with("query:") {
                Pattern::query(&pattern.as_str()[6..])
            } else {
                Pattern::query(&pattern.as_str())
            }
        }
    }

    /// Constructs a Pattern::Query with the given FieldPattern string
    pub fn query(s: &str) -> Pattern {
        Pattern::Query(FieldPattern(s.into()))
    }

    /// Constructs a Pattern::Mutation with then given FieldPattern string
    pub fn mutation(s: &str) -> Pattern {
        Pattern::Mutation(FieldPattern(s.into()))
    }

    /// Compares this Pattern against the GraphQL AST Field if it matches
    ///
    /// # Examples
    ///
    /// ```
    /// use arboric::graphql::Pattern;
    /// use graphql_parser::query::Definition::Operation;
    /// use graphql_parser::query::OperationDefinition;
    ///
    /// let doc = graphql_parser::parse_query("{hero{id name}}").unwrap();
    /// let op = doc.definitions.first().unwrap();
    /// if let Operation(od) = op {
    ///     assert!(Pattern::parse("*").matches(od));
    ///     assert!(Pattern::parse("query:*").matches(od));
    ///     assert!(Pattern::parse("query:hero").matches(od));
    ///     assert!(!Pattern::parse("mutation:createHero").matches(od));
    /// }
    ///
    pub fn matches(&self, operation_definition: &OperationDefinition) -> bool {
        trace!("matches({:?}, {:?})", &self, &operation_definition);
        match self {
            Pattern::Any => true,
            Pattern::Query(ref field_pattern) => match operation_definition {
                OperationDefinition::Query(query) => {
                    query
                        .selection_set
                        .items
                        .iter()
                        .any(|selection| match selection {
                            Selection::Field(field) => field_pattern.matches(field),
                            _ => false,
                        })
                }
                OperationDefinition::SelectionSet(selection_set) => {
                    selection_set.items.iter().any(|selection| match selection {
                        Selection::Field(field) => field_pattern.matches(field),
                        _ => false,
                    })
                }
                _ => false,
            },
            Pattern::Mutation(ref field_pattern) => {
                match operation_definition {
                    OperationDefinition::Mutation(mutation) => mutation
                        .selection_set
                        .items
                        .iter()
                        .any(|selection| match selection {
                            Selection::Field(field) => field_pattern.matches(field),
                            _ => false,
                        }),
                    _ => false,
                }
            }
        }
    }
}

impl fmt::Display for Pattern {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pattern::Any => write!(f, "*"),
            Pattern::Query(ref field_pattern) => write!(f, "query:{}", field_pattern),
            Pattern::Mutation(ref field_pattern) => write!(f, "mutation:{}", field_pattern),
        }
    }
}

/// A FieldPattern matches a query or mutation field
#[derive(Debug, Clone, PartialEq)]
pub struct FieldPattern(String);

impl FieldPattern {
    pub fn matches<F: Borrow<Field>>(&self, field: F) -> bool {
        let FieldPattern(s) = self;
        // TODO: compile Regex once
        Regex::new(&s.replace("*", ".*"))
            .unwrap()
            .is_match(field.borrow().name.as_str())
    }
}

impl fmt::Display for FieldPattern {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let FieldPattern(ref s) = *self;
        write!(f, "{}", &s)
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    use graphql_parser::query::Definition::Operation;
    use graphql_parser::query::{Field, OperationDefinition, Selection, SelectionSet};

    #[test]
    fn test_pattern_parse() {
        crate::initialize_logging();
        assert_eq!(
            Pattern::parse("__type"),
            Pattern::Query(FieldPattern("__type".into()))
        );
        assert_eq!(Pattern::parse("*"), Pattern::Any);
        assert_eq!(
            Pattern::parse("__schema"),
            Pattern::Query(FieldPattern("__schema".into()))
        );
        assert_eq!(
            Pattern::parse("query:*"),
            Pattern::Query(FieldPattern("*".into()))
        );
        assert_eq!(
            Pattern::parse("mutation:*"),
            Pattern::Mutation(FieldPattern("*".into()))
        );
    }

    #[test]
    fn test_pattern_matches() {
        crate::initialize_logging();
        let doc = graphql_parser::parse_query("{hero{id name}}").unwrap();
        let op = doc.definitions.first().unwrap();
        if let Operation(od) = op {
            assert!(Pattern::parse("*").matches(od));
            assert!(Pattern::parse("query:*").matches(od));
            assert!(Pattern::parse("query:hero").matches(od));
            assert!(!Pattern::parse("mutation:createHero").matches(od));
        } else {
            panic!(
                "Expected Definition::Operation(OperationDefintion), got {:?}!",
                &op
            );
        }
    }

    fn pos(line: usize, column: usize) -> graphql_parser::Pos {
        graphql_parser::Pos {
            line: line,
            column: column,
        }
    }

    fn field(name: &str) -> Field {
        Field {
            position: pos(1, 1),
            alias: None,
            name: name.into(),
            arguments: Vec::new(),
            directives: Vec::new(),
            selection_set: SelectionSet {
                span: (pos(1, 1), pos(1, 1)),
                items: Vec::new(),
            },
        }
    }

    fn query(s: &str) -> Field {
        let doc = graphql_parser::parse_query(&s).unwrap();
        match doc.definitions.iter().next().unwrap() {
            Operation(OperationDefinition::SelectionSet(ref selection_set)) => {
                let first = selection_set.items.iter().next().unwrap();
                match first {
                    Selection::Field(field) => field.clone(),
                    x => panic!("Don't know what to do with {:?}!", x),
                }
            }
            x => panic!("Don't know what to do with {:?}!", x),
        }
    }

    #[test]
    fn test_field_pattern_matches() {
        assert!(FieldPattern("*".into()).matches(field("foo")));
        assert!(FieldPattern("foo".into()).matches(field("foo")));
        assert!(FieldPattern("foo".into()).matches(query("{foo{id}}")));
    }
}
