//! Lotion core: one SQLite workspace file, opened with the schema from
//! `docs/03-DATA-MODEL.md` (ADR-018: the core is Rust).

pub mod host;

use rusqlite::Connection;
use std::path::Path;

/// Initial schema, generated from docs/03-DATA-MODEL.md.
pub const SCHEMA_V1: &str = include_str!("../migrations/0001_init.sql");

/// Current schema version stored in `PRAGMA user_version`.
pub const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("workspace schema version {found} is newer than this build ({supported})")]
    TooNew { found: i64, supported: i64 },
    #[error("DB worker thread stopped before opening the workspace")]
    WorkerFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Answer to the `ping` command: proves the core reached SQLite.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pong {
    pub sqlite_version: String,
    pub schema_version: i64,
}

/// An open workspace.
pub struct Workspace {
    conn: Connection,
}

impl Workspace {
    /// Opens (creating if needed) a workspace file and brings its schema up to date.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    /// Opens a private in-memory workspace (tests, scratch).
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;",
        )?;
        let found: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if found > SCHEMA_VERSION {
            return Err(Error::TooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        if found < 1 {
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch(SCHEMA_V1)?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            tx.commit()?;
        }
        Ok(Self { conn })
    }

    /// The `ping` command.
    pub fn ping(&self) -> Result<Pong> {
        let sqlite_version = self
            .conn
            .query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
        let schema_version = self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))?;
        Ok(Pong {
            sqlite_version,
            schema_version,
        })
    }

    /// Raw connection, for tests inside this crate (engine modules will use it too).
    #[cfg(test)]
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

/// New time-ordered id (UUIDv7), as text.
pub fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests;
