//! Where the blog's data is stored, like a Laravel app's `app/Repositories`.

mod post_repository;
mod post_view_repository;

pub use post_repository::PostRepository;
pub use post_view_repository::PostViewRepository;
