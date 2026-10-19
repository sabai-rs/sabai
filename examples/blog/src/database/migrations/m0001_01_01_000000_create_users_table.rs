use sabai::orm::{Migration, Schema};

pub struct CreateUsersTable;

impl Migration for CreateUsersTable {
    fn up(&self, schema: &mut Schema) {
        schema.create("users", |table| {
            table.id();
            table.string("name");
            table.string("email").unique();
            table.timestamp("email_verified_at").nullable();
            table.string("password");
            table.remember_token();
            table.timestamps();
        });
    }

    fn down(&self, schema: &mut Schema) {
        schema.drop_if_exists("users");
    }
}
