use super::{Column, IndexKind, Kind};

/// The SQL dialect of a connection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Driver {
    /// MySQL 8.
    MySql,
    /// SQLite 3.
    Sqlite,
}

impl Driver {
    pub(crate) fn create_table(self, table: &str, columns: &[Column]) -> String {
        let columns: Vec<_> = columns.iter().map(|column| self.column(column)).collect();
        let suffix = match self {
            Driver::MySql => " default character set utf8mb4 collate 'utf8mb4_unicode_ci'",
            Driver::Sqlite => "",
        };
        format!(
            "create table {} ({}){suffix}",
            self.quote(table),
            columns.join(", ")
        )
    }

    pub(crate) fn drop_table_if_exists(self, table: &str) -> String {
        format!("drop table if exists {}", self.quote(table))
    }

    pub(crate) fn add_index(self, table: &str, kind: IndexKind, columns: &[String]) -> String {
        let (mysql_kind, sqlite_kind, suffix) = match kind {
            IndexKind::Unique => ("unique", "unique index", "unique"),
            IndexKind::Index => ("index", "index", "index"),
        };
        let index = self.quote(&index_name(table, columns, suffix));
        let quoted: Vec<_> = columns.iter().map(|column| self.quote(column)).collect();
        let (table, columns) = (self.quote(table), quoted.join(", "));
        match self {
            Driver::MySql => format!("alter table {table} add {mysql_kind} {index}({columns})"),
            Driver::Sqlite => format!("create {sqlite_kind} {index} on {table} ({columns})"),
        }
    }

    pub(crate) fn has_table(self) -> &'static str {
        match self {
            Driver::MySql => {
                "select count(*) as tables from information_schema.tables \
                 where table_schema = database() and table_name = ?"
            }
            Driver::Sqlite => {
                "select count(*) as tables from sqlite_master where type = 'table' and name = ?"
            }
        }
    }

    pub(crate) fn last_batch(self) -> &'static str {
        match self {
            Driver::MySql => {
                "select cast(coalesce(max(batch), 0) as signed) as batch from migrations"
            }
            Driver::Sqlite => "select coalesce(max(batch), 0) as batch from migrations",
        }
    }

    fn column(self, column: &Column) -> String {
        let name = self.quote(&column.name);
        let kind = self.type_of(column.kind);
        if matches!(column.kind, Kind::Id | Kind::Increments) {
            return match self {
                Driver::MySql => format!("{name} {kind} not null auto_increment primary key"),
                Driver::Sqlite => format!("{name} {kind} primary key autoincrement not null"),
            };
        }
        match (self, column.nullable) {
            (_, false) => format!("{name} {kind} not null"),
            (Driver::MySql, true) => format!("{name} {kind} null"),
            (Driver::Sqlite, true) => format!("{name} {kind}"),
        }
    }

    fn type_of(self, kind: Kind) -> String {
        match (self, kind) {
            (Driver::MySql, Kind::Id) => "bigint unsigned".to_owned(),
            (Driver::MySql, Kind::Increments) => "int unsigned".to_owned(),
            (Driver::MySql, Kind::Integer) => "int".to_owned(),
            (Driver::MySql, Kind::UnsignedBigInteger) => "bigint unsigned".to_owned(),
            (
                Driver::Sqlite,
                Kind::Id | Kind::Increments | Kind::Integer | Kind::UnsignedBigInteger,
            ) => "integer".to_owned(),
            (Driver::MySql, Kind::String(length)) => format!("varchar({length})"),
            (Driver::Sqlite, Kind::String(_)) => "varchar".to_owned(),
            (_, Kind::Text) => "text".to_owned(),
            (Driver::MySql, Kind::Timestamp) => "timestamp".to_owned(),
            (Driver::Sqlite, Kind::Timestamp) => "datetime".to_owned(),
        }
    }

    pub(crate) fn quote(self, name: &str) -> String {
        match self {
            Driver::MySql => format!("`{}`", name.replace('`', "``")),
            Driver::Sqlite => format!("\"{}\"", name.replace('"', "\"\"")),
        }
    }
}

/// An index name: `users_email_unique`, `posts_commentable_type_commentable_id_index`.
fn index_name(table: &str, columns: &[String], kind: &str) -> String {
    format!("{table}_{}_{kind}", columns.join("_"))
        .to_lowercase()
        .replace(['-', '.'], "_")
}

#[cfg(test)]
mod tests {
    use super::Driver;
    use crate::Schema;

    fn create_posts() -> Schema {
        let mut schema = Schema::default();
        schema.create("posts", |table| {
            table.id();
            table.string("title");
            table.text("body").nullable();
            table.unsigned_big_integer("author_id");
            table.timestamps();
        });
        schema.drop_if_exists("posts");
        schema
    }

    #[test]
    fn mysql_gets_unsigned_ids_and_lengths_in_create_table() {
        assert_eq!(
            create_posts().to_sql(Driver::MySql),
            [
                "create table `posts` (`id` bigint unsigned not null auto_increment primary key, \
                 `title` varchar(255) not null, `body` text null, `author_id` bigint unsigned not null, \
                 `created_at` timestamp null, \
                 `updated_at` timestamp null) default character set utf8mb4 collate 'utf8mb4_unicode_ci'",
                "drop table if exists `posts`",
            ]
        );
    }

    #[test]
    fn sqlite_gets_its_own_dialect() {
        assert_eq!(
            create_posts().to_sql(Driver::Sqlite),
            [
                "create table \"posts\" (\"id\" integer primary key autoincrement not null, \
                 \"title\" varchar not null, \"body\" text, \"author_id\" integer not null, \
                 \"created_at\" datetime, \
                 \"updated_at\" datetime)",
                "drop table if exists \"posts\"",
            ]
        );
    }

    #[test]
    fn a_quote_in_a_name_cannot_end_the_identifier() {
        let mut schema = Schema::default();
        schema.drop_if_exists("posts` ; drop table users; --");

        assert_eq!(
            schema.to_sql(Driver::MySql),
            ["drop table if exists `posts`` ; drop table users; --`"]
        );
    }

    fn create_users() -> Schema {
        let mut schema = Schema::default();
        schema.create("users", |table| {
            table.id();
            table.string("email").unique();
            table.remember_token();
        });
        schema
    }

    #[test]
    fn mysql_adds_a_unique_index_after_the_table() {
        assert_eq!(
            create_users().to_sql(Driver::MySql),
            [
                "create table `users` (`id` bigint unsigned not null auto_increment primary key, \
                 `email` varchar(255) not null, `remember_token` varchar(100) null) \
                 default character set utf8mb4 collate 'utf8mb4_unicode_ci'",
                "alter table `users` add unique `users_email_unique`(`email`)",
            ]
        );
    }

    #[test]
    fn sqlite_adds_a_unique_index_after_the_table() {
        assert_eq!(
            create_users().to_sql(Driver::Sqlite),
            [
                "create table \"users\" (\"id\" integer primary key autoincrement not null, \
                 \"email\" varchar not null, \"remember_token\" varchar)",
                "create unique index \"users_email_unique\" on \"users\" (\"email\")",
            ]
        );
    }

    fn create_comments() -> Schema {
        let mut schema = Schema::default();
        schema.create("comments", |table| {
            table.id();
            table.morphs("commentable");
            table.string("author_name").length(100);
            table.string("slug").length(64).unique();
            table.text("body");
            table.string("status").length(20).index();
            table.timestamps();
        });
        schema
    }

    #[test]
    fn mysql_adds_the_morphs_index_then_each_column_index_in_order() {
        assert_eq!(
            create_comments().to_sql(Driver::MySql),
            [
                "create table `comments` (`id` bigint unsigned not null auto_increment primary key, \
                 `commentable_type` varchar(255) not null, `commentable_id` bigint unsigned not null, \
                 `author_name` varchar(100) not null, `slug` varchar(64) not null, \
                 `body` text not null, `status` varchar(20) not null, \
                 `created_at` timestamp null, `updated_at` timestamp null) \
                 default character set utf8mb4 collate 'utf8mb4_unicode_ci'",
                "alter table `comments` add index \
                 `comments_commentable_type_commentable_id_index`(`commentable_type`, `commentable_id`)",
                "alter table `comments` add unique `comments_slug_unique`(`slug`)",
                "alter table `comments` add index `comments_status_index`(`status`)",
            ]
        );
    }

    #[test]
    fn sqlite_adds_the_same_indexes_as_create_index() {
        assert_eq!(
            create_comments().to_sql(Driver::Sqlite),
            [
                "create table \"comments\" (\"id\" integer primary key autoincrement not null, \
                 \"commentable_type\" varchar not null, \"commentable_id\" integer not null, \
                 \"author_name\" varchar not null, \"slug\" varchar not null, \
                 \"body\" text not null, \"status\" varchar not null, \
                 \"created_at\" datetime, \"updated_at\" datetime)",
                "create index \"comments_commentable_type_commentable_id_index\" \
                 on \"comments\" (\"commentable_type\", \"commentable_id\")",
                "create unique index \"comments_slug_unique\" on \"comments\" (\"slug\")",
                "create index \"comments_status_index\" on \"comments\" (\"status\")",
            ]
        );
    }

    #[test]
    fn a_column_with_both_gets_its_unique_index_before_its_plain_one() {
        let mut schema = Schema::default();
        schema.create("users", |table| {
            table.string("email").index().unique();
        });

        let sql = schema.to_sql(Driver::Sqlite);

        assert_eq!(
            sql[1..],
            [
                "create unique index \"users_email_unique\" on \"users\" (\"email\")",
                "create index \"users_email_index\" on \"users\" (\"email\")",
            ]
        );
    }
}
