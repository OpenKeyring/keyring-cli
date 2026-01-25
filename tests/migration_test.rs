use keyring_cli::db::migration::{Migration, Migrator};
use tempfile::TempDir;

#[test]
fn test_migration_version_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut migrator = Migrator::new(&db_path).unwrap();

    // Initial version should be 0 (no migrations applied)
    assert_eq!(migrator.current_version(), 0);

    // Apply a test migration
    migrator.apply_migration(&TestMigration).unwrap();

    // Version should now be 1
    assert_eq!(migrator.current_version(), 1);
}

struct TestMigration;

impl Migration for TestMigration {
    fn version(&self) -> i64 {
        1
    }

    fn name(&self) -> &str {
        "test_migration"
    }

    fn up(&self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        // Create a test table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test_table (id INTEGER PRIMARY KEY)",
            [],
        )?;
        Ok(())
    }

    fn down(&self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        conn.execute("DROP TABLE IF EXISTS test_table", [])?;
        Ok(())
    }
}
