mod m2026_10_15_000000_create_posts_table;

use sabai::orm::Migrations;

use m2026_10_15_000000_create_posts_table::CreatePostsTable;

/// Every migration, in the order it runs.
pub fn all() -> Migrations {
    Migrations::new().add("2026_10_15_000000_create_posts_table", CreatePostsTable)
}
