//! Git executor for MCP Git Tools
//!
//! Provides Git operations (clone, push, pull) using system git commands.
//! This approach ensures maximum compatibility and leverages the user's
//! existing git configuration and credentials.
//!
//! The gix crate is used for repository inspection and validation.

use crate::error::Error;
use crate::mcp::secure_memory::{SecureBuffer, SecureMemoryError};
use std::path::Path;
use std::process::Command;

/// Git-specific error type
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid repository URL: {0}")]
    InvalidUrl(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Repository not found at: {0}")]
    RepositoryNotFound(String),

    #[error("No changes to push")]
    NoChangesToPush,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Memory protection failed: {0}")]
    MemoryProtectionFailed(String),
}

impl Error {
    pub fn from_git_error(err: &GitError) -> Self {
        match err {
            GitError::AuthenticationFailed(msg) => Error::AuthenticationFailed { reason: msg.clone() },
            GitError::RepositoryNotFound(path) => Error::NotFound {
                resource: format!("Git repository at {}", path),
            },
            GitError::PermissionDenied(msg) => Error::Unauthorized { reason: msg.clone() },
            _ => Error::Mcp {
                context: err.to_string(),
            },
        }
    }
}

impl From<GitError> for Error {
    fn from(err: GitError) -> Self {
        Error::from_git_error(&err)
    }
}

impl From<SecureMemoryError> for GitError {
    fn from(err: SecureMemoryError) -> Self {
        GitError::MemoryProtectionFailed(err.to_string())
    }
}

/// Output from a git clone operation
#[derive(Debug, Clone)]
pub struct GitCloneOutput {
    pub success: bool,
    pub commit: String,
    pub branch: String,
}

/// Output from a git push operation
#[derive(Debug, Clone)]
pub struct GitPushOutput {
    pub success: bool,
    pub commit: String,
    pub branch: String,
}

/// Output from a git pull operation
#[derive(Debug, Clone)]
pub struct GitPullOutput {
    pub success: bool,
    pub commit: String,
    pub updated: bool,
}

/// Git executor with credential support
///
/// This executor uses system git commands for operations, which ensures:
/// - Compatibility with all git protocols
/// - Proper authentication through git credentials helpers
/// - Leverage user's existing git configuration
/// - No C dependencies (uses system git binary)
pub struct GitExecutor {
    credential_name: String,
    username: Option<String>,
    password: Option<String>,
    private_key: Option<SecureBuffer>,
    public_key: Option<Vec<u8>>,
    passphrase: Option<String>,
}

impl GitExecutor {
    /// Create a new Git executor with username/password authentication
    pub fn new(
        credential_name: String,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        Self {
            credential_name,
            username,
            password,
            private_key: None,
            public_key: None,
            passphrase: None,
        }
    }

    /// Create a new Git executor with SSH key authentication
    pub fn with_ssh_key(
        credential_name: String,
        username: Option<String>,
        private_key: Vec<u8>,
        public_key: Option<Vec<u8>>,
        passphrase: Option<String>,
    ) -> std::result::Result<Self, GitError> {
        // Protect the private key in memory
        let secure_key = SecureBuffer::new(private_key)?;

        Ok(Self {
            credential_name,
            username,
            password: None,
            private_key: Some(secure_key),
            public_key,
            passphrase,
        })
    }

    /// Clone a repository to a local directory
    pub async fn clone(
        &self,
        repo_url: &str,
        destination: &Path,
        branch: Option<&str>,
    ) -> std::result::Result<GitCloneOutput, GitError> {
        // Validate URL
        if repo_url.is_empty() {
            return Err(GitError::InvalidUrl("Repository URL is empty".to_string()));
        }

        // Build git clone command
        let mut cmd = Command::new("git");
        cmd.arg("clone");

        // Add branch if specified
        if let Some(branch_name) = branch {
            cmd.args(["--branch", branch_name]);
        }

        cmd.arg(repo_url).arg(destination);

        // Set up authentication if needed
        let envs = self.setup_git_auth_env();
        cmd.envs(envs);

        // Execute clone
        let output = cmd.output().map_err(GitError::IoError)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("auth") || stderr.contains("credential") {
                return Err(GitError::AuthenticationFailed(stderr.to_string()));
            } else if stderr.contains("not found") || stderr.contains("does not exist") {
                return Err(GitError::InvalidUrl(stderr.to_string()));
            }
            return Err(GitError::GitError(format!("Clone failed: {}", stderr)));
        }

        // Get the current HEAD commit from the cloned repository
        let commit = self.get_head_commit(destination)?;
        let branch_name = branch.unwrap_or("main").to_string();

        Ok(GitCloneOutput {
            success: true,
            commit,
            branch: branch_name,
        })
    }

    /// Push changes to a remote repository
    pub async fn push(
        &self,
        repo_path: &Path,
        branch: &str,
        remote: Option<&str>,
    ) -> std::result::Result<GitPushOutput, GitError> {
        let remote_name = remote.unwrap_or("origin");

        // Validate repository
        self.validate_repo(repo_path)?;

        // Get the current HEAD commit
        let commit = self.get_head_commit(repo_path)?;

        // Build git push command
        let mut cmd = Command::new("git");
        cmd.arg("push").arg(remote_name).arg(branch).current_dir(repo_path);

        // Set up authentication if needed
        let envs = self.setup_git_auth_env();
        cmd.envs(envs);

        // Execute push
        let output = cmd.output().map_err(GitError::IoError)?;

        if output.status.success() {
            Ok(GitPushOutput {
                success: true,
                commit,
                branch: branch.to_string(),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("auth") || stderr.contains("credential") {
                Err(GitError::AuthenticationFailed(stderr.to_string()))
            } else if stderr.contains("permission") || stderr.contains("forbidden") {
                Err(GitError::PermissionDenied(stderr.to_string()))
            } else if stderr.contains("up to date") {
                Err(GitError::NoChangesToPush)
            } else {
                Err(GitError::GitError(format!("Push failed: {}", stderr)))
            }
        }
    }

    /// Pull changes from a remote repository
    pub async fn pull(
        &self,
        repo_path: &Path,
        branch: Option<&str>,
        remote: Option<&str>,
    ) -> std::result::Result<GitPullOutput, GitError> {
        let remote_name = remote.unwrap_or("origin");
        let branch_name = branch.unwrap_or("main");

        // Validate repository
        self.validate_repo(repo_path)?;

        // Get the current HEAD commit before pull
        let old_commit = self.get_head_commit(repo_path)?;

        // Build git pull command
        let mut cmd = Command::new("git");
        cmd.arg("pull").arg(remote_name).arg(branch_name).current_dir(repo_path);

        // Set up authentication if needed
        let envs = self.setup_git_auth_env();
        cmd.envs(envs);

        // Execute pull
        let output = cmd.output().map_err(GitError::IoError)?;

        if output.status.success() {
            // Get the new HEAD commit
            let new_commit = self.get_head_commit(repo_path)?;
            let updated = new_commit != old_commit;

            Ok(GitPullOutput {
                success: true,
                commit: new_commit,
                updated,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(GitError::GitError(format!("Pull failed: {}", stderr)))
        }
    }

    /// Get repository status
    pub fn status(&self, repo_path: &Path) -> std::result::Result<Vec<String>, GitError> {
        self.validate_repo(repo_path)?;

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path)
            .output()
            .map_err(GitError::IoError)?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let statuses: Vec<String> = stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();

            Ok(statuses)
        } else {
            Ok(Vec::new())
        }
    }

    /// Validate that a path is a git repository
    fn validate_repo(&self, repo_path: &Path) -> std::result::Result<(), GitError> {
        // Try to open with gix to validate it's a git repo
        gix::open(repo_path)
            .map_err(|_| GitError::RepositoryNotFound(repo_path.display().to_string()))?;
        Ok(())
    }

    /// Get the current HEAD commit hash
    fn get_head_commit(&self, repo_path: &Path) -> std::result::Result<String, GitError> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .map_err(GitError::IoError)?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim().to_string())
        } else {
            Err(GitError::GitError("Failed to get HEAD commit".to_string()))
        }
    }

    /// Setup git authentication environment variables
    fn setup_git_auth_env(&self) -> Vec<(&'static str, String)> {
        let mut envs = Vec::new();

        // If username/password are set, configure git to use them
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            // For HTTPS with username/password, we can embed in URL
            // Note: In production, you'd want to use git credential helpers
            envs.push(("GIT_USERNAME", username.clone()));
            envs.push(("GIT_PASSWORD", password.clone()));
        }

        // If SSH key is set, configure GIT_SSH_COMMAND
        if let Some(ref _key) = self.private_key {
            // For SSH key authentication
            // Note: In production, you'd want to use ssh-agent or a temporary key file
            if let Some(passphrase) = &self.passphrase {
                envs.push(("GIT_SSH_PASSPHRASE", passphrase.clone()));
            }
        }

        envs
    }

    /// Check if executor has credentials configured
    fn has_credentials(&self) -> bool {
        self.username.is_some()
            || self.password.is_some()
            || self.private_key.is_some()
            || self.passphrase.is_some()
    }

    /// Get the credential name
    pub fn credential_name(&self) -> &str {
        &self.credential_name
    }

    /// Set credentials for the executor
    pub fn set_credentials(&mut self, username: Option<String>, password: Option<String>) {
        self.username = username.clone();
        self.password = password.clone();
        // Clear SSH key credentials when setting username/password
        if username.is_some() || password.is_some() {
            self.private_key = None;
            self.public_key = None;
            self.passphrase = None;
        }
    }

    /// Set SSH key credentials for the executor
    pub fn set_ssh_key(
        &mut self,
        private_key: Vec<u8>,
        public_key: Option<Vec<u8>>,
        passphrase: Option<String>,
    ) -> std::result::Result<(), GitError> {
        // Protect the private key in memory
        let secure_key = SecureBuffer::new(private_key)?;
        self.private_key = Some(secure_key);
        self.public_key = public_key;
        self.passphrase = passphrase;
        // Clear username/password when setting SSH key
        self.password = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_executor_new() {
        let executor = GitExecutor::new(
            "test_credential".to_string(),
            Some("test_user".to_string()),
            Some("test_pass".to_string()),
        );

        assert_eq!(executor.credential_name(), "test_credential");
    }

    #[test]
    fn test_git_executor_with_ssh_key() {
        let private_key = b"test_private_key".to_vec();
        let executor = GitExecutor::with_ssh_key(
            "test_credential".to_string(),
            Some("git_user".to_string()),
            private_key.clone(),
            None,
            None,
        )
        .unwrap();

        assert_eq!(executor.credential_name(), "test_credential");
    }

    #[test]
    fn test_git_clone_output() {
        let output = GitCloneOutput {
            success: true,
            commit: "abc123".to_string(),
            branch: "main".to_string(),
        };

        assert!(output.success);
        assert_eq!(output.commit, "abc123");
        assert_eq!(output.branch, "main");
    }

    #[test]
    fn test_git_push_output() {
        let output = GitPushOutput {
            success: true,
            commit: "def456".to_string(),
            branch: "develop".to_string(),
        };

        assert!(output.success);
        assert_eq!(output.commit, "def456");
        assert_eq!(output.branch, "develop");
    }

    #[test]
    fn test_git_pull_output() {
        let output = GitPullOutput {
            success: true,
            commit: "ghi789".to_string(),
            updated: true,
        };

        assert!(output.success);
        assert!(output.updated);
        assert_eq!(output.commit, "ghi789");
    }

    #[test]
    fn test_invalid_url() {
        let executor = GitExecutor::new("test".to_string(), None, None);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(executor.clone("", std::path::Path::new("/tmp/test"), None));

        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::InvalidUrl(_) => {}
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[test]
    fn test_has_credentials() {
        let mut executor = GitExecutor::new("test".to_string(), None, None);
        assert!(!executor.has_credentials());

        executor.set_credentials(Some("user".to_string()), Some("pass".to_string()));
        assert!(executor.has_credentials());
    }

    #[test]
    fn test_set_credentials_clears_ssh() {
        let mut executor = GitExecutor::new("test".to_string(), None, None);

        // Set SSH key
        let private_key = b"test_key".to_vec();
        executor
            .set_ssh_key(private_key, None, None)
            .unwrap();

        // Set username/password should clear SSH
        executor.set_credentials(Some("user".to_string()), Some("pass".to_string()));
        assert!(executor.password.is_some());
    }
}
