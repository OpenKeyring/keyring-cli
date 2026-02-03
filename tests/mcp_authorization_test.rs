use keyring_cli::error::KeyringError;
use keyring_cli::mcp::policy::token::ConfirmationToken;

#[test]
fn test_token_encoding_decoding() {
    let token = ConfirmationToken::new(
        "test_credential".to_string(),
        "ssh_exec".to_string(),
        "session-123".to_string(),
        b"test_secret_key",
    ).unwrap();

    // Test encoding
    let encoded = token.encode().unwrap();
    assert!(!encoded.is_empty());
    assert!(!encoded.contains(":")); // Should be base64, not plain text

    // Test decoding
    let decoded = ConfirmationToken::decode(&encoded).expect("Failed to decode token");
    assert_eq!(decoded.credential_name, "test_credential");
    assert_eq!(decoded.tool, "ssh_exec");
    assert_eq!(decoded.session_id, "session-123");
    assert_eq!(decoded.nonce, token.nonce);
    assert_eq!(decoded.signature, token.signature);
}

#[test]
fn test_token_signature_generation() {
    let token = ConfirmationToken::new(
        "test_credential".to_string(),
        "api_get".to_string(),
        "session-456".to_string(),
        b"test_secret_key",
    ).unwrap();

    // Signature should be non-empty
    assert!(!token.signature.is_empty());
    assert_eq!(token.signature.len(), 64); // HMAC-SHA256 produces 32 bytes = 64 hex chars
}

#[test]
fn test_token_verification_with_valid_session() {
    let token = ConfirmationToken::new(
        "test_credential".to_string(),
        "ssh_exec".to_string(),
        "session-789".to_string(),
        b"test_secret_key",
    ).unwrap();

    // Should verify successfully with correct session and key
    let result = token.verify_with_session(b"test_secret_key", "session-789");
    assert!(result.is_ok());
}

#[test]
fn test_token_verification_with_wrong_session() {
    let token = ConfirmationToken::new(
        "test_credential".to_string(),
        "ssh_exec".to_string(),
        "session-789".to_string(),
        b"test_secret_key",
    ).unwrap();

    // Should fail with different session ID
    let result = token.verify_with_session(b"test_secret_key", "different-session");
    assert!(result.is_err());
    match result {
        Err(KeyringError::Unauthorized { reason }) => {
            assert!(reason.contains("session"));
        }
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_token_verification_with_wrong_key() {
    let token = ConfirmationToken::new(
        "test_credential".to_string(),
        "ssh_exec".to_string(),
        "session-789".to_string(),
        b"test_secret_key",
    ).unwrap();

    // Should fail with different signing key
    let result = token.verify_with_session(b"wrong_secret_key", "session-789");
    assert!(result.is_err());
    match result {
        Err(KeyringError::Unauthorized { reason }) => {
            assert!(reason.contains("signature"));
        }
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_token_signature_only_verification() {
    let token = ConfirmationToken::new(
        "test_credential".to_string(),
        "api_get".to_string(),
        "session-abc".to_string(),
        b"test_secret_key",
    ).unwrap();

    // Should verify signature with correct key
    let result = token.verify(b"test_secret_key");
    assert!(result.is_ok());

    // Should fail with wrong key
    let result = token.verify(b"wrong_key");
    assert!(result.is_err());
}

#[test]
fn test_token_nonce_uniqueness() {
    let token1 = ConfirmationToken::new(
        "test_credential".to_string(),
        "ssh_exec".to_string(),
        "session-123".to_string(),
        b"test_secret_key",
    ).unwrap();

    let token2 = ConfirmationToken::new(
        "test_credential".to_string(),
        "ssh_exec".to_string(),
        "session-123".to_string(),
        b"test_secret_key",
    ).unwrap();

    // Nonces should be different
    assert_ne!(token1.nonce, token2.nonce);

    // Signatures should also be different due to different nonces
    assert_ne!(token1.signature, token2.signature);
}

#[test]
fn test_token_timestamp() {
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let token = ConfirmationToken::new(
        "test_credential".to_string(),
        "ssh_exec".to_string(),
        "session-123".to_string(),
        b"test_secret_key",
    ).unwrap();

    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Timestamp should be between before and after (with some tolerance)
    assert!(token.timestamp >= before - 1);
    assert!(token.timestamp <= after + 1);
}

#[test]
fn test_invalid_base64_decode() {
    let invalid_encoded = "not-valid-base64!!!";
    let result = ConfirmationToken::decode(invalid_encoded);
    assert!(result.is_err());
}

#[test]
fn test_malformed_token_decode() {
    // Valid base64 but doesn't contain expected format
    let valid_base64 = base64::encode("invalid_token_format");
    let result = ConfirmationToken::decode(&valid_base64);
    assert!(result.is_err());
}
