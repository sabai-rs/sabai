//! The single integration-test binary. Each area of the framework is one `mod`.

// `allow-unwrap-in-tests` does not cover helpers outside `#[test]` functions.
#![allow(clippy::unwrap_used)]

mod app;
mod config;
mod container;
mod contracts;
mod error;
mod events;
mod facade;
mod log;
mod request;
mod response;
mod router;
