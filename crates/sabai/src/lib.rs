//! Sabai: a batteries-included web framework for Rust, built for developer experience and fast
//! compiles. Rust, but sabai.

pub use sabai_core::*;
pub use sabai_macros::ConfigSection;

pub use sabai_http as http;

pub mod console;

#[cfg(feature = "orm")]
pub use sabai_orm as orm;

// Every query goes through a database driver, so the ORM is useless without one.
#[cfg(all(feature = "orm", not(any(feature = "mysql", feature = "sqlite"))))]
compile_error!(
    "the `orm` feature needs a database driver: turn on `mysql` or `sqlite`, \
     such as sabai = { ..., features = [\"orm\", \"sqlite\"] }"
);

#[cfg(feature = "auth")]
pub use sabai_auth as auth;

#[cfg(feature = "queue")]
pub use sabai_queue as queue;
