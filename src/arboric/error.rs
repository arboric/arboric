//! The ArboricError is an Error type used to 'wrap' most other
//! runtime errors
use failure::Fail;

/// Arboric error type to 'wrap' other, underlying error causes
#[derive(Debug, Fail)]
pub enum ArboricError {
    #[fail(display = "{}", message)]
    GeneralError { message: String },

    #[fail(display = "{}", message)]
    EnvVarError {
        message: String,
        #[cause]
        cause: std::env::VarError,
    },

    #[fail(display = "{}", message)]
    HexDecodeError {
        message: String,
        #[cause]
        cause: hex::FromHexError,
    },

    #[fail(display = "{}", message)]
    Base64DecodeError {
        message: String,
        #[cause]
        cause: base64::DecodeError,
    },

    #[fail(display = "{}", message)]
    IoError {
        message: String,
        #[cause]
        cause: std::io::Error,
    },

    #[fail(display = "{}", message)]
    JsonError {
        message: String,
        #[cause]
        cause: serde_json::Error,
    },

    #[fail(display = "{}", message)]
    YamlError {
        message: String,
        #[cause]
        cause: serde_yaml::Error,
    },

    #[fail(display = "{}", message)]
    GraphqlParserError {
        message: String,
        #[cause]
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

// macro_rules! impl_from {
//     ($($type:ty),+) => {
//         $(
//             impl From<$type> for ArboricError {
//                 fn from(error: $type) -> Self {
//                     ArboricError::GeneralError { message: format!("{:?}", error) }
//                 }
//             }
//         )*
//     };
// }
// impl_from!(hex::FromHexError);

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

impl From<serde_yaml::Error> for ArboricError {
    fn from(yaml_error: serde_yaml::Error) -> Self {
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
