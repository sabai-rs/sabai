mod grammar;

pub use grammar::Driver;

/// The table changes a migration makes, like Laravel's `Schema` facade.
#[derive(Default)]
pub struct Schema {
    changes: Vec<Change>,
}

enum Change {
    Create(Blueprint),
    DropIfExists(String),
}

/// The columns of a table being created, like Laravel's `Blueprint $table`.
pub struct Blueprint {
    table: String,
    columns: Vec<Column>,
    indexes: Vec<Index>,
}

/// One column; chain modifiers such as [`nullable`](Column::nullable).
pub struct Column {
    name: String,
    kind: Kind,
    nullable: bool,
    unique: bool,
    index: bool,
}

struct Index {
    kind: IndexKind,
    columns: Vec<String>,
}

#[derive(Clone, Copy)]
pub(crate) enum IndexKind {
    Unique,
    Index,
}

#[derive(Clone, Copy)]
enum Kind {
    Id,
    Increments,
    Integer,
    UnsignedBigInteger,
    String(u32),
    Text,
    Timestamp,
}

impl Schema {
    /// Creates `table` with the columns `columns` adds: `schema.create("posts", |table| { table.id(); })`.
    pub fn create<F: FnOnce(&mut Blueprint)>(&mut self, table: &str, columns: F) {
        let mut blueprint = Blueprint::new(table);
        columns(&mut blueprint);
        self.changes.push(Change::Create(blueprint));
    }

    /// Drops `table` if it exists.
    pub fn drop_if_exists(&mut self, table: &str) {
        self.changes.push(Change::DropIfExists(table.to_owned()));
    }

    /// The SQL statements for `driver`, in order, as `migrate --pretend` prints them.
    pub fn to_sql(&self, driver: Driver) -> Vec<String> {
        self.changes
            .iter()
            .flat_map(|change| match change {
                Change::Create(blueprint) => blueprint.to_sql(driver),
                Change::DropIfExists(table) => vec![driver.drop_table_if_exists(table)],
            })
            .collect()
    }
}

impl Blueprint {
    fn new(table: &str) -> Self {
        Self {
            table: table.to_owned(),
            columns: Vec::new(),
            indexes: Vec::new(),
        }
    }

    /// An auto-incrementing `bigint unsigned` primary key named `id`.
    pub fn id(&mut self) -> &mut Column {
        self.add("id", Kind::Id)
    }

    /// An auto-incrementing `int unsigned` primary key.
    pub fn increments(&mut self, name: &str) -> &mut Column {
        self.add(name, Kind::Increments)
    }

    /// An `int` column.
    pub fn integer(&mut self, name: &str) -> &mut Column {
        self.add(name, Kind::Integer)
    }

    /// A `bigint unsigned` column, such as an id from another table.
    pub fn unsigned_big_integer(&mut self, name: &str) -> &mut Column {
        self.add(name, Kind::UnsignedBigInteger)
    }

    /// A `varchar(255)` column.
    pub fn string(&mut self, name: &str) -> &mut Column {
        self.add(name, Kind::String(255))
    }

    /// A `text` column.
    pub fn text(&mut self, name: &str) -> &mut Column {
        self.add(name, Kind::Text)
    }

    /// A timestamp column.
    pub fn timestamp(&mut self, name: &str) -> &mut Column {
        self.add(name, Kind::Timestamp)
    }

    /// A nullable `varchar(100)` named `remember_token`, for "remember me" logins.
    pub fn remember_token(&mut self) -> &mut Column {
        self.add("remember_token", Kind::String(100)).nullable()
    }

    /// `{name}_type` and `{name}_id` for a model of any type, with an index on both, like
    /// Laravel's `morphs('commentable')`.
    pub fn morphs(&mut self, name: &str) {
        let (kind, id) = (format!("{name}_type"), format!("{name}_id"));
        self.string(&kind);
        self.unsigned_big_integer(&id);
        self.indexes.push(Index {
            kind: IndexKind::Index,
            columns: vec![kind, id],
        });
    }

    /// Nullable `created_at` and `updated_at` timestamps.
    pub fn timestamps(&mut self) {
        self.timestamp("created_at").nullable();
        self.timestamp("updated_at").nullable();
    }

    fn add(&mut self, name: &str, kind: Kind) -> &mut Column {
        let index = self.columns.len();
        self.columns.push(Column {
            name: name.to_owned(),
            kind,
            nullable: false,
            unique: false,
            index: false,
        });
        &mut self.columns[index]
    }

    // Like Laravel, indexes are their own statements after `create table`: first the ones added
    // by name, such as `morphs`, then each column's own, its unique index before its plain one.
    fn to_sql(&self, driver: Driver) -> Vec<String> {
        let column_indexes = self.columns.iter().flat_map(|column| {
            let unique = column.unique.then_some(IndexKind::Unique);
            let index = column.index.then_some(IndexKind::Index);
            [unique, index]
                .into_iter()
                .flatten()
                .map(|kind| (kind, std::slice::from_ref(&column.name)))
        });
        let indexes = self
            .indexes
            .iter()
            .map(|index| (index.kind, index.columns.as_slice()))
            .chain(column_indexes)
            .map(|(kind, columns)| driver.add_index(&self.table, kind, columns));
        [driver.create_table(&self.table, &self.columns)]
            .into_iter()
            .chain(indexes)
            .collect()
    }
}

impl Column {
    /// Allows `NULL`.
    pub fn nullable(&mut self) -> &mut Self {
        self.nullable = true;
        self
    }

    /// Adds a unique index named like Laravel's: `users_email_unique`.
    pub fn unique(&mut self) -> &mut Self {
        self.unique = true;
        self
    }

    /// Adds an index named like Laravel's: `comments_status_index`.
    pub fn index(&mut self) -> &mut Self {
        self.index = true;
        self
    }

    /// The most characters a string column holds, like Laravel's `string('status', 20)`;
    /// `string()` holds 255. Only string columns have a length.
    pub fn length(&mut self, length: u32) -> &mut Self {
        if let Kind::String(_) = self.kind {
            self.kind = Kind::String(length);
        }
        self
    }
}
