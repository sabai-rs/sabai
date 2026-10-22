//! The blog's data, like Laravel's `app/Models`.

mod post;
mod post_view;
mod user;

pub use post::Post;
pub use post_view::PostView;
pub use user::User;
