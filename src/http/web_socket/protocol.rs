use bytes::Bytes;
use bytestring::ByteString;

#[derive(Debug, PartialEq, Eq)]
pub enum WSMessage {
    /// Text message.
    Text(ByteString),

    /// Binary message.
    Binary(Bytes),

    /// Ping message.
    Ping(Bytes),

    /// Pong message.
    Pong(Bytes),

    /// Close message with optional reason.
    Close(Option<CloseReason>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// Reason for closing the connection
pub struct CloseReason {
    /// Exit code
    pub code: CloseCode,

    /// Optional description of the exit code
    pub description: Option<String>,
}

impl From<CloseCode> for CloseReason {
    fn from(code: CloseCode) -> Self {
        CloseReason {
            code,
            description: None,
        }
    }
}

/// Status code used to indicate why an endpoint is closing the WebSocket connection.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CloseCode {
    /// Indicates a normal closure, meaning that the purpose for which the connection was
    /// established has been fulfilled.
    Normal,

    /// Indicates that an endpoint is "going away", such as a server going down or a browser having
    /// navigated away from a page.
    Away,

    /// Indicates that an endpoint is terminating the connection due to a protocol error.
    Protocol,

    /// Indicates that an endpoint is terminating the connection because it has received a type of
    /// data it cannot accept (e.g., an endpoint that understands only text data MAY send this if it
    /// receives a binary message).
    Unsupported,

    /// Indicates an abnormal closure. If the abnormal closure was due to an error, this close code
    /// will not be used. Instead, the `on_error` method of the handler will be called with
    /// the error. However, if the connection is simply dropped, without an error, this close code
    /// will be sent to the handler.
    Abnormal,

    /// Indicates that an endpoint is terminating the connection because it has received data within
    /// a message that was not consistent with the type of the message (e.g., non-UTF-8 \[RFC 3629\]
    /// data within a text message).
    Invalid,

    /// Indicates that an endpoint is terminating the connection because it has received a message
    /// that violates its policy. This is a generic status code that can be returned when there is
    /// no other more suitable status code (e.g., Unsupported or Size) or if there is a need to hide
    /// specific details about the policy.
    Policy,

    /// Indicates that an endpoint is terminating the connection because it has received a message
    /// that is too big for it to process.
    Size,

    /// Indicates that an endpoint (client) is terminating the connection because it has expected
    /// the server to negotiate one or more extension, but the server didn't return them in the
    /// response message of the WebSocket handshake.  The list of extensions that are needed should
    /// be given as the reason for closing. Note that this status code is not used by the server,
    /// because it can fail the WebSocket handshake instead.
    Extension,

    /// Indicates that a server is terminating the connection because it encountered an unexpected
    /// condition that prevented it from fulfilling the request.
    Error,

    /// Indicates that the server is restarting. A client may choose to reconnect, and if it does,
    /// it should use a randomized delay of 5-30 seconds between attempts.
    Restart,

    /// Indicates that the server is overloaded and the client should either connect to a different
    /// IP (when multiple targets exist), or reconnect to the same IP when a user has performed
    /// an action.
    Again,

    #[doc(hidden)]
    Tls,

    #[doc(hidden)]
    Other(u16),
}

impl From<CloseCode> for u16 {
    fn from(code: CloseCode) -> u16 {
        use self::CloseCode::*;

        match code {
            Normal => 1000,
            Away => 1001,
            Protocol => 1002,
            Unsupported => 1003,
            Abnormal => 1006,
            Invalid => 1007,
            Policy => 1008,
            Size => 1009,
            Extension => 1010,
            Error => 1011,
            Restart => 1012,
            Again => 1013,
            Tls => 1015,
            Other(code) => code,
        }
    }
}

impl From<u16> for CloseCode {
    fn from(code: u16) -> CloseCode {
        use self::CloseCode::*;

        match code {
            1000 => Normal,
            1001 => Away,
            1002 => Protocol,
            1003 => Unsupported,
            1006 => Abnormal,
            1007 => Invalid,
            1008 => Policy,
            1009 => Size,
            1010 => Extension,
            1011 => Error,
            1012 => Restart,
            1013 => Again,
            1015 => Tls,
            _ => Other(code),
        }
    }
}

use derive_more::{Display, Error, From};

/// WebSocket protocol errors.
#[derive(Debug, Display, Error, From)]
pub enum ProtocolError {
    /// Received an unmasked frame from client.
    #[display("received an unmasked frame from client")]
    UnmaskedFrame,

    /// Received a masked frame from server.
    #[display("received a masked frame from server")]
    MaskedFrame,

    /// Encountered invalid opcode.
    #[display("invalid opcode ({})", _0)]
    InvalidOpcode(#[error(not(source))] u8),

    /// Invalid control frame length
    #[display("invalid control frame length ({})", _0)]
    InvalidLength(#[error(not(source))] usize),

    /// Bad opcode.
    #[display("bad opcode")]
    BadOpCode,

    /// A payload reached size limit.
    #[display("payload reached size limit")]
    Overflow,

    /// Continuation has not started.
    #[display("continuation has not started")]
    ContinuationNotStarted,

    /// Received new continuation but it is already started.
    #[display("received new continuation but it has already started")]
    ContinuationStarted,

    /// Unknown continuation fragment.
    #[display("unknown continuation fragment: {}", _0)]
    ContinuationFragment(#[error(not(source))] OpCode),

    /// I/O error.
    #[display("I/O error: {}", _0)]
    Io(std::io::Error),
}

/// Operation codes defined in [RFC 6455 §11.8].
///
/// [RFC 6455]: https://datatracker.ietf.org/doc/html/rfc6455#section-11.8
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum OpCode {
    /// Indicates a continuation frame of a fragmented message.
    Continue,

    /// Indicates a text data frame.
    Text,

    /// Indicates a binary data frame.
    Binary,

    /// Indicates a close control frame.
    Close,

    /// Indicates a ping control frame.
    Ping,

    /// Indicates a pong control frame.
    Pong,

    /// Indicates an invalid opcode was received.
    Bad,
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OpCode::*;

        match self {
            Continue => write!(f, "CONTINUE"),
            Text => write!(f, "TEXT"),
            Binary => write!(f, "BINARY"),
            Close => write!(f, "CLOSE"),
            Ping => write!(f, "PING"),
            Pong => write!(f, "PONG"),
            Bad => write!(f, "BAD"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> u8 {
        use self::OpCode::*;

        match op {
            Continue => 0,
            Text => 1,
            Binary => 2,
            Close => 8,
            Ping => 9,
            Pong => 10,
            Bad => {
                tracing::error!("Attempted to convert invalid opcode to u8. This is a bug.");
                8 // if this somehow happens, a close frame will help us tear down quickly
            }
        }
    }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> OpCode {
        use self::OpCode::*;

        match byte {
            0 => Continue,
            1 => Text,
            2 => Binary,
            8 => Close,
            9 => Ping,
            10 => Pong,
            _ => Bad,
        }
    }
}
