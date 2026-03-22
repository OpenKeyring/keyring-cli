// tests/sync/nonce_validator_test.rs
use keyring_cli::sync::nonce_validator::{NonceStatus, NonceValidator, RecoveryStrategy};

#[test]
fn test_nonce_validator_creation() {
    let validator = NonceValidator::new();
    let _ = validator;
}

#[test]
fn test_nonce_validator_default() {
    let validator = NonceValidator;
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
    let local_nonce = [1u8; 12];
    let remote_nonce = [2u8; 12];
    let result = validator.prompt_user_resolution(&local_nonce, &remote_nonce);

    // Should return Ok strategy (currently defaults to UseLocal)
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RecoveryStrategy::UseLocal);
}

#[test]
fn test_prompt_user_resolution_different_record_names() {
    let validator = NonceValidator::new();

    // Test with different nonces
    for i in 0..4 {
        let local = [i; 12];
        let remote = [i + 1; 12];
        let result = validator.prompt_user_resolution(&local, &remote);
        assert!(result.is_ok(), "Should return strategy for nonces");
    }
}
