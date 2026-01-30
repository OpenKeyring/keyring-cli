//! Integration tests for complete sync flow
//!
//! These tests verify the full end-to-end sync functionality:
//! - Passkey -> Root MK -> CryptoManager -> CloudSyncService flow
//! - Cross-device key derivation
//! - Sync record export/import

use base64::Engine;
use keyring_cli::crypto::hkdf::DeviceIndex;
use keyring_cli::crypto::{passkey::Passkey, CryptoManager};
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::db::vault::Vault;
use keyring_cli::sync::import::{JsonSyncImporter, SyncImporter};
use keyring_cli::sync::service::SyncService;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn test_full_sync_flow_with_passkey() {
    // Create temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let sync_dir = temp_dir.path().join("sync");
    std::fs::create_dir_all(&sync_dir).unwrap();

    // Step 1: Generate Passkey
    let passkey = Passkey::generate(24).unwrap();
    let words = passkey.to_words();
    assert_eq!(words.len(), 24);

    // Step 2: Convert Passkey to seed
    let seed = passkey.to_seed(None).unwrap();
    assert_eq!(seed.get().len(), 64);

    // Step 3: Derive root master key from Passkey seed
    let salt = [1u8; 16]; // In production, this would be a random salt
    let root_master_key = seed.derive_root_master_key(&salt).unwrap();
    assert_eq!(root_master_key.len(), 32);

    // Step 4: Initialize CryptoManager with Passkey (simulating device 1)
    let mut crypto_manager = CryptoManager::new();
    let kdf_nonce = [2u8; 32]; // In production, this would be random

    crypto_manager
        .initialize_with_passkey(
            &passkey,
            "device-password",
            &root_master_key,
            DeviceIndex::MacOS,
            &kdf_nonce,
        )
        .unwrap();

    // Verify CryptoManager is initialized
    assert!(crypto_manager.is_initialized());
    assert!(crypto_manager.get_device_key().is_some());

    // Step 5: Create and encrypt a test record
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let plaintext_password = b"my-secure-password-123";
    let (encrypted_data, nonce) = crypto_manager.encrypt(plaintext_password).unwrap();

    let test_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data,
        nonce,
        tags: vec!["test".to_string(), "integration".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
    };

    // Add record to vault
    vault.add_record(&test_record).unwrap();

    // Step 6: Export record to sync directory
    let sync_service = SyncService::new();
    let exported_records = sync_service
        .export_pending_records(&vault, &sync_dir)
        .unwrap();

    assert_eq!(exported_records.len(), 1);
    assert_eq!(exported_records[0].id, test_record.id.to_string());

    // Step 7: Verify exported record structure
    let sync_record = &exported_records[0];

    // Verify metadata doesn't contain sensitive information
    let metadata_json = serde_json::to_string(&sync_record.metadata).unwrap();
    assert!(!metadata_json.contains("passkey"));
    assert!(!metadata_json.contains("master_key"));
    assert!(!metadata_json.contains("private_key"));

    // Verify encrypted data is base64 encoded
    assert!(
        base64::engine::general_purpose::STANDARD
            .decode(&sync_record.encrypted_data)
            .is_ok()
    );

    // Step 8: Simulate cross-device sync (import on device 2)
    // In production, this would be a different device with the same Passkey
    let importer = JsonSyncImporter;
    let sync_file_path = sync_dir.join(format!("{}.json", test_record.id));

    let imported_sync_record = importer.import_from_file(&sync_file_path).unwrap();
    let imported_record = importer.sync_record_to_db(imported_sync_record).unwrap();

    // Verify imported record matches original
    assert_eq!(imported_record.id, test_record.id);
    assert_eq!(imported_record.record_type, test_record.record_type);
    assert_eq!(imported_record.encrypted_data, test_record.encrypted_data);
    assert_eq!(imported_record.nonce, test_record.nonce);
    assert_eq!(imported_record.tags, test_record.tags);

    // Step 9: Decrypt on device 2 to verify data integrity
    // In production, device 2 would derive its own device-specific key
    // from the same root master key
    let decrypted_data = crypto_manager
        .decrypt(&imported_record.encrypted_data, &imported_record.nonce)
        .unwrap();

    assert_eq!(decrypted_data, plaintext_password);
}

#[tokio::test]
async fn test_cross_device_key_derivation() {
    // Generate Passkey
    let passkey = Passkey::generate(24).unwrap();
    let seed = passkey.to_seed(None).unwrap();

    // Derive root master key
    let salt = [1u8; 16];
    let root_master_key = seed.derive_root_master_key(&salt).unwrap();

    // Simulate two devices
    let kdf_nonce = [2u8; 32];

    // Device 1: macOS
    let mut crypto_macos = CryptoManager::new();
    crypto_macos
        .initialize_with_passkey(
            &passkey,
            "macos-password",
            &root_master_key,
            DeviceIndex::MacOS,
            &kdf_nonce,
        )
        .unwrap();

    // Device 2: iOS
    let mut crypto_ios = CryptoManager::new();
    crypto_ios
        .initialize_with_passkey(
            &passkey,
            "ios-password",
            &root_master_key,
            DeviceIndex::IOS,
            &kdf_nonce,
        )
        .unwrap();

    // Both devices should have different device keys
    let macos_key = crypto_macos.get_device_key().unwrap();
    let ios_key = crypto_ios.get_device_key().unwrap();
    assert_ne!(macos_key, ios_key);

    // But they should be able to encrypt/decrypt the same data
    // if they use the same device-specific key (this is a simplified test)
    let plaintext = b"cross-device-test-data";
    let (encrypted, nonce) = crypto_macos.encrypt(plaintext).unwrap();
    let decrypted = crypto_macos.decrypt(&encrypted, &nonce).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[tokio::test]
async fn test_passkey_seed_pbkdf2_derivation() {
    // Test PBKDF2 derivation with different parameters
    let passkey = Passkey::generate(12).unwrap();
    let seed = passkey.to_seed(None).unwrap();

    let salt1 = [1u8; 16];
    let salt2 = [2u8; 16];

    // Same seed with different salts should produce different keys
    let key1 = seed.derive_root_master_key(&salt1).unwrap();
    let key2 = seed.derive_root_master_key(&salt2).unwrap();

    assert_ne!(key1, key2);

    // Same seed with same salt should produce same key
    let key3 = seed.derive_root_master_key(&salt1).unwrap();
    assert_eq!(key1, key3);

    // Verify key length
    assert_eq!(key1.len(), 32);

    // Verify key is not all zeros (basic sanity check)
    let mut is_all_zeros = true;
    for &byte in &key1 {
        if byte != 0 {
            is_all_zeros = false;
            break;
        }
    }
    assert!(!is_all_zeros, "Derived key should not be all zeros");
}

#[tokio::test]
async fn test_sync_roundtrip_with_encrypted_data() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let sync_dir = temp_dir.path().join("sync");
    std::fs::create_dir_all(&sync_dir).unwrap();

    // Initialize crypto
    let mut crypto = CryptoManager::new();
    crypto.initialize("test-password").unwrap();

    // Create vault
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Create multiple test records
    let test_data: Vec<(&str, &[u8])> = vec![
        ("github", b"github-password-123"),
        ("aws", b"aws-access-key"),
        ("email", b"email-secret-456"),
    ];

    for (name, password) in &test_data {
        let (encrypted, nonce) = crypto.encrypt(*password).unwrap();
        let record = StoredRecord {
            id: Uuid::new_v4(),
            record_type: RecordType::Password,
            encrypted_data: encrypted,
            nonce,
            tags: vec![name.to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };
        vault.add_record(&record).unwrap();
    }

    // Export all records
    let sync_service = SyncService::new();
    let exported = sync_service
        .export_pending_records(&vault, &sync_dir)
        .unwrap();

    assert_eq!(exported.len(), 3);

    // Import all records
    let stats = sync_service
        .import_from_directory(
            &mut Vault::open(&db_path, "test-password").unwrap(),
            &sync_dir,
            keyring_cli::sync::conflict::ConflictResolution::Newer,
        )
        .unwrap();

    // Verify import statistics
    assert_eq!(stats.imported + stats.updated, 3);

    // Verify all exported files exist
    for record in &exported {
        let file_path = sync_dir.join(format!("{}.json", record.id));
        assert!(file_path.exists());
    }
}

#[tokio::test]
async fn test_passkey_word_validation() {
    // Test BIP39 word validation
    assert!(Passkey::is_valid_word("abandon"));
    assert!(Passkey::is_valid_word("zoo"));
    assert!(!Passkey::is_valid_word("invalid-word"));
    assert!(!Passkey::is_valid_word(""));

    // Test Passkey generation with different word counts
    let passkey_12 = Passkey::generate(12).unwrap();
    assert_eq!(passkey_12.to_words().len(), 12);

    let passkey_24 = Passkey::generate(24).unwrap();
    assert_eq!(passkey_24.to_words().len(), 24);

    // Test that invalid word count fails
    assert!(Passkey::generate(11).is_err());
    assert!(Passkey::generate(25).is_err());
}
