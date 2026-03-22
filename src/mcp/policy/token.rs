use crate::error::KeyringError;
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Confirmation token for MCP authorization flow.
///
/// Tokens are used in the two-phase authorization flow where AI queries first,
/// gets a confirmation_id, then calls again after user approval.
///
/// # Security Properties
/// - HMAC-SHA256 signed tokens prevent tampering
/// - Random nonce ensures uniqueness
/// - Session binding prevents token reuse across sessions
/// - Timestamp enables expiration checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationToken {
    /// Random 16-byte nonce for uniqueness
    pub nonce: String,
    /// Which credential is being accessed
    pub credential_name: String,
    /// Which tool is being invoked (ssh_exec, api_get, etc.)
    pub tool: String,
    /// MCP session UUID for session binding
    pub session_id: String,
    /// Unix timestamp for expiration checking
    pub timestamp: i64,
    /// HMAC-SHA256 signature
    pub signature: String,
}

impl ConfirmationToken {
    const NONCE_SIZE: usize = 16;

    /// Create a new confirmation token with a signature.
    ///
    /// # Arguments
    /// * `credential_name` - The credential being accessed
    /// * `tool` - The tool being invoked
    /// * `session_id` - The MCP session ID for binding
    /// * `signing_key` - The secret key for HMAC signing
    ///
    /// # Returns
    /// A signed confirmation token
    pub fn new(
        credential_name: String,
        tool: String,
        session_id: String,
        signing_key: &[u8],
    ) -> Result<Self, KeyringError> {
        let nonce = Self::generate_nonce();
        let timestamp = Self::current_timestamp();

        let token = Self {
            nonce,
            credential_name,
            tool,
            session_id,
            timestamp,
            signature: String::new(), // Will be set below
        };

        let signature = token.sign(signing_key)?;
        Ok(Self { signature, ..token })
    }

    /// Generate a random nonce for token uniqueness.
    fn generate_nonce() -> String {
        let mut rng = rand::rng();
        let nonce_bytes: Vec<u8> = (0..Self::NONCE_SIZE).map(|_| rng.random::<u8>()).collect();
        hex::encode(nonce_bytes)
    }

    /// Get the current Unix timestamp.
    fn current_timestamp() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    /// Sign the token with HMAC-SHA256.
    ///
    /// The signature covers: nonce, credential_name, tool, and session_id
    fn sign(&self, key: &[u8]) -> Result<String, KeyringError> {
        let message = format!(
            "{}:{}:{}:{}",
            self.nonce, self.credential_name, self.tool, self.session_id
        );
        let mut mac = HmacSha256::new_from_slice(key).map_err(|_| KeyringError::Crypto {
            context: "Invalid HMAC key length".to_string(),
        })?;
        mac.update(message.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    /// Encode the token as a base64 string.
    ///
    /// This encodes the entire token (excluding signature) as JSON,
    /// then base64-encodes it. The signature is appended separately.
    pub fn encode(&self) -> Result<String, KeyringError> {
        let token_data = TokenData {
            nonce: &self.nonce,
            credential_name: &self.credential_name,
            tool: &self.tool,
            session_id: &self.session_id,
            timestamp: self.timestamp,
            signature: &self.signature,
        };

        let json = serde_json::to_string(&token_data).map_err(|e| KeyringError::Crypto {
            context: format!("Token serialization failed: {}", e),
        })?;
        Ok(STANDARD.encode(json))
    }

    /// Decode a token from a base64 string.
    ///
    /// # Arguments
    /// * `encoded` - The base64-encoded token string
    ///
    /// # Returns
    /// A decoded ConfirmationToken
    ///
    /// # Errors
    /// Returns KeyringError if the input is invalid base64 or malformed
    pub fn decode(encoded: &str) -> Result<Self, KeyringError> {
        let json = STANDARD
            .decode(encoded)
            .map_err(|_| KeyringError::Unauthorized {
                reason: "Invalid token encoding".to_string(),
            })?;

        let json_str = String::from_utf8(json).map_err(|_| KeyringError::Unauthorized {
            reason: "Invalid token encoding".to_string(),
        })?;

        let data: TokenData =
            serde_json::from_str(&json_str).map_err(|_| KeyringError::Unauthorized {
                reason: "Invalid token format".to_string(),
            })?;

        Ok(Self {
            nonce: data.nonce.to_string(),
            credential_name: data.credential_name.to_string(),
            tool: data.tool.to_string(),
            session_id: data.session_id.to_string(),
            timestamp: data.timestamp,
            signature: data.signature.to_string(),
        })
    }

    /// Verify the token's signature and session binding.
    ///
    /// This method checks both:
    /// 1. The HMAC signature is valid for the given key
    /// 2. The session_id matches the expected session
    ///
    /// # Arguments
    /// * `signing_key` - The key used to verify the signature
    /// * `expected_session_id` - The session ID to validate against
    ///
    /// # Returns
    /// Ok(()) if both signature and session are valid
    ///
    /// # Errors
    /// Returns KeyringError::Unauthorized if verification fails
    pub fn verify_with_session(
        &self,
        signing_key: &[u8],
        expected_session_id: &str,
    ) -> Result<(), KeyringError> {
        // First, verify the signature
        self.verify(signing_key)?;

        // Then, verify the session binding
        if self.session_id != expected_session_id {
            return Err(KeyringError::Unauthorized {
                reason: format!(
                    "Session mismatch: expected {}, got {}",
                    expected_session_id, self.session_id
                ),
            });
        }

        Ok(())
    }

    /// Verify only the token's signature.
    ///
    /// Use this when you want to check signature validity without
    /// session binding.
    ///
    /// # Arguments
    /// * `signing_key` - The key used to verify the signature
    ///
    /// # Returns
    /// Ok(()) if the signature is valid
    ///
    /// # Errors
    /// Returns KeyringError::Unauthorized if the signature is invalid
    pub fn verify(&self, signing_key: &[u8]) -> Result<(), KeyringError> {
        let expected_signature = self.sign(signing_key)?;

        // Constant-time comparison to prevent timing attacks
        if !self.constant_time_compare(&self.signature, &expected_signature) {
            return Err(KeyringError::Unauthorized {
                reason: "Invalid token signature".to_string(),
            });
        }

        Ok(())
    }

    /// Constant-time string comparison to prevent timing attacks.
    fn constant_time_compare(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }
}

/// Internal struct for serialization.
#[derive(Serialize, Deserialize)]
struct TokenData<'a> {
    nonce: &'a str,
    credential_name: &'a str,
    tool: &'a str,
    session_id: &'a str,
    timestamp: i64,
    signature: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_length() {
        let token = ConfirmationToken::new(
            "test".to_string(),
            "test_tool".to_string(),
            "session".to_string(),
            b"key",
        )
        .unwrap();
        // 16 bytes = 32 hex chars
        assert_eq!(token.nonce.len(), 32);
    }
}
