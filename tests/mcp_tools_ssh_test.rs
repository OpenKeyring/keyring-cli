//! Integration tests for SSH tool definitions
//!
//! These tests verify that all SSH tool input/output structures
//! properly serialize/deserialize and comply with MCP protocol requirements.

use keyring_cli::mcp::tools::ssh::{
    CommandResult, SshCheckConnectionInput, SshCheckConnectionOutput, SshDownloadFileInput,
    SshDownloadFileOutput, SshExecInput, SshExecInteractiveInput, SshExecInteractiveOutput,
    SshExecOutput, SshHostInfo, SshListHostsInput, SshListHostsOutput, SshUploadFileInput,
    SshUploadFileOutput,
};

#[test]
fn test_ssh_exec_input_full_serialization() {
    let input = SshExecInput {
        credential_name: "production-db".to_string(),
        command: "ps aux | grep postgres".to_string(),
        timeout: 60,
        confirmation_id: Some("conf-abc-123".to_string()),
        user_decision: Some("approve".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&input).unwrap();
    println!("SshExecInput JSON:\n{}", json);

    // Verify all fields are present
    assert!(json.contains("production-db"));
    assert!(json.contains("ps aux | grep postgres"));
    assert!(json.contains("60"));
    assert!(json.contains("conf-abc-123"));
    assert!(json.contains("approve"));

    // Deserialize back
    let deserialized: SshExecInput = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.credential_name, "production-db");
    assert_eq!(deserialized.command, "ps aux | grep postgres");
    assert_eq!(deserialized.timeout, 60);
    assert_eq!(
        deserialized.confirmation_id,
        Some("conf-abc-123".to_string())
    );
    assert_eq!(deserialized.user_decision, Some("approve".to_string()));
}

#[test]
fn test_ssh_exec_input_minimal() {
    let json = r#"{"credential_name":"test-host","command":"whoami"}"#;
    let input: SshExecInput = serde_json::from_str(json).unwrap();

    assert_eq!(input.credential_name, "test-host");
    assert_eq!(input.command, "whoami");
    assert_eq!(input.timeout, 30); // default
    assert!(input.confirmation_id.is_none());
    assert!(input.user_decision.is_none());
}

#[test]
fn test_ssh_exec_output_with_error() {
    let output = SshExecOutput {
        stdout: "".to_string(),
        stderr: "bash: command not found".to_string(),
        exit_code: 127,
        duration_ms: 123,
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshExecOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.exit_code, 127);
    assert!(deserialized.stderr.contains("command not found"));
    assert_eq!(deserialized.duration_ms, 123);
}

#[test]
fn test_ssh_exec_interactive_multiple_commands() {
    let input = SshExecInteractiveInput {
        credential_name: "api-server".to_string(),
        commands: vec![
            "cd /opt/app".to_string(),
            "git pull".to_string(),
            "systemctl restart app".to_string(),
            "systemctl status app".to_string(),
        ],
        timeout: 120,
        confirmation_id: None,
        user_decision: None,
    };

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: SshExecInteractiveInput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.commands.len(), 4);
    assert_eq!(deserialized.commands[1], "git pull");
    assert_eq!(deserialized.timeout, 120);
}

#[test]
fn test_ssh_exec_interactive_output() {
    let output = SshExecInteractiveOutput {
        results: vec![
            CommandResult {
                command: "cd /tmp".to_string(),
                stdout: "".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
            CommandResult {
                command: "ls".to_string(),
                stdout: "file1\nfile2\n".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
        ],
        total_duration_ms: 567,
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshExecInteractiveOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.results.len(), 2);
    assert_eq!(deserialized.results[0].command, "cd /tmp");
    assert_eq!(deserialized.results[1].stdout, "file1\nfile2\n");
    assert_eq!(deserialized.total_duration_ms, 567);
}

#[test]
fn test_ssh_list_hosts_with_tags() {
    let input = SshListHostsInput {
        filter_tags: Some(vec!["staging".to_string(), "database".to_string()]),
    };

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: SshListHostsInput = serde_json::from_str(&json).unwrap();

    let tags = deserialized.filter_tags.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"staging".to_string()));
    assert!(tags.contains(&"database".to_string()));
}

#[test]
fn test_ssh_list_hosts_no_filter() {
    let input = SshListHostsInput { filter_tags: None };

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: SshListHostsInput = serde_json::from_str(&json).unwrap();

    assert!(deserialized.filter_tags.is_none());
}

#[test]
fn test_ssh_host_info_complete() {
    let host = SshHostInfo {
        name: "redis-primary".to_string(),
        host: "redis.example.com".to_string(),
        username: "redis".to_string(),
        port: Some(6379),
        tags: vec![
            "production".to_string(),
            "cache".to_string(),
            "critical".to_string(),
        ],
    };

    let json = serde_json::to_string(&host).unwrap();
    println!("SshHostInfo JSON:\n{}", json);

    let deserialized: SshHostInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "redis-primary");
    assert_eq!(deserialized.host, "redis.example.com");
    assert_eq!(deserialized.port, Some(6379));
    assert_eq!(deserialized.tags.len(), 3);
}

#[test]
fn test_ssh_host_info_default_port() {
    let host = SshHostInfo {
        name: "simple-host".to_string(),
        host: "10.0.0.1".to_string(),
        username: "user".to_string(),
        port: None, // Default SSH port
        tags: vec![],
    };

    let json = serde_json::to_string(&host).unwrap();
    let deserialized: SshHostInfo = serde_json::from_str(&json).unwrap();

    assert!(deserialized.port.is_none());
}

#[test]
fn test_ssh_list_hosts_output() {
    let output = SshListHostsOutput {
        hosts: vec![
            SshHostInfo {
                name: "host1".to_string(),
                host: "192.168.1.1".to_string(),
                username: "admin".to_string(),
                port: Some(22),
                tags: vec!["web".to_string()],
            },
            SshHostInfo {
                name: "host2".to_string(),
                host: "192.168.1.2".to_string(),
                username: "admin".to_string(),
                port: Some(2222),
                tags: vec!["db".to_string()],
            },
        ],
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshListHostsOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.hosts.len(), 2);
    assert_eq!(deserialized.hosts[0].name, "host1");
    assert_eq!(deserialized.hosts[1].name, "host2");
}

#[test]
fn test_ssh_upload_file_with_approval() {
    let input = SshUploadFileInput {
        credential_name: "deploy-server".to_string(),
        local_path: "./build/app.jar".to_string(),
        remote_path: "/opt/app/app.jar".to_string(),
        confirmation_id: Some("upload-confirm-456".to_string()),
        user_decision: Some("approve".to_string()),
    };

    let json = serde_json::to_string(&input).unwrap();
    println!("SshUploadFileInput JSON:\n{}", json);

    let deserialized: SshUploadFileInput = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.credential_name, "deploy-server");
    assert_eq!(deserialized.local_path, "./build/app.jar");
    assert_eq!(
        deserialized.confirmation_id,
        Some("upload-confirm-456".to_string())
    );
}

#[test]
fn test_ssh_upload_file_output() {
    let output = SshUploadFileOutput {
        success: true,
        bytes_uploaded: 1024 * 1024 * 50, // 50 MB
        duration_ms: 5000,
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshUploadFileOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.success, true);
    assert_eq!(deserialized.bytes_uploaded, 52_428_800);
    assert_eq!(deserialized.duration_ms, 5000);
}

#[test]
fn test_ssh_upload_file_failed() {
    let output = SshUploadFileOutput {
        success: false,
        bytes_uploaded: 0,
        duration_ms: 100,
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshUploadFileOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.success, false);
    assert_eq!(deserialized.bytes_uploaded, 0);
}

#[test]
fn test_ssh_download_file_input() {
    let input = SshDownloadFileInput {
        credential_name: "log-archive".to_string(),
        remote_path: "/var/log/nginx/access.log.1".to_string(),
        local_path: "./logs/access.log.1".to_string(),
        confirmation_id: None,
        user_decision: None,
    };

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: SshDownloadFileInput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.remote_path, "/var/log/nginx/access.log.1");
    assert_eq!(deserialized.local_path, "./logs/access.log.1");
}

#[test]
fn test_ssh_download_file_output() {
    let output = SshDownloadFileOutput {
        success: true,
        bytes_downloaded: 1024 * 1024 * 250, // 250 MB
        duration_ms: 15000,
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshDownloadFileOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.success, true);
    assert_eq!(deserialized.bytes_downloaded, 262_144_000);
    assert_eq!(deserialized.duration_ms, 15000);
}

#[test]
fn test_ssh_check_connection_input() {
    let input = SshCheckConnectionInput {
        credential_name: "test-connection".to_string(),
    };

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: SshCheckConnectionInput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.credential_name, "test-connection");
}

#[test]
fn test_ssh_check_connection_success() {
    let output = SshCheckConnectionOutput {
        connected: true,
        latency_ms: 23,
        error: None,
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshCheckConnectionOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.connected, true);
    assert_eq!(deserialized.latency_ms, 23);
    assert!(deserialized.error.is_none());
}

#[test]
fn test_ssh_check_connection_failure() {
    let output = SshCheckConnectionOutput {
        connected: false,
        latency_ms: 0,
        error: Some("Connection timed out".to_string()),
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: SshCheckConnectionOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.connected, false);
    assert_eq!(deserialized.latency_ms, 0);
    assert_eq!(deserialized.error, Some("Connection timed out".to_string()));
}

#[test]
fn test_all_tool_inputs_optional_fields_serialization() {
    // Test that optional confirmation fields are properly omitted when None

    let input = SshExecInput {
        credential_name: "test".to_string(),
        command: "test".to_string(),
        timeout: 30,
        confirmation_id: None,
        user_decision: None,
    };

    let json = serde_json::to_string(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // These fields should not be present when None
    assert!(parsed.get("confirmation_id").is_none());
    assert!(parsed.get("user_decision").is_none());
}

#[test]
fn test_mcp_protocol_compliance() {
    // Verify all structures can be serialized to valid JSON
    // This is a requirement for MCP protocol compliance

    // Test each input type separately
    let input1 = SshExecInput {
        credential_name: "test".to_string(),
        command: "test".to_string(),
        timeout: 30,
        confirmation_id: None,
        user_decision: None,
    };
    let json = serde_json::to_string(&input1).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();

    let input2 = SshExecInteractiveInput {
        credential_name: "test".to_string(),
        commands: vec!["ls".to_string()],
        timeout: 30,
        confirmation_id: None,
        user_decision: None,
    };
    let json = serde_json::to_string(&input2).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();

    let input3 = SshListHostsInput { filter_tags: None };
    let json = serde_json::to_string(&input3).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();

    let input4 = SshUploadFileInput {
        credential_name: "test".to_string(),
        local_path: "/tmp/file".to_string(),
        remote_path: "/remote/file".to_string(),
        confirmation_id: None,
        user_decision: None,
    };
    let json = serde_json::to_string(&input4).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();

    let input5 = SshDownloadFileInput {
        credential_name: "test".to_string(),
        remote_path: "/remote/file".to_string(),
        local_path: "/local/file".to_string(),
        confirmation_id: None,
        user_decision: None,
    };
    let json = serde_json::to_string(&input5).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();

    let input6 = SshCheckConnectionInput {
        credential_name: "test".to_string(),
    };
    let json = serde_json::to_string(&input6).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();
}
