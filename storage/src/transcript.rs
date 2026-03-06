//! JSONL transcript read/write/repair utilities.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::StorageError;

/// Result statistics from transcript repair.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairReport {
    /// Number of invalid JSON lines removed.
    pub invalid_lines_removed: usize,
    /// Number of unmatched tool result entries removed.
    pub orphan_tool_results_removed: usize,
    /// Number of synthetic tool result entries inserted.
    pub synthesized_tool_results: usize,
}

/// JSONL transcript helper bound to one file path.
#[derive(Debug, Clone)]
pub struct TranscriptStore {
    path: PathBuf,
}

impl TranscriptStore {
    /// Creates a transcript store and parent directories if needed.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StorageError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        Ok(Self { path })
    }

    /// Returns transcript file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends one JSON object line and fsyncs for durability.
    pub fn append(&self, value: &Value) -> Result<(), StorageError> {
        let line = serde_json::to_string(value)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| StorageError::Io {
                path: self.path.clone(),
                source,
            })?;

        file.write_all(line.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .map_err(|source| StorageError::Io {
                path: self.path.clone(),
                source,
            })?;
        file.sync_data().map_err(|source| StorageError::Io {
            path: self.path.clone(),
            source,
        })?;
        Ok(())
    }

    /// Writes all transcript entries, replacing current file.
    pub fn write_all(&self, values: &[Value]) -> Result<(), StorageError> {
        let mut body = String::new();
        for value in values {
            body.push_str(&serde_json::to_string(value)?);
            body.push('\n');
        }

        fs::write(&self.path, body).map_err(|source| StorageError::Io {
            path: self.path.clone(),
            source,
        })
    }

    /// Reads transcript entries with optional offset/limit pagination.
    pub fn read(&self, offset: usize, limit: Option<usize>) -> Result<Vec<Value>, StorageError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = std::fs::File::open(&self.path).map_err(|source| StorageError::Io {
            path: self.path.clone(),
            source,
        })?;

        let reader = BufReader::new(file);
        let mut values = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            if idx < offset {
                continue;
            }
            if let Some(max) = limit {
                if values.len() >= max {
                    break;
                }
            }

            let line = line.map_err(|source| StorageError::Io {
                path: self.path.clone(),
                source,
            })?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                values.push(value);
            }
        }

        Ok(values)
    }

    /// Rewrites the transcript by summarizing old entries into one summary record.
    pub fn compact(&self, keep_last: usize) -> Result<(), StorageError> {
        let all = self.read(0, None)?;
        if all.len() <= keep_last {
            return Ok(());
        }

        let split_at = all.len() - keep_last;
        let dropped = &all[..split_at];
        let kept = &all[split_at..];

        let summary = serde_json::json!({
            "type": "summary",
            "count": dropped.len(),
            "approxTokens": dropped.iter().map(approx_token_count).sum::<usize>(),
        });

        let mut next = Vec::with_capacity(kept.len() + 1);
        next.push(summary);
        next.extend_from_slice(kept);
        self.write_all(&next)
    }

    /// Repairs invalid JSON lines and broken `tool_use`/`tool_result` pairs.
    pub fn repair_tool_pairs(&self) -> Result<RepairReport, StorageError> {
        let mut report = RepairReport::default();
        let file = match std::fs::File::open(&self.path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(report),
            Err(source) => {
                return Err(StorageError::Io {
                    path: self.path.clone(),
                    source,
                });
            }
        };

        let mut repaired = Vec::<Value>::new();
        let mut open_tool_ids: Vec<String> = Vec::new();

        for line in BufReader::new(file).lines() {
            let line = line.map_err(|source| StorageError::Io {
                path: self.path.clone(),
                source,
            })?;
            if line.trim().is_empty() {
                continue;
            }

            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                report.invalid_lines_removed += 1;
                continue;
            };

            let msg_type = value.get("type").and_then(Value::as_str).unwrap_or("");
            let tool_id = value
                .get("tool_use_id")
                .or_else(|| value.get("toolUseId"))
                .and_then(Value::as_str)
                .map(str::to_string);

            match (msg_type, tool_id) {
                ("tool_use", Some(id)) => {
                    open_tool_ids.push(id);
                    repaired.push(value);
                }
                ("tool_result", Some(id)) => {
                    if let Some(pos) = open_tool_ids.iter().position(|x| x == &id) {
                        open_tool_ids.remove(pos);
                        repaired.push(value);
                    } else {
                        report.orphan_tool_results_removed += 1;
                    }
                }
                _ => repaired.push(value),
            }
        }

        for id in open_tool_ids {
            repaired.push(serde_json::json!({
                "type": "tool_result",
                "tool_use_id": id,
                "status": "error",
                "error": "synthetic tool_result inserted during transcript repair"
            }));
            report.synthesized_tool_results += 1;
        }

        self.write_all(&repaired)?;
        Ok(report)
    }
}

/// Returns an approximate token count for one transcript message.
pub fn approx_token_count(value: &Value) -> usize {
    let text = value.to_string();
    let words = text.split_whitespace().count();
    (words as f64 * 1.3).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_read_and_repair() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = TranscriptStore::new(temp.path().join("session.jsonl")).expect("store");
        store
            .append(&serde_json::json!({"type":"tool_use","tool_use_id":"a"}))
            .expect("append use");
        store
            .append(&serde_json::json!({"type":"tool_result","tool_use_id":"z"}))
            .expect("append orphan");

        let report = store.repair_tool_pairs().expect("repair");
        assert_eq!(report.orphan_tool_results_removed, 1);
        assert_eq!(report.synthesized_tool_results, 1);

        let values = store.read(0, None).expect("read");
        assert_eq!(values.len(), 2);
    }
}
