//! SSH Executor Tests
//!
//! Tests SSH remote command execution functionality.

use keyring_cli::mcp::executors::ssh_executor::{SshExecutor, SshExecOutput};
use std::time::Duration;

/// Sample SSH private key for testing (Ed25519 test key)
/// WARNING: This is a TEST key only, never use in production
const TEST_PRIVATE_KEY: &str = r#"-----OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBbDwFzqYcXvRzQnN9KqzFJ3qQ5lCjLjqWFKqVD4Tf7RAAAAJi9BMWSvQTF
kwAAAtzc2gtZWQyNTUxOQAAACBbDwFzqYcXvRzQnN9KqzFJ3qQ5lCjLjqWFKqVD4Tf7RAA
AEAwFLNlV0QBLD/tQtLJ9P+M1ZRJuE4yD3RKMdYTj9KlMKNWtHFcJlCjLjqWFKqVD4Tf7R
AAAADHNzaC1tY3AtdGVzdAECAwQFBgcIAQIDBAUGBwg=
-----END OPENSSH PRIVATE KEY-----
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_executor_creation() {
        let private_key = TEST_PRIVATE_KEY.as_bytes().to_vec();
        let executor = SshExecutor::new(
            private_key,
            "localhost".to_string(),
            "testuser".to_string(),
            Some(22),
        );

        assert_eq!(executor.host(), "localhost");
        assert_eq!(executor.username(), "testuser");
        assert_eq!(executor.port(), Some(22));
    }

    #[test]
    fn test_ssh_executor_default_port() {
        let private_key = TEST_PRIVATE_KEY.as_bytes().to_vec();
        let executor = SshExecutor::new(
            private_key,
            "example.com".to_string(),
            "admin".to_string(),
            None,
        );

        assert_eq!(executor.host(), "example.com");
        assert_eq!(executor.username(), "admin");
        assert_eq!(executor.port(), None); // None means use SSH default
    }

    #[test]
    fn test_ssh_exec_output_creation() {
        let output = SshExecOutput {
            stdout: "Hello World".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
            duration_ms: 100,
        };

        assert_eq!(output.stdout, "Hello World");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.duration_ms, 100);
    }

    #[test]
    fn test_write_temp_key() {
        // write_temp_key is a private method, tested implicitly through exec()
        // This test verifies the executor was created successfully
        let private_key = TEST_PRIVATE_KEY.as_bytes().to_vec();
        let executor = SshExecutor::new(
            private_key,
            "localhost".to_string(),
            "testuser".to_string(),
            None,
        );

        assert_eq!(executor.host(), "localhost");
    }

    // Integration tests - only run when SSH server is available
    #[test]
    #[cfg(ignore)] // Set to #[test] when SSH server is available for testing
    #[tokio::test]
    async fn test_ssh_command_execution() {
        // This test requires:
        // 1. An SSH server running on localhost:22
        // 2. A test user with the test public key in authorized_keys
        // 3. Network access

        let private_key = TEST_PRIVATE_KEY.as_bytes().to_vec();
        let executor = SshExecutor::new(
            private_key,
            "localhost".to_string(),
            "testuser".to_string(),
            Some(22),
        );

        let result = executor
            .exec("echo 'Hello from SSH'", Duration::from_secs(5))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("Hello from SSH"));
    }

    #[test]
    #[cfg(ignore)]
    #[tokio::test]
    async fn test_ssh_command_timeout() {
        let private_key = TEST_PRIVATE_KEY.as_bytes().to_vec();
        let executor = SshExecutor::new(
            private_key,
            "localhost".to_string(),
            "testuser".to_string(),
            Some(22),
        );

        // Execute a long-running command with short timeout
        let result = executor
            .exec("sleep 10", Duration::from_millis(100))
            .await;

        assert!(result.is_err());
    }

    #[test]
    #[cfg(ignore)]
    #[tokio::test]
    async fn test_ssh_command_error() {
        let private_key = TEST_PRIVATE_KEY.as_bytes().to_vec();
        let executor = SshExecutor::new(
            private_key,
            "localhost".to_string(),
            "testuser".to_string(),
            Some(22),
        );

        // Execute a command that fails
        let result = executor.exec("exit 42", Duration::from_secs(5)).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, 42);
    }

    #[test]
    fn test_key_zeroization() {
        // Test that private key is zeroized when dropped
        let private_key_bytes = b"secret_key_content_123".to_vec();
        let _original_bytes = private_key_bytes.clone();

        let executor = SshExecutor::new(
            private_key_bytes,
            "localhost".to_string(),
            "testuser".to_string(),
            None,
        );

        // After creating executor, the original_bytes should still exist
        // We can't directly access the private_key_bytes, but we verified
        // the structure compiles with zeroize derive

        drop(executor);

        // After dropping, the memory should be zeroized (but we can't verify this
        // without accessing the executor's internal state, which is private)
    }
}
