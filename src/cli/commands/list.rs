use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::Vault;
use crate::error::Result;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct ListArgs {
    #[clap(short, long)]
    pub r#type: Option<String>,
    #[clap(short, long)]
    pub tags: Vec<String>,
    #[clap(short, long)]
    pub limit: Option<usize>,
}

pub async fn list_records(args: ListArgs) -> Result<()> {
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    let vault = Vault::open(&db_path, "")?;
    let records = vault.list_records()?;

    // Filter by type if specified
    let filtered: Vec<_> = if let Some(type_str) = args.r#type {
        let record_type = crate::db::models::RecordType::from(type_str);
        records.into_iter()
            .filter(|r| r.record_type == record_type)
            .collect()
    } else {
        records.into_iter().collect()
    };

    // Filter by tags if specified
    let filtered: Vec<_> = if !args.tags.is_empty() {
        filtered.into_iter()
            .filter(|record| {
                args.tags.iter().all(|tag| record.tags.contains(tag))
            })
            .collect()
    } else {
        filtered
    };

    // Apply limit if specified
    let mut filtered: Vec<_> = filtered.into_iter().collect();
    if let Some(limit) = args.limit {
        filtered.truncate(limit);
    }

    if filtered.is_empty() {
        println!("📋 No records found");
    } else {
        println!("📋 Found {} records:", filtered.len());
        for record in filtered {
            println!("  - {} ({})", record.id,
                format!("{:?}", record.record_type).to_lowercase());
        }
    }

    Ok(())
}
