//! Agent Control Protocol runtime for managing external coding-agent subprocesses.

mod config;
mod runtime;

pub use config::{AgentHarnessConfig, AgentId};
pub use runtime::{
    AcpEvent, AcpEventStream, AcpRuntime, AcpSession, AcpSessionRequest, AcpSessionStatus,
    AcpxRequest,
};
