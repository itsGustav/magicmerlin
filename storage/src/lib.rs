//! Persistent storage primitives for MagicMerlin.

mod db;
mod error;
mod locking;
mod memory;
mod transcript;

pub use db::Storage;
pub use error::StorageError;
pub use locking::SessionFileLock;
pub use memory::MemoryManager;
pub use transcript::{approx_token_count, RepairReport, TranscriptStore};
