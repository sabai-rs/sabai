use crate::Schema;

/// One schema change, like a Laravel migration class: `up` makes it, `down` undoes it.
pub trait Migration: Send + Sync {
    /// Describes the change.
    fn up(&self, schema: &mut Schema);

    /// Describes how to undo it.
    fn down(&self, schema: &mut Schema);
}

/// The app's migrations, run in the order they are added.
#[derive(Default)]
pub struct Migrations {
    list: Vec<(String, Box<dyn Migration>)>,
}

impl Migrations {
    /// No migrations yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `migration` under its name: `.add("2026_10_15_000000_create_posts_table", CreatePostsTable)`.
    pub fn add(self, name: &str, migration: impl Migration + 'static) -> Self {
        self.push(name, Box::new(migration))
    }

    fn push(mut self, name: &str, migration: Box<dyn Migration>) -> Self {
        self.list.push((name.to_owned(), migration));
        self
    }

    pub(crate) fn pending<'a>(
        &'a self,
        ran: &'a [String],
    ) -> impl Iterator<Item = &'a (String, Box<dyn Migration>)> {
        self.list.iter().filter(|(name, _)| !ran.contains(name))
    }
}
