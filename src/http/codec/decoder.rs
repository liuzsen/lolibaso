use serde::Deserialize;

use crate::{
    http::{
        codec::{Format, Json, SimpleCodec, UrlEncodedQuery},
        error::BizError,
    },
    provider::Provider,
};

pub trait Decoder<'a, T, F: Format = Json>: Provider {
    fn decode(&self, input: &'a [u8]) -> Result<T, DecodeError>;
}

#[derive(Debug, derive_more::Display)]
pub enum DecodeError {
    #[display("custom decode error: {err_name} {err_msg:?}")]
    Custom {
        err_name: String,
        err_msg: Option<String>,
    },
    BizErr(BizError),
}

impl std::error::Error for DecodeError {}

impl<'a, T> Decoder<'a, T, Json> for SimpleCodec
where
    T: Deserialize<'a>,
{
    fn decode(&self, input: &'a [u8]) -> Result<T, DecodeError> {
        match serde_json::from_slice(input) {
            Ok(v) => Ok(v),
            Err(e) => {
                if e.is_data() {
                    let s = e.to_string();
                    let Some((err_name, err_msg)) = extract_error_name(&s) else {
                        return Err(DecodeError::BizErr(BizError::InvalidJson.with_context(s)));
                    };
                    Err(DecodeError::Custom { err_name, err_msg })
                } else {
                    Err(DecodeError::BizErr(
                        BizError::InvalidJson.with_context(e.to_string()),
                    ))
                }
            }
        }
    }
}

impl<'a, T> Decoder<'a, T, UrlEncodedQuery> for SimpleCodec
where
    T: Deserialize<'a>,
{
    fn decode(&self, input: &'a [u8]) -> Result<T, DecodeError> {
        let res = serde_urlencoded::from_bytes::<T>(input);
        match res {
            Ok(t) => return Ok(t),
            Err(err) => {
                let s = err.to_string();
                let Some((err_name, err_msg)) = extract_error_name(&s) else {
                    return Err(DecodeError::BizErr(BizError::InvalidQuery.with_context(s)));
                };

                return Err(DecodeError::Custom { err_name, err_msg });
            }
        }
    }
}

fn extract_error_name(s: &str) -> Option<(String, Option<String>)> {
    let start = s.find("##")? + 2;
    if start >= s.len() {
        return None;
    }

    let end = s[start..].find("##")?;
    let err_name = &s[start..start + end];
    let mut split = err_name.split("(");
    let err_name = split.next()?.trim();
    let err_msg = if let Some(err_msg) = split.next() {
        let err_msg = err_msg.trim();
        if err_msg.ends_with(")") {
            Some(err_msg.trim_end_matches(")"))
        } else {
            Some(err_msg)
        }
    } else {
        None
    };

    Some((err_name.to_string(), err_msg.map(String::from)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_error_name() {
        let s = "##InvalidPortNumber(invalid digit found in string)## at line 1 column 10";
        let (err_name, err_msg) = extract_error_name(s).unwrap();
        assert_eq!(err_name, "InvalidPortNumber");
        assert_eq!(err_msg, Some("invalid digit found in string".to_string()));

        let s = "##InvalidPortNumber## at line 1 column 10";
        let (err_name, err_msg) = extract_error_name(s).unwrap();
        assert_eq!(err_name, "InvalidPortNumber");
        assert_eq!(err_msg, None);
    }
}
