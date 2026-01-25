//! WAL (Write-Ahead Log) management for SQLite

use anyhow::Result;
use rusqlite::Connection;

/// Get the current WAL file size in bytes
///
/// Returns the size of the -wal file associated with the database.
pub fn get_wal_size(conn: &Connection) -> Result<u64> {
    // Try to get actual WAL file size
    let wal_size: i64 = conn.pragma_query_value(None, "wal_size(DATABASE)", |row| row.get(0))
        .unwrap_or(0);

    Ok(wal_size as u64)
}

/// Run a WAL checkpoint
///
/// A checkpoint moves frames from the WAL file back into the main database.
/// This is the standard checkpoint mode (TRUNCATE).
pub fn checkpoint(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
    Ok(())
}

/// Run an aggressive WAL checkpoint that truncates the WAL file
///
/// This mode:
/// 1. Writes all WAL frames back to the database
/// 2. Syncs the database file
/// 3. Truncates the WAL file to zero bytes
///
/// Use this when you want to minimize disk usage.
pub fn truncate(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
    Ok(())
}

/// Run a PASSIVE checkpoint
///
/// Only checkpoint if no readers are using the WAL. Safe to run while
/// other processes are reading the database.
pub fn checkpoint_passive(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "PASSIVE")?;
    Ok(())
}

/// Run a FULL checkpoint
///
/// Like TRUNCATE but doesn't truncate the WAL file. Use this when
/// you want to ensure consistency but don't need to reclaim space.
pub fn checkpoint_full(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "FULL")?;
    Ok(())
}

/// Run a RESTART checkpoint
///
/// Checkpoints and then restarts the WAL file. This is useful for
/// ensuring the database is in a consistent state for backup.
pub fn checkpoint_restart(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "RESTART")?;
    Ok(())
}

/// Get WAL auto-checkpoint setting
///
/// Returns the number of WAL frames before automatic checkpoint.
/// 0 means auto-checkpoint is disabled.
pub fn get_auto_checkpoint(conn: &Connection) -> Result<i64> {
    let value: i64 = conn.pragma_query_value(None, "wal_autocheckpoint", |row| row.get(0))?;
    Ok(value)
}

/// Set WAL auto-checkpoint threshold
///
/// Set to 0 to disable automatic checkpointing.
/// Recommended: 1000 (default) or higher for better performance.
pub fn set_auto_checkpoint(conn: &Connection, frames: i64) -> Result<()> {
    conn.pragma_update(None, "wal_autocheckpoint", frames)?;
    Ok(())
}

/// Disable automatic checkpointing
///
/// Manual checkpointing can be more efficient for bulk operations.
pub fn disable_auto_checkpoint(conn: &Connection) -> Result<()> {
    set_auto_checkpoint(conn, 0)
}

/// Re-enable automatic checkpointing with default value
pub fn enable_auto_checkpoint(conn: &Connection) -> Result<()> {
    set_auto_checkpoint(conn, 1000)
}
