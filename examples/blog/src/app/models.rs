//! The blog's data, like Laravel's `app/Models`.

mod post;
mod post_view;

pub use post::Post;
pub use post_view::PostView;
