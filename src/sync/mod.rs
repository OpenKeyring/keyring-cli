pub mod export;
pub mod import;
pub mod conflict;

pub use export::SyncExporter;
pub use import::SyncImporter;
pub use conflict::{ConflictResolution, ConflictResolver};

pub enum SyncStatus {
    Idle,
    Syncing,
    Completed,
    Error(String),
}

pub struct SyncConfig {
    pub enabled: bool,
    pub provider: String,
    pub remote_path: String,
    pub auto_sync: bool,
    pub conflict_resolution: ConflictResolution,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "icloud".to_string(),
            remote_path: "/OpenKeyring".to_string(),
            auto_sync: false,
            conflict_resolution: ConflictResolution::Newer,
        }
    }
}