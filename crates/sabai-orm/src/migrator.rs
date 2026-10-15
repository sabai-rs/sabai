use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, Result};

use crate::database::unreachable;
use crate::{Database, Migration, Migrations, Schema};

const TABLE: &str = "migrations";

impl Database {
    /// Runs every migration that has not run yet, in order, as one new batch, like
    /// `php artisan migrate`; returns their names.
    pub fn migrate<'a>(&'a self, migrations: &'a Migrations) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(self.run_pending(migrations))
    }

    async fn run_pending(&self, migrations: &Migrations) -> Result<Vec<String>> {
        self.create_migrations_table().await?;
        let ran: Vec<String> = sqlx::query_scalar("select migration from migrations")
            .fetch_all(self.pool())
            .await?;
        let batch = self.last_batch().await? + 1;
        let mut done = Vec::new();
        for (name, migration) in migrations.pending(&ran) {
            self.run_up(migration.as_ref())
                .await
                .map_err(|error| error.context(format!("migration `{name}` failed")))?;
            sqlx::query("insert into migrations (migration, batch) values (?, ?)")
                .bind(name)
                .bind(batch)
                .execute(self.pool())
                .await?;
            done.push(name.clone());
        }
        Ok(done)
    }

    async fn run_up(&self, migration: &dyn Migration) -> Result<()> {
        let mut schema = Schema::default();
        migration.up(&mut schema);
        for sql in schema.to_sql(self.driver()) {
            sqlx::query(&sql).execute(self.pool()).await?;
        }
        Ok(())
    }

    async fn create_migrations_table(&self) -> Result<()> {
        let exists: i64 = sqlx::query_scalar(self.driver().has_table())
            .bind(TABLE)
            .fetch_one(self.pool())
            .await
            .map_err(unreachable)?;
        if exists > 0 {
            return Ok(());
        }
        let mut schema = Schema::default();
        schema.create(TABLE, |table| {
            table.increments("id");
            table.string("migration");
            table.integer("batch");
        });
        for sql in schema.to_sql(self.driver()) {
            sqlx::query(&sql).execute(self.pool()).await?;
        }
        Ok(())
    }

    async fn last_batch(&self) -> Result<i64> {
        sqlx::query_scalar(self.driver().last_batch())
            .fetch_one(self.pool())
            .await
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CreatePostsTable;

    impl Migration for CreatePostsTable {
        fn up(&self, schema: &mut Schema) {
            schema.create("posts", |table| {
                table.id();
                table.string("title");
                table.timestamps();
            });
        }

        fn down(&self, schema: &mut Schema) {
            schema.drop_if_exists("posts");
        }
    }

    struct CreateTagsTable;

    impl Migration for CreateTagsTable {
        fn up(&self, schema: &mut Schema) {
            schema.create("tags", |table| {
                table.id();
            });
        }

        fn down(&self, schema: &mut Schema) {
            schema.drop_if_exists("tags");
        }
    }

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    async fn batch_of(database: &Database, name: &str) -> i64 {
        sqlx::query_scalar("select batch from migrations where migration = ?")
            .bind(name)
            .fetch_one(database.pool())
            .await
            .unwrap()
    }

    #[test]
    fn migrate_runs_each_migration_once_and_each_run_is_a_new_batch() {
        let database = Database::in_memory();
        let first = Migrations::new().add("2026_10_15_000000_create_posts_table", CreatePostsTable);
        let both = Migrations::new()
            .add("2026_10_15_000000_create_posts_table", CreatePostsTable)
            .add("2026_10_16_000000_create_tags_table", CreateTagsTable);

        block_on(async {
            let ran = database.migrate(&first).await.unwrap();
            let again = database.migrate(&first).await.unwrap();
            let later = database.migrate(&both).await.unwrap();

            assert_eq!(ran, ["2026_10_15_000000_create_posts_table"]);
            assert!(again.is_empty());
            assert_eq!(later, ["2026_10_16_000000_create_tags_table"]);
            assert_eq!(
                batch_of(&database, "2026_10_15_000000_create_posts_table").await,
                1
            );
            assert_eq!(
                batch_of(&database, "2026_10_16_000000_create_tags_table").await,
                2
            );
        });
    }

    #[test]
    fn a_failing_migration_is_named_and_not_recorded() {
        let database = Database::in_memory();
        let twice = Migrations::new()
            .add("2026_10_15_000000_create_posts_table", CreatePostsTable)
            .add(
                "2026_10_15_000001_create_posts_table_again",
                CreatePostsTable,
            );

        block_on(async {
            let error = database.migrate(&twice).await.err().unwrap().to_string();
            let recorded: Vec<String> = sqlx::query_scalar("select migration from migrations")
                .fetch_all(database.pool())
                .await
                .unwrap();

            assert!(
                error.starts_with("migration `2026_10_15_000001_create_posts_table_again` failed"),
                "{error}"
            );
            assert_eq!(recorded, ["2026_10_15_000000_create_posts_table"]);
        });
    }
}
