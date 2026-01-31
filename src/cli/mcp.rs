//! CLI MCP Commands
//!
//! This module provides CLI commands for managing the MCP server,
//! including start, stop, status, and logs commands.

use crate::cli::ConfigManager;
use crate::error::{Error, Result};
use crate::mcp::audit::AuditLogger;
use crate::mcp::config::McpConfig;
use crate::mcp::lock::{is_locked, McpLock};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs;

/// MCP CLI commands
#[derive(Subcommand, Debug)]
pub enum MCPCommands {
    /// 启动 MCP 服务器（stdio 模式）
    Start {
        /// 详细输出
        #[arg(short, long)]
        verbose: bool,
    },

    /// 停止 MCP 服务器
    Stop,

    /// 查看服务状态
    Status,

    /// 查看审计日志
    Logs {
        /// 只显示今天的日志
        #[arg(long)]
        today: bool,

        /// 按工具过滤
        #[arg(long)]
        tool: Option<String>,

        /// 按状态过滤
        #[arg(long)]
        status: Option<String>,

        /// 按凭证过滤
        #[arg(long)]
        credential: Option<String>,

        /// 显示最近 N 条
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
}

/// Audit log entry for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub tool: String,
    pub credential: String,
    pub operation: String,
    pub authorization: String,
    pub status: String,
}

/// Query parameters for audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    pub today: bool,
    pub tool: Option<String>,
    pub status: Option<String>,
    pub credential: Option<String>,
    pub limit: usize,
}

/// Handle MCP CLI commands
pub async fn handle_mcp_command(cmd: MCPCommands) -> Result<()> {
    match cmd {
        MCPCommands::Start { verbose } => {
            handle_start_command(verbose).await
        }

        MCPCommands::Stop => {
            handle_stop_command()
        }

        MCPCommands::Status => {
            handle_status_command()
        }

        MCPCommands::Logs { today, tool, status, credential, limit } => {
            handle_logs_command(today, tool, status, credential, limit).await
        }
    }
}

/// Handle the MCP start command
async fn handle_start_command(verbose: bool) -> Result<()> {
    // Check if already running
    if is_locked() {
        return Err(Error::Mcp {
            context: "MCP server is already running".to_string(),
        });
    }

    // Prompt for master password
    let master_password = dialoguer::Password::new()
        .with_prompt("请输入主密码以解密 MCP 密钥缓存")
        .interact()
        .map_err(|e| Error::InvalidInput {
            context: format!("Password prompt failed: {}", e),
        })?;

    // Get database path from config
    let config_manager = ConfigManager::new()?;
    let db_config = config_manager.get_database_config()?;
    let db_path = std::path::PathBuf::from(db_config.path);

    // TODO: Initialize key cache (McpKeyCache doesn't exist yet, so we'll skip this for now)
    // The actual MCP server implementation will need to be completed separately

    // Load config
    let mcp_config = McpConfig::load_or_default(&McpConfig::config_path())?;

    if verbose {
        eprintln!("MCP server configuration loaded:");
        eprintln!("  Max concurrent requests: {}", mcp_config.max_concurrent_requests);
        eprintln!("  Max SSH response size: {} bytes", mcp_config.max_response_size_ssh);
        eprintln!("  Max API response size: {} bytes", mcp_config.max_response_size_api);
        eprintln!("  Session cache TTL: {} seconds", mcp_config.session_cache.ttl_seconds);
        eprintln!();
        eprintln!("Database path: {}", db_path.display());
    }

    // Acquire lock
    let _lock = McpLock::acquire()?;

    if verbose {
        eprintln!("MCP server lock acquired");
        eprintln!();
        eprintln!("MCP server starting on stdio...");
        eprintln!("Press Ctrl+C to stop the server");
    }

    // TODO: Start actual MCP server with rmcp
    // For now, we'll just run indefinitely until interrupted
    // This is a placeholder until the full MCP server implementation is complete

    // Simulate running the server
    eprintln!("MCP server running (PID: {})", std::process::id());
    eprintln!();
    eprintln!("Note: Full MCP server implementation is pending.");
    eprintln!("This is a placeholder that demonstrates the CLI structure.");

    // Wait for interrupt signal
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| Error::Mcp {
            context: format!("Failed to listen for shutdown signal: {}", e),
        })?;

    eprintln!();
    eprintln!("MCP server stopped");

    Ok(())
}

/// Handle the MCP stop command
fn handle_stop_command() -> Result<()> {
    if is_locked() {
        eprintln!("MCP 服务器正在运行");
        eprintln!("请按 Ctrl+C 停止服务器");
        eprintln!();
        eprintln!("或者在另一个终端运行:");
        eprintln!("  kill $(cat /tmp/open-keyring-mcp.lock)");
        Ok(())
    } else {
        eprintln!("MCP 服务器未运行");
        Ok(())
    }
}

/// Handle the MCP status command
fn handle_status_command() -> Result<()> {
    let config = McpConfig::load_or_default(&McpConfig::config_path())?;

    eprintln!("OpenKeyring MCP Server");
    eprintln!();

    if is_locked() {
        eprintln!("状态: 运行中");
        eprintln!("PID: {}", std::process::id());
    } else {
        eprintln!("状态: 未运行");
    }

    eprintln!();
    eprintln!("配置:");
    eprintln!("  最大并发请求: {}", config.max_concurrent_requests);
    eprintln!("  SSH 响应大小限制: {} MB", config.max_response_size_ssh / (1024 * 1024));
    eprintln!("  API 响应大小限制: {} MB", config.max_response_size_api / (1024 * 1024));
    eprintln!("  会话缓存 TTL: {} 秒 ({} 分钟)",
        config.session_cache.ttl_seconds,
        config.session_cache.ttl_seconds / 60
    );
    eprintln!("  会话缓存最大条目: {}", config.session_cache.max_entries);

    Ok(())
}

/// Handle the MCP logs command
async fn handle_logs_command(
    today: bool,
    tool: Option<String>,
    status: Option<String>,
    credential: Option<String>,
    limit: usize,
) -> Result<()> {
    let logger = AuditLogger::new();

    // Read and parse audit logs
    let entries = parse_audit_logs(&logger, today, tool, status, credential, limit)?;

    display_audit_logs(&entries);

    Ok(())
}

/// Parse audit logs from file
fn parse_audit_logs(
    logger: &AuditLogger,
    today: bool,
    tool_filter: Option<String>,
    status_filter: Option<String>,
    credential_filter: Option<String>,
    limit: usize,
) -> Result<Vec<AuditEntry>> {
    let log_path = std::env::var("OK_MCP_AUDIT_LOG")
        .unwrap_or_else(|_| "mcp_audit.log".to_string());

    // Check if log file exists
    if !std::path::Path::new(&log_path).exists() {
        eprintln!("审计日志文件不存在: {}", log_path);
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&log_path)
        .map_err(|e| Error::Io(e))?;

    let mut entries = Vec::new();

    for line in content.lines() {
        // Parse log line format: [timestamp] event_type | id | success=bool | client=X | details=...
        if let Some(entry) = parse_log_line(line) {
            // Apply filters
            if today {
                let entry_date = entry.timestamp.date_naive();
                let today = Utc::now().date_naive();
                if entry_date != today {
                    continue;
                }
            }

            if let Some(ref tool) = tool_filter {
                if !entry.tool.contains(tool) {
                    continue;
                }
            }

            if let Some(ref status) = status_filter {
                let entry_status = if entry.status == "success" {
                    "success"
                } else if entry.status == "failed" {
                    "failed"
                } else if entry.status == "denied" {
                    "denied"
                } else {
                    &entry.status
                };
                if entry_status != status {
                    continue;
                }
            }

            if let Some(ref cred) = credential_filter {
                if !entry.credential.contains(cred) {
                    continue;
                }
            }

            entries.push(entry);
        }
    }

    // Sort by timestamp (newest first) and limit
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(limit);

    Ok(entries)
}

/// Parse a single log line
fn parse_log_line(line: &str) -> Option<AuditEntry> {
    // Expected format: [2025-01-30 10:30:45 UTC] tool_execution | id | success=true | client=X | details=...
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Extract timestamp
    let timestamp_start = line.find('[')?;
    let timestamp_end = line.find(']')?;
    let timestamp_str = &line[timestamp_start + 1..timestamp_end];

    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
        .or_else(|_| DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S %Z"))
        .ok()?
        .with_timezone(&Utc);

    // Extract event type and details
    let rest = &line[timestamp_end + 1..];
    let parts: Vec<&str> = rest.split('|').collect();

    if parts.len() < 4 {
        return None;
    }

    let event_type = parts[0].trim().to_string();
    let _id = parts[1].trim().to_string();

    // Parse success status
    let success_part = parts[2].trim();
    let is_success = success_part.contains("true");

    // Parse details
    let details_part = parts.get(3).and_then(|p| p.strip_prefix("details=")).unwrap_or("{}");

    // Try to parse details as JSON
    let details: serde_json::Value = serde_json::from_str(details_part).unwrap_or_else(|_| serde_json::json!({}));

    // Extract fields from details or use defaults
    let tool = details.get("tool_name")
        .and_then(|v| v.as_str())
        .unwrap_or(&event_type)
        .to_string();

    let credential = details.get("credential")
        .and_then(|v| v.as_str())
        .unwrap_or("N/A")
        .to_string();

    let operation = details.get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("execute")
        .to_string();

    let authorization = details.get("authorization")
        .and_then(|v| v.as_str())
        .unwrap_or("N/A")
        .to_string();

    let status = if is_success {
        "success".to_string()
    } else {
        "failed".to_string()
    };

    Some(AuditEntry {
        timestamp,
        tool,
        credential,
        operation,
        authorization,
        status,
    })
}

/// Display audit logs in a formatted table
fn display_audit_logs(entries: &[AuditEntry]) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                           MCP 审计日志                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();

    if entries.is_empty() {
        println!("没有找到审计日志");
        println!();
        return;
    }

    for entry in entries {
        println!("┌────────────────────────────────────────────────────────────────────────────┐");
        println!("│ {} │", entry.timestamp.format("%Y-%m-%d %H:%M:%S"));
        println!("│ 工具: {}  │", entry.tool);
        println!("│ 凭证: {}  │", entry.credential);
        println!("│ 操作: {} │", entry.operation);
        println!("│ 授权: {}  │", entry.authorization);
        println!("│ 状态: {}  │", match entry.status.as_str() {
            "success" => "✓ 成功",
            "failed" => "✗ 失败",
            "denied" => "⊘ 拒绝",
            _ => &entry.status,
        });
        println!("└────────────────────────────────────────────────────────────────────────────┘");
    }

    println!();
    println!("共 {} 条记录", entries.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_commands_clap() {
        // Test that MCPCommands can be parsed by clap
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(subcommand)]
            mcp: MCPCommands,
        }

        // Test start command
        let cli = TestCli::parse_from(["test", "start", "--verbose"]);
        match cli.mcp {
            MCPCommands::Start { verbose } => {
                assert!(verbose);
            }
            _ => panic!("Expected Start command"),
        }

        // Test logs command
        let cli = TestCli::parse_from(["test", "logs", "--today", "--limit", "10"]);
        match cli.mcp {
            MCPCommands::Logs { today, tool, status, credential, limit } => {
                assert!(today);
                assert_eq!(limit, 10);
                assert!(tool.is_none());
                assert!(status.is_none());
                assert!(credential.is_none());
            }
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn test_audit_query_default() {
        let query = AuditQuery::default();
        assert!(!query.today);
        assert!(query.tool.is_none());
        assert!(query.status.is_none());
        assert!(query.credential.is_none());
        assert_eq!(query.limit, 0);
    }
}
