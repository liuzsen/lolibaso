use serde::Deserialize;

pub trait Parser<'a, T, F: Format = Json> {
    fn parse(&self, input: &'a [u8]) -> Result<T, ParseError>;
}

pub trait Format {}

pub enum ParseError {
    Custom {
        err_name: String,
        err_msg: Option<String>,
    },
    Other(anyhow::Error),
}

pub struct SimpleParser;

pub struct Json;

impl Format for Json {}

impl<'a, T> Parser<'a, T, Json> for SimpleParser
where
    T: Deserialize<'a>,
{
    fn parse(&self, input: &'a [u8]) -> Result<T, ParseError> {
        match serde_json::from_slice(input) {
            Ok(v) => Ok(v),
            Err(e) => {
                if e.is_data() {
                    let s = e.to_string();
                    let Some((err_name, err_msg)) = extract_error_name(&s) else {
                        return Err(ParseError::Other(From::from(e)));
                    };
                    Err(ParseError::Custom { err_name, err_msg })
                } else {
                    Err(ParseError::Other(From::from(e)))
                }
            }
        }
    }
}

pub struct UrlEncodedQuery;

impl Format for UrlEncodedQuery {}

impl<'a, T> Parser<'a, T, UrlEncodedQuery> for SimpleParser
where
    T: Deserialize<'a>,
{
    fn parse(&self, input: &'a [u8]) -> Result<T, ParseError> {
        let res = serde_urlencoded::from_bytes::<T>(input);
        match res {
            Ok(t) => return Ok(t),
            Err(err) => {
                let s = err.to_string();
                let Some((err_name, err_msg)) = extract_error_name(&s) else {
                    return Err(ParseError::Other(From::from(err)));
                };

                return Err(ParseError::Custom { err_name, err_msg });
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
