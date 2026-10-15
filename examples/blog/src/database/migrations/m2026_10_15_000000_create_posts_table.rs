use sabai::orm::{Migration, Schema};

pub struct CreatePostsTable;

impl Migration for CreatePostsTable {
    fn up(&self, schema: &mut Schema) {
        schema.create("posts", |table| {
            table.id();
            table.string("title");
            table.text("body");
            table.timestamps();
        });
    }

    fn down(&self, schema: &mut Schema) {
        schema.drop_if_exists("posts");
    }
}
