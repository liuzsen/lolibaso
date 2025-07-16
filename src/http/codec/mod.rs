use crate::provider::Provider;

pub mod decoder;
pub mod encoder;

pub trait Codec<'a, T, F: Format>: encoder::Encoder<T, F> + decoder::Decoder<'a, T, F> {}

impl<'a, A, T, F: Format> Codec<'a, T, F> for A where
    A: encoder::Encoder<T, F> + decoder::Decoder<'a, T, F>
{
}

pub trait CodecBytes<'a, T, F: Format>:
    encoder::EncoderBytes<T, F> + decoder::Decoder<'a, T, F>
{
}

impl<'a, A, T, F: Format> CodecBytes<'a, T, F> for A where
    A: encoder::EncoderBytes<T, F> + decoder::Decoder<'a, T, F>
{
}

pub struct SimpleCodec;

impl Provider for SimpleCodec {
    fn build(_ctx: &mut crate::provider::ProviderContext) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

pub trait Format {}

pub struct Json;

impl Format for Json {}

pub struct UrlEncodedQuery;

impl Format for UrlEncodedQuery {}
