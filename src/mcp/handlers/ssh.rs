//! SSH Tool Handler with Authorization
//!
//! This module implements the SSH tool handler that connects SSH tool definitions
//! to the SSH executor with proper authorization flow and confirmation handling.

use crate::db::models::RecordType;
use crate::db::vault::Vault;
use crate::error::KeyringError;
use crate::mcp::policy::{ConfirmationToken, OperationType, PolicyEngine, SessionCache, UsedTokenCache};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// SSH execution input from the AI/tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshExecInput {
    /// Name of the SSH credential to use
    pub credential_name: String,

    /// Command to execute on the remote host
    pub command: String,

    /// Optional: Working directory on remote host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Optional: Environment variables to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,

    /// Optional: Timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,

    /// Confirmation ID from a previous pending confirmation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,

    /// User decision (approve/deny) when providing confirmation_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// SSH execution output returned to the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshExecOutput {
    /// Whether the command succeeded
    pub success: bool,

    /// Standard output from the command
    pub stdout: String,

    /// Standard error from the command
    pub stderr: String,

    /// Exit code from the command
    pub exit_code: i32,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Host that was connected to
    pub host: String,

    /// Username that was used
    pub username: String,
}

/// SSH credential extracted from database
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used when real SSH executor is implemented
struct SshCredential {
    /// Name/identifier
    pub name: String,
    /// Host to connect to
    pub host: String,
    /// Username for authentication
    pub username: String,
    /// Port (default 22)
    pub port: u16,
    /// Private key content
    pub private_key: String,
    /// Optional passphrase for the key
    pub passphrase: Option<String>,
    /// Tags for policy evaluation
    pub tags: HashSet<String>,
}

/// Handler error types
#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Credential '{name}' not found")]
    CredentialNotFound { name: String },

    #[error("Invalid confirmation token: {reason}")]
    InvalidToken { reason: String },

    #[error("Operation denied by user")]
    DeniedByUser,

    #[error("Invalid user decision: {decision}")]
    InvalidDecision { decision: String },

    #[error("Pending confirmation: {confirmation_id}")]
    PendingConfirmation {
        confirmation_id: String,
        prompt: String,
        policy: String,
    },

    #[error("SSH execution failed: {0}")]
    SshError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] KeyringError),

    #[error("Policy denied this operation")]
    DeniedByPolicy,
}

impl From<HandlerError> for KeyringError {
    fn from(err: HandlerError) -> Self {
        match err {
            HandlerError::CredentialNotFound { name } => KeyringError::RecordNotFound { name },
            HandlerError::InvalidToken { reason } => KeyringError::Unauthorized {
                reason: format!("Invalid confirmation token: {}", reason),
            },
            HandlerError::DeniedByUser => KeyringError::Unauthorized {
                reason: "Operation denied by user".to_string(),
            },
            HandlerError::DeniedByPolicy => KeyringError::Unauthorized {
                reason: "Policy denied this operation".to_string(),
            },
            HandlerError::SshError(msg) => KeyringError::Mcp {
                context: format!("SSH execution failed: {}", msg),
            },
            HandlerError::DatabaseError(e) => e,
            HandlerError::InvalidDecision { .. } | HandlerError::PendingConfirmation { .. } => {
                KeyringError::Mcp {
                    context: err.to_string(),
                }
            }
        }
    }
}

/// Handle SSH exec tool call with authorization flow
///
/// # Authorization Flow
/// 1. AI calls tool without confirmation_id
/// 2. Handler checks policy engine for credential tags
/// 3. If AutoApprove → execute immediately
/// 4. If SessionApprove → check session cache → if authorized, execute
/// 5. If AlwaysConfirm or no session auth → return PendingConfirmation with confirmation_id
/// 6. User confirms via AI (AI calls again with confirmation_id + user_decision)
/// 7. Handler validates token and executes
///
/// # Arguments
/// * `input` - SSH execution input parameters
/// * `vault` - Vault for accessing encrypted credentials
/// * `signing_key` - Key for signing confirmation tokens
/// * `session_cache` - Session authorization cache
/// * `used_tokens` - Used token cache for replay protection
/// * `session_id` - Current MCP session ID
///
/// # Returns
/// * `Ok(SshExecOutput)` - Command executed successfully
/// * `Err(HandlerError::PendingConfirmation)` - User confirmation required
/// * `Err(HandlerError)` - Other errors
pub async fn handle_ssh_exec(
    input: SshExecInput,
    vault: &Vault,
    signing_key: &[u8],
    session_cache: &mut SessionCache,
    used_tokens: &mut UsedTokenCache,
    session_id: &str,
) -> Result<SshExecOutput, HandlerError> {
    // 1. Load credential from database
    let ssh_credential = load_ssh_credential(vault, &input.credential_name)?;

    // 2. Check if confirmation_id present (user approved)
    if let Some(ref cid) = input.confirmation_id {
        return handle_confirmed_exec(
            cid,
            input.clone(), // Clone to avoid move
            ssh_credential,
            vault,
            signing_key,
            session_cache,
            used_tokens,
            session_id,
        )
        .await;
    }

    // 3. Check policy engine
    let engine = PolicyEngine::new();
    let decision = engine.decide(&ssh_credential.tags, OperationType::Write, "ssh_exec");

    // 4. Handle based on decision
    match decision {
        crate::mcp::policy::AuthDecision::AutoApprove => {
            // Execute immediately without confirmation
            log::debug!("AutoApprove: executing SSH command immediately");
            return execute_ssh(input, ssh_credential).await;
        }
        crate::mcp::policy::AuthDecision::SessionApprove => {
            // Check session cache
            if session_cache.is_authorized(&input.credential_name) {
                log::debug!("SessionApprove: credential authorized in session cache");
                return execute_ssh(input, ssh_credential).await;
            }
            log::debug!("SessionApprove: credential not in session cache, requiring confirmation");
        }
        crate::mcp::policy::AuthDecision::AlwaysConfirm => {
            log::debug!("AlwaysConfirm: requiring user confirmation");
        }
        crate::mcp::policy::AuthDecision::Deny => {
            return Err(HandlerError::DeniedByPolicy);
        }
    }

    // 5. Generate confirmation token
    let token = ConfirmationToken::new(
        input.credential_name.clone(),
        "ssh_exec".to_string(),
        session_id.to_string(),
        signing_key,
    );

    // 6. Return pending confirmation
    let prompt = format!(
        "Execute SSH command '{}' on {}@{}?",
        input.command, ssh_credential.username, ssh_credential.host
    );

    Err(HandlerError::PendingConfirmation {
        confirmation_id: token.encode(),
        prompt,
        policy: format!("{:?}", decision),
    })
}

/// Handle confirmed SSH execution (user provided confirmation_id)
async fn handle_confirmed_exec(
    confirmation_id: &str,
    input: SshExecInput,
    ssh_credential: SshCredential,
    _vault: &Vault,
    signing_key: &[u8],
    session_cache: &mut SessionCache,
    used_tokens: &mut UsedTokenCache,
    session_id: &str,
) -> Result<SshExecOutput, HandlerError> {
    // 1. Decode and verify token
    let token = ConfirmationToken::decode(confirmation_id).map_err(|e| HandlerError::InvalidToken {
        reason: e.to_string(),
    })?;

    // 2. Verify signature and session binding
    token.verify_with_session(signing_key, session_id)
        .map_err(|e| HandlerError::InvalidToken {
            reason: e.to_string(),
        })?;

    // 3. Check if token was already used (replay protection)
    if used_tokens.is_used(&token.nonce) {
        return Err(HandlerError::InvalidToken {
            reason: "Token already used".to_string(),
        });
    }

    // 4. Check user decision if provided
    if let Some(ref decision) = input.user_decision {
        match decision.to_lowercase().as_str() {
            "approve" | "yes" | "true" => {
                // User approved, continue
            }
            "deny" | "no" | "false" => {
                return Err(HandlerError::DeniedByUser);
            }
            _ => {
                return Err(HandlerError::InvalidDecision {
                    decision: decision.clone(),
                });
            }
        }
    }

    // 5. Mark token as used
    used_tokens.mark_used(&token.nonce).map_err(|e| HandlerError::InvalidToken {
        reason: e.to_string(),
    })?;

    // 6. Authorize in session cache (for SessionApprove policy)
    let _ = session_cache.authorize(&input.credential_name);

    // 7. Execute SSH command
    execute_ssh(input, ssh_credential).await
}

/// Execute SSH command using the executor
///
/// This is a placeholder that will be replaced with actual SSH executor
/// once the executor module is implemented.
async fn execute_ssh(
    input: SshExecInput,
    credential: SshCredential,
) -> Result<SshExecOutput, HandlerError> {
    // TODO: Replace with actual SSH executor call
    // For now, this is a placeholder that simulates execution

    log::info!(
        "Executing SSH command '{}' on {}@{}",
        input.command,
        credential.username,
        credential.host
    );

    // Simulate execution time
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Placeholder response - in real implementation, this would call SshExecutor
    Ok(SshExecOutput {
        success: true,
        stdout: format!("Command '{}' executed on {}", input.command, credential.host),
        stderr: String::new(),
        exit_code: 0,
        execution_time_ms: 100,
        host: credential.host,
        username: credential.username,
    })
}

/// Load SSH credential from the vault
///
/// Decrypts and parses the SSH credential from the database.
fn load_ssh_credential(
    vault: &Vault,
    credential_name: &str,
) -> Result<SshCredential, HandlerError> {
    // Find the record by name (returns encrypted record)
    let stored_record = vault
        .find_record_by_name(credential_name)
        .map_err(|e| HandlerError::DatabaseError(KeyringError::Database {
            context: format!("Failed to find credential: {}", e),
        }))?
        .ok_or_else(|| HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        })?;

    // Check record type
    if stored_record.record_type != RecordType::SshKey {
        return Err(HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        });
    }

    // Parse SSH credential from encrypted data
    // Note: The data is encrypted, so we need to parse the encrypted JSON structure
    // This is a placeholder - in production, this would need proper decryption
    // For now, we'll try to parse the encrypted data as UTF-8 (this won't work with real encrypted data)
    let credential_json = String::from_utf8(stored_record.encrypted_data.clone())
        .map_err(|_| HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        })?;

    let credential_data: serde_json::Value = serde_json::from_str(&credential_json).map_err(|_| {
        HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        }
    })?;

    // The actual SSH credential data should be in a "password" or "data" field
    let ssh_data_str = credential_data
        .get("password")
        .or_else(|| credential_data.get("data"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        })?;

    // Parse the SSH credential JSON (which is stored as a string in the password field)
    let ssh_data: serde_json::Value = serde_json::from_str(ssh_data_str).map_err(|_| {
        HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        }
    })?;

    let host = ssh_data["host"]
        .as_str()
        .ok_or_else(|| HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        })?
        .to_string();

    let username = ssh_data["username"]
        .as_str()
        .ok_or_else(|| HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        })?
        .to_string();

    let port = ssh_data["port"].as_u64().unwrap_or(22) as u16;

    let private_key = ssh_data["private_key"]
        .as_str()
        .ok_or_else(|| HandlerError::CredentialNotFound {
            name: credential_name.to_string(),
        })?
        .to_string();

    let passphrase = ssh_data["passphrase"].as_str().map(|s| s.to_string());

    let tags: HashSet<String> = stored_record.tags.into_iter().collect();

    Ok(SshCredential {
        name: credential_name.to_string(),
        host,
        username,
        port,
        private_key,
        passphrase,
        tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_exec_input_deserialize() {
        let json = r#"{
            "credential_name": "test-server",
            "command": "ls -la",
            "working_dir": "/home/user",
            "timeout_secs": 30
        }"#;

        let input: SshExecInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.credential_name, "test-server");
        assert_eq!(input.command, "ls -la");
        assert_eq!(input.working_dir, Some("/home/user".to_string()));
        assert_eq!(input.timeout_secs, Some(30));
    }

    #[test]
    fn test_ssh_exec_output_serialize() {
        let output = SshExecOutput {
            success: true,
            stdout: "file1.txt\nfile2.txt".to_string(),
            stderr: String::new(),
            exit_code: 0,
            execution_time_ms: 150,
            host: "example.com".to_string(),
            username: "admin".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("file1.txt"));
    }

    #[test]
    fn test_handler_error_display() {
        let err = HandlerError::CredentialNotFound {
            name: "test-cred".to_string(),
        };
        assert_eq!(err.to_string(), "Credential 'test-cred' not found");

        let err = HandlerError::DeniedByUser;
        assert_eq!(err.to_string(), "Operation denied by user");

        let err = HandlerError::DeniedByPolicy;
        assert_eq!(err.to_string(), "Policy denied this operation");
    }

    #[test]
    fn test_pending_confirmation_error() {
        let err = HandlerError::PendingConfirmation {
            confirmation_id: "test-token-abc123".to_string(),
            prompt: "Execute 'ls' on host?".to_string(),
            policy: "AlwaysConfirm".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("test-token-abc123"));
        assert!(msg.contains("Pending confirmation"));
    }
}
