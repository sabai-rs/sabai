//! Query builder, models, relations, schema builder and migrator for Sabai.

// Crates built on the ORM, like sabai-auth, compile without a driver and leave the choice to the
// app; the `sabai` crate refuses an app without one. Without a driver the query code is unused.
#![cfg_attr(not(any(feature = "mysql", feature = "sqlite")), allow(unused))]

mod commands;
mod config;
mod database;
mod manager;
mod migration;
mod migrator;
mod model;
#[cfg(feature = "mysql")]
mod mysql_timestamp;
mod pool;
mod provider;
mod query;
mod row;
mod schema;
mod timestamps;
mod unique;
mod value;

pub use commands::{MigrateCommand, migrate};
pub use database::Database;
pub use manager::DatabaseManager;
pub use migration::{Migration, Migrations};
pub use model::Model;
pub use provider::DatabaseServiceProvider;
pub use query::Query;
pub use row::{FromColumn, Row};
pub use sabai_core::Result;
pub use sabai_macros::Model;
pub use schema::{Blueprint, Column, Driver, Schema};
pub use timestamps::{Timestamp, Timestamps};
pub use unique::is_unique_violation;
pub use value::{SqlValue, ToColumn};
