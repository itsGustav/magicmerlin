//! PID-based lock-file manager for session transcripts.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::StorageError;

/// Held lock guard that removes lock file on drop.
#[derive(Debug)]
pub struct SessionFileLock {
    lock_path: PathBuf,
}

impl SessionFileLock {
    /// Acquires a lock for `<session>.jsonl.lock` with timeout.
    pub fn acquire(
        session_path: impl AsRef<Path>,
        timeout: Duration,
    ) -> Result<Self, StorageError> {
        let session_path = session_path.as_ref();
        let lock_path = PathBuf::from(format!("{}.lock", session_path.display()));
        let start = Instant::now();

        loop {
            match try_create_lock(&lock_path) {
                Ok(()) => return Ok(Self { lock_path }),
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    if is_stale_lock(&lock_path)? {
                        std::fs::remove_file(&lock_path).map_err(|source| StorageError::Io {
                            path: lock_path.clone(),
                            source,
                        })?;
                        continue;
                    }

                    if start.elapsed() >= timeout {
                        return Err(StorageError::LockTimeout(lock_path));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(source) => {
                    return Err(StorageError::Io {
                        path: lock_path,
                        source,
                    });
                }
            }
        }
    }

    /// Returns current lock-file path.
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }
}

impl Drop for SessionFileLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

fn try_create_lock(lock_path: &Path) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(lock_path)?;

    let pid = std::process::id();
    writeln!(file, "pid={pid}")?;
    writeln!(file, "created_unix={}", chrono::Utc::now().timestamp())?;
    file.sync_data()
}

fn is_stale_lock(lock_path: &Path) -> Result<bool, StorageError> {
    let raw = std::fs::read_to_string(lock_path).map_err(|source| StorageError::Io {
        path: lock_path.to_path_buf(),
        source,
    })?;

    let pid_line = raw
        .lines()
        .find(|line| line.starts_with("pid="))
        .ok_or_else(|| StorageError::InvalidLock(raw.clone()))?;
    let pid: i32 = pid_line
        .trim_start_matches("pid=")
        .parse()
        .map_err(|_| StorageError::InvalidLock(raw.clone()))?;

    Ok(!is_pid_alive(pid))
}

fn is_pid_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }

    let status = std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status();

    matches!(status, Ok(status) if status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_lifecycle() {
        let temp = tempfile::tempdir().expect("tempdir");
        let session = temp.path().join("abc.jsonl");
        let lock = SessionFileLock::acquire(&session, Duration::from_millis(10)).expect("lock");
        assert!(lock.lock_path().exists());
        drop(lock);
        assert!(!session.with_extension("jsonl.lock").exists());
    }
}
