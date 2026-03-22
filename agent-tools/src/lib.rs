//! Tool execution runtime for MagicMerlin agents.

mod error;
pub mod gateway;
mod process;
mod registry;
mod tools;

pub use error::{Result, ToolError};
pub use gateway::{gateway_call, gateway_url};
pub use process::{ProcessManager, ProcessSummary};
pub use registry::{DeliveryContext, NodeConfig, Tool, ToolContext, ToolRegistry, ToolResult};
pub use tools::register_default_tools;
