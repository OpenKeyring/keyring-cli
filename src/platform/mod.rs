//! Platform detection and platform-specific functionality
//!
//! This module provides cross-platform abstractions for:
//! - Memory protection (mlock on Unix, CryptProtectMemory on Windows)
//! - SSH binary detection
//! - Platform-specific utilities

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::*;
    } else if #[cfg(target_os = "macos")] {
        mod macos;
        pub use macos::*;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::*;
    } else {
        compile_error!("Unsupported platform");
    }
}

use crate::error::Error;
use std::path::Path;
use std::process::Command;

/// Platform-specific error types
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Memory protection failed: {0}")]
    MemoryProtectionFailed(String),

    #[error("SSH binary not found")]
    SshNotFound,

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

impl From<PlatformError> for Error {
    fn from(err: PlatformError) -> Self {
        Error::Internal {
            context: err.to_string(),
        }
    }
}

/// Detect if SSH binary is available on the system
///
/// Returns the path to the SSH binary if found, None otherwise.
/// Checks common SSH installation paths based on the platform.
pub fn which_ssh() -> Option<String> {
    #[cfg(unix)]
    {
        // Common Unix SSH paths
        let paths = vec![
            "/usr/bin/ssh",
            "/usr/local/bin/ssh",
            "/bin/ssh",
            "/opt/homebrew/bin/ssh",          // macOS Apple Silicon
            "/usr/local/opt/openssh/bin/ssh", // macOS Intel Homebrew
        ];

        for path in paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }

        // Fall back to 'which' command
        if let Ok(output) = Command::new("which").arg("ssh").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim();
                    if !path.is_empty() {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows SSH paths (PowerShell, Git Bash, WSL, etc.)
        let paths = vec![
            "C:\\Windows\\System32\\OpenSSH\\ssh.exe",
            "C:\\Program Files\\Git\\usr\\bin\\ssh.exe",
            "C:\\Program Files\\OpenSSH\\bin\\ssh.exe",
        ];

        for path in paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }

        // Fall back to 'where' command
        if let Ok(output) = Command::new("where").arg("ssh").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim().lines().next().unwrap_or("");
                    if !path.is_empty() {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Check if SSH is available on the system
pub fn has_ssh() -> bool {
    which_ssh().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_detection() {
        // This test might be skipped in CI environments without SSH
        let ssh_path = which_ssh();
        if ssh_path.is_some() {
            assert!(Path::new(ssh_path.as_ref().unwrap()).exists());
        }
    }

    #[test]
    fn test_has_ssh() {
        // has_ssh should be consistent with which_ssh
        let ssh_path = which_ssh();
        assert_eq!(has_ssh(), ssh_path.is_some());
    }
}
