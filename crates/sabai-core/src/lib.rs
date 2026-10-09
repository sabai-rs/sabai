//! Core types and contracts for Sabai: errors, requests, responses, config and the service container.

mod error;

pub use error::{Error, Result};
pub use http::StatusCode;
