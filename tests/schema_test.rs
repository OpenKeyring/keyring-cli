use keyring_cli::db::schema;
use tempfile::TempDir;

#[test]
fn test_mcp_sessions_table_exists() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let conn = schema::initialize_database(&db_path).unwrap();

    // Check that mcp_sessions table exists
    let table_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mcp_sessions'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(table_exists, 1, "mcp_sessions table should exist");
}

#[test]
fn test_mcp_sessions_table_schema() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let conn = schema::initialize_database(&db_path).unwrap();

    // Insert a test session
    let session_id = "test-session-123";
    let approved_credentials = r#"["cred-1", "cred-2"]"#;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO mcp_sessions (id, approved_credentials, created_at, last_activity, ttl_seconds)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (session_id, approved_credentials, now, now, 3600),
    ).unwrap();

    // Verify the data
    let (id, creds, created, last_activity, ttl): (String, String, i64, i64, i64) = conn
        .query_row(
            "SELECT id, approved_credentials, created_at, last_activity, ttl_seconds
         FROM mcp_sessions WHERE id = ?1",
            [session_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();

    assert_eq!(id, session_id);
    assert_eq!(creds, approved_credentials);
    assert_eq!(ttl, 3600);
}

#[test]
fn test_mcp_policies_table_exists() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let conn = schema::initialize_database(&db_path).unwrap();

    // Check that mcp_policies table exists
    let table_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mcp_policies'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(table_exists, 1, "mcp_policies table should exist");
}

#[test]
fn test_mcp_policies_table_schema() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let conn = schema::initialize_database(&db_path).unwrap();

    // Insert a test policy
    let credential_id = "cred-123";
    let tag = "env:prod";
    let authz_mode = "confirm";
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO mcp_policies (credential_id, tag, authz_mode, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        (credential_id, tag, authz_mode, now),
    )
    .unwrap();

    // Try to insert duplicate (should fail due to primary key)
    let result = conn.execute(
        "INSERT INTO mcp_policies (credential_id, tag, authz_mode, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        (credential_id, tag, "auto", now),
    );
    assert!(result.is_err(), "Duplicate policy should fail");

    // Verify the data
    let (cred_id, tag_out, mode): (String, String, String) = conn
        .query_row(
            "SELECT credential_id, tag, authz_mode FROM mcp_policies
         WHERE credential_id = ?1 AND tag = ?2",
            (credential_id, tag),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();

    assert_eq!(cred_id, credential_id);
    assert_eq!(tag_out, tag);
    assert_eq!(mode, authz_mode);
}
