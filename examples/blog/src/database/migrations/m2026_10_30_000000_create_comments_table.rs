use sabai::orm::{Migration, Schema};

pub struct CreateCommentsTable;

impl Migration for CreateCommentsTable {
    fn up(&self, schema: &mut Schema) {
        schema.create("comments", |table| {
            table.id();
            table.morphs("commentable");
            table.string("author_name").length(100);
            table.text("body");
            table.string("status").length(20).index();
            table.timestamps();
        });
    }

    fn down(&self, schema: &mut Schema) {
        schema.drop_if_exists("comments");
    }
}
