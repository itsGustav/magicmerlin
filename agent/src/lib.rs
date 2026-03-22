//! Agent runtime turn loop, prompt assembly, sessions, and queue control.

pub mod engine;
pub mod error;
pub mod heartbeat;
pub mod queue;
pub mod registry;
pub mod session;
pub mod system_prompt;

pub use engine::{
    AbortSignal, AgentEngine, AgentEngineConfig, AgentReply, ToolExecutionResult, ToolExecutor,
};
pub use error::AgentError;
pub use heartbeat::{
    default_state_path, load_state, run_heartbeat, run_heartbeat_with_state, save_state,
    HeartbeatOutcome, HeartbeatRunResult, HeartbeatState, HeartbeatTask,
};
pub use queue::{MessageQueue, QueueStats, QueuedMessage};
pub use registry::{AgentConfig, AgentDescriptor, AgentRegistry, RouteDecision};
pub use session::{CompactionResult, SessionKey, SessionManager, SessionMetadata, SessionRecord};
pub use system_prompt::{
    discover_skills, smart_clip_markdown, InboundContext, PromptRuntimeMetadata,
    SkillDescriptor, SystemPromptAssembler, ToolSchemaDescriptor,
};
