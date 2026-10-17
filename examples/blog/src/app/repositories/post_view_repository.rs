use sabai::orm::Model;
use sabai::{App, FromApp, Result};

use crate::app::models::PostView;

/// Records reads of posts in the `report` database.
pub struct PostViewRepository;

impl FromApp for PostViewRepository {
    fn new(_app: &App) -> Result<Self> {
        Ok(Self)
    }
}

impl PostViewRepository {
    pub async fn record(&self, post_id: u64) -> Result<()> {
        PostView { id: 0, post_id }.save().await
    }
}
