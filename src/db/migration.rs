//! Database migration system for schema versioning

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

/// Migration trait for database schema changes
pub trait Migration {
    /// Unique version number for this migration
    fn version(&self) -> i64;

    /// Human-readable name for this migration
    fn name(&self) -> &str;

    /// Apply the migration (upgrade schema)
    fn up(&self, conn: &Connection) -> Result<()>;

    /// Rollback the migration (downgrade schema)
    fn down(&self, conn: &Connection) -> Result<()>;
}

/// Database migrator that tracks and applies migrations
pub struct Migrator {
    conn: Connection,
}

impl Migrator {
    /// Create a new migrator for the database at the given path
    ///
    /// Initializes the migration tracking table if it doesn't exist.
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open database for migration")?;

        // Initialize migration tracking table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            )",
            [],
        )
        .context("Failed to create schema_migrations table")?;

        Ok(Self { conn })
    }

    /// Get the current migration version
    pub fn current_version(&self) -> i64 {
        self.conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
    }

    /// Check if a migration has been applied
    pub fn is_applied(&self, migration: &dyn Migration) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
            [migration.version()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Apply a migration if not already applied
    ///
    /// Wraps the migration in a transaction for safety.
    pub fn apply_migration(&mut self, migration: &dyn Migration) -> Result<()> {
        if self.is_applied(migration)? {
            log::info!(
                "Migration {} already applied, skipping",
                migration.version()
            );
            return Ok(());
        }

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction for migration")?;

        log::info!(
            "Applying migration {}: {}",
            migration.version(),
            migration.name()
        );

        // Apply the migration
        migration.up(&tx).with_context(|| {
            format!(
                "Migration {} ({}) failed",
                migration.version(),
                migration.name()
            )
        })?;

        // Record the migration
        tx.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            params![
                migration.version(),
                migration.name(),
                chrono::Utc::now().timestamp()
            ],
        )
        .context("Failed to record migration")?;

        tx.commit()
            .context("Failed to commit migration transaction")?;

        log::info!("Migration {} applied successfully", migration.version());
        Ok(())
    }

    /// Rollback a migration (applies the down migration)
    pub fn rollback_migration(&mut self, migration: &dyn Migration) -> Result<()> {
        if !self.is_applied(migration)? {
            return Err(anyhow::anyhow!(
                "Migration {} is not applied",
                migration.version()
            ));
        }

        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to start transaction for rollback")?;

        log::info!(
            "Rolling back migration {}: {}",
            migration.version(),
            migration.name()
        );

        // Apply the rollback
        migration
            .down(&tx)
            .with_context(|| format!("Rollback of migration {} failed", migration.version()))?;

        // Remove migration record
        tx.execute(
            "DELETE FROM schema_migrations WHERE version = ?1",
            [migration.version()],
        )
        .context("Failed to remove migration record")?;

        tx.commit()
            .context("Failed to commit rollback transaction")?;

        log::info!("Migration {} rolled back successfully", migration.version());
        Ok(())
    }

    /// Get the connection for direct database access during migration
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get mutable connection for direct database access during migration
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_migrator_creates_tracking_table() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let migrator = Migrator::new(&db_path).unwrap();

        // Check that the table exists
        let table_exists: i64 = migrator.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(table_exists, 1);
    }

    #[test]
    fn test_initial_version_is_zero() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let migrator = Migrator::new(&db_path).unwrap();
        assert_eq!(migrator.current_version(), 0);
    }
}
