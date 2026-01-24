use crate::error::KeyringError;
use crate::mcp::AuditLogger;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub client_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<String>,
}

#[derive(Debug)]
pub struct AuthManager {
    tokens: HashMap<String, AuthToken>,
    active_clients: HashMap<String, ClientSession>,
    audit_logger: AuditLogger,
}

#[derive(Debug)]
struct ClientSession {
    id: String,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    permissions: Vec<String>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            active_clients: HashMap::new(),
            audit_logger: AuditLogger::new(),
        }
    }

    pub fn generate_token(&mut self, client_id: String, permissions: Vec<String>) -> Result<AuthToken, KeyringError> {
        let token = self.generate_random_token();
        let issued_at = Utc::now();
        let expires_at = issued_at + chrono::Duration::hours(24);

        let auth_token = AuthToken {
            token: token.clone(),
            client_id: client_id.clone(),
            issued_at,
            expires_at,
            permissions: permissions.clone(),
        };

        self.tokens.insert(token.clone(), auth_token.clone());
        self.register_client_session(client_id.clone(), permissions);

        Ok(auth_token)
    }

    pub fn validate_token(&mut self, token: &str) -> Result<AuthToken, KeyringError> {
        if let Some(auth_token) = self.tokens.get(token).cloned() {
            if auth_token.expires_at > Utc::now() {
                self.update_client_activity(&auth_token.client_id)?;
                return Ok(auth_token);
            }
        }

        Err(KeyringError::Unauthorized { reason: "Invalid or expired token".to_string() })
    }

    pub fn revoke_token(&mut self, token: &str) -> Result<(), KeyringError> {
        if let Some(auth_token) = self.tokens.remove(token) {
            self.active_clients.remove(&auth_token.client_id);
            self.audit_logger.log_event("token_revoked", &serde_json::to_string(&auth_token)?);
        }
        Ok(())
    }

    pub fn list_active_clients(&self) -> Vec<ClientInfo> {
        self.active_clients.values()
            .map(|session| ClientInfo {
                id: session.id.clone(),
                created_at: session.created_at,
                last_activity: session.last_activity,
                permissions: session.permissions.clone(),
            })
            .collect()
    }

    fn generate_random_token(&self) -> String {
        format!("ok_mcp_{}", Uuid::new_v4())
    }

    fn register_client_session(&mut self, client_id: String, permissions: Vec<String>) {
        let now = Utc::now();
        self.active_clients.insert(client_id.clone(), ClientSession {
            id: client_id.clone(),
            created_at: now,
            last_activity: now,
            permissions,
        });
    }

    fn update_client_activity(&mut self, client_id: &str) -> Result<(), KeyringError> {
        if let Some(session) = self.active_clients.get_mut(client_id) {
            session.last_activity = Utc::now();
            Ok(())
        } else {
            Err(KeyringError::Unauthorized { reason: "Client not found".to_string() })
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub permissions: Vec<String>,
}