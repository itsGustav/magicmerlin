//! Multi-provider LLM routing, auth, and failover primitives.

pub mod auth;
pub mod error;
pub mod model_registry;
pub mod providers;
pub mod router;
pub mod types;

pub use auth::{AuthProfile, AuthProfiles, OAuthTokenConfig};
pub use error::{ProviderError, Result};
pub use model_registry::{ModelCapabilities, ModelDefinition, ModelRegistry};
pub use router::{ProviderRouter, RetryConfig};
pub use types::{
    CompletionRequest, CompletionResponse, ContentBlock, ContentPart, Message, MessageContent,
    Role, StopReason, StreamChunk, ToolCall, ToolDefinition, ToolResultContent, Usage,
};
