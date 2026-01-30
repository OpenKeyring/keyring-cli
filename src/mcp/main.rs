//! OpenKeyring MCP Server - Standalone Binary
//!
//! This is the main entry point for the standalone MCP server binary (ok-mcp-server).
//! It communicates via stdio transport following the Model Context Protocol (MCP).

use keyring_cli::mcp::config::McpConfig;
use keyring_cli::mcp::server::McpServer;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    // Load MCP configuration
    let config_path = McpConfig::config_path();
    let config = McpConfig::load_or_default(&config_path)
        .map_err(|e| format!("Failed to load MCP config: {}", e))?;

    // Create database and key cache placeholders
    // TODO: Initialize actual database connection
    let db = Arc::new(RwLock::new(()));
    let key_cache = Arc::new(RwLock::new(()));

    // Create the MCP server
    let server = McpServer::new(db, key_cache, config);

    eprintln!(
        "OpenKeyring MCP Server starting (session: {})",
        server.session_id()
    );
    eprintln!("Communicating via stdio transport...");

    // Run the server with stdio transport
    server.run_stdio().await?;

    Ok(())
}
