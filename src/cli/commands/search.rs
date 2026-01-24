use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::DatabaseManager;
use crate::error::{KeyringError, Result};
use crate::cli::utils::PrettyPrinter;

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
    let mut config = ConfigManager::new()?;
    let mut db = DatabaseManager::new(&config.get_database_config()?).await?;

    let records = db.search_records(&args.query, args.r#type, args.tags, args.limit).await?;

    if records.is_empty() {
        println!("🔍 No records found matching '{}'", args.query);
    } else {
        println!("🔍 Found {} records matching '{}':", records.len(), args.query);
        PrettyPrinter::print_records(&records);
    }

    Ok(())
}