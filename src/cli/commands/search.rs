use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::Vault;
use crate::error::Result;
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
    let records = vault.search_records(&args.query)?;

    if records.is_empty() {
        println!("🔍 No records found matching '{}'", args.query);
    } else {
        println!("🔍 Found {} records matching '{}':", records.len(), args.query);
        for record in records {
            println!("  - {}", record.id);
        }
    }

    Ok(())
}
