//! SQLite storage pool and schema migration helpers.

use std::path::{Path, PathBuf};

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

use crate::StorageError;

/// SQLite-backed storage handle with connection pool.
#[derive(Clone)]
pub struct Storage {
    db_path: PathBuf,
    pool: Pool<SqliteConnectionManager>,
}

impl Storage {
    /// Opens a SQLite database and runs migrations.
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| StorageError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder().max_size(8).build(manager)?;
        let storage = Self { db_path, pool };
        storage.migrate()?;
        Ok(storage)
    }

    /// Returns the configured database path.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Borrows a pooled SQLite connection.
    pub fn connection(&self) -> Result<PooledConnection<SqliteConnectionManager>, StorageError> {
        self.pool.get().map_err(StorageError::Pool)
    }

    /// Applies idempotent schema migrations.
    pub fn migrate(&self) -> Result<(), StorageError> {
        let conn = self.connection()?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;

            CREATE TABLE IF NOT EXISTS sessions (
              id          TEXT PRIMARY KEY,
              agent       TEXT,
              status      TEXT NOT NULL,
              started_at  INTEGER NOT NULL,
              updated_at  INTEGER NOT NULL,
              metadata    TEXT
            );

            CREATE TABLE IF NOT EXISTS cron_jobs (
              id              INTEGER PRIMARY KEY AUTOINCREMENT,
              name            TEXT NOT NULL,
              schedule        TEXT NOT NULL,
              kind            TEXT NOT NULL,
              payload         TEXT NOT NULL,
              enabled         INTEGER NOT NULL DEFAULT 1,
              attempts        INTEGER NOT NULL DEFAULT 0,
              max_attempts    INTEGER NOT NULL DEFAULT 3,
              backoff_seconds INTEGER NOT NULL DEFAULT 30,
              last_run_at     INTEGER,
              next_run_at     INTEGER,
              last_status     TEXT,
              last_error      TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_cron_jobs_next_run ON cron_jobs(enabled, next_run_at);

            CREATE TABLE IF NOT EXISTS cron_runs (
              id          INTEGER PRIMARY KEY AUTOINCREMENT,
              job_id      INTEGER NOT NULL,
              started_at  INTEGER NOT NULL,
              ended_at    INTEGER,
              status      TEXT NOT NULL,
              error       TEXT,
              metadata    TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_cron_runs_job ON cron_runs(job_id, started_at DESC);

            CREATE TABLE IF NOT EXISTS approvals (
              id          INTEGER PRIMARY KEY AUTOINCREMENT,
              agent       TEXT NOT NULL DEFAULT '*',
              key         TEXT NOT NULL,
              value       TEXT NOT NULL,
              updated_at  INTEGER NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_approvals_agent_key ON approvals(agent, key);

            CREATE TABLE IF NOT EXISTS plugins (
              id          INTEGER PRIMARY KEY AUTOINCREMENT,
              name        TEXT NOT NULL,
              version     TEXT,
              enabled     INTEGER NOT NULL DEFAULT 1,
              config      TEXT,
              updated_at  INTEGER NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_plugins_name ON plugins(name);

            CREATE TABLE IF NOT EXISTS dead_letters (
              id            INTEGER PRIMARY KEY AUTOINCREMENT,
              source        TEXT NOT NULL,
              payload       TEXT NOT NULL,
              error         TEXT NOT NULL,
              created_at    INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_dead_letters_created ON dead_letters(created_at DESC);
            "#,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_tables() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("magicmerlin.db");
        let storage = Storage::new(&db_path).expect("storage");
        let conn = storage.connection().expect("connection");

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='cron_jobs'",
                [],
                |row| row.get(0),
            )
            .expect("query");

        assert_eq!(exists, 1);
    }
}
