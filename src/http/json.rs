use serde::de::DeserializeOwned;

pub trait JsonParser {
    fn parse<T: DeserializeOwned>(&self, body: &[u8]) -> Result<T, JsonError>;
}

pub enum JsonError {
    Custom {
        err_name: String,
        err_msg: Option<String>,
    },
    InvalidJson(Box<dyn std::error::Error + Send + Sync>),
}

pub struct SerdeJsonParser;

impl SerdeJsonParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl JsonParser for SerdeJsonParser {
    fn parse<T: DeserializeOwned>(&self, body: &[u8]) -> Result<T, JsonError> {
        match serde_json::from_slice(body) {
            Ok(v) => Ok(v),
            Err(e) => {
                if e.is_data() {
                    let s = e.to_string();
                    let Some((err_name, err_msg)) = extract_error_name(&s) else {
                        return Err(JsonError::InvalidJson(Box::new(e)));
                    };
                    Err(JsonError::Custom { err_name, err_msg })
                } else {
                    Err(JsonError::InvalidJson(Box::new(e)))
                }
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
