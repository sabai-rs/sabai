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
}

/// One column; chain modifiers such as [`nullable`](Column::nullable).
pub struct Column {
    name: String,
    kind: Kind,
    nullable: bool,
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
            .map(|change| match change {
                Change::Create(blueprint) => {
                    driver.create_table(&blueprint.table, &blueprint.columns)
                }
                Change::DropIfExists(table) => driver.drop_table_if_exists(table),
            })
            .collect()
    }
}

impl Blueprint {
    fn new(table: &str) -> Self {
        Self {
            table: table.to_owned(),
            columns: Vec::new(),
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
        });
        &mut self.columns[index]
    }
}

impl Column {
    /// Allows `NULL`.
    pub fn nullable(&mut self) -> &mut Self {
        self.nullable = true;
        self
    }
}
