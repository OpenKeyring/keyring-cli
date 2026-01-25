pub mod audit;
pub mod authorization;
pub mod executors;
pub mod server;
pub mod tools;

pub use audit::{AuditEvent, AuditLogger};
pub use authorization::{AuthManager, AuthToken};
pub use executors::ExecutionResult;
pub use server::{McpServer, ServerConfig};
pub use tools::{McpToolRegistry, ToolDefinition};

pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
pub const MAX_TOOL_EXECUTION_TIME: u64 = 30; // seconds
