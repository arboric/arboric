//! The ArboricError is an Error type used to 'wrap' most other
//! runtime errors

/// Arboric error type to 'wrap' other, underlying error causes
#[derive(Debug, thiserror::Error)]
pub enum ArboricError {
    #[error("{message}")]
    GeneralError { message: String },

    #[error("Unable to find or decode ENV['{message}']")]
    EnvVarError {
        message: String,
        #[source]
        cause: std::env::VarError,
    },

    #[error("{message}")]
    HexDecodeError {
        message: String,
        #[source]
        cause: hex::FromHexError,
    },

    #[error("{message}")]
    Base64DecodeError {
        message: String,
        #[source]
        cause: base64::DecodeError,
    },

    #[error("{message}")]
    IoError {
        message: String,
        #[source]
        cause: std::io::Error,
    },

    #[error("{message}")]
    JsonError {
        message: String,
        #[source]
        cause: serde_json::Error,
    },

    #[error("{message}")]
    YamlError {
        message: String,
        #[source]
        cause: yaml_serde::Error,
    },

    #[error("{message}")]
    GraphqlParserError {
        message: String,
        cause: graphql_parser::query::ParseError,
    },
}

impl ArboricError {
    pub fn general<S: Into<String>>(message: S) -> ArboricError {
        ArboricError::GeneralError {
            message: message.into(),
        }
    }
}

impl From<std::env::VarError> for ArboricError {
    fn from(error: std::env::VarError) -> Self {
        ArboricError::EnvVarError {
            message: format!("{:?}", error),
            cause: error,
        }
    }
}

impl From<hex::FromHexError> for ArboricError {
    fn from(error: hex::FromHexError) -> Self {
        ArboricError::HexDecodeError {
            message: format!("{:?}", error),
            cause: error,
        }
    }
}

impl From<base64::DecodeError> for ArboricError {
    fn from(error: base64::DecodeError) -> Self {
        ArboricError::Base64DecodeError {
            message: format!("{:?}", error),
            cause: error,
        }
    }
}

impl From<std::io::Error> for ArboricError {
    fn from(io_error: std::io::Error) -> Self {
        ArboricError::IoError {
            message: format!("{:?}", io_error),
            cause: io_error,
        }
    }
}

impl From<serde_json::Error> for ArboricError {
    fn from(json_error: serde_json::Error) -> Self {
        ArboricError::JsonError {
            message: format!("{:?}", json_error),
            cause: json_error,
        }
    }
}

impl From<yaml_serde::Error> for ArboricError {
    fn from(yaml_error: yaml_serde::Error) -> Self {
        ArboricError::YamlError {
            message: format!("{:?}", yaml_error),
            cause: yaml_error,
        }
    }
}

impl From<graphql_parser::query::ParseError> for ArboricError {
    fn from(parser_error: graphql_parser::query::ParseError) -> Self {
        ArboricError::GraphqlParserError {
            message: format!("{:?}", parser_error),
            cause: parser_error,
        }
    }
}
