use serde::de::DeserializeOwned;

pub trait QueryDecoder {
    fn decode<T: DeserializeOwned>(&self, query_str: &str) -> Result<T, QueryParseError>;
}

pub enum QueryParseError {
    Custom {
        err_name: String,
        err_msg: Option<String>,
    },
    InvalidQuery(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(feature = "serde_urlencoded")]
pub use impl_serde_urlencoded::QueryParserSerde;

#[cfg(feature = "serde_urlencoded")]
mod impl_serde_urlencoded {
    use serde::de::DeserializeOwned;

    use super::{QueryDecoder, QueryParseError};
    use crate::http::json::extract_error_name;

    pub struct QueryParserSerde;

    impl QueryDecoder for QueryParserSerde {
        fn decode<T: DeserializeOwned>(&self, query_str: &str) -> Result<T, QueryParseError> {
            let res = serde_urlencoded::from_str::<T>(query_str);
            match res {
                Ok(t) => return Ok(t),
                Err(err) => {
                    let s = err.to_string();
                    let Some((err_name, err_msg)) = extract_error_name(&s) else {
                        return Err(QueryParseError::InvalidQuery(Box::new(err)));
                    };

                    return Err(QueryParseError::Custom { err_name, err_msg });
                }
            }
        }
    }
}
