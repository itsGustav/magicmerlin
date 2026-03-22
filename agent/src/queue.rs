//! Message queue with collect/debounce, priority bypass, dedup, persistence, and abort signaling.

use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify};

use crate::error::{AgentError, Result};

/// One queued message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    /// Message body.
    pub text: String,
    /// Priority score (higher means more urgent).
    pub priority: u8,
    /// Optional session key.
    pub session_key: Option<String>,
    /// Unix timestamp seconds when created.
    pub created_at: i64,
}

impl QueuedMessage {
    /// Creates a regular-priority message.
    pub fn normal(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            priority: 0,
            session_key: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Returns true when this message should bypass batching.
    pub fn is_urgent(&self) -> bool {
        self.priority >= 200
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedQueue {
    queue: Vec<QueuedMessage>,
}

/// Queue statistics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueStats {
    /// Current in-memory queue size.
    pub size: usize,
    /// Number of dropped duplicates.
    pub duplicates_dropped: u64,
    /// Number of rejected pushes due to backpressure.
    pub rejected_backpressure: u64,
    /// Number of pops/batches served.
    pub batches_served: u64,
}

/// Queue for inbound messages with debounce collection.
#[derive(Clone)]
pub struct MessageQueue {
    queue: Arc<Mutex<VecDeque<QueuedMessage>>>,
    dedup: Arc<Mutex<HashSet<u64>>>,
    notify: Arc<Notify>,
    abort_notify: Arc<Notify>,
    persistence_path: Option<PathBuf>,
    max_per_session: usize,
    stats: Arc<Mutex<QueueStats>>,
}

impl MessageQueue {
    /// Creates a new queue with bounded per-session capacity.
    pub fn new(max_per_session: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            dedup: Arc::new(Mutex::new(HashSet::new())),
            notify: Arc::new(Notify::new()),
            abort_notify: Arc::new(Notify::new()),
            persistence_path: None,
            max_per_session,
            stats: Arc::new(Mutex::new(QueueStats::default())),
        }
    }

    /// Creates a queue with persistence file and loads any existing messages.
    pub async fn new_persistent(max_per_session: usize, path: impl AsRef<Path>) -> Result<Self> {
        let mut queue = Self::new(max_per_session);
        queue.persistence_path = Some(path.as_ref().to_path_buf());
        queue.restore().await?;
        Ok(queue)
    }

    /// Pushes one message into queue with dedup and backpressure checks.
    pub async fn push(&self, message: QueuedMessage) -> Result<bool> {
        let hash = hash_message(&message);
        {
            let mut dedup = self.dedup.lock().await;
            if dedup.contains(&hash) {
                let mut stats = self.stats.lock().await;
                stats.duplicates_dropped = stats.duplicates_dropped.saturating_add(1);
                return Ok(false);
            }
            dedup.insert(hash);
            if dedup.len() > 10_000 {
                // Keep dedup memory bounded by clearing old fingerprints.
                dedup.clear();
                dedup.insert(hash);
            }
        }

        let mut guard = self.queue.lock().await;
        let session_count = if let Some(session_key) = &message.session_key {
            guard
                .iter()
                .filter(|m| m.session_key.as_ref() == Some(session_key))
                .count()
        } else {
            0
        };

        if session_count >= self.max_per_session {
            let mut stats = self.stats.lock().await;
            stats.rejected_backpressure = stats.rejected_backpressure.saturating_add(1);
            return Ok(false);
        }

        if message.is_urgent() {
            guard.push_front(message);
        } else {
            guard.push_back(message);
        }
        drop(guard);

        self.persist().await?;
        self.notify.notify_one();
        Ok(true)
    }

    /// Collects a batch by waiting for one message and then debouncing.
    pub async fn collect_batch(&self, debounce: Duration) -> Result<Vec<QueuedMessage>> {
        loop {
            let mut guard = self.queue.lock().await;
            if let Some(first) = guard.pop_front() {
                let urgent = first.is_urgent();
                let mut batch = vec![first];
                drop(guard);

                if !urgent {
                    let start = Instant::now();
                    while start.elapsed() < debounce {
                        tokio::time::sleep(Duration::from_millis(15)).await;
                        let mut guard = self.queue.lock().await;
                        if let Some(next) = guard.front() {
                            if next.is_urgent() {
                                break;
                            }
                        }
                        if let Some(next) = guard.pop_front() {
                            batch.push(next);
                            continue;
                        }
                        drop(guard);
                    }
                }

                self.persist().await?;
                let mut stats = self.stats.lock().await;
                stats.batches_served = stats.batches_served.saturating_add(1);
                stats.size = self.queue.lock().await.len();
                return Ok(batch);
            }
            drop(guard);
            self.notify.notified().await;
        }
    }

    /// Pops one message immediately if available.
    pub async fn pop_now(&self) -> Result<Option<QueuedMessage>> {
        let mut guard = self.queue.lock().await;
        let value = guard.pop_front();
        drop(guard);
        self.persist().await?;
        Ok(value)
    }

    /// Clears queue and persistence file.
    pub async fn clear(&self) -> Result<()> {
        self.queue.lock().await.clear();
        self.persist().await
    }

    /// Notifies in-progress turn to abort.
    pub fn abort_in_progress(&self) {
        self.abort_notify.notify_waiters();
    }

    /// Waits for abort notification.
    pub async fn wait_abort(&self) {
        self.abort_notify.notified().await;
    }

    /// Returns queue stats.
    pub async fn stats(&self) -> QueueStats {
        let mut stats = self.stats.lock().await.clone();
        stats.size = self.queue.lock().await.len();
        stats
    }

    /// Returns current queue size.
    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Returns whether queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }

    async fn persist(&self) -> Result<()> {
        let Some(path) = &self.persistence_path else {
            return Ok(());
        };

        let snapshot = PersistedQueue {
            queue: self.queue.lock().await.iter().cloned().collect(),
        };
        let body = serde_json::to_vec_pretty(&snapshot)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|source| AgentError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
        }
        tokio::fs::write(path, body)
            .await
            .map_err(|source| AgentError::Io {
                path: path.clone(),
                source,
            })?;
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        let Some(path) = &self.persistence_path else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }
        let body = tokio::fs::read(path)
            .await
            .map_err(|source| AgentError::Io {
                path: path.clone(),
                source,
            })?;
        if body.is_empty() {
            return Ok(());
        }

        let parsed = serde_json::from_slice::<PersistedQueue>(&body)?;
        let mut queue = self.queue.lock().await;
        for message in parsed.queue {
            queue.push_back(message);
        }
        Ok(())
    }
}

fn hash_message(message: &QueuedMessage) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    message.text.hash(&mut hasher);
    message.priority.hash(&mut hasher);
    message.session_key.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn batches_non_urgent_and_prioritizes_urgent() {
        let queue = MessageQueue::new(100);
        queue
            .push(QueuedMessage {
                text: "normal-1".to_string(),
                priority: 0,
                session_key: Some("s1".to_string()),
                created_at: 1,
            })
            .await
            .expect("push");
        queue
            .push(QueuedMessage {
                text: "urgent".to_string(),
                priority: 255,
                session_key: Some("s1".to_string()),
                created_at: 2,
            })
            .await
            .expect("push");

        let first = queue
            .collect_batch(Duration::from_millis(50))
            .await
            .expect("batch");
        assert_eq!(first[0].text, "urgent");
    }

    #[tokio::test]
    async fn dedup_and_backpressure_work() {
        let queue = MessageQueue::new(1);
        let msg = QueuedMessage {
            text: "dup".to_string(),
            priority: 0,
            session_key: Some("s1".to_string()),
            created_at: 1,
        };

        assert!(queue.push(msg.clone()).await.expect("push1"));
        assert!(!queue.push(msg).await.expect("push2"));

        let reject = queue
            .push(QueuedMessage {
                text: "new".to_string(),
                priority: 0,
                session_key: Some("s1".to_string()),
                created_at: 2,
            })
            .await
            .expect("push3");
        assert!(!reject);
    }

    #[tokio::test]
    async fn persistence_round_trip() {
        let temp = tempfile::tempdir().expect("tmp");
        let path = temp.path().join("queue.json");
        let queue = MessageQueue::new_persistent(10, &path)
            .await
            .expect("queue");
        queue
            .push(QueuedMessage::normal("hello"))
            .await
            .expect("push");
        assert!(path.exists());

        let queue2 = MessageQueue::new_persistent(10, &path)
            .await
            .expect("queue2");
        assert_eq!(queue2.len().await, 1);
    }
}
