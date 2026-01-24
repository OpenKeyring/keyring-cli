pub mod server;
pub mod authorization;
pub mod tools;
pub mod executors;
pub mod audit;

pub use server::{McpServer, ServerConfig};
pub use authorization::{AuthManager, AuthToken};
pub use tools::{McpToolRegistry, ToolDefinition};
pub use executors::ExecutionResult;
pub use audit::{AuditLogger, AuditEvent};

pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
pub const MAX_TOOL_EXECUTION_TIME: u64 = 30; // seconds