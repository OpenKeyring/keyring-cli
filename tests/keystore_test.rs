use keyring_cli::crypto::keystore::KeyStore;

#[test]
fn keystore_roundtrip_unlock() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("keystore.json");
    let master = "correct-horse-battery-staple";

    let keystore = KeyStore::initialize(&path, master).unwrap();
    assert!(path.exists());
    assert_eq!(keystore.dek.len(), 32);

    let unlocked = KeyStore::unlock(&path, master).unwrap();
    assert_eq!(unlocked.dek.len(), 32);
}
