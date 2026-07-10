//! Storage backends for ICM.
//!
//! The store is pluggable (issue #301) and backends are **additive**: any
//! combination can be compiled into one binary, and the active backend is
//! selected at **runtime** via the `ICM_DB_BACKEND` environment variable
//! (`sqlite` (default) / `postgres` / `opensearch` / `turso`). This follows the
//! idiomatic Rust pattern (e.g. SurrealDB's `Surreal<Any>`): features only
//! control which backends are *available*, not which one runs.
//!
//! - **`backend-sqlite`** (default) — in-process SQLite via `rusqlite` +
//!   `sqlite-vec`. Lightweight, no external service.
//! - **`postgres`** — network-accessible PostgreSQL (`pgvector` + FTS) so
//!   replicas share one memory store.
//! - **`opensearch`** — network-accessible OpenSearch (BM25 + `knn_vector`).
//! - **`turso`** — network-accessible libSQL/Turso via the `libsql` async
//!   client wrapped in a synchronous facade (`dbcompat`). Same SQL as
//!   `backend-sqlite`; works with sqld/Turso remote databases.
//!
//! `icm-cli` / `icm-mcp` use the [`Store`] enum, which dispatches every
//! call to whichever backend variant is active.

// At least one backend must be compiled in.
#[cfg(not(any(
    feature = "backend-sqlite",
    feature = "postgres",
    feature = "opensearch",
    feature = "turso",
)))]
compile_error!(
    "at least one storage backend must be enabled: `backend-sqlite` (default), \
     `postgres`, `opensearch`, and/or `turso`"
);

mod backend;
mod common;

#[cfg(feature = "backend-sqlite")]
mod schema;
#[cfg(feature = "backend-sqlite")]
mod store;

#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "opensearch")]
mod opensearch;

#[cfg(feature = "turso")]
#[macro_use]
pub mod dbcompat;
#[cfg(feature = "turso")]
mod turso_schema;
#[cfg(feature = "turso")]
mod turso_store;

// Shared row types (backend-agnostic).
pub use common::{CodeArea, HookEvent, HookEventInsert, HookStatsRow, PendingRow};

// The runtime-dispatched store and the backend selector.
pub use backend::{BackendKind, Store};

// Concrete backend types, exposed for direct use / tests.
#[cfg(feature = "opensearch")]
pub use opensearch::OpenSearchStore;
#[cfg(feature = "postgres")]
pub use postgres::PostgresStore;
#[cfg(feature = "backend-sqlite")]
pub use store::SqliteStore;
#[cfg(feature = "turso")]
pub use turso_store::TursoStore;
