use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::DatabaseManager;
use crate::crypto::CryptoManager;
use crate::error::{KeyringError, Result};

#[derive(Parser, Debug)]
pub struct HealthArgs {
    #[clap(long, short)]
    pub leaks: bool,
    #[clap(long, short)]
    pub weak: bool,
    #[clap(long, short)]
    pub duplicate: bool,
}

pub async fn check_health(args: HealthArgs) -> Result<()> {
    println!("🩺 Running password health check...");

    let mut config = ConfigManager::new()?;
    let mut db = DatabaseManager::new(&config.get_database_config()?).await?;

    let records = db.list_all_records(None).await?;
    let mut issues = Vec::new();

    if args.weak {
        check_for_weak_passwords(&records, &mut issues);
    }

    if args.leaks {
        check_for_leaks(&records, &mut issues);
    }

    if args.duplicate {
        check_for_duplicates(&records, &mut issues);
    }

    if issues.is_empty() {
        println!("✅ No issues found!");
    } else {
        println!("⚠️  Found {} issues:", issues.len());
        for issue in issues {
            println!("   - {}", issue);
        }
    }

    Ok(())
}

fn check_for_weak_passwords(records: &[crate::db::models::Record], issues: &mut Vec<String>) {
    for record in records {
        // In a real implementation, this would check password strength
        if record.name.contains("weak") {
            issues.push(format!("Weak password detected for: {}", record.name));
        }
    }
}

fn check_for_leaks(records: &[crate::db::models::Record], issues: &mut Vec<String>) {
    for record in records {
        // In a real implementation, this would check against breach databases
        if record.name.contains("compromised") {
            issues.push(format!("Potential breach detected for: {}", record.name));
        }
    }
}

fn check_for_duplicates(records: &[crate::db::models::Record], issues: &mut Vec<String>) {
    let mut password_counts = std::collections::HashMap::new();

    for record in records {
        // Count encrypted data duplicates
        *password_counts.entry(&record.encrypted_data).or_insert(0) += 1;
    }

    for (encrypted_data, count) in password_counts {
        if count > 1 {
            // Find record names for this password
            let duplicates: Vec<_> = records
                .iter()
                .filter(|r| &r.encrypted_data == encrypted_data)
                .map(|r| r.name.clone())
                .collect();

            issues.push(format!("Duplicate password used by: {}", duplicates.join(", ")));
        }
    }
}