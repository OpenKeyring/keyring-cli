//! MCP Audit Logging Integration Tests
//!
//! Tests for the audit logging functionality
//!
//! # Important: Run tests sequentially
//!
//! These tests use environment variables to configure the log path and must
//! be run sequentially to avoid interference. Run with:
//!   cargo test --test mcp_audit_integration_test -- --test-threads=1

#[cfg(test)]
mod mcp_audit_integration_tests {
    use keyring_cli::mcp::audit::AuditLogger;
    use serial_test::serial;
    use std::env;

    /// Helper to set a unique log path for each test
    fn set_test_log_path(test_name: &str) -> String {
        let log_path = format!("/tmp/test_audit_{}.log", test_name);
        env::set_var("OK_MCP_AUDIT_LOG", &log_path);
        log_path
    }

    fn cleanup_test_log(log_path: &str) {
        let _ = std::fs::remove_file(log_path);
    }

    #[serial]    #[test]
    fn test_audit_logger_creation() {
        let log_path = set_test_log_path("creation");
        let _logger = AuditLogger::new();
        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_log_single_event() {
        let log_path = set_test_log_path("single");
        let logger = AuditLogger::new();

        logger.log_event("ssh_exec", "test operation").unwrap();

        let content = std::fs::read_to_string(&log_path).expect("Should read log file");
        assert!(!content.is_empty());
        assert!(content.contains("ssh_exec"));

        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_log_multiple_events() {
        let log_path = set_test_log_path("multiple");
        let logger = AuditLogger::new();

        for i in 0..3 {
            logger
                .log_event(&format!("event_{}", i), "test details")
                .expect("Should log event");
        }

        let content = std::fs::read_to_string(&log_path).expect("Should read log file");
        assert!(!content.is_empty());

        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_log_contains_event_type() {
        let log_path = set_test_log_path("event_type");
        let logger = AuditLogger::new();

        logger.log_event("api_get", "test details").unwrap();

        let content = std::fs::read_to_string(&log_path).expect("Should read log file");
        assert!(content.contains("api_get"));

        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_log_contains_success_status() {
        let log_path = set_test_log_path("success");
        let logger = AuditLogger::new();

        logger.log_event("test_event", "details").unwrap();

        let content = std::fs::read_to_string(&log_path).expect("Should read log file");
        assert!(content.contains("success="));

        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_tool_execution_logging() {
        let log_path = set_test_log_path("tool_exec");
        let logger = AuditLogger::new();

        logger
            .log_tool_execution(
                "ssh_exec",
                "test-client",
                &serde_json::json!({"command": "test"}),
                None,
                true,
            )
            .unwrap();

        let content = std::fs::read_to_string(&log_path).expect("Should read log file");
        assert!(content.contains("tool_execution"));

        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_auth_event_logging() {
        let log_path = set_test_log_path("auth_event");
        let logger = AuditLogger::new();

        logger
            .log_authentication_event("test-client", "login", true, None)
            .unwrap();

        let content = std::fs::read_to_string(&log_path).expect("Should read log file");
        assert!(content.contains("auth_login"));

        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_failed_operation_logging() {
        let log_path = set_test_log_path("failed");
        let logger = AuditLogger::new();

        logger
            .log_tool_execution("ssh_exec", "test-client", &serde_json::json!({}), None, false)
            .unwrap();

        let content = std::fs::read_to_string(&log_path).expect("Should read log file");
        assert!(content.contains("success=false"));

        cleanup_test_log(&log_path);
    }

    #[serial]    #[test]
    fn test_clear_logs() {
        let log_path = set_test_log_path("clear");
        let logger = AuditLogger::new();

        logger.log_event("test1", "details 1").unwrap();
        assert!(std::path::Path::new(&log_path).exists());

        logger.clear_logs().expect("Should clear logs");
        assert!(!std::path::Path::new(&log_path).exists());
    }

    #[serial]    #[test]
    fn test_disable_logging() {
        let log_path = set_test_log_path("disable");
        let mut logger = AuditLogger::new();
        logger.set_enabled(false);

        logger.log_event("test", "not logged").unwrap();
        assert!(!std::path::Path::new(&log_path).exists());

        cleanup_test_log(&log_path);
    }
}
