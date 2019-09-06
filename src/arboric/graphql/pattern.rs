//! Represents a pattern that can be used to match incoming
//! GraphQL requests (queries or mutations) by field, type, etc.
//! Used for ABAC/ACLs, and selective logging.

use graphql_parser::query::Field;
use regex::Regex;
use std::fmt;

/// A graphql::Pattern can be one of:
///   * `Any` - or `*` will match anything
///   * `Query` - or `query:...` will match a query
///   * `Mutation` - or `mutation:...` will match a mutation
#[derive(PartialEq, Debug)]
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
    /// ```
    pub fn parse(pattern: &str) -> Pattern {
        if pattern == "*" {
            Pattern::Any
        } else {
            let s = pattern.to_string();
            if s.starts_with("mutation:") {
                Pattern::mutation(&pattern[9..])
            } else if s.starts_with("query:") {
                Pattern::query(&pattern[6..])
            } else {
                Pattern::query(&s)
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

    pub fn matches(&self, field: &Field) -> bool {
        match self {
            Pattern::Any => true,
            Pattern::Query(ref field_pattern) => field_pattern.matches(field),
            Pattern::Mutation(ref field_pattern) => field_pattern.matches(field),
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
#[derive(PartialEq, Debug)]
pub struct FieldPattern(String);

impl FieldPattern {
    pub fn matches(&self, field: &Field) -> bool {
        let FieldPattern(s) = self;
        // TODO: compile Regex once
        Regex::new(&s.replace("*", ".*"))
            .unwrap()
            .is_match(field.name.as_str())
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
        let s: String = String::from("__type");
        assert_eq!(
            Pattern::parse(&s),
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
        assert!(FieldPattern("*".into()).matches(&field("foo")));
        assert!(FieldPattern("foo".into()).matches(&field("foo")));
        assert!(FieldPattern("foo".into()).matches(&query("{foo{id}}")));
    }
}
