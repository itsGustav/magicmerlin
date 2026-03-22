//! Multi-provider LLM routing, auth, and failover primitives.

pub mod auth;
pub mod error;
mod model_catalog;
pub mod model_registry;
pub mod providers;
pub mod router;
pub mod types;

pub use auth::{AuthHealth, AuthProfile, AuthProfiles, OAuthTokenConfig};
pub use error::{ProviderError, Result, RetryHint};
pub use model_registry::{ModelCapabilities, ModelDefinition, ModelRegistry, ModelRequirements};
pub use router::{
    CircuitBreakerConfig, ProviderRouter, RequestMiddleware, ResponseMiddleware, RetryConfig,
    RouterMetrics, TokenBucketConfig,
};
pub use types::{
    CompletionRequest, CompletionResponse, ContentBlock, ContentPart, Message, MessageContent,
    ResponseFormat, ResponseFormatMode, Role, StopReason, StreamChunk, ToolCall, ToolDefinition,
    ToolResultContent, Usage,
};
