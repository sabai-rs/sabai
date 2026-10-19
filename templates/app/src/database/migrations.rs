mod m0001_01_01_000000_create_users_table;

use sabai::orm::Migrations;

use m0001_01_01_000000_create_users_table::CreateUsersTable;

/// Every migration, in the order it runs.
pub fn all() -> Migrations {
    Migrations::new().add("0001_01_01_000000_create_users_table", CreateUsersTable)
}
