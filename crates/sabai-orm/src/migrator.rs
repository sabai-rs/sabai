use sabai_core::Result;
use sabai_core::contracts::BoxFuture;

use crate::database::unreachable;
use sabai_core::App;

use crate::{Database, DatabaseManager, Migration, Migrations, Schema, SqlValue};

const TABLE: &str = "migrations";

impl Database {
    /// Runs every migration that has not run yet, in order, as one new batch, like
    /// `php artisan migrate`; returns their names.
    pub fn migrate<'a>(&'a self, migrations: &'a Migrations) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(self.run_pending(migrations))
    }

    async fn run_pending(&self, migrations: &Migrations) -> Result<Vec<String>> {
        migrations.check_order()?;
        self.create_migrations_table().await?;
        let ran = self
            .pool()?
            .fetch_all("select migration from migrations", Vec::new())
            .await?
            .iter()
            .map(|row| row.get::<String>("migration"))
            .collect::<Result<Vec<_>>>()?;
        let batch = self.last_batch().await? + 1;
        let mut done = Vec::new();
        for (name, migration) in migrations.pending(&ran) {
            self.run_up(migration.as_ref())
                .await
                .map_err(|error| error.context(format!("migration `{name}` failed")))?;
            let values = vec![SqlValue::Text(name.clone()), SqlValue::Integer(batch)];
            self.pool()?
                .execute(
                    "insert into migrations (migration, batch) values (?, ?)",
                    values,
                )
                .await?;
            done.push(name.clone());
        }
        Ok(done)
    }

    async fn run_up(&self, migration: &dyn Migration) -> Result<()> {
        let mut schema = Schema::default();
        migration.up(&mut schema);
        let other;
        let target = match migration.connection() {
            Some(name) => {
                other = App::current()?.get::<DatabaseManager>()?.connection(name)?;
                other.as_ref()
            }
            None => self,
        };
        for sql in schema.to_sql(target.driver()) {
            target.pool()?.execute(&sql, Vec::new()).await?;
        }
        Ok(())
    }

    async fn create_migrations_table(&self) -> Result<()> {
        let found = self
            .pool()?
            .fetch_all(
                self.driver().has_table(),
                vec![SqlValue::Text(TABLE.to_owned())],
            )
            .await
            .map_err(unreachable)?;
        if found
            .first()
            .map(|row| row.get::<i64>("tables"))
            .transpose()?
            > Some(0)
        {
            return Ok(());
        }
        let mut schema = Schema::default();
        schema.create(TABLE, |table| {
            table.increments("id");
            table.string("migration");
            table.integer("batch");
        });
        for sql in schema.to_sql(self.driver()) {
            self.pool()?.execute(&sql, Vec::new()).await?;
        }
        Ok(())
    }

    async fn last_batch(&self) -> Result<i64> {
        let rows = self
            .pool()?
            .fetch_all(self.driver().last_batch(), Vec::new())
            .await?;
        rows.first().map_or(Ok(0), |row| row.get::<i64>("batch"))
    }
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
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
        let rows = database
            .pool()
            .unwrap()
            .fetch_all(
                "select batch from migrations where migration = ?",
                vec![SqlValue::Text(name.to_owned())],
            )
            .await
            .expect("the batch can be read");
        rows[0].get("batch").expect("the batch is a number")
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
            let rows = database
                .pool()
                .unwrap()
                .fetch_all("select migration from migrations", Vec::new())
                .await
                .unwrap();
            let recorded: Vec<String> = rows
                .iter()
                .map(|row| row.get("migration").unwrap())
                .collect();

            assert!(
                error.starts_with("migration `2026_10_15_000001_create_posts_table_again` failed"),
                "{error}"
            );
            assert_eq!(recorded, ["2026_10_15_000000_create_posts_table"]);
        });
    }

    #[test]
    fn a_list_out_of_name_order_or_with_a_repeat_is_refused_before_anything_runs() {
        let database = Database::in_memory();
        let unsorted = Migrations::new()
            .add("2026_10_16_000000_create_tags_table", CreateTagsTable)
            .add("2026_10_15_000000_create_posts_table", CreatePostsTable);
        let twice = Migrations::new()
            .add("2026_10_15_000000_create_posts_table", CreatePostsTable)
            .add("2026_10_15_000000_create_posts_table", CreatePostsTable);

        block_on(async {
            let unsorted = database.migrate(&unsorted).await.err().unwrap().to_string();
            let twice = database.migrate(&twice).await.err().unwrap().to_string();

            assert_eq!(
                unsorted,
                "migration `2026_10_15_000000_create_posts_table` is added after \
                 `2026_10_16_000000_create_tags_table`; move it up so names run in order"
            );
            assert_eq!(
                twice,
                "migration `2026_10_15_000000_create_posts_table` is added twice"
            );
        });
    }
}
