//! Tests for Git executor

use keyring_cli::mcp::executors::git::{
    GitCloneOutput, GitError, GitExecutor, GitPullOutput, GitPushOutput,
};
use tempfile::TempDir;
use std::path::PathBuf;
use std::fs;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test creating a new Git executor with username/password
    #[test]
    fn test_git_executor_new_with_credentials() {
        let executor = GitExecutor::new(
            "github".to_string(),
            Some("test_user".to_string()),
            Some("test_password".to_string()),
        );

        assert_eq!(executor.credential_name(), "github");
    }

    /// Test creating a new Git executor without credentials
    #[test]
    fn test_git_executor_new_without_credentials() {
        let executor = GitExecutor::new(
            "github".to_string(),
            None,
            None,
        );

        assert_eq!(executor.credential_name(), "github");
    }

    /// Test creating Git executor with SSH key
    #[test]
    fn test_git_executor_with_ssh_key() {
        let private_key = b"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA2X8dZkKhGkV2cOJ7uVLdHZ2xNnDu0I3KXKdK5hZp9m8f2w8
-----END RSA PRIVATE KEY-----".to_vec();

        let executor = GitExecutor::with_ssh_key(
            "github".to_string(),
            Some("git_user".to_string()),
            private_key,
            None,
            None,
        ).unwrap();

        assert_eq!(executor.credential_name(), "github");
    }

    /// Test Git executor with SSH key and passphrase
    #[test]
    fn test_git_executor_with_ssh_key_and_passphrase() {
        let private_key = b"test_key".to_vec();
        let passphrase = Some("test_passphrase".to_string());

        let executor = GitExecutor::with_ssh_key(
            "github".to_string(),
            Some("git_user".to_string()),
            private_key,
            None,
            passphrase,
        ).unwrap();

        assert_eq!(executor.credential_name(), "github");
    }

    /// Test setting credentials on existing executor
    #[test]
    fn test_set_credentials() {
        let mut executor = GitExecutor::new(
            "github".to_string(),
            None,
            None,
        );

        executor.set_credentials(
            Some("new_user".to_string()),
            Some("new_password".to_string()),
        );

        // Verify credentials are set (we can't directly access them,
        // but this demonstrates the API works)
        assert_eq!(executor.credential_name(), "github");
    }

    /// Test setting SSH key on existing executor
    #[test]
    fn test_set_ssh_key() {
        let mut executor = GitExecutor::new(
            "github".to_string(),
            None,
            None,
        );

        let private_key = b"test_key".to_vec();
        executor.set_ssh_key(private_key, None, None).unwrap();

        assert_eq!(executor.credential_name(), "github");
    }

    /// Test GitCloneOutput struct
    #[test]
    fn test_git_clone_output() {
        let output = GitCloneOutput {
            success: true,
            commit: "abc123def456".to_string(),
            branch: "main".to_string(),
        };

        assert!(output.success);
        assert_eq!(output.commit, "abc123def456");
        assert_eq!(output.branch, "main");
    }

    /// Test GitPushOutput struct
    #[test]
    fn test_git_push_output() {
        let output = GitPushOutput {
            success: true,
            commit: "def456ghi789".to_string(),
            branch: "develop".to_string(),
        };

        assert!(output.success);
        assert_eq!(output.commit, "def456ghi789");
        assert_eq!(output.branch, "develop");
    }

    /// Test GitPullOutput struct
    #[test]
    fn test_git_pull_output() {
        let output = GitPullOutput {
            success: true,
            commit: "ghi789jkl012".to_string(),
            updated: true,
        };

        assert!(output.success);
        assert!(output.updated);
        assert_eq!(output.commit, "ghi789jkl012");
    }

    /// Test GitError::InvalidUrl
    #[tokio::test]
    async fn test_git_error_invalid_url() {
        let executor = GitExecutor::new("test".to_string(), None, None);

        // This test verifies that empty URLs are rejected
        let temp_dir = TempDir::new().unwrap();
        let result = executor.clone("", temp_dir.path(), None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::InvalidUrl(msg) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    /// Test repository not found error
    #[test]
    fn test_repository_not_found() {
        let executor = GitExecutor::new("test".to_string(), None, None);
        let non_existent_path = PathBuf::from("/tmp/non_existent_repo_12345");

        let result = executor.status(&non_existent_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::RepositoryNotFound(_) => {}
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }

    /// Test error conversion from GitError to KeyringError
    #[test]
    fn test_git_error_conversion() {
        use keyring_cli::error::Error;

        let git_error = GitError::AuthenticationFailed("Test auth failed".to_string());
        let keyring_error: Error = git_error.into();

        match keyring_error {
            Error::AuthenticationFailed { .. } => {}
            _ => panic!("Expected AuthenticationFailed error"),
        }
    }

    /// Test error conversion for repository not found
    #[test]
    fn test_git_error_conversion_not_found() {
        use keyring_cli::error::Error;

        let git_error = GitError::RepositoryNotFound("/test/path".to_string());
        let keyring_error: Error = git_error.into();

        match keyring_error {
            Error::NotFound { .. } => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    /// Test error conversion for permission denied
    #[test]
    fn test_git_error_conversion_permission_denied() {
        use keyring_cli::error::Error;

        let git_error = GitError::PermissionDenied("Access denied".to_string());
        let keyring_error: Error = git_error.into();

        match keyring_error {
            Error::Unauthorized { .. } => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    /// Test cloning behavior with invalid URL formats
    #[tokio::test]
    async fn test_invalid_url_formats() {
        let executor = GitExecutor::new("test".to_string(), None, None);

        let invalid_urls = vec![
            "",
            "not-a-url",
            "ftp://invalid.com",
            "http://",
        ];

        for url in invalid_urls {
            let temp_dir = TempDir::new().unwrap();
            let result = executor.clone(url, temp_dir.path(), None).await;

            // We expect these to fail, though the specific error may vary
            assert!(result.is_err(), "Expected failure for URL: {}", url);
        }
    }

    /// Test Git executor credential switching
    #[test]
    fn test_credential_switching() {
        let mut executor = GitExecutor::new(
            "github".to_string(),
            Some("user1".to_string()),
            Some("pass1".to_string()),
        );

        // Switch to SSH key
        let private_key = b"ssh_key".to_vec();
        executor.set_ssh_key(private_key, None, None).unwrap();

        // Switch back to username/password
        executor.set_credentials(
            Some("user2".to_string()),
            Some("pass2".to_string()),
        );

        assert_eq!(executor.credential_name(), "github");
    }

    /// Test empty branch handling in clone
    #[tokio::test]
    async fn test_clone_with_none_branch() {
        let executor = GitExecutor::new("test".to_string(), None, None);

        // This will fail due to invalid URL, but tests the branch parameter
        let temp_dir = TempDir::new().unwrap();
        let result = executor.clone("https://github.com/test/repo.git", temp_dir.path(), None).await;

        // Should fail due to authentication/network, not due to branch handling
        assert!(result.is_err());
    }

    /// Test push error handling
    #[tokio::test]
    async fn test_push_with_non_existent_repo() {
        let executor = GitExecutor::new("test".to_string(), None, None);
        let non_existent_path = PathBuf::from("/tmp/non_existent_push_repo_12345");

        let result = executor.push(&non_existent_path, "main", None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::RepositoryNotFound(_) => {}
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }

    /// Test pull error handling
    #[tokio::test]
    async fn test_pull_with_non_existent_repo() {
        let executor = GitExecutor::new("test".to_string(), None, None);
        let non_existent_path = PathBuf::from("/tmp/non_existent_pull_repo_12345");

        let result = executor.pull(&non_existent_path, None, None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::RepositoryNotFound(_) => {}
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }

    /// Test status with non-existent repository
    #[test]
    fn test_status_non_existent() {
        let executor = GitExecutor::new("test".to_string(), None, None);
        let non_existent_path = PathBuf::from("/tmp/non_existent_status_repo_12345");

        let result = executor.status(&non_existent_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::RepositoryNotFound(_) => {}
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }

    /// Test GitError display implementation
    #[test]
    fn test_git_error_display() {
        let err = GitError::InvalidUrl("test://bad-url".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("Invalid URL"));
        assert!(display_str.contains("test://bad-url"));
    }

    /// Test GitError Debug implementation
    #[test]
    fn test_git_error_debug() {
        let err = GitError::AuthenticationFailed("bad credentials".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("AuthenticationFailed"));
    }

    /// Test creating local repository for status check
    #[test]
    fn test_local_repository_status() {
        use git2::Repository;

        let executor = GitExecutor::new("test".to_string(), None, None);
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");

        // Initialize a git repository
        let _repo = Repository::init(&repo_path).unwrap();

        // Check status (should be empty for new repo)
        let result = executor.status(&repo_path);

        assert!(result.is_ok());
        let statuses = result.unwrap();
        // New repo should have no changes
        assert!(statuses.is_empty() || statuses.len() == 0);
    }
}

/// Unit tests for error handling
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_git_error_from_git2_error() {
        use git2::ErrorCode;
        use git2::ErrorClass;

        let git2_err = git2::Error::new(
            ErrorCode::GenericError,
            ErrorClass::None,
            "Test git2 error"
        );

        let git_error = GitError::from(git2_err);
        assert!(matches!(git_error, GitError::GitError(_)));
    }

    #[test]
    fn test_git_error_from_io_error() {
        let io_err = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found"
        );

        let git_error = GitError::from(io_err);
        assert!(matches!(git_error, GitError::IoError(_)));
    }
}
