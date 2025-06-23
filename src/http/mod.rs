pub mod adapter;
pub mod api_macro;
pub mod error;
pub mod json;
pub mod query_decoder;
pub mod request;
pub mod response;

#[cfg(feature = "web-socket")]
pub mod web_socket;

pub use adapter::Adapter;
pub use response::ApiResponse;
