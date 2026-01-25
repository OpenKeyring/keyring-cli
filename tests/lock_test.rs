use keyring_cli::db::lock::VaultLock;
use tempfile::TempDir;

#[test]
fn test_write_lock_exclusive() {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // First lock should succeed
    let lock1 = VaultLock::acquire_write(vault_path, 1000).unwrap();

    // Second lock attempt should fail or timeout
    let lock2_result = VaultLock::acquire_write(vault_path, 100);

    // Either fail immediately or timeout
    assert!(lock2_result.is_err(), "Second write lock should fail");

    // Release first lock
    lock1.release().unwrap();

    // Now second lock should succeed
    let lock2 = VaultLock::acquire_write(vault_path, 1000).unwrap();
    lock2.release().unwrap();
}

#[test]
fn test_read_lock_concurrent() {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path();

    // Multiple read locks should be allowed
    let lock1 = VaultLock::acquire_read(vault_path, 1000).unwrap();
    let lock2 = VaultLock::acquire_read(vault_path, 1000).unwrap();

    lock1.release().unwrap();
    lock2.release().unwrap();
}
