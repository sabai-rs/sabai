//! Where the blog's data is stored, like a Laravel app's `app/Repositories`.

mod in_memory_post_repository;
mod post_repository;

pub use in_memory_post_repository::InMemoryPostRepository;
pub use post_repository::PostRepository;
