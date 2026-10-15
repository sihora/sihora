//! In-memory storage backend for web (wasm32).

use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::reactive::{create_memo, create_signal, get, set, Signal};

use super::{DataError, Handle, Migration, Model, Value};
use std::marker::PhantomData;

struct TableData {
    rows: Vec<Vec<Value>>,
    next_id: i64,
}

struct StoreInner {
    tables: HashMap<String, TableData>,
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

fn with_store_mut<R>(
    f: impl FnOnce(&mut StoreInner) -> Result<R, DataError>,
) -> Result<R, DataError> {
    STORE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let store = borrow.as_mut().ok_or(DataError::NotOpen)?;
        f(store)
    })
}

/// Open an in-memory data store (path is ignored on web).
pub fn open(_path: &str) -> Result<(), DataError> {
    let version = create_signal(0_u64);
    STORE.with(|cell| {
        *cell.borrow_mut() = Some(StoreInner {
            tables: HashMap::new(),
            version,
        });
    });
    Ok(())
}

/// Register a model's table.
pub fn register<M: Model>() -> Result<(), DataError> {
    with_store_mut(|store| {
        let name = M::table_name().to_string();
        store.tables.entry(name).or_insert_with(|| TableData {
            rows: Vec::new(),
            next_id: 1,
        });
        Ok(())
    })
}

/// Apply migrations (no-op for in-memory backend).
pub fn migrate(_migrations: &[Migration]) -> Result<(), DataError> {
    with_store(|_| Ok(()))
}

/// Insert a model and return a handle.
pub fn insert<M: Model>(model: M) -> Result<Handle<M>, DataError> {
    let (handle, ver_sig, new_ver) = with_store_mut(|store| {
        let name = M::table_name().to_string();
        let table = store
            .tables
            .get_mut(&name)
            .ok_or_else(|| DataError::Storage(format!("table {name} not registered")))?;

        let new_id = table.next_id;
        table.next_id += 1;

        let mut values = vec![Value::Integer(new_id)];
        values.extend(model.to_params());
        table.rows.push(values);

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
        let name = M::table_name();
        let table = store
            .tables
            .get(name)
            .ok_or_else(|| DataError::Storage(format!("table {name} not registered")))?;

        for row in &table.rows {
            if let Some(Value::Integer(id)) = row.first() {
                if *id == handle.id() {
                    return Ok(Some(M::from_values(row)));
                }
            }
        }
        Ok(None)
    })
}

/// Update an existing row.
pub fn save<M: Model>(handle: Handle<M>, model: &M) -> Result<(), DataError> {
    let (ver_sig, new_ver) = with_store_mut(|store| {
        let name = M::table_name();
        let table = store
            .tables
            .get_mut(name)
            .ok_or_else(|| DataError::Storage(format!("table {name} not registered")))?;

        for row in &mut table.rows {
            if let Some(Value::Integer(id)) = row.first() {
                if *id == handle.id() {
                    let mut new_values = vec![Value::Integer(handle.id())];
                    new_values.extend(model.to_params());
                    *row = new_values;
                    break;
                }
            }
        }

        let v = get(store.version);
        Ok((store.version, v + 1))
    })?;
    set(ver_sig, new_ver);
    Ok(())
}

/// Delete a row by handle.
pub fn delete<M: Model>(handle: Handle<M>) -> Result<(), DataError> {
    let (ver_sig, new_ver) = with_store_mut(|store| {
        let name = M::table_name();
        let table = store
            .tables
            .get_mut(name)
            .ok_or_else(|| DataError::Storage(format!("table {name} not registered")))?;

        table.rows.retain(|row| {
            if let Some(Value::Integer(id)) = row.first() {
                *id != handle.id()
            } else {
                true
            }
        });

        let v = get(store.version);
        Ok((store.version, v + 1))
    })?;
    set(ver_sig, new_ver);
    Ok(())
}

/// Reactive query: returns a signal of all rows that updates on writes.
pub fn query_all<M: Model>() -> Signal<Vec<M>> {
    let ver_sig = with_store(|store| Ok(store.version)).expect("store not open");
    create_memo(move || {
        let _ = get(ver_sig);
        with_store(|store| {
            let name = M::table_name();
            let table = match store.tables.get(name) {
                Some(t) => t,
                None => return Ok(Vec::new()),
            };
            Ok(table.rows.iter().map(|row| M::from_values(row)).collect())
        })
        .unwrap_or_default()
    })
}

/// Close the store.
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
