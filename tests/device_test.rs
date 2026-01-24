use keyring_cli::db::vault::Vault;
use keyring_cli::device::{generate_device_id, get_or_create_device_id};
use tempfile::tempdir;

#[test]
fn device_id_format_has_three_parts() {
    let device_id = generate_device_id();
    let parts: Vec<&str> = device_id.splitn(3, '-').collect();
    assert_eq!(parts.len(), 3);
    assert!(!parts[0].is_empty());
    assert!(!parts[1].is_empty());
    assert!(!parts[2].is_empty());
}

#[test]
fn device_id_persists_in_metadata() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("device_id.db");

    let mut vault = Vault::open(&db_path, "").unwrap();
    let first = get_or_create_device_id(&mut vault).unwrap();
    let second = get_or_create_device_id(&mut vault).unwrap();

    assert_eq!(first, second);
}
