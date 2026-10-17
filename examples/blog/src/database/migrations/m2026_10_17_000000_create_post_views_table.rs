use sabai::orm::{Migration, Schema};

pub struct CreatePostViewsTable;

impl Migration for CreatePostViewsTable {
    fn up(&self, schema: &mut Schema) {
        schema.create("post_views", |table| {
            table.id();
            table.unsigned_big_integer("post_id");
        });
    }

    fn down(&self, schema: &mut Schema) {
        schema.drop_if_exists("post_views");
    }

    fn connection(&self) -> Option<&'static str> {
        Some("report")
    }
}
