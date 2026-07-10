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

use std::path::Path;

use icm_core::{IcmError, IcmResult};

/// Open the store's writable connection, picked from the environment:
///
/// * `TURSO_DATABASE_URL` (or `LIBSQL_URL`) + `ICM_TURSO_REPLICA` truthy →
///   local embedded replica at `path` syncing to the remote primary.
/// * URL set otherwise → remote libSQL/Turso server (`sqld`/Turso); every ICM
///   process shares it, so concurrent writes from multiple machines are
///   serialised by the server. `path` is ignored.
/// * no URL → local libSQL file at `path` (same semantics as the sqlite
///   backend, minus rusqlite).
///
/// Auth token: `TURSO_AUTH_TOKEN` (or `LIBSQL_AUTH_TOKEN`); empty is fine for
/// an unauthenticated self-hosted `sqld`.
pub(crate) fn open_backend(path: &Path) -> IcmResult<sql::Connection> {
    let url = std::env::var("TURSO_DATABASE_URL")
        .or_else(|_| std::env::var("LIBSQL_URL"))
        .ok()
        .filter(|s| !s.trim().is_empty());
    let token = std::env::var("TURSO_AUTH_TOKEN")
        .or_else(|_| std::env::var("LIBSQL_AUTH_TOKEN"))
        .unwrap_or_default();
    match url {
        Some(url) => {
            let replica = std::env::var("ICM_TURSO_REPLICA")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if replica {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                sql::Connection::open_replica(path, url, token).map_err(|e| {
                    IcmError::Database(format!("cannot open Turso embedded replica: {e}"))
                })
            } else {
                sql::Connection::open_remote(url, token).map_err(|e| {
                    IcmError::Database(format!("cannot connect to libSQL/Turso server: {e}"))
                })
            }
        }
        None => sql::Connection::open(path)
            .map_err(|e| IcmError::Database(format!("cannot open database: {e}"))),
    }
}

/// A remote server manages journaling/locking itself, so only foreign-key
/// enforcement is requested there (best-effort); local/replica files get the
/// full PRAGMAs.
pub(crate) fn apply_pragmas(conn: &sql::Connection) -> IcmResult<()> {
    if conn.is_remote() {
        let _ = conn.execute_batch("PRAGMA foreign_keys=ON;");
        Ok(())
    } else {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=30000;",
        )
        .map_err(|e| IcmError::Database(e.to_string()))
    }
}
