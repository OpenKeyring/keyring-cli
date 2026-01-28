use crate::cli::ConfigManager;
use crate::db::{models::RecordType, Vault};
use crate::error::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct SearchArgs {
    pub query: String,
    #[clap(short, long)]
    pub r#type: Option<String>,
    #[clap(short, long)]
    pub tags: Vec<String>,
    #[clap(short, long)]
    pub limit: Option<usize>,
}

pub async fn search_records(args: SearchArgs) -> Result<()> {
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    let vault = Vault::open(&db_path, "")?;
    let mut records = vault.search_records(&args.query)?;

    // Apply type filter
    if let Some(ref type_str) = args.r#type {
        let filter_type = match type_str.as_str() {
            "password" => RecordType::Password,
            "ssh_key" | "ssh-key" | "ssh" => RecordType::SshKey,
            "api_key" | "api-key" | "apicredential" => RecordType::ApiCredential,
            "mnemonic" => RecordType::Mnemonic,
            "private_key" | "private-key" | "key" => RecordType::PrivateKey,
            _ => {
                println!("⚠️  Unknown record type: {}", type_str);
                return Ok(());
            }
        };
        records.retain(|r| r.record_type == filter_type);
    }

    // Apply tags filter (records must have ALL specified tags)
    if !args.tags.is_empty() {
        records.retain(|r| args.tags.iter().all(|tag| r.tags.contains(tag)));
    }

    // Apply limit
    if let Some(limit) = args.limit {
        records.truncate(limit);
    }

    if records.is_empty() {
        println!("🔍 No records found matching '{}'", args.query);
    } else {
        println!(
            "🔍 Found {} records matching '{}':",
            records.len(),
            args.query
        );
        for record in records {
            println!("  - {}", record.id);
        }
    }

    Ok(())
}
