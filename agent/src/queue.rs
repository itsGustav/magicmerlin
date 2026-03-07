//! Message queue with collect/debounce and abort signaling.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, Notify};

/// One queued message.
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    /// Message body.
    pub text: String,
    /// Priority score (higher means more urgent).
    pub priority: u8,
}

/// Queue for inbound messages with debounce collection.
#[derive(Clone)]
pub struct MessageQueue {
    tx: mpsc::Sender<QueuedMessage>,
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<QueuedMessage>>>,
    abort_notify: Arc<Notify>,
}

impl MessageQueue {
    /// Creates a new queue with bounded capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            tx,
            rx: Arc::new(tokio::sync::Mutex::new(rx)),
            abort_notify: Arc::new(Notify::new()),
        }
    }

    /// Sends one message into queue.
    pub async fn push(
        &self,
        message: QueuedMessage,
    ) -> std::result::Result<(), mpsc::error::SendError<QueuedMessage>> {
        self.tx.send(message).await
    }

    /// Collects a batch by waiting for one message and then debouncing.
    pub async fn collect_batch(&self, debounce: Duration) -> Vec<QueuedMessage> {
        let mut guard = self.rx.lock().await;
        let mut batch = Vec::new();

        let Some(first) = guard.recv().await else {
            return batch;
        };
        batch.push(first);

        while let Ok(Some(next)) = tokio::time::timeout(debounce, guard.recv()).await {
            batch.push(next);
        }
        batch
    }

    /// Notifies in-progress turn to abort.
    pub fn abort_in_progress(&self) {
        self.abort_notify.notify_waiters();
    }

    /// Waits for abort notification.
    pub async fn wait_abort(&self) {
        self.abort_notify.notified().await;
    }
}
