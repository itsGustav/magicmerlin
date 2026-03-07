//! Tool execution runtime for MagicMerlin agents.

mod error;
mod process;
mod registry;
mod tools;

pub use error::{Result, ToolError};
pub use process::{ProcessManager, ProcessSummary};
pub use registry::{DeliveryContext, Tool, ToolContext, ToolRegistry, ToolResult};
pub use tools::register_default_tools;
