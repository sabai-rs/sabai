use crate::app::models::Post;

/// Stores posts. Not async yet: an `async fn` is not allowed in a `dyn` trait, and the ORM (M2)
/// will decide how repositories wait on the database.
pub trait PostRepository: Send + Sync {
    fn create(&self, title: String, body: String) -> Post;
    fn find(&self, id: u64) -> Option<Post>;
}
