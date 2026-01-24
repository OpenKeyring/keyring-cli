use keyring_cli::onboarding::{initialize_keystore, is_initialized};
use tempfile::tempdir;

#[test]
fn onboarding_initializes_keystore_file() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("keystore.json");

    assert!(!is_initialized(&path));
    let keystore = initialize_keystore(&path, "correct-horse-battery-staple").unwrap();
    assert!(path.exists());
    assert_eq!(keystore.dek.len(), 32);
    assert!(is_initialized(&path));
}
