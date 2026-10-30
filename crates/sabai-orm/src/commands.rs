//! The database's console commands, such as `migrate`.

mod migrate;

pub use migrate::{MigrateCommand, migrate};
