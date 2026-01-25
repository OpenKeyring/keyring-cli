# Database Completion Documentation

## Overview

This document describes the completed database features beyond the basic CRUD operations.

## Soft Delete Implementation

Records are soft-deleted to support sync and recovery:

```rust
vault.delete_record(&record_id)?;
```

- Sets `deleted = 1` in the records table
- Updates `updated_at` timestamp
- Marks record as pending sync

## MCP Tables

### mcp_sessions

Stores active MCP sessions for authorization tracking:

```sql
CREATE TABLE mcp_sessions (
    id TEXT PRIMARY KEY,
    approved_credentials TEXT NOT NULL,  -- JSON array
    created_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL,
    ttl_seconds INTEGER NOT NULL
);
```

### mcp_policies

Stores custom authorization policies per credential/tag:

```sql
CREATE TABLE mcp_policies (
    credential_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    authz_mode TEXT NOT NULL,  -- "auto" | "confirm" | "deny"
    created_at INTEGER NOT NULL,
    PRIMARY KEY (credential_id, tag)
);
```

## File Locking

Cross-platform file locking for CLI ↔ App coordination:

```rust
use keyring_cli::db::lock::VaultLock;

// Exclusive write lock
let lock = VaultLock::acquire_write(&vault_path, 5000)?;

// ... perform write operations ...

lock.release()?;
```

### Platform Support

- **Unix/macOS**: `flock()` system call
- **Windows**: `LockFileEx()` API

### Using with Vault

```rust
use keyring_cli::db::vault::Vault;

// For write operations
let (vault, _lock) = Vault::with_write_lock(&vault_path, "password", 5000)?;
// ... write operations ...
// Lock is automatically released when _lock goes out of scope

// For read operations
let (vault, _lock) = Vault::with_read_lock(&vault_path, "password", 5000)?;
// ... read operations ...
```

## WAL Checkpoint Management

Control WAL file growth:

```rust
use keyring_cli::db::wal;

// Standard checkpoint
wal::checkpoint(&mut conn)?;

// Aggressive truncation
wal::truncate(&mut conn)?;

// Disable auto-checkpoint for bulk operations
wal::disable_auto_checkpoint(&conn)?;
// ... bulk operations ...
wal::enable_auto_checkpoint(&conn)?;
```

### Checkpoint Modes

- **PASSIVE**: Only checkpoint if no readers are using the WAL
- **TRUNCATE**: Write all frames, sync, and truncate WAL file (default)
- **FULL**: Write all frames and sync, but don't truncate
- **RESTART**: Checkpoint and restart WAL file for backups

## Migration System

Version-controlled schema changes:

```rust
use keyring_cli::db::{Migrator, Migration};

struct MyMigration;
impl Migration for MyMigration {
    fn version(&self) -> i64 { 1 }
    fn name(&self) -> &str { "my_migration" }
    fn up(&self, conn: &Connection) -> Result<()> {
        conn.execute("CREATE TABLE ...", [])?;
        Ok(())
    }
    fn down(&self, conn: &Connection) -> Result<()> {
        conn.execute("DROP TABLE ...", [])?;
        Ok(())
    }
}

let mut migrator = Migrator::new(&db_path)?;
migrator.apply_migration(&MyMigration)?;
```

### Migration Tracking

Migrations are tracked in the `schema_migrations` table:

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at INTEGER NOT NULL
);
```

## Module Exports

All database features are re-exported from `src/db/mod.rs`:

```rust
// Core
pub use schema::initialize_database;
pub use vault::Vault;
pub use models::{RecordType, StoredRecord, SyncStatus, SyncState};

// Locking
pub use lock::VaultLock;

// WAL
pub use wal::{checkpoint, truncate, checkpoint_passive};

// Migration
pub use migration::{Migration, Migrator};
```

## Database Schema Summary

### Tables

1. **records** - Encrypted password records with soft delete support
2. **tags** - Tag definitions for categorization
3. **record_tags** - Many-to-many relationship between records and tags
4. **metadata** - Key-value metadata storage
5. **sync_state** - Sync status tracking for each record
6. **mcp_sessions** - MCP session tracking
7. **mcp_policies** - MCP authorization policies
8. **schema_migrations** - Database version tracking

### Indexes

- `idx_records_type` - Records by type
- `idx_records_updated` - Records by update time (descending)
- `idx_records_deleted` - Records by deleted status
- `idx_mcp_sessions_last_activity` - MCP sessions by last activity
- `idx_mcp_policies_credential` - MCP policies by credential ID

## Performance Considerations

### WAL Mode

- Enables concurrent readers and writers
- Better performance than rollback journal
- Requires periodic checkpointing to manage disk space

### File Locking

- Prevents concurrent write conflicts
- Allows multiple concurrent readers
- Automatic release on file close

### Soft Delete

- Preserves data for sync and recovery
- Query optimization via `deleted` index
- List operations automatically filter deleted records

## Testing

All database features have comprehensive tests:

```bash
# Run all database tests
cargo test --test vault_test
cargo test --test schema_test
cargo test --test lock_test
cargo test --test wal_test
cargo test --test migration_test
```

## Future Enhancements

Out of scope for this completion phase but planned for future:

- MCP session manager implementation
- MCP policy manager implementation
- Automatic WAL checkpoint scheduling
- Production migration scripts
- Database backup/restore functionality
- Performance testing for concurrent access patterns
