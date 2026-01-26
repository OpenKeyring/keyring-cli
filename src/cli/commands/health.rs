use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::DatabaseManager;
use crate::crypto::CryptoManager;
use crate::health::{HealthChecker, HealthReport};
use crate::error::{KeyringError, Result};
use std::collections::HashMap;

#[derive(Parser, Debug)]
pub struct HealthArgs {
    /// Check for leaked passwords (HIBP API)
    #[clap(long, short)]
    pub leaks: bool,

    /// Check for weak passwords
    #[clap(long, short)]
    pub weak: bool,

    /// Check for duplicate passwords
    #[clap(long, short)]
    pub duplicate: bool,

    /// Run all checks
    #[clap(long, short)]
    pub all: bool,
}

pub async fn check_health(args: HealthArgs) -> Result<()> {
    println!("🩺 Running password health check...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let mut db = DatabaseManager::new(&db_config.path)?;

    // Initialize crypto manager (prompt for master password if needed)
    let mut crypto = CryptoManager::new();

    // Check if vault is initialized
    {
        let conn = db.connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM sqlite_master WHERE name='records'")?;
        let count: i64 = stmt.query_row((), |row| row.get(0))?;
        if count == 0 {
            println!("❌ Vault not initialized. Run 'ok init' first.");
            return Err(KeyringError::NotFound {
                resource: "Vault not initialized".to_string(),
            });
        }
    }

    // Prompt for master password
    let password = rpassword::prompt_password("Enter master password: ")?;
    crypto.initialize(&password)?;

    // Get all records from database
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, record_type, encrypted_data, nonce, tags, created_at, updated_at
         FROM records WHERE deleted = 0"
    )?;

    let records_vec = stmt.query_map((), |row| {
        use crate::db::models::{RecordType, StoredRecord};
        use chrono::DateTime;

        // Parse UUID from string
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
        })
    })?;

    let mut records = Vec::new();
    for record in records_vec {
        records.push(record?);
    }

    if records.is_empty() {
        println!("📭 No passwords found in vault.");
        return Ok(());
    }

    println!("📊 Checking {} records...\n", records.len());

    // Determine which checks to run
    let run_weak = args.all || args.weak;
    let run_duplicates = args.all || args.duplicate;
    let run_leaks = args.all || args.leaks;

    if !run_weak && !run_duplicates && !run_leaks {
        println!("⚠️  No checks selected. Use --weak, --duplicate, --leaks, or --all");
        return Ok(());
    }

    // Create health checker and run checks
    let checker = HealthChecker::new(crypto)
        .with_weak(run_weak)
        .with_duplicates(run_duplicates)
        .with_leaks(run_leaks);

    let issues = checker.check_all(&records).await;
    let report = HealthReport::from_issues(records.len(), issues);

    // Display results
    print_health_report(&report, run_weak, run_duplicates, run_leaks);

    Ok(())
}

fn print_health_report(report: &HealthReport, show_weak: bool, show_dupes: bool, show_leaks: bool) {
    // Print summary
    println!("Health Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total records checked: {}", report.total_records);

    let mut _total_issues = 0;

    if show_weak {
        println!("Weak passwords:       {}", report.weak_password_count);
        _total_issues += report.weak_password_count;
    }

    if show_dupes {
        println!("Duplicate passwords:  {}", report.duplicate_password_count);
        _total_issues += report.duplicate_password_count;
    }

    if show_leaks {
        println!("Compromised:          {}", report.compromised_password_count);
        _total_issues += report.compromised_password_count;
    }

    println!();

    if report.is_healthy() {
        println!("✅ All passwords are healthy!");
        return;
    }

    // Group issues by severity
    let mut by_severity: HashMap<String, Vec<_>> = HashMap::new();
    for issue in &report.issues {
        let severity = format!("{:?}", issue.severity);
        by_severity.entry(severity).or_insert_with(Vec::new).push(issue);
    }

    // Display issues by severity
    for severity in ["Critical", "High", "Medium", "Low"] {
        if let Some(issues) = by_severity.get(severity) {
            println!("{} Issues:", severity);
            for issue in issues {
                let icon = match issue.severity {
                    crate::health::report::Severity::Critical => "🔴",
                    crate::health::report::Severity::High => "🟠",
                    crate::health::report::Severity::Medium => "🟡",
                    crate::health::report::Severity::Low => "🟢",
                };
                println!("  {} {} - {}", icon, issue.record_names.join(", "), issue.description);
            }
            println!();
        }
    }

    // Print recommendations
    println!("Recommendations:");

    if report.weak_password_count > 0 {
        println!("  • Update weak passwords to improve security");
        println!("    Use: ok generate random -n <name> -l 16");
    }

    if report.duplicate_password_count > 0 {
        println!("  • Use unique passwords for each account");
        println!("    Use: ok generate random -n <name> -l 20");
    }

    if report.compromised_password_count > 0 {
        println!("  • Change compromised passwords immediately!");
        println!("    These passwords have been found in data breaches.");
    }

    println!();
}
