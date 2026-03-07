//! Agent runtime turn loop, prompt assembly, sessions, and queue control.

pub mod engine;
pub mod error;
pub mod heartbeat;
pub mod queue;
pub mod registry;
pub mod session;
pub mod system_prompt;

pub use engine::{AgentEngine, AgentEngineConfig, AgentReply, ToolExecutionResult, ToolExecutor};
pub use error::AgentError;
pub use heartbeat::{run_heartbeat, HeartbeatOutcome};
pub use queue::{MessageQueue, QueuedMessage};
pub use registry::{AgentDescriptor, AgentRegistry};
pub use session::{SessionKey, SessionManager, SessionRecord};
pub use system_prompt::{discover_skills, PromptRuntimeMetadata, SystemPromptAssembler};
