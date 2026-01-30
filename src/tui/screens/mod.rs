//! TUI Screens
//!
//! Individual screen implementations for the TUI mode.

pub mod conflict;
pub mod help;
pub mod provider_config;
pub mod provider_select;
pub mod settings;

pub use conflict::ConflictResolutionScreen;
pub use help::{HelpSection, HelpScreen, Shortcut};
pub use provider_config::{ConfigField, ProviderConfig, ProviderConfigScreen};
pub use provider_select::{Provider, ProviderSelectScreen};
pub use settings::{SettingsAction, SettingsItem, SettingsScreen, SettingsSection};
