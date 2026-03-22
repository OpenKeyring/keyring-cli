use keyring_cli::crypto::keystore::KeyStore;

#[test]
fn keystore_roundtrip_unlock() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("keystore.json");
    let master = "correct-horse-battery-staple";

    let keystore = KeyStore::initialize(&path, master).unwrap();
    assert!(path.exists());
    assert_eq!(keystore.dek.get().len(), 32);

    let unlocked = KeyStore::unlock(&path, master).unwrap();
    assert_eq!(unlocked.dek.get().len(), 32);
}

#[test]
fn recovery_key_verification() {
    use keyring_cli::crypto::verify_recovery_key;

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("keystore.json");
    let master = "test-password";

    let keystore = KeyStore::initialize(&path, master).unwrap();
    let recovery_key = keystore.recovery_key.as_ref().unwrap();

    // Get the hash from the keystore file
    let content = std::fs::read_to_string(&path).unwrap();
    let keystore_file: serde_json::Value = serde_json::from_str(&content).unwrap();
    let recovery_key_hash = keystore_file["recovery_key_hash"].as_str().unwrap();

    // Verify the recovery key matches the hash
    assert!(verify_recovery_key(recovery_key, recovery_key_hash));

    // Verify wrong recovery key fails
    let wrong_key = "wrong recovery key phrase";
    assert!(!verify_recovery_key(wrong_key, recovery_key_hash));
}
