//! The blog's controllers, like Laravel's `app/Http/Controllers`.

mod post_controller;
mod registered_user_controller;
mod stats_controller;

pub use post_controller::PostController;
pub use registered_user_controller::RegisteredUserController;
pub use stats_controller::StatsController;
