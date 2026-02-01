//! SSH Executor - Remote command execution via SSH
//!
//! Provides secure SSH command execution using system ssh command.
//! Private keys are never exposed to the AI and are zeroized after use.

use crate::mcp::secure_memory::{SecureBuffer, SecureMemoryError};
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use thiserror::Error;

/// SSH execution errors
#[derive(Debug, Error)]
pub enum SshError {
    #[error("SSH connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Command timed out after {0:?}")]
    Timeout(Duration),

    #[error("Key file error: {0}")]
    KeyFileError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("SSH session error: {0}")]
    SessionError(String),

    #[error("Memory protection failed: {0}")]
    MemoryProtectionFailed(String),
}

impl From<SecureMemoryError> for SshError {
    fn from(err: SecureMemoryError) -> Self {
        SshError::MemoryProtectionFailed(err.to_string())
    }
}

/// Output from SSH command execution
#[derive(Debug, Clone)]
pub struct SshExecOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

/// SSH executor for remote command execution
///
/// # Security
///
/// - Private keys are stored in protected memory (mlock on Unix, CryptProtectMemory on Windows)
/// - Keys are automatically zeroized and unprotected on drop
/// - Temporary key files are created with 0o600 permissions
/// - Keys are automatically cleaned up after execution
///
/// # Example
///
/// ```no_run
/// use keyring_cli::mcp::executors::ssh::SshExecutor;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let private_key = std::fs::read("/path/to/private/key")?;
///     let executor = SshExecutor::new(
///         private_key,
///         "example.com".to_string(),
///         "user".to_string(),
///         Some(22),
///     )?;
///
///     let output = executor.exec("ls -la")?;
///     println!("{}", output.stdout);
///
///     Ok(())
/// }
/// ```
pub struct SshExecutor {
    /// Private key bytes (protected in memory)
    private_key: Option<SecureBuffer>,

    /// SSH host
    host: String,

    /// SSH username
    username: String,

    /// SSH port (None = use SSH default)
    port: Option<u16>,
}

impl SshExecutor {
    /// Create a new SSH executor
    ///
    /// # Arguments
    ///
    /// * `private_key_bytes` - SSH private key in bytes
    /// * `host` - Target hostname or IP address
    /// * `username` - SSH username
    /// * `port` - SSH port (None for default 22)
    pub fn new(
        private_key_bytes: Vec<u8>,
        host: String,
        username: String,
        port: Option<u16>,
    ) -> Result<Self, SshError> {
        // Protect the private key in memory
        let secure_key = SecureBuffer::new(private_key_bytes)?;

        Ok(Self {
            private_key: Some(secure_key),
            host,
            username,
            port,
        })
    }

    /// Get the host
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Get the port
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Execute a command on the remote host
    ///
    /// # Arguments
    ///
    /// * `command` - Command string to execute
    ///
    /// # Returns
    ///
    /// `SshExecOutput` containing stdout, stderr, exit code, and duration
    pub fn exec(&self, command: &str) -> Result<SshExecOutput, SshError> {
        let start = std::time::Instant::now();

        // Get private key bytes from protected memory
        let secure_key = self
            .private_key
            .as_ref()
            .ok_or_else(|| SshError::KeyFileError("Private key not available".to_string()))?;

        // Write temporary key file
        let key_path = self.write_temp_key(secure_key.as_slice())?;

        // Build ssh command
        let mut cmd = Command::new("ssh");
        cmd.arg("-i").arg(&key_path);
        cmd.arg("-o").arg("StrictHostKeyChecking=no");
        cmd.arg("-o").arg("UserKnownHostsFile=/dev/null");

        if let Some(port) = self.port {
            cmd.arg("-p").arg(port.to_string());
        }

        cmd.arg(format!("{}@{}", self.username, self.host));
        cmd.arg(command);

        // Execute
        let output = cmd
            .output()
            .map_err(|e| SshError::ExecutionFailed(e.to_string()))?;

        // Clean up temp key file
        let _ = fs::remove_file(&key_path);

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(SshExecOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            duration_ms,
        })
    }

    /// Write private key to a temporary file with secure permissions
    ///
    /// # Security
    ///
    /// - File is created in $TEMP directory
    /// - Permissions are set to 0o600 (owner read/write only)
    /// - File path includes PID for uniqueness
    ///
    /// # Returns
    ///
    /// Path to the temporary key file
    fn write_temp_key(&self, key_bytes: &[u8]) -> Result<PathBuf, SshError> {
        // Get temp directory
        let temp_dir = env::temp_dir();

        // Create unique filename with PID
        let pid = std::process::id();
        let key_filename = format!(".ok-ssh-{}-test_key", pid);
        let key_path = temp_dir.join(&key_filename);

        // Create file with restrictive permissions
        // Note: .mode(0o600) is Unix-only; on Windows we skip it
        #[cfg(unix)]
        let mut file = fs::File::options()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&key_path)
            .map_err(|e| SshError::KeyFileError(format!("Failed to create temp file: {}", e)))?;

        #[cfg(windows)]
        let mut file = fs::File::options()
            .write(true)
            .create_new(true)
            .open(&key_path)
            .map_err(|e| SshError::KeyFileError(format!("Failed to create temp file: {}", e)))?;

        // Write key bytes
        file.write_all(key_bytes)
            .map_err(|e| SshError::KeyFileError(format!("Failed to write key: {}", e)))?;

        file.flush()
            .map_err(|e| SshError::KeyFileError(format!("Failed to flush key: {}", e)))?;

        Ok(key_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_error_display() {
        let err = SshError::ConnectionFailed("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn test_ssh_executor_creation() {
        let key = b"test_key".to_vec();
        let executor = SshExecutor::new(
            key,
            "example.com".to_string(),
            "user".to_string(),
            Some(2222),
        );

        assert!(executor.is_ok());
        let executor = executor.unwrap();
        assert_eq!(executor.host(), "example.com");
        assert_eq!(executor.username(), "user");
        assert_eq!(executor.port(), Some(2222));
    }
}
