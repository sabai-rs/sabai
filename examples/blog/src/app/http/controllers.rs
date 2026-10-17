//! The blog's controllers, like Laravel's `app/Http/Controllers`.

mod post_controller;
mod stats_controller;

pub use post_controller::PostController;
pub use stats_controller::StatsController;
