//! Turso/libSQL backend wrapper: imports the shared store/schema with the
//! dbcompat shim (synchronous rusqlite-shaped facade over the async libsql
//! client).
//!
//! `#[path]` here is relative to this file's directory (`src/`), so
//! `schema.rs` and `store.rs` at `src/schema.rs` / `src/store.rs` are loaded
//! under the `turso_backend::` namespace.

/// SQL provider = dbcompat (synchronous libSQL facade, rusqlite-shaped).
#[allow(unused_imports)]
pub(crate) mod sql {
    pub use crate::dbcompat::{
        ffi,
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
}

#[path = "schema.rs"]
pub(crate) mod schema;

#[path = "store.rs"]
pub(crate) mod store;
