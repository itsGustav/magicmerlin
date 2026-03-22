//! Signal connection monitor with automatic reconnection.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::framework::InboundMessage;

use super::SignalChannel;

/// Wraps a [`SignalChannel`] with automatic reconnection on receive loop failure.
pub struct SignalMonitor {
    channel: Arc<SignalChannel>,
    retry_delay: Duration,
    max_retries: u32,
}

impl SignalMonitor {
    /// Creates a new monitor.
    pub fn new(channel: Arc<SignalChannel>, retry_delay: Duration, max_retries: u32) -> Self {
        Self {
            channel,
            retry_delay,
            max_retries,
        }
    }

    /// Runs the receive loop with automatic retry on failure.
    ///
    /// Stops when either:
    /// - The receive loop returns `Ok(())` (clean shutdown / receiver dropped)
    /// - The maximum number of consecutive retries is exceeded
    pub async fn run(&self, tx: mpsc::Sender<InboundMessage>) {
        let mut consecutive_errors = 0u32;

        loop {
            match self.channel.run_receive_loop(tx.clone()).await {
                Ok(()) => break,
                Err(e) => {
                    consecutive_errors += 1;
                    tracing::error!(
                        "Signal receive loop error ({consecutive_errors}/{}): {e}",
                        self.max_retries
                    );

                    if consecutive_errors >= self.max_retries {
                        tracing::error!("Signal monitor: max retries reached, stopping");
                        break;
                    }

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
}
