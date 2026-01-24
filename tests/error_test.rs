use keyring_cli::error::Error;

#[test]
fn test_crypto_error_display() {
    let err = Error::Crypto {
        context: "test failed".to_string(),
    };
    assert_eq!(err.to_string(), "Crypto error: test failed");
}

#[test]
fn test_database_error_display() {
    let err = Error::Database {
        context: "query failed".to_string(),
    };
    assert_eq!(err.to_string(), "Database error: query failed");
}

#[test]
fn test_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("internal error");
    let err: Error = anyhow_err.into();
    assert!(matches!(err, Error::Internal { .. }));
}

#[test]
fn test_invalid_input_error_display() {
    let err = Error::InvalidInput {
        context: "empty password".to_string(),
    };
    assert_eq!(err.to_string(), "Invalid input: empty password");
}

#[test]
fn test_not_found_error_display() {
    let err = Error::NotFound {
        resource: "password record".to_string(),
    };
    assert_eq!(err.to_string(), "Not found: password record");
}

#[test]
fn test_authentication_failed_error_display() {
    let err = Error::AuthenticationFailed {
        reason: "wrong master password".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "Authentication failed: wrong master password"
    );
}

#[test]
fn test_clipboard_error_display() {
    let err = Error::Clipboard {
        context: "pbcopy not found".to_string(),
    };
    assert_eq!(err.to_string(), "Clipboard error: pbcopy not found");
}

#[test]
fn test_sync_error_display() {
    let err = Error::Sync {
        context: "cloud connection failed".to_string(),
    };
    assert_eq!(err.to_string(), "Sync error: cloud connection failed");
}

#[test]
fn test_mcp_error_display() {
    let err = Error::Mcp {
        context: "MCP protocol error".to_string(),
    };
    assert_eq!(err.to_string(), "MCP error: MCP protocol error");
}
