//! Migration v002: Add groups table and group_id to records

use crate::db::Migration;
use anyhow::Result;
use rusqlite::Connection;

pub struct V002AddGroups;

impl Migration for V002AddGroups {
    fn version(&self) -> i64 {
        2
    }

    fn name(&self) -> &str {
        "add_groups_table"
    }

    fn up(&self, conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
                sort_order INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(name, parent_id)
            );
            CREATE INDEX IF NOT EXISTS idx_groups_parent ON groups(parent_id);"
        )?;

        // Add group_id to records if not exists
        let has_col: bool = conn
            .prepare("PRAGMA table_info(records)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .any(|col| col == "group_id");

        if !has_col {
            conn.execute(
                "ALTER TABLE records ADD COLUMN group_id TEXT REFERENCES groups(id) ON DELETE SET NULL",
                [],
            )?;
        }

        Ok(())
    }

    fn down(&self, conn: &Connection) -> Result<()> {
        conn.execute_batch("DROP TABLE IF EXISTS groups;")?;
        // Note: SQLite doesn't support DROP COLUMN; group_id stays but is unused
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Migration;
    use tempfile::TempDir;

    #[test]
    fn test_v002_migration_applies_groups_table() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = crate::db::schema::initialize_database(&db_path).unwrap();

        // Migration should apply cleanly (even though schema.rs already creates the table)
        let migration = V002AddGroups;
        migration.up(&conn).unwrap();

        // Verify groups table works
        conn.execute(
            "INSERT INTO groups (id, name, created_at, updated_at) VALUES ('g1', 'Test', 0, 0)",
            [],
        ).unwrap();

        let name: String = conn.query_row(
            "SELECT name FROM groups WHERE id = 'g1'", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(name, "Test");
    }

    #[test]
    fn test_v002_migration_version() {
        let m = V002AddGroups;
        assert_eq!(m.version(), 2);
        assert_eq!(m.name(), "add_groups_table");
    }
}
