use sabai::orm::Model;
use sabai::{Json, Result};
use serde::Serialize;

use crate::app::models::PostView;

pub struct StatsController;

#[derive(Serialize)]
pub struct Stats {
    post_views: usize,
}

impl StatsController {
    pub async fn index() -> Result<Json<Stats>> {
        let post_views = PostView::all().await?.len();
        Ok(Json(Stats { post_views }))
    }
}
