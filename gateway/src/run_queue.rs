use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::{watch, Mutex, Notify};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Aborted,
    TimedOut,
    Dropped,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    pub run_id: String,
    pub session_id: String,
    pub queued_at_unix_ms: i64,
    pub started_at_unix_ms: Option<i64>,
    pub ended_at_unix_ms: Option<i64>,
    pub timeout_seconds: u64,
    pub status: RunStatus,
    pub error: Option<String>,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct RunQueueConfig {
    pub max_depth_per_session: usize,
    pub default_timeout: Duration,
}

impl Default for RunQueueConfig {
    fn default() -> Self {
        Self {
            max_depth_per_session: 5,
            default_timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
struct SessionQueue {
    lock: Arc<Mutex<()>>,
    queue: VecDeque<String>,
    records: HashMap<String, RunRecord>,
    abort_sender: Option<watch::Sender<bool>>,
    notify: Arc<Notify>,
}

impl Default for SessionQueue {
    fn default() -> Self {
        Self {
            lock: Arc::new(Mutex::new(())),
            queue: VecDeque::new(),
            records: HashMap::new(),
            abort_sender: None,
            notify: Arc::new(Notify::new()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunQueue {
    config: RunQueueConfig,
    sessions: Arc<Mutex<HashMap<String, SessionQueue>>>,
}

impl Default for RunQueue {
    fn default() -> Self {
        Self::new(RunQueueConfig::default())
    }
}

impl RunQueue {
    pub fn new(config: RunQueueConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn session_lock(&self, session_id: &str) -> Arc<Mutex<()>> {
        let mut sessions = self.sessions.lock().await;
        sessions
            .entry(session_id.to_string())
            .or_insert_with(SessionQueue::default)
            .lock
            .clone()
    }

    pub async fn enqueue(
        &self,
        session_id: &str,
        run_id: &str,
        timeout: Option<Duration>,
    ) -> Result<RunRecord, String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .entry(session_id.to_string())
            .or_insert_with(SessionQueue::default);

        if session.queue.len() >= self.config.max_depth_per_session {
            return Err(format!(
                "queue depth exceeded for session {session_id} (max={})",
                self.config.max_depth_per_session
            ));
        }

        let record = RunRecord {
            run_id: run_id.to_string(),
            session_id: session_id.to_string(),
            queued_at_unix_ms: now_unix_ms(),
            started_at_unix_ms: None,
            ended_at_unix_ms: None,
            timeout_seconds: timeout
                .unwrap_or(self.config.default_timeout)
                .as_secs()
                .max(1),
            status: RunStatus::Pending,
            error: None,
            position: session.queue.len(),
        };

        session.queue.push_back(run_id.to_string());
        session.records.insert(run_id.to_string(), record.clone());
        session.notify.notify_waiters();

        Ok(record)
    }

    pub async fn wait_turn(
        &self,
        session_id: &str,
        run_id: &str,
        queue_timeout: Duration,
    ) -> Result<(), String> {
        let start = Instant::now();
        loop {
            let notify = {
                let mut sessions = self.sessions.lock().await;
                let Some(session) = sessions.get_mut(session_id) else {
                    return Err("session queue missing".to_string());
                };

                let Some(front) = session.queue.front() else {
                    return Err("queue is empty".to_string());
                };

                if front == run_id {
                    if let Some(record) = session.records.get_mut(run_id) {
                        record.status = RunStatus::Running;
                        record.started_at_unix_ms = Some(now_unix_ms());
                        record.position = 0;
                    }
                    return Ok(());
                }

                // update queue positions
                for (idx, id) in session.queue.iter().enumerate() {
                    if let Some(record) = session.records.get_mut(id) {
                        record.position = idx;
                    }
                }

                session.notify.clone()
            };

            if start.elapsed() > queue_timeout {
                self.complete(session_id, run_id, RunStatus::Dropped, Some("queue timeout".into()))
                    .await;
                return Err(format!("timed out waiting in queue for session {session_id}"));
            }

            tokio::time::timeout(Duration::from_secs(1), notify.notified())
                .await
                .ok();
        }
    }

    pub async fn register_abort(&self, session_id: &str) -> watch::Receiver<bool> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .entry(session_id.to_string())
            .or_insert_with(SessionQueue::default);
        let (tx, rx) = watch::channel(false);
        session.abort_sender = Some(tx);
        rx
    }

    pub async fn clear_abort(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.abort_sender = None;
        }
    }

    pub async fn abort_session(&self, session_id: &str) -> bool {
        let sender = {
            let sessions = self.sessions.lock().await;
            sessions
                .get(session_id)
                .and_then(|session| session.abort_sender.clone())
        };

        if let Some(sender) = sender {
            let _ = sender.send(true);
            true
        } else {
            false
        }
    }

    pub async fn complete(
        &self,
        session_id: &str,
        run_id: &str,
        status: RunStatus,
        error: Option<String>,
    ) {
        let mut sessions = self.sessions.lock().await;
        let Some(session) = sessions.get_mut(session_id) else {
            return;
        };

        if let Some(record) = session.records.get_mut(run_id) {
            record.status = status;
            record.error = error;
            record.ended_at_unix_ms = Some(now_unix_ms());
        }

        if session.queue.front().is_some_and(|front| front == run_id) {
            session.queue.pop_front();
        } else if let Some(idx) = session.queue.iter().position(|item| item == run_id) {
            session.queue.remove(idx);
        }

        for (idx, id) in session.queue.iter().enumerate() {
            if let Some(row) = session.records.get_mut(id) {
                row.position = idx;
            }
        }

        session.notify.notify_waiters();
    }

    pub async fn list_session_runs(&self, session_id: &str) -> Vec<RunRecord> {
        let sessions = self.sessions.lock().await;
        let Some(session) = sessions.get(session_id) else {
            return Vec::new();
        };
        let mut rows = session.records.values().cloned().collect::<Vec<_>>();
        rows.sort_by_key(|row| row.queued_at_unix_ms);
        rows
    }

    pub async fn list_runs(&self) -> Vec<RunRecord> {
        let sessions = self.sessions.lock().await;
        let mut all: Vec<RunRecord> = sessions
            .values()
            .flat_map(|q| q.records.values().cloned())
            .collect();
        all.sort_by_key(|r| std::cmp::Reverse(r.queued_at_unix_ms));
        all
    }

    pub async fn get_run_status(&self, run_id: &str) -> Option<RunRecord> {
        let sessions = self.sessions.lock().await;
        for session in sessions.values() {
            if let Some(record) = session.records.get(run_id) {
                return Some(record.clone());
            }
        }
        None
    }

    pub async fn snapshot(&self) -> HashMap<String, Vec<RunRecord>> {
        let sessions = self.sessions.lock().await;
        sessions
            .iter()
            .map(|(session_id, queue)| {
                let mut rows = queue.records.values().cloned().collect::<Vec<_>>();
                rows.sort_by_key(|row| row.queued_at_unix_ms);
                (session_id.clone(), rows)
            })
            .collect()
    }
}

fn now_unix_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn queue_rejects_depth_overflow() {
        let queue = RunQueue::new(RunQueueConfig {
            max_depth_per_session: 1,
            default_timeout: Duration::from_secs(10),
        });

        queue
            .enqueue("s1", "r1", Some(Duration::from_secs(10)))
            .await
            .expect("first enqueue should work");

        let err = queue
            .enqueue("s1", "r2", Some(Duration::from_secs(10)))
            .await
            .expect_err("second enqueue should fail");
        assert!(err.contains("queue depth exceeded"));
    }

    #[tokio::test]
    async fn queue_turn_order_is_fifo() {
        let queue = RunQueue::default();

        queue
            .enqueue("s1", "r1", Some(Duration::from_secs(10)))
            .await
            .expect("r1 enqueue");
        queue
            .enqueue("s1", "r2", Some(Duration::from_secs(10)))
            .await
            .expect("r2 enqueue");

        queue
            .wait_turn("s1", "r1", Duration::from_secs(1))
            .await
            .expect("r1 turn");

        let wait_r2 = tokio::spawn({
            let queue = queue.clone();
            async move { queue.wait_turn("s1", "r2", Duration::from_secs(2)).await }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        queue.complete("s1", "r1", RunStatus::Completed, None).await;

        wait_r2
            .await
            .expect("task should run")
            .expect("r2 should get turn");
    }

    #[tokio::test]
    async fn abort_signal_flows_to_receiver() {
        let queue = RunQueue::default();
        let mut rx = queue.register_abort("session-1").await;

        let ok = queue.abort_session("session-1").await;
        assert!(ok);

        rx.changed().await.expect("rx changed");
        assert!(*rx.borrow());
    }

    #[tokio::test]
    async fn queue_timeout_marks_run_dropped() {
        let queue = RunQueue::default();
        queue
            .enqueue("s1", "r1", Some(Duration::from_secs(10)))
            .await
            .expect("enqueue r1");
        queue
            .enqueue("s1", "r2", Some(Duration::from_secs(10)))
            .await
            .expect("enqueue r2");

        let err = queue
            .wait_turn("s1", "r2", Duration::from_millis(50))
            .await
            .expect_err("r2 should timeout in queue");
        assert!(err.contains("timed out waiting in queue"));

        let rows = queue.list_session_runs("s1").await;
        let dropped = rows
            .into_iter()
            .find(|row| row.run_id == "r2")
            .expect("r2 must exist");
        assert_eq!(dropped.status, RunStatus::Dropped);
    }
}
