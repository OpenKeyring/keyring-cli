//! Git tool definitions for MCP server.
//!
//! This module defines input/output structures for Git-related MCP tools:
//! - git_clone: Clone a repository (low risk, no confirmation)
//! - git_pull: Pull changes from remote (low risk, no confirmation)
//! - git_push: Push changes to remote (requires confirmation)
//! - git_list_credentials: List stored Git credentials (low risk, no confirmation)
//! - git_get_current_head: Get current branch and commit (low risk, no confirmation)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input for git_clone tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitCloneInput {
    /// URL of the Git repository to clone
    pub repo_url: String,
    /// Optional destination directory path
    pub destination: Option<String>,
    /// Optional branch or tag to clone
    pub branch: Option<String>,
}

/// Output for git_clone tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitCloneOutput {
    /// Whether the clone operation succeeded
    pub success: bool,
    /// The commit hash that was checked out
    pub commit: String,
}

/// Input for git_pull tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitPullInput {
    /// URL of the Git repository to pull from
    pub repo_url: String,
    /// Optional branch to pull
    pub branch: Option<String>,
    /// Optional repository path (defaults to current directory)
    pub destination: Option<String>,
}

/// Output for git_pull tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitPullOutput {
    /// Whether the pull operation succeeded
    pub success: bool,
    /// The commit hash after pulling
    pub commit: String,
    /// Number of files changed in the pull
    pub files_changed: usize,
}

/// Input for git_push tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitPushInput {
    /// Name of the stored credential to use for authentication
    pub credential_name: String,
    /// URL of the Git repository to push to
    pub repo_url: String,
    /// Optional branch to push
    pub branch: Option<String>,
    /// Optional repository path (defaults to current directory)
    pub destination: Option<String>,
    /// Optional confirmation token ID (required for authorization)
    pub confirmation_id: Option<String>,
    /// User's decision (approve/deny)
    pub user_decision: Option<String>,
}

/// Output for git_push tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitPushOutput {
    /// Whether the push operation succeeded
    pub success: bool,
    /// The commit hash that was pushed
    pub commit: String,
}

/// Input for git_list_credentials tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitListCredentialsInput {
    /// Optional filter by tags (e.g., ["production", "github"])
    pub filter_tags: Option<Vec<String>>,
}

/// Information about a stored Git credential
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitCredentialInfo {
    /// Name/identifier of the credential
    pub name: String,
    /// Repository URL this credential is for
    pub repo_url: String,
    /// Tags associated with this credential
    pub tags: Vec<String>,
}

/// Output for git_list_credentials tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitListCredentialsOutput {
    /// List of stored Git credentials
    pub credentials: Vec<GitCredentialInfo>,
}

/// Input for git_get_current_head tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitGetCurrentHeadInput {
    /// Path to the Git repository
    pub destination: String,
}

/// Output for git_get_current_head tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitGetCurrentHeadOutput {
    /// Current branch name
    pub branch: String,
    /// Current commit hash
    pub commit: String,
    /// Commit message
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_value;

    #[test]
    fn test_git_clone_input_serialization() {
        let input = GitCloneInput {
            repo_url: "https://github.com/user/repo".to_string(),
            destination: Some("/tmp/repo".to_string()),
            branch: Some("main".to_string()),
        };

        let json = to_value(&input).expect("Failed to serialize");
        assert_eq!(json["repo_url"], "https://github.com/user/repo");
        assert_eq!(json["destination"], "/tmp/repo");
        assert_eq!(json["branch"], "main");
    }

    #[test]
    fn test_git_push_input_serialization() {
        let input = GitPushInput {
            credential_name: "my-credential".to_string(),
            repo_url: "https://github.com/user/repo".to_string(),
            branch: Some("main".to_string()),
            destination: Some("/tmp/repo".to_string()),
            confirmation_id: Some("confirm-123".to_string()),
            user_decision: Some("approve".to_string()),
        };

        let json = to_value(&input).expect("Failed to serialize");
        assert_eq!(json["credential_name"], "my-credential");
        assert_eq!(json["confirmation_id"], "confirm-123");
    }
}
