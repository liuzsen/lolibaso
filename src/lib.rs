#![allow(async_fn_in_trait)]

pub mod configs;
pub mod entity;
pub mod flake_id;
pub mod http;
pub mod provider;
pub mod repository;
pub mod result;
pub mod use_case;

pub use lolibaso_macros::*;

pub use derive_more;
pub use flaken;
