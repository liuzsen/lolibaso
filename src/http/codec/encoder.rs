use crate::http::codec::{Format, Json, SimpleCodec, UrlEncodedQuery};

pub trait Encoder<T, F: Format> {
    fn encode(&self, input: &T) -> anyhow::Result<String>;
}

pub trait EncoderBytes<T, F: Format> {
    fn encode(&self, input: &T) -> anyhow::Result<Vec<u8>>;
}

impl<T, TT, F> EncoderBytes<T, F> for TT
where
    F: Format,
    TT: Encoder<T, F>,
{
    fn encode(&self, input: &T) -> anyhow::Result<Vec<u8>> {
        self.encode(input).map(|s| s.into_bytes())
    }
}

impl<T> Encoder<T, Json> for SimpleCodec
where
    T: serde::Serialize,
{
    fn encode(&self, input: &T) -> anyhow::Result<String> {
        serde_json::to_string(input).map_err(|e| anyhow::anyhow!("encode json error: {}", e))
    }
}

impl<T> Encoder<T, UrlEncodedQuery> for SimpleCodec
where
    T: serde::Serialize,
{
    fn encode(&self, input: &T) -> anyhow::Result<String> {
        serde_urlencoded::to_string(input)
            .map_err(|e| anyhow::anyhow!("encode urlencoded query error: {}", e))
    }
}
