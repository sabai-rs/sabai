//! The single integration-test binary. Each area of the framework is one `mod`.

// Everything in this binary is test code, including helpers outside `#[test]` fns,
// which `allow-unwrap-in-tests` in clippy.toml does not cover.
#![allow(clippy::unwrap_used)]

mod config;
mod error;
mod facade;
mod log;
mod request;
mod response;
