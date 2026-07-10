//! SQLite backend wrapper: imports the shared store/schema with real rusqlite.
//!
//! `#[path]` here is relative to this file's directory (`src/`), so
//! `schema.rs` and `store.rs` at `src/schema.rs` / `src/store.rs` are loaded
//! under the `sqlite_backend::` namespace.

/// SQL provider = real rusqlite.
#[allow(unused_imports)]
pub(crate) mod sql {
    pub use rusqlite::{
        params,
        Connection,
        Error,
        MappedRows,
        OpenFlags,
        OptionalExtension,
        Result,
        Row,
        Statement,
        ToSql,
        types,
    };
    // Re-export the ffi module so store.rs can do `use super::sql::ffi::...`.
    pub use rusqlite::ffi;
}

#[path = "schema.rs"]
pub(crate) mod schema;

#[path = "store.rs"]
pub(crate) mod store;

use std::path::Path;

use icm_core::{IcmError, IcmResult};

/// Open the store's writable connection: always the local SQLite file.
/// (Provider-specific so the turso backend can dispatch on env vars.)
pub(crate) fn open_backend(path: &Path) -> IcmResult<sql::Connection> {
    sql::Connection::open(path)
        .map_err(|e| IcmError::Database(format!("cannot open database: {e}")))
}

/// Connection PRAGMAs (WAL + foreign keys + busy timeout) for the local file.
pub(crate) fn apply_pragmas(conn: &sql::Connection) -> IcmResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=30000;",
    )
    .map_err(|e| IcmError::Database(e.to_string()))
}
