//! Group CRUD operations for the vault

use anyhow::Result;
use rusqlite::Connection;
use uuid::Uuid;

/// Stored group record from database
#[derive(Debug, Clone)]
pub struct StoredGroup {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Create a new group
pub fn create_group(conn: &mut Connection, name: &str) -> Result<StoredGroup> {
    // Enforce name uniqueness at the application level (SQLite NULL != NULL in UNIQUE)
    if group_name_exists(conn, name)? {
        anyhow::bail!("Group name already exists: {}", name);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    // Get next sort_order
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM groups",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    conn.execute(
        "INSERT INTO groups (id, name, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, name, max_order + 1, now, now],
    )?;

    Ok(StoredGroup {
        id,
        name: name.to_string(),
        parent_id: None,
        sort_order: max_order + 1,
        created_at: now,
        updated_at: now,
    })
}

/// Rename a group
pub fn rename_group(conn: &Connection, id: &str, new_name: &str) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let rows = conn.execute(
        "UPDATE groups SET name = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_name, now, id],
    )?;
    if rows == 0 {
        anyhow::bail!("Group not found: {}", id);
    }
    Ok(())
}

/// Delete a group (passwords move to ungrouped via ON DELETE SET NULL)
pub fn delete_group(conn: &Connection, id: &str) -> Result<()> {
    let rows = conn.execute("DELETE FROM groups WHERE id = ?1", [id])?;
    if rows == 0 {
        anyhow::bail!("Group not found: {}", id);
    }
    Ok(())
}

/// List all groups ordered by sort_order, name
pub fn list_groups(conn: &Connection) -> Result<Vec<StoredGroup>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, sort_order, created_at, updated_at
         FROM groups ORDER BY sort_order, name",
    )?;

    let groups = stmt
        .query_map([], |row| {
            Ok(StoredGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(groups)
}

/// Move a password to a group (None = ungrouped)
pub fn move_password_to_group(
    conn: &Connection,
    password_id: &str,
    group_id: Option<&str>,
) -> Result<()> {
    let rows = conn.execute(
        "UPDATE records SET group_id = ?1 WHERE id = ?2 AND deleted = 0",
        rusqlite::params![group_id, password_id],
    )?;
    if rows == 0 {
        anyhow::bail!("Record not found or deleted: {}", password_id);
    }
    Ok(())
}

/// Check if a group name already exists
pub fn group_name_exists(conn: &Connection, name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM groups WHERE name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_db() -> (Connection, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = crate::db::schema::initialize_database(&db_path).unwrap();
        (conn, temp_dir)
    }

    #[test]
    fn test_create_group() {
        let (mut conn, _dir) = setup_db();
        let group = create_group(&mut conn, "Work").unwrap();
        assert_eq!(group.name, "Work");
        assert!(!group.id.is_empty());
        assert_eq!(group.sort_order, 0);
    }

    #[test]
    fn test_list_groups_ordered() {
        let (mut conn, _dir) = setup_db();
        create_group(&mut conn, "Beta").unwrap();
        create_group(&mut conn, "Alpha").unwrap();

        let groups = list_groups(&conn).unwrap();
        assert_eq!(groups.len(), 2);
        // Ordered by sort_order (creation order), not name
        assert_eq!(groups[0].name, "Beta");
        assert_eq!(groups[1].name, "Alpha");
    }

    #[test]
    fn test_rename_group() {
        let (mut conn, _dir) = setup_db();
        let group = create_group(&mut conn, "Old Name").unwrap();
        rename_group(&conn, &group.id, "New Name").unwrap();

        let groups = list_groups(&conn).unwrap();
        assert_eq!(groups[0].name, "New Name");
    }

    #[test]
    fn test_delete_group() {
        let (mut conn, _dir) = setup_db();
        let group = create_group(&mut conn, "ToDelete").unwrap();
        delete_group(&conn, &group.id).unwrap();

        let groups = list_groups(&conn).unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_duplicate_name_rejected() {
        let (mut conn, _dir) = setup_db();
        create_group(&mut conn, "Unique").unwrap();
        let result = create_group(&mut conn, "Unique");
        assert!(result.is_err(), "Duplicate group name should fail");
    }

    #[test]
    fn test_group_name_exists() {
        let (mut conn, _dir) = setup_db();
        create_group(&mut conn, "Exists").unwrap();
        assert!(group_name_exists(&conn, "Exists").unwrap());
        assert!(!group_name_exists(&conn, "Nope").unwrap());
    }
}
