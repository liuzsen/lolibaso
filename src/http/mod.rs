pub mod adapter;
pub mod api_macro;
pub mod error;
pub mod parser;
pub mod request;
pub mod response;

#[cfg(feature = "web-socket")]
pub mod web_socket;

pub use adapter::HttpAdapter;
pub use response::ApiResponse;
