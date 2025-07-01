use bytes::Bytes;
use bytestring::ByteString;
use futures_util::StreamExt;

use crate::{
    http::{
        adapter::HttpRequestModel,
        parser::{Json, Parser, UrlEncodedQuery},
        web_socket::protocol::WSMessage,
    },
    result::BizResult,
};

use super::{error::BizError, request::HttpRequest};
use protocol::ProtocolError;

#[cfg(feature = "actix-ws")]
pub mod impl_actix_ws;
pub mod protocol;

#[derive(Debug)]
pub struct Closed;

impl std::fmt::Display for Closed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WebSocket session is closed")
    }
}

impl std::error::Error for Closed {}

pub trait WSSession: 'static {
    async fn text(&mut self, text: impl Into<ByteString>) -> Result<(), Closed>;

    async fn binary(&mut self, binary: impl Into<Bytes>) -> Result<(), Closed>;

    async fn ping(&mut self, ping: &[u8]) -> Result<(), Closed>;

    async fn pong(&mut self, pong: &[u8]) -> Result<(), Closed>;

    async fn close(self) -> Result<(), Closed>;
}

pub trait WebSocketChan:
    futures_util::Stream<Item = Result<WSMessage, ProtocolError>> + WSSession + Unpin + 'static
{
    fn next_msg(&mut self) -> impl Future<Output = Option<Result<WSMessage, ProtocolError>>> {
        StreamExt::next(self)
    }
}

pub trait WSAdapter<P> {
    type Request: HttpRequestModel<Body = ()>;

    fn accept<R, F, W>(self, request: &R, parser: P, get_ws: F) -> BizResult<(), BizError>
    where
        R: HttpRequest,
        F: FnOnce() -> anyhow::Result<W>,
        W: WebSocketChan,
        for<'a> P: Parser<'a, <Self::Request as HttpRequestModel>::Query, UrlEncodedQuery>,
        for<'a> P: Parser<'a, <Self::Request as HttpRequestModel>::Body, Json>;
}
