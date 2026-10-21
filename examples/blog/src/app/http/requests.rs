//! The input of each action, like Laravel's `app/Http/Requests`.

mod import_posts_request;
mod store_post_request;

pub use import_posts_request::ImportPostsRequest;
pub use store_post_request::StorePostRequest;
