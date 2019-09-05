//! Represents a pattern that can be used to match incoming
//! GraphQL requests (queries or mutations) by field, type, etc.
//! Used for ABAC/ACLs, and selective logging.

use log::{debug, error, info, trace, warn};
use std::fmt;

pub type Patterns = Vec<Pattern>;

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
    /// assert_eq!(Pattern::Any, Pattern::parse("*"));
    /// ```
    pub fn parse(pattern: &str) -> Pattern {
        if pattern == "*" {
            Pattern::Any
        } else {
            let s = pattern.to_string();
            if s.starts_with("mutation:") {
                Pattern::Mutation(FieldPattern(pattern[9..].to_owned()))
            } else if s.starts_with("query:") {
                Pattern::Query(FieldPattern(pattern[6..].to_owned()))
            } else {
                Pattern::Query(FieldPattern(s))
            }
        }
    }
}

impl fmt::Display for Pattern {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Pattern::Any => write!(f, "*"),
            Pattern::Query(ref field_pattern) => write!(f, "query:{}", field_pattern),
            Pattern::Mutation(ref field_pattern) => write!(f, "mutation:{}", field_pattern),
        }
    }
}

pub type Fields = Vec<FieldPattern>;

/// A FieldPattern matches a query or mutation field
#[derive(PartialEq, Debug)]
pub struct FieldPattern(String);

impl FieldPattern {}

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

    #[test]
    fn test_pattern_parse() {
        crate::initialize_logging();
        print!("test_pattern_parse()");
        let s: String = String::from("__type");
        assert_eq!(
            Pattern::parse(&s),
            Pattern::Query(FieldPattern("__type".into()))
        );
        println!("s => {}", s);
        assert_eq!(Pattern::parse("*"), Pattern::Any);
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
}
