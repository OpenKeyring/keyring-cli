use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::models::{StoredRecord, RecordType};
use crate::error::Result;
use crate::cli::utils::PrettyPrinter;

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
    let mut config = ConfigManager::new()?;
    let mut db = crate::db::DatabaseManager::new(&config.get_database_config()?).await?;

    let records = if args.r#type.is_some() {
        let record_type = RecordType::from(args.r#type.unwrap());
        db.list_records_by_type(record_type, args.limit).await?
    } else {
        db.list_all_records(args.limit).await?
    };

    // Filter by tags if specified
    let mut filtered_records = records;
    if !args.tags.is_empty() {
        filtered_records = records.into_iter()
            .filter(|record| {
                args.tags.iter().all(|tag| record.tags.contains(tag))
            })
            .collect();
    }

    PrettyPrinter::print_records(&filtered_records);

    Ok(())
}