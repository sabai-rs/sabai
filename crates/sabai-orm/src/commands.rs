//! The database's console commands, like Laravel's `Illuminate\Database\Console`.

mod migrate;

pub use migrate::{MigrateCommand, migrate};
