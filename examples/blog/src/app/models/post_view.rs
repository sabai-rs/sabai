use sabai::orm::Model;

/// One read of a post, kept in the `report` database.
#[derive(Model)]
#[model(connection = "report")]
pub struct PostView {
    pub id: u64,
    pub post_id: u64,
}
