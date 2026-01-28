use chrono::Utc;
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::db::vault::Vault;
use keyring_cli::sync::export::{JsonSyncExporter, SyncExporter};
use keyring_cli::sync::import::{JsonSyncImporter, SyncImporter};
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn sync_export_import_roundtrip() {
    // 创建临时目录用于测试
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let sync_dir = temp_dir.path().join("sync");
    std::fs::create_dir_all(&sync_dir).unwrap();

    // 创建 vault
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // 创建测试记录
    let test_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        tags: vec!["test".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // 添加记录到 vault
    vault.add_record(&test_record).unwrap();

    // Export: 导出记录到目录
    let exporter = JsonSyncExporter;
    let sync_record = exporter.export_record(&test_record).unwrap();
    let export_path = sync_dir.join(format!("{}.json", test_record.id));
    exporter.write_to_file(&sync_record, &export_path).unwrap();

    // Import: 从目录导入记录
    let importer = JsonSyncImporter;
    let imported_sync_record = importer.import_from_file(&export_path).unwrap();
    let imported_record = importer.sync_record_to_db(imported_sync_record).unwrap();

    // 断言记录数量、updated_at、nonce 一致
    assert_eq!(imported_record.id, test_record.id);
    assert_eq!(imported_record.record_type, test_record.record_type);
    assert_eq!(imported_record.encrypted_data, test_record.encrypted_data);
    assert_eq!(imported_record.nonce, test_record.nonce);
    assert_eq!(imported_record.tags, test_record.tags);
    assert_eq!(
        imported_record.updated_at.timestamp(),
        test_record.updated_at.timestamp()
    );
}
