mod m0001_01_01_000000_create_users_table;
mod m2026_10_15_000000_create_posts_table;
mod m2026_10_17_000000_create_post_views_table;

use sabai::orm::Migrations;

use m0001_01_01_000000_create_users_table::CreateUsersTable;
use m2026_10_15_000000_create_posts_table::CreatePostsTable;
use m2026_10_17_000000_create_post_views_table::CreatePostViewsTable;

/// Every migration, in the order it runs.
pub fn all() -> Migrations {
    Migrations::new()
        .add("0001_01_01_000000_create_users_table", CreateUsersTable)
        .add("2026_10_15_000000_create_posts_table", CreatePostsTable)
        .add(
            "2026_10_17_000000_create_post_views_table",
            CreatePostViewsTable,
        )
}
