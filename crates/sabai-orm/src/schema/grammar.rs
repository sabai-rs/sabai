use super::{Column, Kind};

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

    pub(crate) fn has_table(self) -> &'static str {
        match self {
            Driver::MySql => {
                "select count(*) from information_schema.tables \
                 where table_schema = database() and table_name = ?"
            }
            Driver::Sqlite => {
                "select count(*) from sqlite_master where type = 'table' and name = ?"
            }
        }
    }

    pub(crate) fn last_batch(self) -> &'static str {
        match self {
            Driver::MySql => "select cast(coalesce(max(batch), 0) as signed) from migrations",
            Driver::Sqlite => "select coalesce(max(batch), 0) from migrations",
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
            (Driver::Sqlite, Kind::Id | Kind::Increments | Kind::Integer) => "integer".to_owned(),
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
            table.timestamps();
        });
        schema.drop_if_exists("posts");
        schema
    }

    #[test]
    fn mysql_gets_laravels_create_table() {
        assert_eq!(
            create_posts().to_sql(Driver::MySql),
            [
                "create table `posts` (`id` bigint unsigned not null auto_increment primary key, \
                 `title` varchar(255) not null, `body` text null, `created_at` timestamp null, \
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
                 \"title\" varchar not null, \"body\" text, \"created_at\" datetime, \
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
}
