use crate::db::models::DecryptedRecord;

pub struct PrettyPrinter;

impl PrettyPrinter {
    pub fn print_records(records: &[DecryptedRecord]) {
        if records.is_empty() {
            println!("📋 No records found");
            return;
        }

        println!("📋 Found {} records:", records.len());
        println!("{}", "─".repeat(80));

        for record in records {
            Self::print_single_record(record);
            println!("{}", "─".repeat(80));
        }
    }

    fn print_single_record(record: &DecryptedRecord) {
        println!("🔹 Name: {}", record.name);
        println!("📝 Type: {:?}", record.record_type);
        println!("🏷️  Tags: {}", if record.tags.is_empty() { "None" } else { record.tags.join(", ") });
        println!("📅 Created: {}", record.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("🔄 Updated: {}", record.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));

        if let Some(username) = &record.username {
            println!("👤 Username: {}", username);
        }

        if let Some(url) = &record.url {
            println!("🌐 URL: {}", url);
        }

        if let Some(notes) = &record.notes {
            println!("📄 Notes: {}", notes);
        }
    }

    pub fn print_record(record: &DecryptedRecord) {
        Self::print_single_record(record);
    }

    pub fn print_success(message: &str) {
        println!("✅ {}", message);
    }

    pub fn print_error(message: &str) {
        eprintln!("❌ {}", message);
    }

    pub fn print_warning(message: &str) {
        println!("⚠️  {}", message);
    }

    pub fn print_info(message: &str) {
        println!("ℹ️  {}", message);
    }
}