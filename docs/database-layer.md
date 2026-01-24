# Database Layer Documentation

## Overview

The database layer provides SQLite-based storage for password records with encryption, tag management, and sync support.

## Components

### Vault

The `Vault` struct provides the main interface for database operations.

#### Basic Operations

```rust
use keyring_cli::db::{Vault, Record, RecordType};
use tempfile::TempDir;
use uuid::Uuid;

let temp_dir = TempDir::new().unwrap();
let db_path = temp_dir.path().join("passwords.db");
let mut vault = Vault::open(&db_path, "master-password")?;

// Add a record
let record = Record {
    id: Uuid::new_v4(),
    record_type: RecordType::Password,
    encrypted_data: "encrypted-data".to_string(),
    name: "github".to_string(),
    username: Some("user@example.com".to_string()),
    url: Some("https://github.com".to_string()),
    notes: None,
    tags: vec!["work".to_string()],
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
vault.add_record(&record)?;

// Get a record
let retrieved = vault.get_record(&record.id.to_string())?;

// List all records
let records = vault.list_records()?;

// Update a record
let mut updated_record = record.clone();
updated_record.encrypted_data = "new-encrypted-data".to_string();
vault.update_record(&updated_record)?;

// Soft delete
vault.delete_record(&record.id.to_string())?;
```

#### Search and Filter

```rust
// Search by pattern (currently searches encrypted_data field)
let results = vault.search_records("data")?;

// Filter by type
let passwords = vault.list_by_type(RecordType::Password)?;

// Filter by tag
let work_items = vault.list_by_tag("work")?;
```

#### Tag Management

```rust
// List all tags
let tags = vault.list_tags()?;

// Rename a tag
vault.rename_tag("old-name", "new-name")?;

// Delete a tag
vault.delete_tag("unused-tag")?;
```

#### Metadata

```rust
// Set metadata
vault.set_metadata("version", "1.0.0")?;
vault.set_metadata("device_id", "device-123")?;

// Get metadata
let version = vault.get_metadata("version")?;

// Delete metadata
vault.delete_metadata("version")?;
```

#### Sync State

```rust
use keyring_cli::db::{SyncState, SyncStatus};

let sync_state = SyncState {
    record_id: record.id.to_string(),
    cloud_updated_at: Some(chrono::Utc::now().timestamp()),
    sync_status: SyncStatus::Synced,
};
vault.set_sync_state(&sync_state)?;

let state = vault.get_sync_state(&record.id.to_string())?;
```

#### Batch Operations

```rust
// Add multiple records in a transaction
let records = vec![record1, record2, record3];
vault.batch_add_records(&records)?;
```

#### Count and Statistics

```rust
// Count all non-deleted records
let total = vault.count_all()?;

// Count records by type
let password_count = vault.count_by_type(RecordType::Password)?;
```

### DatabaseManager

Manages database connections and provides vault access.

```rust
use keyring_cli::db::DatabaseManager;

let mut db_manager = DatabaseManager::new(&db_path)?;
db_manager.open()?;

// Get a Vault instance
let vault = db_manager.vault()?;

// Use vault for operations
let records = vault.list_records()?;

// Close the database
db_manager.close()?;
```

## Schema

### records table
- `id`: TEXT (UUID, primary key)
- `record_type`: TEXT (password, ssh_key, api_credential, mnemonic, private_key)
- `encrypted_data`: TEXT (encrypted record data)
- `nonce`: TEXT (encryption nonce, currently placeholder)
- `created_at`: INTEGER (timestamp)
- `updated_at`: INTEGER (timestamp)
- `updated_by`: TEXT (device ID)
- `version`: INTEGER (auto-incremented on update)
- `deleted`: INTEGER (0=active, 1=deleted)

### tags table
- `id`: INTEGER (primary key, auto-increment)
- `name`: TEXT (unique)

### record_tags table
- `record_id`: TEXT (foreign key to records.id)
- `tag_id`: INTEGER (foreign key to tags.id)
- PRIMARY KEY (record_id, tag_id)

### metadata table
- `key`: TEXT (primary key)
- `value`: TEXT

### sync_state table
- `record_id`: TEXT (primary key, foreign key to records.id)
- `cloud_updated_at`: INTEGER (nullable timestamp)
- `sync_status`: INTEGER (0=pending, 1=synced, 2=conflict)

## Best Practices

1. **Always use transactions** for batch operations
2. **Soft delete** is used for record retention - records are marked as deleted but not removed
3. **WAL mode** enables concurrent access - multiple readers and one writer
4. **Filter by deleted=0** to exclude soft-deleted records in queries
5. **Version field** auto-increments for conflict resolution during sync
6. **Tag deduplication** - tags are automatically deduplicated when adding records
7. **Use GROUP_CONCAT** - list operations use optimized queries with GROUP_CONCAT to avoid N+1 query patterns

## Performance Considerations

- **WAL Mode**: SQLite uses Write-Ahead Logging for better concurrency
- **Indexes**: Indexes are created on `record_type`, `updated_at`, and `deleted` columns
- **Query Optimization**: List operations use JOINs with GROUP_CONCAT to load tags efficiently
- **Transaction Safety**: Batch operations wrap all inserts in a single transaction

## Error Handling

All operations return `Result<T>` types:
- `Ok(T)` for successful operations
- `Err(anyhow::Error)` for failures

Common error scenarios:
- Record not found: Returns error when trying to get/update/delete non-existent records
- Tag not found: Returns error when trying to rename/delete non-existent tags
- Database not open: Returns error when DatabaseManager methods are called before `open()`

## Testing

All database operations have comprehensive test coverage in `tests/vault_test.rs`:

- Record CRUD operations
- Tag management
- Metadata operations
- Sync state management
- Batch operations
- Count and statistics
- DatabaseManager integration

Run tests with:
```bash
cargo test --test vault_test
```

## Future Enhancements

- Encrypted data serialization/deserialization (when crypto module is integrated)
- Database migration support
- Vault locking mechanism for CLI <-> App coordination
- Performance benchmarks
- Database backup/restore functionality
- Full-text search on decrypted name field (after crypto integration)
