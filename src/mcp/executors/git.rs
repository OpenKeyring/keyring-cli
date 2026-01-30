//! Git executor for MCP Git Tools
//!
//! Provides Git operations (clone, push, pull) using the git2 crate.

use crate::error::{Error, Result};
use git2::{
    Cred, ObjectType, Oid, PushOptions, RemoteCallbacks, Repository, ResetType,
    Signature,
};
use std::path::Path;

/// Git-specific error type
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

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
}

impl From<GitError> for Error {
    fn from(err: GitError) -> Self {
        match err {
            GitError::AuthenticationFailed(msg) => Error::AuthenticationFailed { reason: msg },
            GitError::RepositoryNotFound(path) => Error::NotFound {
                resource: format!("Git repository at {}", path),
            },
            GitError::PermissionDenied(msg) => Error::Unauthorized { reason: msg },
            _ => Error::Mcp {
                context: err.to_string(),
            },
        }
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
pub struct GitExecutor {
    credential_name: String,
    username: Option<String>,
    password: Option<String>,
    private_key: Option<Vec<u8>>,
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
    ) -> Self {
        Self {
            credential_name,
            username,
            password: None,
            private_key: Some(private_key),
            public_key,
            passphrase,
        }
    }

    /// Clone a repository to a local directory
    pub async fn clone(
        &self,
        repo_url: &str,
        destination: &Path,
        branch: Option<&str>,
    ) -> Result<GitCloneOutput, GitError> {
        // Validate URL
        if repo_url.is_empty() {
            return Err(GitError::InvalidUrl("Repository URL is empty".to_string()));
        }

        // Build clone options with credential callbacks
        let mut builder = git2::Repository::clone_opts(repo_url, destination, self.clone_opts()?)?;

        // Configure branch if specified
        if let Some(branch_name) = branch {
            builder.branch(branch_name);
        }

        // Perform the clone
        let repo = Repository::clone(repo_url, destination)?;

        // Get the current HEAD commit
        let head = repo.head()?;
        let commit_oid = head.target().ok_or_else(|| {
            GitError::GitError(git2::Error::from_str(
                "Failed to get HEAD commit OID",
            ))
        })?;
        let commit = repo.find_commit(commit_oid)?;

        // Get the branch name
        let branch_name = branch
            .map(|s| s.to_string())
            .or_else(|| {
                head.shorthand()
                    .and_then(|s| s.strip_prefix("refs/heads/"))
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "main".to_string());

        Ok(GitCloneOutput {
            success: true,
            commit: commit_oid.to_string(),
            branch: branch_name,
        })
    }

    /// Push changes to a remote repository
    pub async fn push(
        &self,
        repo_path: &Path,
        branch: &str,
        remote: Option<&str>,
    ) -> Result<GitPushOutput, GitError> {
        let repo = Repository::open(repo_path)
            .map_err(|_| GitError::RepositoryNotFound(repo_path.display().to_string()))?;

        let remote_name = remote.unwrap_or("origin");

        // Find the remote
        let mut remote_obj = repo
            .find_remote(remote_name)
            .map_err(|_| GitError::GitError(git2::Error::from_str(&format!(
                "Remote '{}' not found",
                remote_name
            ))))?;

        // Get the current HEAD commit
        let head = repo.head()?;
        let commit_oid = head.target().ok_or_else(|| {
            GitError::GitError(git2::Error::from_str("No HEAD commit"))
        })?;

        // Prepare push options with credentials
        let mut push_options = PushOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        let repo_clone = repo.clone();
        let username_clone = self.username.clone();
        let password_clone = self.password.clone();
        let private_key_clone = self.private_key.clone();
        let public_key_clone = self.public_key.clone();
        let passphrase_clone = self.passphrase.clone();

        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            // Try SSH key first if available
            if let Some(ref key) = private_key_clone {
                let username = username_clone
                    .as_deref()
                    .or_else(|| username_from_url)
                    .unwrap_or("git");

                let result = if let Some(ref passphrase) = passphrase_clone {
                    Cred::ssh_key_from_memory(username, None, key, passphrase)
                } else {
                    Cred::ssh_key_from_memory(username, None, key, None)
                };

                return result.map_err(|e| {
                    git2::Error::new(
                        git2::ErrorCode::Auth,
                        git2::ErrorClass::Authentication,
                        &format!("SSH key authentication failed: {}", e),
                    )
                });
            }

            // Fall back to username/password
            if let (Some(username), Some(password)) = (&username_clone, &password_clone) {
                return Cred::new(username, password).map_err(|e| {
                    git2::Error::new(
                        git2::ErrorCode::Auth,
                        git2::ErrorClass::Authentication,
                        &format!("Password authentication failed: {}", e),
                    )
                });
            }

            // Try default SSH agent
            if let Some(username) = username_clone.as_deref().or_else(|| username_from_url) {
                let result = Cred::ssh_key_from_agent(username);
                if result.is_ok() {
                    return result;
                }
            }

            Err(git2::Error::new(
                git2::ErrorCode::Auth,
                git2::ErrorClass::Authentication,
                "No authentication credentials available",
            ))
        });

        push_options.remote_callbacks(callbacks);

        // Prepare the refspec
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);

        // Push
        remote_obj
            .push(&[&refspec], Some(&mut push_options))
            .map_err(|e| {
                if e.code() == git2::ErrorCode::Auth {
                    GitError::AuthenticationFailed(e.message().to_string())
                } else if e.code() == git2::ErrorCode::Certificate {
                    GitError::PermissionDenied(e.message().to_string())
                } else {
                    GitError::GitError(e)
                }
            })?;

        Ok(GitPushOutput {
            success: true,
            commit: commit_oid.to_string(),
            branch: branch.to_string(),
        })
    }

    /// Pull changes from a remote repository
    pub async fn pull(
        &self,
        repo_path: &Path,
        branch: Option<&str>,
        remote: Option<&str>,
    ) -> Result<GitPullOutput, GitError> {
        let repo = Repository::open(repo_path)
            .map_err(|_| GitError::RepositoryNotFound(repo_path.display().to_string()))?;

        let remote_name = remote.unwrap_or("origin");

        // Find the remote
        let mut remote_obj = repo
            .find_remote(remote_name)
            .map_err(|_| GitError::GitError(git2::Error::from_str(&format!(
                "Remote '{}' not found",
                remote_name
            ))))?;

        // Fetch from remote
        let mut fetch_options = self.fetch_options()?;
        remote_obj.fetch(&[branch.unwrap_or("main")], Some(&mut fetch_options), None)?;

        // Get the branch name
        let branch_name = branch.unwrap_or("main");

        // Get the remote commit
        let remote_branch_name = format!("{}/{}", remote_name, branch_name);
        let remote_oid = repo
            .refname_to_id(&format!("refs/remotes/{}", remote_branch_name))
            .map_err(|_| GitError::BranchNotFound(remote_branch_name.clone()))?;

        // Get the current HEAD
        let head_oid = repo.head()?.target().ok_or_else(|| {
            GitError::GitError(git2::Error::from_str("No HEAD commit"))
        })?;

        // Check if there are updates
        let updated = remote_oid != head_oid;

        if updated {
            // Merge the remote branch
            let remote_commit = repo.find_commit(remote_oid)?;
            let head_commit = repo.find_commit(head_oid)?;

            // Get the annotated commit
            let remote_annotated = repo
                .find_annotated_commit(remote_oid)
                .map_err(|e| GitError::GitError(e))?;

            // Perform the merge
            let _merge_analysis = repo.merge_analysis(&[&remote_annotated])?.0;

            // Checkout the remote commit
            repo.checkout_tree(remote_commit.as_object(), None)?;
            repo.set_head(&format!("refs/heads/{}", branch_name))?;

            // Reset to the remote commit
            repo.reset(remote_commit.as_object(), ResetType::Hard, None)?;
        }

        Ok(GitPullOutput {
            success: true,
            commit: remote_oid.to_string(),
            updated,
        })
    }

    /// Get repository status
    pub fn status(&self, repo_path: &Path) -> Result<Vec<String>, GitError> {
        let repo = Repository::open(repo_path)
            .map_err(|_| GitError::RepositoryNotFound(repo_path.display().to_string()))?;

        let mut statuses = Vec::new();
        let repo_statuses = repo.statuses(None).map_err(GitError::GitError)?;

        for entry in repo_statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown").to_string();

            if status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_wt_new()
                || status.is_wt_modified()
                || status.is_wt_deleted()
            {
                statuses.push(path);
            }
        }

        Ok(statuses)
    }

    /// Build clone options
    fn clone_opts(&self) -> Result<git2::CloneOptions, GitError> {
        let mut opts = git2::CloneOptions::new();
        let fetch_opts = self.fetch_options()?;
        opts.fetch_options(fetch_opts);
        Ok(opts)
    }

    /// Build fetch options with authentication
    fn fetch_options(&self) -> Result<git2::FetchOptions, GitError> {
        let mut opts = git2::FetchOptions::new();

        let mut callbacks = RemoteCallbacks::new();
        let username_clone = self.username.clone();
        let password_clone = self.password.clone();
        let private_key_clone = self.private_key.clone();
        let passphrase_clone = self.passphrase.clone();

        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            // Try SSH key first if available
            if let Some(ref key) = private_key_clone {
                let username = username_clone
                    .as_deref()
                    .or_else(|| username_from_url)
                    .unwrap_or("git");

                let result = if let Some(ref passphrase) = passphrase_clone {
                    Cred::ssh_key_from_memory(username, None, key, passphrase)
                } else {
                    Cred::ssh_key_from_memory(username, None, key, None)
                };

                return result.map_err(|e| {
                    git2::Error::new(
                        git2::ErrorCode::Auth,
                        git2::ErrorClass::Authentication,
                        &format!("SSH key authentication failed: {}", e),
                    )
                });
            }

            // Fall back to username/password
            if let (Some(username), Some(password)) = (&username_clone, &password_clone) {
                return Cred::new(username, password).map_err(|e| {
                    git2::Error::new(
                        git2::ErrorCode::Auth,
                        git2::ErrorClass::Authentication,
                        &format!("Password authentication failed: {}", e),
                    )
                });
            }

            // Try default SSH agent
            if let Some(username) = username_clone.as_deref().or_else(|| username_from_url) {
                let result = Cred::ssh_key_from_agent(username);
                if result.is_ok() {
                    return result;
                }
            }

            Err(git2::Error::new(
                git2::ErrorCode::Auth,
                git2::ErrorClass::Authentication,
                "No authentication credentials available",
            ))
        });

        opts.remote_callbacks(callbacks);
        Ok(opts)
    }

    /// Get the credential name
    pub fn credential_name(&self) -> &str {
        &self.credential_name
    }

    /// Set credentials for the executor
    pub fn set_credentials(&mut self, username: Option<String>, password: Option<String>) {
        self.username = username;
        self.password = password;
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
    ) {
        self.private_key = Some(private_key);
        self.public_key = public_key;
        self.passphrase = passphrase;
        // Clear username/password when setting SSH key
        self.password = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
        );

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
    fn test_git_error_from() {
        let git_err = git2::Error::new(
            git2::ErrorCode::Auth,
            git2::ErrorClass::Authentication,
            "Test auth error",
        );
        let git_error = GitError::GitError(git_err);

        // Test conversion to Error
        let keyring_error: Error = git_error.into();
        match keyring_error {
            Error::AuthenticationFailed { .. } => {}
            _ => panic!("Expected AuthenticationFailed error"),
        }
    }

    #[test]
    fn test_invalid_url() {
        let executor = GitExecutor::new("test".to_string(), None, None);
        let temp_dir = TempDir::new().unwrap();

        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(executor.clone("", temp_dir.path(), None))
        })
        .join()
        .unwrap();

        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::InvalidUrl(_) => {}
            _ => panic!("Expected InvalidUrl error"),
        }
    }
}
