//! SQLite storage backend (desktop).

use std::cell::RefCell;
use std::marker::PhantomData;

use crate::core::reactive::{create_memo, create_signal, get, set, Signal};
use rusqlite::{Connection, OptionalExtension};

use super::{ColumnType, DataError, Handle, Migration, Model, Value};

struct StoreInner {
    conn: Connection,
    version: Signal<u64>,
}

thread_local! {
    static STORE: RefCell<Option<StoreInner>> = const { RefCell::new(None) };
}

fn with_store<R>(f: impl FnOnce(&StoreInner) -> Result<R, DataError>) -> Result<R, DataError> {
    STORE.with(|cell| {
        let borrow = cell.borrow();
        let store = borrow.as_ref().ok_or(DataError::NotOpen)?;
        f(store)
    })
}

/// Open (or create) a SQLite database at `path`.
///
/// Use `":memory:"` for an in-memory database.
pub fn open(path: &str) -> Result<(), DataError> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    let version = create_signal(0_u64);
    STORE.with(|cell| {
        *cell.borrow_mut() = Some(StoreInner { conn, version });
    });
    Ok(())
}

/// Register a model's table (CREATE TABLE IF NOT EXISTS).
pub fn register<M: Model>() -> Result<(), DataError> {
    with_store(|store| {
        let sql = create_table_sql::<M>();
        store.conn.execute_batch(&sql)?;
        Ok(())
    })
}

/// Apply pending migrations.
pub fn migrate(migrations: &[Migration]) -> Result<(), DataError> {
    with_store(|store| {
        store.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY)",
        )?;

        for m in migrations {
            let applied: bool = store.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM _migrations WHERE version = ?1)",
                [m.version],
                |row| row.get(0),
            )?;
            if !applied {
                store.conn.execute_batch(m.sql)?;
                store
                    .conn
                    .execute("INSERT INTO _migrations (version) VALUES (?1)", [m.version])?;
            }
        }
        Ok(())
    })
}

/// Insert a model and return a [`Handle`] to it.
pub fn insert<M: Model>(model: M) -> Result<Handle<M>, DataError> {
    let (handle, ver_sig, new_ver) = with_store(|store| {
        let cols = non_id_columns::<M>();
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{i}")).collect();
        let col_names: Vec<&str> = cols.iter().map(|c| c.name).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            M::table_name(),
            col_names.join(", "),
            placeholders.join(", ")
        );

        let params = model.to_params();
        let refs = params_to_sql(&params);
        store.conn.execute(&sql, refs.as_slice())?;
        let new_id = store.conn.last_insert_rowid();

        let v = get(store.version);
        Ok((
            Handle {
                id: new_id,
                _marker: PhantomData,
            },
            store.version,
            v + 1,
        ))
    })?;
    set(ver_sig, new_ver);
    Ok(handle)
}

/// Read a model by handle.
pub fn find<M: Model>(handle: Handle<M>) -> Result<Option<M>, DataError> {
    with_store(|store| {
        let col_names = all_column_names::<M>();
        let sql = format!(
            "SELECT {} FROM {} WHERE id = ?1",
            col_names.join(", "),
            M::table_name()
        );
        let result = store
            .conn
            .query_row(&sql, [handle.id()], |row| Ok(row_to_model::<M>(row)))
            .optional()?;
        Ok(result)
    })
}

/// Update an existing row.
pub fn save<M: Model>(handle: Handle<M>, model: &M) -> Result<(), DataError> {
    let (ver_sig, new_ver) = with_store(|store| {
        let cols = non_id_columns::<M>();
        let sets: Vec<String> = cols
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{} = ?{}", c.name, i + 1))
            .collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?{}",
            M::table_name(),
            sets.join(", "),
            cols.len() + 1
        );

        let mut params = model.to_params();
        params.push(Value::Integer(handle.id()));
        let refs = params_to_sql(&params);
        store.conn.execute(&sql, refs.as_slice())?;

        let v = get(store.version);
        Ok((store.version, v + 1))
    })?;
    set(ver_sig, new_ver);
    Ok(())
}

/// Delete a row by handle.
pub fn delete<M: Model>(handle: Handle<M>) -> Result<(), DataError> {
    let (ver_sig, new_ver) = with_store(|store| {
        let sql = format!("DELETE FROM {} WHERE id = ?1", M::table_name());
        store.conn.execute(&sql, [handle.id()])?;
        let v = get(store.version);
        Ok((store.version, v + 1))
    })?;
    set(ver_sig, new_ver);
    Ok(())
}

/// Reactive query: returns a [`Signal`] of all rows that updates on writes.
pub fn query_all<M: Model>() -> Signal<Vec<M>> {
    let ver_sig = with_store(|store| Ok(store.version)).expect("store not open");
    create_memo(move || {
        let _ = get(ver_sig);
        with_store(|store| {
            let col_names = all_column_names::<M>();
            let sql = format!("SELECT {} FROM {}", col_names.join(", "), M::table_name());
            let mut stmt = store.conn.prepare(&sql)?;
            let rows = stmt.query_map([], |row| Ok(row_to_model::<M>(row)))?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
    })
}

/// Close the store and release the database connection.
pub fn close() {
    STORE.with(|cell| {
        cell.borrow_mut().take();
    });
}

/// Reset the store (for tests).
pub fn reset() {
    close();
    crate::core::reactive::reset();
}

// ── Internal helpers ─────────────────────────────────────────────────────

fn create_table_sql<M: Model>() -> String {
    let cols: Vec<String> = M::columns()
        .iter()
        .map(|c| {
            let ty = match c.ty {
                ColumnType::Integer => "INTEGER",
                ColumnType::Real => "REAL",
                ColumnType::Text => "TEXT",
                ColumnType::Bool => "INTEGER",
            };
            if c.primary_key {
                format!("{} {} PRIMARY KEY AUTOINCREMENT", c.name, ty)
            } else {
                format!("{} {} NOT NULL", c.name, ty)
            }
        })
        .collect();

    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        M::table_name(),
        cols.join(", ")
    )
}

fn non_id_columns<M: Model>() -> Vec<&'static super::ColumnDef> {
    M::columns().iter().filter(|c| !c.primary_key).collect()
}

fn all_column_names<M: Model>() -> Vec<&'static str> {
    M::columns().iter().map(|c| c.name).collect()
}

fn params_to_sql(values: &[Value]) -> Vec<&dyn rusqlite::types::ToSql> {
    values
        .iter()
        .map(|v| v as &dyn rusqlite::types::ToSql)
        .collect()
}

impl rusqlite::types::ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            Value::Null => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Null,
            )),
            Value::Integer(i) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*i),
            )),
            Value::Real(r) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Real(*r),
            )),
            Value::Text(s) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(s.clone()),
            )),
            Value::Bool(b) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(i64::from(*b)),
            )),
        }
    }
}

fn row_to_model<M: Model>(row: &rusqlite::Row<'_>) -> M {
    let cols = M::columns();
    let values: Vec<Value> = cols
        .iter()
        .enumerate()
        .map(|(i, c)| match c.ty {
            ColumnType::Integer => Value::Integer(row.get::<_, i64>(i).unwrap_or(0)),
            ColumnType::Real => Value::Real(row.get::<_, f64>(i).unwrap_or(0.0)),
            ColumnType::Text => Value::Text(row.get::<_, String>(i).unwrap_or_default()),
            ColumnType::Bool => Value::Bool(row.get::<_, i64>(i).unwrap_or(0) != 0),
        })
        .collect();
    M::from_values(&values)
}
