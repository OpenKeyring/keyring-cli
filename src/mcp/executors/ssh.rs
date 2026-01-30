//! SSH MCP Tool Definitions
//!
//! This module defines input/output structures for SSH-related MCP tools.
//! All structures implement JsonSchema for MCP protocol compliance.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default timeout value (30 seconds)
fn default_timeout() -> u64 {
    30
}

// ============================================================================
// Tool 1: ssh_exec (by tag - first/always confirm)
// ============================================================================

/// Input for ssh_exec tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshExecInput {
    /// Name of the SSH credential to use
    pub credential_name: String,
    /// Command to execute on the remote host
    pub command: String,
    /// Timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Confirmation ID for authorization flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,
    /// User decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output for ssh_exec tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshExecOutput {
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code of the command
    pub exit_code: i32,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 2: ssh_exec_interactive (by tag)
// ============================================================================

/// Input for ssh_exec_interactive tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshExecInteractiveInput {
    /// Name of the SSH credential to use
    pub credential_name: String,
    /// List of commands to execute sequentially
    pub commands: Vec<String>,
    /// Timeout in seconds per command (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Confirmation ID for authorization flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,
    /// User decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Result of a single command execution
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CommandResult {
    /// The command that was executed
    pub command: String,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code of the command
    pub exit_code: i32,
}

/// Output for ssh_exec_interactive tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshExecInteractiveOutput {
    /// Results for each command executed
    pub results: Vec<CommandResult>,
    /// Total execution duration in milliseconds
    pub total_duration_ms: u64,
}

// ============================================================================
// Tool 3: ssh_list_hosts (low risk - no confirmation)
// ============================================================================

/// Input for ssh_list_hosts tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshListHostsInput {
    /// Optional filter by tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_tags: Option<Vec<String>>,
}

/// Information about a single SSH host
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshHostInfo {
    /// Name identifier for the host
    pub name: String,
    /// Host address (hostname or IP)
    pub host: String,
    /// SSH username
    pub username: String,
    /// SSH port (default: 22)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Tags associated with this host
    pub tags: Vec<String>,
}

/// Output for ssh_list_hosts tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshListHostsOutput {
    /// List of SSH hosts
    pub hosts: Vec<SshHostInfo>,
}

// ============================================================================
// Tool 4: ssh_upload_file (by tag)
// ============================================================================

/// Input for ssh_upload_file tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshUploadFileInput {
    /// Name of the SSH credential to use
    pub credential_name: String,
    /// Local file path to upload
    pub local_path: String,
    /// Remote destination path
    pub remote_path: String,
    /// Confirmation ID for authorization flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,
    /// User decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output for ssh_upload_file tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshUploadFileOutput {
    /// Whether the upload succeeded
    pub success: bool,
    /// Number of bytes uploaded
    pub bytes_uploaded: u64,
    /// Upload duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 5: ssh_download_file (by tag)
// ============================================================================

/// Input for ssh_download_file tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshDownloadFileInput {
    /// Name of the SSH credential to use
    pub credential_name: String,
    /// Remote file path to download
    pub remote_path: String,
    /// Local destination path
    pub local_path: String,
    /// Confirmation ID for authorization flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,
    /// User decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output for ssh_download_file tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshDownloadFileOutput {
    /// Whether the download succeeded
    pub success: bool,
    /// Number of bytes downloaded
    pub bytes_downloaded: u64,
    /// Download duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 6: ssh_check_connection (low risk - no confirmation)
// ============================================================================

/// Input for ssh_check_connection tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshCheckConnectionInput {
    /// Name of the SSH credential to check
    pub credential_name: String,
}

/// Output for ssh_check_connection tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SshCheckConnectionOutput {
    /// Whether the connection succeeded
    pub connected: bool,
    /// Connection latency in milliseconds
    pub latency_ms: u64,
    /// Error message if connection failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_exec_input_serialization() {
        let input = SshExecInput {
            credential_name: "my-server".to_string(),
            command: "ls -la".to_string(),
            timeout: 30,
            confirmation_id: None,
            user_decision: None,
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("my-server"));
        assert!(json.contains("ls -la"));

        // Test deserialization
        let deserialized: SshExecInput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.credential_name, "my-server");
        assert_eq!(deserialized.command, "ls -la");
        assert_eq!(deserialized.timeout, 30);
    }

    #[test]
    fn test_ssh_exec_input_with_confirmation() {
        let input = SshExecInput {
            credential_name: "my-server".to_string(),
            command: "cat /etc/hosts".to_string(),
            timeout: 60,
            confirmation_id: Some("confirm-123".to_string()),
            user_decision: Some("approve".to_string()),
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("confirm-123"));
        assert!(json.contains("approve"));

        let deserialized: SshExecInput = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.confirmation_id,
            Some("confirm-123".to_string())
        );
        assert_eq!(deserialized.user_decision, Some("approve".to_string()));
    }

    #[test]
    fn test_ssh_exec_output_serialization() {
        let output = SshExecOutput {
            stdout: "file1.txt\nfile2.txt\n".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
            duration_ms: 245,
        };

        let json = serde_json::to_string(&output).unwrap();
        let deserialized: SshExecOutput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.stdout, "file1.txt\nfile2.txt\n");
        assert_eq!(deserialized.exit_code, 0);
        assert_eq!(deserialized.duration_ms, 245);
    }

    #[test]
    fn test_ssh_exec_interactive_serialization() {
        let input = SshExecInteractiveInput {
            credential_name: "db-server".to_string(),
            commands: vec![
                "cd /var/log".to_string(),
                "tail -100 syslog".to_string(),
                "exit".to_string(),
            ],
            timeout: 45,
            confirmation_id: None,
            user_decision: None,
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: SshExecInteractiveInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.commands.len(), 3);
        assert_eq!(deserialized.commands[0], "cd /var/log");
        assert_eq!(deserialized.timeout, 45);
    }

    #[test]
    fn test_command_result_serialization() {
        let result = CommandResult {
            command: "pwd".to_string(),
            stdout: "/home/user\n".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: CommandResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.command, "pwd");
        assert_eq!(deserialized.stdout, "/home/user\n");
    }

    #[test]
    fn test_ssh_list_hosts_input() {
        let input = SshListHostsInput {
            filter_tags: Some(vec!["production".to_string(), "web".to_string()]),
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: SshListHostsInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.filter_tags.unwrap().len(), 2);
    }

    #[test]
    fn test_ssh_host_info_serialization() {
        let host = SshHostInfo {
            name: "web-server-1".to_string(),
            host: "192.168.1.100".to_string(),
            username: "admin".to_string(),
            port: Some(2222),
            tags: vec!["production".to_string(), "web".to_string()],
        };

        let json = serde_json::to_string(&host).unwrap();
        let deserialized: SshHostInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "web-server-1");
        assert_eq!(deserialized.host, "192.168.1.100");
        assert_eq!(deserialized.port, Some(2222));
        assert_eq!(deserialized.tags.len(), 2);
    }

    #[test]
    fn test_ssh_upload_file_serialization() {
        let input = SshUploadFileInput {
            credential_name: "backup-server".to_string(),
            local_path: "/tmp/backup.tar.gz".to_string(),
            remote_path: "/backups/daily.tar.gz".to_string(),
            confirmation_id: None,
            user_decision: None,
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: SshUploadFileInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.local_path, "/tmp/backup.tar.gz");
        assert_eq!(deserialized.remote_path, "/backups/daily.tar.gz");
    }

    #[test]
    fn test_ssh_download_file_serialization() {
        let input = SshDownloadFileInput {
            credential_name: "log-server".to_string(),
            remote_path: "/var/log/app.log".to_string(),
            local_path: "./app.log".to_string(),
            confirmation_id: None,
            user_decision: None,
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: SshDownloadFileInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.remote_path, "/var/log/app.log");
        assert_eq!(deserialized.local_path, "./app.log");
    }

    #[test]
    fn test_ssh_check_connection_serialization() {
        let input = SshCheckConnectionInput {
            credential_name: "test-server".to_string(),
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: SshCheckConnectionInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.credential_name, "test-server");
    }

    #[test]
    fn test_ssh_check_connection_output() {
        let output = SshCheckConnectionOutput {
            connected: true,
            latency_ms: 42,
            error: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        let deserialized: SshCheckConnectionOutput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.connected, true);
        assert_eq!(deserialized.latency_ms, 42);
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_default_timeout() {
        let json = r#"{"credential_name":"test","command":"ls"}"#;
        let input: SshExecInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.timeout, 30);
    }

    #[test]
    fn test_json_schema_generation() {
        // Test that JsonSchema can be generated for all structs
        use schemars::schema_for;

        let _schema = schema_for!(SshExecInput);
        let _schema = schema_for!(SshExecOutput);
        let _schema = schema_for!(SshExecInteractiveInput);
        let _schema = schema_for!(CommandResult);
        let _schema = schema_for!(SshExecInteractiveOutput);
        let _schema = schema_for!(SshListHostsInput);
        let _schema = schema_for!(SshHostInfo);
        let _schema = schema_for!(SshListHostsOutput);
        let _schema = schema_for!(SshUploadFileInput);
        let _schema = schema_for!(SshUploadFileOutput);
        let _schema = schema_for!(SshDownloadFileInput);
        let _schema = schema_for!(SshDownloadFileOutput);
        let _schema = schema_for!(SshCheckConnectionInput);
        let _schema = schema_for!(SshCheckConnectionOutput);
    }
}
