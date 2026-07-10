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
