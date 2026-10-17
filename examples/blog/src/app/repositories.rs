//! Where the blog's data is stored, like a Laravel app's `app/Repositories`.

mod post_repository;

pub use post_repository::PostRepository;
