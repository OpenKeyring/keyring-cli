use keyring_cli::mcp::tools::git::{
    GitCloneInput, GitCloneOutput, GitCredentialInfo, GitGetCurrentHeadInput,
    GitGetCurrentHeadOutput, GitListCredentialsInput, GitListCredentialsOutput, GitPullInput,
    GitPullOutput, GitPushInput, GitPushOutput,
};
use serde_json::{from_value, to_value};

#[test]
fn test_git_clone_input_serialization() {
    let input = GitCloneInput {
        repo_url: "https://github.com/user/repo".to_string(),
        destination: Some("/tmp/repo".to_string()),
        branch: Some("main".to_string()),
    };

    // Test JSON serialization
    let json = to_value(&input).expect("Failed to serialize GitCloneInput");
    assert_eq!(json["repo_url"], "https://github.com/user/repo");
    assert_eq!(json["destination"], "/tmp/repo");
    assert_eq!(json["branch"], "main");
}

#[test]
fn test_git_clone_input_minimal() {
    let input = GitCloneInput {
        repo_url: "https://github.com/user/repo".to_string(),
        destination: None,
        branch: None,
    };

    let json = to_value(&input).expect("Failed to serialize GitCloneInput");
    assert_eq!(json["repo_url"], "https://github.com/user/repo");
    assert!(json.get("destination").is_none() || json["destination"].is_null());
    assert!(json.get("branch").is_none() || json["branch"].is_null());
}

#[test]
fn test_git_clone_output_serialization() {
    let output = GitCloneOutput {
        success: true,
        commit: "abc123def456".to_string(),
    };

    let json = to_value(&output).expect("Failed to serialize GitCloneOutput");
    assert_eq!(json["success"], true);
    assert_eq!(json["commit"], "abc123def456");
}

#[test]
fn test_git_pull_input_serialization() {
    let input = GitPullInput {
        repo_url: "https://github.com/user/repo".to_string(),
        branch: Some("develop".to_string()),
        destination: Some("/tmp/repo".to_string()),
    };

    let json = to_value(&input).expect("Failed to serialize GitPullInput");
    assert_eq!(json["repo_url"], "https://github.com/user/repo");
    assert_eq!(json["branch"], "develop");
    assert_eq!(json["destination"], "/tmp/repo");
}

#[test]
fn test_git_pull_output_serialization() {
    let output = GitPullOutput {
        success: true,
        commit: "def456ghi789".to_string(),
        files_changed: 5,
    };

    let json = to_value(&output).expect("Failed to serialize GitPullOutput");
    assert_eq!(json["success"], true);
    assert_eq!(json["commit"], "def456ghi789");
    assert_eq!(json["files_changed"], 5);
}

#[test]
fn test_git_push_input_serialization() {
    let input = GitPushInput {
        credential_name: "my-git-credential".to_string(),
        repo_url: "https://github.com/user/repo".to_string(),
        branch: Some("feature".to_string()),
        destination: Some("/tmp/repo".to_string()),
        confirmation_id: Some("confirm-123".to_string()),
        user_decision: Some("approve".to_string()),
    };

    let json = to_value(&input).expect("Failed to serialize GitPushInput");
    assert_eq!(json["credential_name"], "my-git-credential");
    assert_eq!(json["repo_url"], "https://github.com/user/repo");
    assert_eq!(json["branch"], "feature");
    assert_eq!(json["destination"], "/tmp/repo");
    assert_eq!(json["confirmation_id"], "confirm-123");
    assert_eq!(json["user_decision"], "approve");
}

#[test]
fn test_git_push_input_minimal() {
    let input = GitPushInput {
        credential_name: "my-git-credential".to_string(),
        repo_url: "https://github.com/user/repo".to_string(),
        branch: None,
        destination: None,
        confirmation_id: None,
        user_decision: None,
    };

    let json = to_value(&input).expect("Failed to serialize GitPushInput");
    assert_eq!(json["credential_name"], "my-git-credential");
    assert_eq!(json["repo_url"], "https://github.com/user/repo");
}

#[test]
fn test_git_push_output_serialization() {
    let output = GitPushOutput {
        success: true,
        commit: "ghi789jkl012".to_string(),
    };

    let json = to_value(&output).expect("Failed to serialize GitPushOutput");
    assert_eq!(json["success"], true);
    assert_eq!(json["commit"], "ghi789jkl012");
}

#[test]
fn test_git_list_credentials_input_serialization() {
    let input = GitListCredentialsInput {
        filter_tags: Some(vec!["production".to_string(), "github".to_string()]),
    };

    let json = to_value(&input).expect("Failed to serialize GitListCredentialsInput");
    assert!(json["filter_tags"].is_array());
    assert_eq!(json["filter_tags"].as_array().unwrap().len(), 2);
}

#[test]
fn test_git_list_credentials_input_empty() {
    let input = GitListCredentialsInput { filter_tags: None };

    let json = to_value(&input).expect("Failed to serialize GitListCredentialsInput");
    assert!(json.get("filter_tags").is_none() || json["filter_tags"].is_null());
}

#[test]
fn test_git_credential_info_serialization() {
    let credential = GitCredentialInfo {
        name: "my-git-cred".to_string(),
        repo_url: "https://github.com/user/repo".to_string(),
        tags: vec!["production".to_string(), "github".to_string()],
    };

    let json = to_value(&credential).expect("Failed to serialize GitCredentialInfo");
    assert_eq!(json["name"], "my-git-cred");
    assert_eq!(json["repo_url"], "https://github.com/user/repo");
    assert!(json["tags"].is_array());
    assert_eq!(json["tags"].as_array().unwrap().len(), 2);
}

#[test]
fn test_git_list_credentials_output_serialization() {
    let output = GitListCredentialsOutput {
        credentials: vec![
            GitCredentialInfo {
                name: "cred-1".to_string(),
                repo_url: "https://github.com/user/repo1".to_string(),
                tags: vec!["github".to_string()],
            },
            GitCredentialInfo {
                name: "cred-2".to_string(),
                repo_url: "https://github.com/user/repo2".to_string(),
                tags: vec!["gitlab".to_string(), "production".to_string()],
            },
        ],
    };

    let json = to_value(&output).expect("Failed to serialize GitListCredentialsOutput");
    assert!(json["credentials"].is_array());
    assert_eq!(json["credentials"].as_array().unwrap().len(), 2);
}

#[test]
fn test_git_get_current_head_input_serialization() {
    let input = GitGetCurrentHeadInput {
        destination: "/tmp/repo".to_string(),
    };

    let json = to_value(&input).expect("Failed to serialize GitGetCurrentHeadInput");
    assert_eq!(json["destination"], "/tmp/repo");
}

#[test]
fn test_git_get_current_head_output_serialization() {
    let output = GitGetCurrentHeadOutput {
        branch: "main".to_string(),
        commit: "abc123".to_string(),
        message: "Initial commit".to_string(),
    };

    let json = to_value(&output).expect("Failed to serialize GitGetCurrentHeadOutput");
    assert_eq!(json["branch"], "main");
    assert_eq!(json["commit"], "abc123");
    assert_eq!(json["message"], "Initial commit");
}

#[test]
fn test_git_clone_input_json_schema() {
    // Verify that JsonSchema is implemented for GitCloneInput
    let schema = schemars::schema_for!(GitCloneInput);
    let obj = schema
        .schema
        .object
        .as_ref()
        .expect("Schema should be an object");
    // Check that we have the expected properties
    assert!(obj.properties.contains_key("repo_url"));
    assert!(obj.properties.contains_key("destination"));
    assert!(obj.properties.contains_key("branch"));
}

#[test]
fn test_git_push_input_json_schema() {
    let schema = schemars::schema_for!(GitPushInput);
    let obj = schema
        .schema
        .object
        .as_ref()
        .expect("Schema should be an object");
    // Check that we have the expected properties
    assert!(obj.properties.contains_key("credential_name"));
    assert!(obj.properties.contains_key("repo_url"));
    assert!(obj.properties.contains_key("branch"));
    assert!(obj.properties.contains_key("destination"));
    assert!(obj.properties.contains_key("confirmation_id"));
    assert!(obj.properties.contains_key("user_decision"));
}

#[test]
fn test_round_trip_git_clone_input() {
    let original = GitCloneInput {
        repo_url: "https://github.com/user/repo".to_string(),
        destination: Some("/tmp/repo".to_string()),
        branch: Some("main".to_string()),
    };

    let json = to_value(&original).expect("Failed to serialize");
    let deserialized: GitCloneInput = from_value(json).expect("Failed to deserialize");

    assert_eq!(deserialized.repo_url, original.repo_url);
    assert_eq!(deserialized.destination, original.destination);
    assert_eq!(deserialized.branch, original.branch);
}

#[test]
fn test_round_trip_git_push_input() {
    let original = GitPushInput {
        credential_name: "my-credential".to_string(),
        repo_url: "https://github.com/user/repo".to_string(),
        branch: Some("feature".to_string()),
        destination: Some("/tmp/repo".to_string()),
        confirmation_id: Some("confirm-abc".to_string()),
        user_decision: Some("approve".to_string()),
    };

    let json = to_value(&original).expect("Failed to serialize");
    let deserialized: GitPushInput = from_value(json).expect("Failed to deserialize");

    assert_eq!(deserialized.credential_name, original.credential_name);
    assert_eq!(deserialized.repo_url, original.repo_url);
    assert_eq!(deserialized.branch, original.branch);
    assert_eq!(deserialized.destination, original.destination);
    assert_eq!(deserialized.confirmation_id, original.confirmation_id);
    assert_eq!(deserialized.user_decision, original.user_decision);
}
