// tests/sync/nonce_validator_test.rs
use keyring_cli::sync::nonce_validator::{NonceValidator, RecoveryStrategy, NonceStatus};

#[test]
fn test_nonce_validator_creation() {
    let validator = NonceValidator::new();
    let _ = validator;
}

#[test]
fn test_nonce_validator_default() {
    let validator = NonceValidator::default();
    let _ = validator;
}

#[test]
fn test_recovery_strategy_valid_nonce() {
    let validator = NonceValidator::new();
    let strategy = validator.get_recovery_strategy(NonceStatus::Valid);
    assert_eq!(strategy, RecoveryStrategy::NoAction);
}

#[test]
fn test_recovery_strategy_mismatch_nonce() {
    let validator = NonceValidator::new();
    let strategy = validator.get_recovery_strategy(NonceStatus::Mismatch);
    assert_eq!(strategy, RecoveryStrategy::AskUser);
}

#[test]
fn test_prompt_user_resolution_returns_strategy() {
    let validator = NonceValidator::new();
    let result = validator.prompt_user_resolution("test-record");

    // Should return Some strategy (currently defaults to UseLocal)
    assert!(result.is_some());
    assert_eq!(result.unwrap(), RecoveryStrategy::UseLocal);
}

#[test]
fn test_prompt_user_resolution_different_record_names() {
    let validator = NonceValidator::new();

    // Test with different record names
    for name in &["github", "gitlab", "aws", "database"] {
        let result = validator.prompt_user_resolution(name);
        assert!(result.is_some(), "Should return strategy for record: {}", name);
    }
}
