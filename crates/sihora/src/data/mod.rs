//! Reactive data layer with persistence.
//!
//! Desktop uses SQLite via rusqlite. Web (wasm32) uses an in-memory store.
//!
//! # Example
//! ```ignore
//! use sihora::data::{Model, open, register, insert, query_all};
//! use sihora::core::reactive::get;
//!
//! #[derive(Clone, Model)]
//! struct Todo {
//!     id: i64,
//!     title: String,
//!     done: bool,
//! }
//!
//! open(":memory:").unwrap();
//! register::<Todo>().unwrap();
//! insert(Todo { id: 0, title: "Test".into(), done: false }).unwrap();
//! let todos = query_all::<Todo>();
//! assert_eq!(get(todos).len(), 1);
//! ```

use std::marker::PhantomData;

pub use sihora_data_derive::Model;

// ── Value types ──────────────────────────────────────────────────────────

/// A database-compatible value.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Bool(bool),
}

/// SQL column type.
#[derive(Debug, Clone, Copy)]
pub enum ColumnType {
    Integer,
    Real,
    Text,
    Bool,
}

/// Column definition for a model field.
#[derive(Debug, Clone, Copy)]
pub struct ColumnDef {
    pub name: &'static str,
    pub ty: ColumnType,
    pub primary_key: bool,
}

// ── Model trait ──────────────────────────────────────────────────────────

/// A persistable data model.
///
/// Derive with `#[derive(Model)]` on a struct whose first field is `id: i64`.
pub trait Model: Clone + PartialEq + 'static {
    /// Table name (snake_case, pluralized).
    fn table_name() -> &'static str;
    /// Column definitions in field order.
    fn columns() -> &'static [ColumnDef];
    /// Primary key value.
    fn id(&self) -> i64;
    /// Set the primary key (called after insert).
    fn set_id(&mut self, id: i64);
    /// Non-id field values for INSERT/UPDATE params.
    fn to_params(&self) -> Vec<Value>;
    /// Reconstruct from a full row (all columns including id).
    fn from_values(values: &[Value]) -> Self;
}

// ── Handle ───────────────────────────────────────────────────────────────

/// A typed, `Copy` reference to a persisted model row.
///
/// Use with [`find`], [`save`], and [`delete`].
#[derive(Debug)]
pub struct Handle<T: Model> {
    id: i64,
    _marker: PhantomData<T>,
}

impl<T: Model> Handle<T> {
    /// Create a handle from a known primary key.
    pub fn from_id(id: i64) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// The row's primary key.
    pub fn id(&self) -> i64 {
        self.id
    }
}

impl<T: Model> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Model> Copy for Handle<T> {}

// ── Error ────────────────────────────────────────────────────────────────

/// Data layer error.
#[derive(Debug)]
pub enum DataError {
    #[cfg(not(target_arch = "wasm32"))]
    /// SQLite error.
    Sql(rusqlite::Error),
    /// Storage error (web backend).
    Storage(String),
    /// Store not opened.
    NotOpen,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<rusqlite::Error> for DataError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Sql(e) => write!(f, "sqlite: {e}"),
            Self::Storage(e) => write!(f, "storage: {e}"),
            Self::NotOpen => write!(f, "store not opened"),
        }
    }
}

impl std::error::Error for DataError {}

// ── Migration ────────────────────────────────────────────────────────────

/// A schema migration step.
pub struct Migration {
    /// Monotonically increasing version number.
    pub version: u32,
    /// SQL to execute for this migration.
    pub sql: &'static str,
}

// ── Backend selection ────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
mod memory;
#[cfg(not(target_arch = "wasm32"))]
mod sqlite;

#[cfg(target_arch = "wasm32")]
pub use memory::{close, delete, find, insert, migrate, open, query_all, register, reset, save};
#[cfg(not(target_arch = "wasm32"))]
pub use sqlite::{close, delete, find, insert, migrate, open, query_all, register, reset, save};

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::reactive::get;

    #[derive(Clone, Debug, PartialEq)]
    struct Task {
        id: i64,
        title: String,
        done: bool,
    }

    impl Model for Task {
        fn table_name() -> &'static str {
            "tasks"
        }
        fn columns() -> &'static [ColumnDef] {
            static COLS: [ColumnDef; 3] = [
                ColumnDef {
                    name: "id",
                    ty: ColumnType::Integer,
                    primary_key: true,
                },
                ColumnDef {
                    name: "title",
                    ty: ColumnType::Text,
                    primary_key: false,
                },
                ColumnDef {
                    name: "done",
                    ty: ColumnType::Bool,
                    primary_key: false,
                },
            ];
            &COLS
        }
        fn id(&self) -> i64 {
            self.id
        }
        fn set_id(&mut self, id: i64) {
            self.id = id;
        }
        fn to_params(&self) -> Vec<Value> {
            vec![Value::Text(self.title.clone()), Value::Bool(self.done)]
        }
        fn from_values(values: &[Value]) -> Self {
            Self {
                id: match &values[0] {
                    Value::Integer(v) => *v,
                    _ => 0,
                },
                title: match &values[1] {
                    Value::Text(v) => v.clone(),
                    _ => String::new(),
                },
                done: match &values[2] {
                    Value::Bool(v) => *v,
                    _ => false,
                },
            }
        }
    }

    #[test]
    fn insert_and_query() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        insert(Task {
            id: 0,
            title: "Buy milk".into(),
            done: false,
        })
        .unwrap();

        insert(Task {
            id: 0,
            title: "Walk dog".into(),
            done: true,
        })
        .unwrap();

        let all = query_all::<Task>();
        let items = get(all);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Buy milk");
        assert!(!items[0].done);
        assert_eq!(items[1].title, "Walk dog");
        assert!(items[1].done);
    }

    #[test]
    fn find_and_save() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        let h = insert(Task {
            id: 0,
            title: "Original".into(),
            done: false,
        })
        .unwrap();

        let mut task = find(h).unwrap().unwrap();
        assert_eq!(task.title, "Original");

        task.title = "Updated".into();
        task.done = true;
        save(h, &task).unwrap();

        let reloaded = find(h).unwrap().unwrap();
        assert_eq!(reloaded.title, "Updated");
        assert!(reloaded.done);
    }

    #[test]
    fn delete_removes_row() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        let h = insert(Task {
            id: 0,
            title: "Temp".into(),
            done: false,
        })
        .unwrap();

        assert!(find(h).unwrap().is_some());
        delete(h).unwrap();
        assert!(find(h).unwrap().is_none());
    }

    #[test]
    fn reactive_query_updates() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        let all = query_all::<Task>();
        assert_eq!(get(all).len(), 0);

        insert(Task {
            id: 0,
            title: "One".into(),
            done: false,
        })
        .unwrap();

        assert_eq!(get(all).len(), 1);

        insert(Task {
            id: 0,
            title: "Two".into(),
            done: false,
        })
        .unwrap();

        assert_eq!(get(all).len(), 2);
    }

    #[test]
    fn migrations() {
        reset();
        open(":memory:").unwrap();
        register::<Task>().unwrap();

        migrate(&[Migration {
            version: 1,
            sql: "ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0",
        }])
        .unwrap();

        migrate(&[Migration {
            version: 1,
            sql: "ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0",
        }])
        .unwrap();
    }
}
