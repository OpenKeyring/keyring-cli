//! TUI Health Command Handler
//!
//! Handles the /health command in TUI mode for password health checks.

use crate::cli::{onboarding, ConfigManager};
use crate::db::DatabaseManager;
use crate::error::{KeyringError, Result};
use crate::health::{HealthChecker, HealthReport};
use std::path::PathBuf;

/// Handle the /health command
///
/// Supports flags: --weak, --duplicate, --leaks, --all
///
/// # Arguments
/// * `args` - Vector of command arguments (flags)
///
/// # Returns
/// * `Result<Vec<String>>` - Formatted output lines for TUI display
#[allow(dead_code)]
pub fn handle_health(args: Vec<&str>) -> Result<Vec<String>> {
    let mut output = vec!["Password Health Check".to_string(), "".to_string()];

    // Parse arguments
    let mut check_weak = false;
    let mut check_duplicates = false;
    let mut check_leaks = false;

    for arg in &args {
        match *arg {
            "--weak" | "-w" => check_weak = true,
            "--duplicate" | "-d" => check_duplicates = true,
            "--leaks" | "-l" => check_leaks = true,
            "--all" | "-a" => {
                check_weak = true;
                check_duplicates = true;
                check_leaks = true;
            }
            _ => {
                // Ignore unknown flags for now
            }
        }
    }

    // If no flags specified, show help message
    if !check_weak && !check_duplicates && !check_leaks {
        output.extend_from_slice(&[
            "No checks selected. Use one or more flags:".to_string(),
            "  --weak, -w       Check for weak passwords".to_string(),
            "  --duplicate, -d  Check for duplicate passwords".to_string(),
            "  --leaks, -l      Check for compromised passwords (HIBP)".to_string(),
            "  --all, -a        Run all checks".to_string(),
            "".to_string(),
            "Example: /health --all".to_string(),
        ]);
        return Ok(output);
    }

    // Initialize components
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path.clone());

    // Check if database exists
    if !db_path.exists() {
        output.push("Vault not initialized.".to_string());
        output.push("   Run 'ok init' first.".to_string());
        return Ok(output);
    }

    // Unlock keystore to decrypt records
    let crypto = match onboarding::unlock_keystore() {
        Ok(crypto) => crypto,
        Err(_) => {
            output.push("Error: Unable to unlock keystore.".to_string());
            output.push("       Make sure you have initialized your vault.".to_string());
            return Ok(output);
        }
    };

    // Open database and get records
    let db = match DatabaseManager::new(&db_config.path) {
        Ok(db) => db,
        Err(e) => {
            output.push(format!("Error: Unable to open database: {}", e));
            return Ok(output);
        }
    };

    let conn = match db.connection() {
        Ok(conn) => conn,
        Err(e) => {
            output.push(format!("Error: Unable to connect to database: {}", e));
            return Ok(output);
        }
    };

    // Check if records table exists
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM sqlite_master WHERE name='records'")?;
    let count: i64 = stmt.query_row((), |row| row.get(0))?;
    if count == 0 {
        output.push("No records found.".to_string());
        return Ok(output);
    }

    // Get all records from database
    let mut stmt = conn.prepare(
        "SELECT id, record_type, encrypted_data, nonce, tags, created_at, updated_at, version
         FROM records WHERE deleted = 0",
    )?;

    let records_vec = stmt.query_map((), |row| {
        use crate::db::models::{RecordType, StoredRecord};
        use chrono::DateTime;

        let id_str: String = row.get(0)?;
        let id = uuid::Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(StoredRecord {
            id,
            record_type: {
                let type_str: String = row.get(1)?;
                match type_str.as_str() {
                    "password" => RecordType::Password,
                    "ssh_key" => RecordType::SshKey,
                    "api_credential" => RecordType::ApiCredential,
                    "mnemonic" => RecordType::Mnemonic,
                    "private_key" => RecordType::PrivateKey,
                    _ => RecordType::Password,
                }
            },
            encrypted_data: row.get(2)?,
            nonce: {
                let nonce_bytes: Vec<u8> = row.get(3)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                nonce
            },
            tags: {
                let tags_str: String = row.get(4)?;
                if tags_str.is_empty() {
                    vec![]
                } else {
                    tags_str.split(',').map(|s| s.to_string()).collect()
                }
            },
            created_at: {
                let ts: i64 = row.get(5)?;
                DateTime::from_timestamp(ts, 0).unwrap_or_default()
            },
            updated_at: {
                let ts: i64 = row.get(6)?;
                DateTime::from_timestamp(ts, 0).unwrap_or_default()
            },
            version: {
                let v: i64 = row.get(7)?;
                v as u64
            },
        })
    })?;

    let mut records = Vec::new();
    for record in records_vec {
        records.push(record?);
    }

    if records.is_empty() {
        output.push("No passwords found in vault.".to_string());
        return Ok(output);
    }

    output.push(format!("Checking {} records...", records.len()));

    // Create health checker and run checks (using a simple blocking approach for TUI)
    let checker = HealthChecker::new(crypto)
        .with_weak(check_weak)
        .with_duplicates(check_duplicates)
        .with_leaks(check_leaks);

    // Run health checks (using tokio runtime for async)
    let issues = tokio::runtime::Runtime::new()
        .map_err(|e| KeyringError::IoError(format!("Failed to create runtime: {}", e)))?
        .block_on(checker.check_all(&records));

    let report = HealthReport::from_issues(records.len(), issues);

    // Format results for TUI display
    output.extend_from_slice(&format_health_report(
        &report,
        check_weak,
        check_duplicates,
        check_leaks,
    ));

    Ok(output)
}

/// Format health report for TUI display
fn format_health_report(
    report: &HealthReport,
    show_weak: bool,
    show_dupes: bool,
    show_leaks: bool,
) -> Vec<String> {
    let mut output = Vec::new();

    // Print summary
    output.push("--------------------------------------------------".to_string());
    output.push(format!("Total records checked: {}", report.total_records));
    output.push("".to_string());

    if show_weak {
        output.push(format!(
            "Weak passwords:       {}",
            report.weak_password_count
        ));
    }

    if show_dupes {
        output.push(format!(
            "Duplicate passwords:  {}",
            report.duplicate_password_count
        ));
    }

    if show_leaks {
        output.push(format!(
            "Compromised:          {}",
            report.compromised_password_count
        ));
    }

    output.push("".to_string());

    if report.is_healthy() {
        output.push("All passwords are healthy!".to_string());
        return output;
    }

    // Group issues by severity
    use std::collections::HashMap;
    let mut by_severity: HashMap<String, Vec<_>> = HashMap::new();
    for issue in &report.issues {
        let severity = format!("{:?}", issue.severity);
        by_severity
            .entry(severity)
            .or_insert_with(Vec::new)
            .push(issue);
    }

    // Display issues by severity
    for severity in ["Critical", "High", "Medium", "Low"] {
        if let Some(issues) = by_severity.get(severity) {
            output.push(format!("{} Issues:", severity));
            for issue in issues {
                let icon = match issue.severity {
                    crate::health::report::Severity::Critical => "[!]",
                    crate::health::report::Severity::High => "[+]",
                    crate::health::report::Severity::Medium => "[*]",
                    crate::health::report::Severity::Low => "[.]",
                };
                output.push(format!(
                    "  {} {} - {}",
                    icon,
                    issue.record_names.join(", "),
                    issue.description
                ));
            }
            output.push("".to_string());
        }
    }

    // Print recommendations
    output.push("Recommendations:".to_string());

    if report.weak_password_count > 0 {
        output.push("  - Update weak passwords to improve security".to_string());
        output.push("    Use: /new to create strong passwords".to_string());
    }

    if report.duplicate_password_count > 0 {
        output.push("  - Use unique passwords for each account".to_string());
    }

    if report.compromised_password_count > 0 {
        output.push("  - Change compromised passwords immediately!".to_string());
        output.push("    These passwords have been found in data breaches.".to_string());
    }

    output
}
