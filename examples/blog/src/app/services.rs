//! Logic shared by controllers, jobs and commands, like a Laravel app's `app/Services`.

mod post_service;

pub use post_service::PostService;
