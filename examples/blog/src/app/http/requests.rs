//! The input of each action, like Laravel's `app/Http/Requests`.

mod import_posts_request;
mod login_request;
mod register_request;
mod store_post_request;

pub use import_posts_request::ImportPostsRequest;
pub use login_request::LoginRequest;
pub use register_request::RegisterRequest;
pub use store_post_request::StorePostRequest;
