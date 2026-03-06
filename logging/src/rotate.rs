//! Size-based rotating file writer for tracing subscribers.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::writer::MakeWriter;

use crate::LoggingError;

/// Thread-safe make-writer for tracing layers with rotation.
#[derive(Clone, Debug)]
pub(crate) struct RotatingMakeWriter {
    inner: Arc<Mutex<RotatingFile>>,
}

impl RotatingMakeWriter {
    /// Creates a new rotating writer at `base_path` with size and retention limits.
    pub(crate) fn new(
        base_path: PathBuf,
        max_bytes: u64,
        keep: usize,
    ) -> Result<Self, LoggingError> {
        let writer = RotatingFile::new(base_path, max_bytes, keep)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(writer)),
        })
    }
}

impl<'a> MakeWriter<'a> for RotatingMakeWriter {
    type Writer = RotatingGuard;

    fn make_writer(&'a self) -> Self::Writer {
        RotatingGuard {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Write adapter that serializes writes through a mutex.
pub(crate) struct RotatingGuard {
    inner: Arc<Mutex<RotatingFile>>,
}

impl io::Write for RotatingGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("log writer poisoned"))?;
        guard.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("log writer poisoned"))?;
        guard.flush()
    }
}

#[derive(Debug)]
struct RotatingFile {
    base_path: PathBuf,
    max_bytes: u64,
    keep: usize,
    size: u64,
    file: File,
}

impl RotatingFile {
    fn new(base_path: PathBuf, max_bytes: u64, keep: usize) -> Result<Self, LoggingError> {
        if let Some(parent) = base_path.parent() {
            fs::create_dir_all(parent).map_err(|source| LoggingError::CreateLogFile {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let file = open_file(&base_path)?;
        let size = file.metadata().map(|m| m.len()).unwrap_or(0);

        Ok(Self {
            base_path,
            max_bytes: max_bytes.max(1),
            keep: keep.max(1),
            size,
            file,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.flush()?;

        for index in (1..=self.keep).rev() {
            let src = self.rotated_path(index);
            if !src.exists() {
                continue;
            }
            if index == self.keep {
                fs::remove_file(&src)?;
            } else {
                let dst = self.rotated_path(index + 1);
                fs::rename(&src, &dst)?;
            }
        }

        if self.base_path.exists() {
            fs::rename(&self.base_path, self.rotated_path(1))?;
        }

        self.file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.base_path)?;
        self.size = 0;
        Ok(())
    }

    fn rotated_path(&self, index: usize) -> PathBuf {
        PathBuf::from(format!("{}.{}", self.base_path.display(), index))
    }
}

impl io::Write for RotatingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.size.saturating_add(buf.len() as u64) > self.max_bytes && self.size > 0 {
            self.rotate().map_err(|source| {
                io::Error::other(LoggingError::Rotate {
                    path: self.base_path.clone(),
                    source,
                })
            })?;
        }

        self.file.write_all(buf)?;
        self.size = self.size.saturating_add(buf.len() as u64);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

fn open_file(path: &Path) -> Result<File, LoggingError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| LoggingError::CreateLogFile {
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_and_keeps_last_n_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("gateway.log");

        let mut rotating = RotatingFile::new(path.clone(), 10, 5).expect("writer");
        rotating.write_all(b"0123456789").expect("write 1");
        rotating.write_all(b"abc").expect("write 2");
        rotating.write_all(b"defghijkl").expect("write 3");
        rotating.write_all(b"mnopqrstu").expect("write 4");
        rotating.write_all(b"vwxyz").expect("write 5");
        rotating.write_all(b"1234567890").expect("write 6");

        assert!(path.exists());
        assert!(temp.path().join("gateway.log.1").exists());
        assert!(temp.path().join("gateway.log.2").exists());
        assert!(temp.path().join("gateway.log.3").exists());
        assert!(temp.path().join("gateway.log.4").exists());
        assert!(temp.path().join("gateway.log.5").exists());
        assert!(!temp.path().join("gateway.log.6").exists());
    }
}
