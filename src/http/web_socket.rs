use bytes::Bytes;
use bytestring::ByteString;

use super::{error::BizError, request::HttpRequest};

pub trait WSMessage: Send + Sync + 'static {}

#[derive(Debug)]
pub struct Closed;

impl std::fmt::Display for Closed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Session is closed")
    }
}

impl std::error::Error for Closed {}

pub trait WSSession: 'static {
    type ProtocolError;
    type Message;

    async fn text(&mut self, text: impl Into<ByteString>) -> Result<(), Closed>;

    async fn binary(&mut self, binary: impl Into<Bytes>) -> Result<(), Closed>;

    async fn ping(&mut self, ping: &[u8]) -> Result<(), Closed>;

    async fn pong(&mut self, pong: &[u8]) -> Result<(), Closed>;

    async fn close(self) -> Result<(), Closed>;
}

pub trait WSSessionStream:
    futures_util::Stream<
        Item = Result<<Self as WSSession>::Message, <Self as WSSession>::ProtocolError>,
    > + WSSession
{
}

pub trait WSAdapter<C> {
    fn accept<R>(&self, req: R) -> Result<(), BizError>
    where
        R: HttpRequest;

    async fn run<W>(self, chan_builder: C, ws: W)
    where
        W: WSSessionStream;
}

#[cfg(feature = "actix-ws")]
mod impl_actix_ws {
    use super::*;
    use actix_ws::AggregatedMessage;

    pub struct ActixWebSocket {
        session: actix_ws::Session,
        stream: actix_ws::AggregatedMessageStream,
    }

    impl futures_util::Stream for ActixWebSocket {
        type Item = Result<AggregatedMessage, actix_ws::ProtocolError>;

        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            let this = self.get_mut();
            let stream = std::pin::Pin::new(&mut this.stream);
            stream.poll_next(cx)
        }
    }

    impl WSSession for ActixWebSocket {
        type ProtocolError = actix_ws::ProtocolError;
        type Message = AggregatedMessage;

        async fn text(&mut self, text: impl Into<ByteString>) -> Result<(), Closed> {
            self.session.text(text).await.map_err(|_| Closed)
        }

        async fn binary(&mut self, binary: impl Into<Bytes>) -> Result<(), Closed> {
            self.session.binary(binary).await.map_err(|_| Closed)
        }

        async fn ping(&mut self, ping: &[u8]) -> Result<(), Closed> {
            self.session.ping(ping).await.map_err(|_| Closed)
        }

        async fn pong(&mut self, pong: &[u8]) -> Result<(), Closed> {
            self.session.pong(pong).await.map_err(|_| Closed)
        }

        async fn close(self) -> Result<(), Closed> {
            self.session.close(None).await.map_err(|_| Closed)
        }
    }

    impl WSSessionStream for ActixWebSocket {}
}
