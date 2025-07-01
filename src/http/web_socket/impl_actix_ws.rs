use crate::http::web_socket::protocol::WSMessage;

use super::*;
use actix_ws::AggregatedMessage;

pub struct WebSocketActix {
    session: actix_ws::Session,
    stream: actix_ws::AggregatedMessageStream,
}

impl From<actix_ws::CloseCode> for protocol::CloseCode {
    fn from(code: actix_ws::CloseCode) -> Self {
        let u16 = u16::from(code);
        protocol::CloseCode::from(u16)
    }
}

impl From<actix_ws::CloseReason> for protocol::CloseReason {
    fn from(reason: actix_ws::CloseReason) -> Self {
        protocol::CloseReason {
            code: From::from(reason.code),
            description: reason.description,
        }
    }
}

impl From<actix_ws::ProtocolError> for protocol::ProtocolError {
    fn from(value: actix_ws::ProtocolError) -> Self {
        match value {
            actix_ws::ProtocolError::UnmaskedFrame => protocol::ProtocolError::UnmaskedFrame,
            actix_ws::ProtocolError::MaskedFrame => protocol::ProtocolError::MaskedFrame,
            actix_ws::ProtocolError::InvalidOpcode(op_code) => {
                protocol::ProtocolError::InvalidOpcode(op_code)
            }
            actix_ws::ProtocolError::InvalidLength(length) => {
                protocol::ProtocolError::InvalidLength(length)
            }
            actix_ws::ProtocolError::BadOpCode => protocol::ProtocolError::BadOpCode,
            actix_ws::ProtocolError::Overflow => protocol::ProtocolError::Overflow,
            actix_ws::ProtocolError::ContinuationNotStarted => {
                protocol::ProtocolError::ContinuationNotStarted
            }
            actix_ws::ProtocolError::ContinuationStarted => {
                protocol::ProtocolError::ContinuationStarted
            }
            actix_ws::ProtocolError::ContinuationFragment(op_code) => {
                protocol::ProtocolError::ContinuationFragment(From::from(op_code))
            }
            actix_ws::ProtocolError::Io(error) => protocol::ProtocolError::Io(error),
        }
    }
}

impl From<actix_http::ws::OpCode> for protocol::OpCode {
    fn from(value: actix_http::ws::OpCode) -> Self {
        match value {
            actix_http::ws::OpCode::Continue => protocol::OpCode::Continue,
            actix_http::ws::OpCode::Text => protocol::OpCode::Text,
            actix_http::ws::OpCode::Binary => protocol::OpCode::Binary,
            actix_http::ws::OpCode::Close => protocol::OpCode::Close,
            actix_http::ws::OpCode::Ping => protocol::OpCode::Ping,
            actix_http::ws::OpCode::Pong => protocol::OpCode::Pong,
            actix_http::ws::OpCode::Bad => protocol::OpCode::Bad,
        }
    }
}

impl futures_util::Stream for WebSocketActix {
    type Item = Result<super::protocol::WSMessage, super::ProtocolError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let stream = std::pin::Pin::new(&mut this.stream);
        let msg = std::task::ready!(stream.poll_next(cx));
        let Some(msg) = msg else {
            return std::task::Poll::Ready(None);
        };

        let msg = match msg {
            Ok(msg) => match msg {
                AggregatedMessage::Text(byte_string) => WSMessage::Text(byte_string),
                AggregatedMessage::Binary(bytes) => WSMessage::Binary(bytes),
                AggregatedMessage::Ping(bytes) => WSMessage::Ping(bytes),
                AggregatedMessage::Pong(bytes) => WSMessage::Pong(bytes),
                AggregatedMessage::Close(close_reason) => {
                    WSMessage::Close(close_reason.map(From::from))
                }
            },
            Err(e) => {
                return std::task::Poll::Ready(Some(Err(ProtocolError::from(e))));
            }
        };

        std::task::Poll::Ready(Some(Ok(msg)))
    }
}

impl WSSession for WebSocketActix {
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

impl WebSocketChan for WebSocketActix {}
