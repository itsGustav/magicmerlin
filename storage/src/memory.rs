//! MEMORY.md and daily memory log file manager.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::StorageError;

/// Memory file manager rooted at a state directory.
#[derive(Debug, Clone)]
pub struct MemoryManager {
    root: PathBuf,
}

impl MemoryManager {
    /// Creates a new memory manager and required directories.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, StorageError> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(root.join("memory")).map_err(|source| StorageError::Io {
            path: root.join("memory"),
            source,
        })?;
        Ok(Self { root })
    }

    /// Returns path to top-level `MEMORY.md`.
    pub fn memory_md_path(&self) -> PathBuf {
        self.root.join("MEMORY.md")
    }

    /// Returns path to daily memory file (`memory/YYYY-MM-DD.md`).
    pub fn daily_path(&self, date: NaiveDate) -> PathBuf {
        self.root.join("memory").join(format!("{date}.md"))
    }

    /// Reads `MEMORY.md` if present.
    pub fn read_memory_md(&self) -> Result<Option<String>, StorageError> {
        let path = self.memory_md_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(Some(content)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(StorageError::Io { path, source }),
        }
    }

    /// Writes `MEMORY.md` content.
    pub fn write_memory_md(&self, content: &str) -> Result<(), StorageError> {
        let path = self.memory_md_path();
        std::fs::write(&path, content).map_err(|source| StorageError::Io { path, source })
    }

    /// Reads one daily memory markdown file.
    pub fn read_daily(&self, date: NaiveDate) -> Result<Option<String>, StorageError> {
        let path = self.daily_path(date);
        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(Some(content)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(StorageError::Io { path, source }),
        }
    }

    /// Appends one entry to daily memory file.
    pub fn append_daily_entry(&self, date: NaiveDate, entry: &str) -> Result<(), StorageError> {
        let path = self.daily_path(date);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| StorageError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|source| StorageError::Io {
                path: path.clone(),
                source,
            })?;

        use std::io::Write as _;
        writeln!(file, "- {entry}").map_err(|source| StorageError::Io {
            path: path.clone(),
            source,
        })?;
        file.sync_data()
            .map_err(|source| StorageError::Io { path, source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_reads_daily_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let manager = MemoryManager::new(temp.path()).expect("manager");
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).expect("date");
        manager
            .append_daily_entry(date, "Bootstrapped storage crate")
            .expect("append");

        let body = manager.read_daily(date).expect("read").expect("content");
        assert!(body.contains("Bootstrapped storage crate"));
    }
}
