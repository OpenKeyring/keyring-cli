//! Type definitions for TUI application
//!
//! Contains error types, screen enumeration, and sync status.

use chrono::{DateTime, Utc};

/// TUI-specific error type
#[derive(Debug)]
pub enum TuiError {
    /// Terminal initialization failed
    InitFailed(String),
    /// Terminal restore failed
    RestoreFailed(String),
    /// I/O error
    IoError(String),
}

impl std::fmt::Display for TuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuiError::InitFailed(msg) => write!(f, "TUI init failed: {}", msg),
            TuiError::RestoreFailed(msg) => write!(f, "TUI restore failed: {}", msg),
            TuiError::IoError(msg) => write!(f, "TUI I/O error: {}", msg),
        }
    }
}

impl std::error::Error for TuiError {}

/// TUI result type
pub type TuiResult<T> = std::result::Result<T, TuiError>;

/// Current active screen in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Main command screen
    Main,
    /// Settings screen (F2)
    Settings,
    /// Provider selection screen
    ProviderSelect,
    /// Provider configuration screen
    ProviderConfig,
    /// Help screen (? or F1)
    Help,
    /// Conflict resolution screen
    ConflictResolution,
    /// Sync screen
    Sync,
    /// Trash screen (deleted passwords)
    Trash,
    /// Onboarding wizard screen
    Wizard,
    /// Unlock screen (enter master password)
    Unlock,
    /// New password screen
    NewPassword,
    /// Edit password screen
    EditPassword,
}

impl Screen {
    /// Get the display name for this screen
    pub fn name(&self) -> &str {
        match self {
            Screen::Main => "Main",
            Screen::Settings => "Settings",
            Screen::ProviderSelect => "Provider Select",
            Screen::ProviderConfig => "Provider Config",
            Screen::Help => "Help",
            Screen::ConflictResolution => "Conflict Resolution",
            Screen::Sync => "Sync",
            Screen::Trash => "Trash",
            Screen::Wizard => "Onboarding Wizard",
            Screen::Unlock => "Unlock",
            Screen::NewPassword => "New Password",
            Screen::EditPassword => "Edit Password",
        }
    }
}

/// Sync status for the statusline
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SyncStatus {
    /// Last sync time
    Synced(DateTime<Utc>),
    /// Not synced
    Unsynced,
    /// Currently syncing
    Syncing,
    /// Sync failed with error message
    Failed(String),
}

impl SyncStatus {
    /// Get display text for sync status
    pub fn display(&self) -> String {
        match self {
            SyncStatus::Synced(dt) => {
                let now = Utc::now();
                let duration = now.signed_duration_since(*dt);
                let mins = duration.num_minutes();
                if mins < 1 {
                    "Just now".to_string()
                } else if mins < 60 {
                    format!("{}m ago", mins)
                } else {
                    let hours = mins / 60;
                    if hours < 24 {
                        format!("{}h ago", hours)
                    } else {
                        let days = hours / 24;
                        format!("{}d ago", days)
                    }
                }
            }
            SyncStatus::Unsynced => "Unsynced".to_string(),
            SyncStatus::Syncing => "Syncing...".to_string(),
            SyncStatus::Failed(msg) => format!("Sync failed: {}", msg),
        }
    }
}
