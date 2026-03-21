//! TUI Application State and Logic
//!
//! Core TUI application handling alternate screen mode, rendering, and event loop.
//!
//! ## Module Structure
//!
//! - [`types`] - Type definitions (Screen, SyncStatus, TuiError)
//! - [`wizard`] - Onboarding wizard handling
//! - [`unlock`] - Vault unlock handling
//! - [`events`] - Keyboard event and command processing
//! - [`render`] - UI rendering methods
//! - [`terminal`] - Terminal lifecycle management

mod events;
mod render;
mod terminal;
mod types;
mod unlock;
mod wizard;

// Re-export public types
pub use types::{Screen, SyncStatus, TuiError};

// Re-export entry point
pub use terminal::run_tui;

use crate::tui::keybindings::KeyBindingManager;
use crate::tui::screens::wizard::WizardState;
use crate::tui::screens::{
    ClipboardTimeoutScreen, EditPasswordScreen, HelpScreen, MainScreen, MasterPasswordScreen,
    NewPasswordScreen, PasskeyGenerateScreen, PasskeyImportScreen, PasskeyVerifyScreen,
    PasswordPolicyScreen, SecurityNoticeScreen, SettingsScreen, SyncScreen, TrashRetentionScreen,
    TrashScreen, UnlockScreen, WelcomeScreen,
};
use crate::tui::state::AppState;

/// TUI Application State
pub struct TuiApp {
    /// Running state
    running: bool,
    /// Keybinding manager
    keybinding_manager: KeyBindingManager,
    /// Lock status
    locked: bool,
    /// Record count
    record_count: usize,
    /// Sync status
    sync_status: SyncStatus,
    /// Version string
    version: String,
    /// Current active screen
    pub current_screen: Screen,
    /// Wizard state (if in onboarding wizard)
    pub wizard_state: Option<WizardState>,
    /// Welcome screen (wizard step 1)
    pub welcome_screen: WelcomeScreen,
    /// Passkey generation screen (wizard step 2)
    pub passkey_generate_screen: PasskeyGenerateScreen,
    /// Passkey import screen (wizard step 2 alt)
    pub passkey_import_screen: PasskeyImportScreen,
    /// Passkey verification screen (wizard step - verify 3 random positions)
    pub passkey_verify_screen: Option<PasskeyVerifyScreen>,
    /// Security notice screen
    pub security_notice_screen: SecurityNoticeScreen,
    /// Password policy screen
    pub password_policy_screen: PasswordPolicyScreen,
    /// Clipboard timeout screen
    pub clipboard_timeout_screen: ClipboardTimeoutScreen,
    /// Trash retention screen
    pub trash_retention_screen: TrashRetentionScreen,
    /// Master password screen (wizard step 4)
    pub master_password_screen: MasterPasswordScreen,
    /// Unlock screen (for existing users)
    pub unlock_screen: UnlockScreen,
    /// Sync screen
    sync_screen: Option<SyncScreen>,
    /// Application state (MVP)
    pub app_state: AppState,
    /// Main screen (MVP)
    pub main_screen: MainScreen,
    /// New password screen
    pub new_password_screen: NewPasswordScreen,
    /// Edit password screen
    pub edit_password_screen: EditPasswordScreen,
    /// Trash screen (deleted passwords)
    pub trash_screen: TrashScreen,
    /// Settings screen
    pub settings_screen: SettingsScreen,
    /// Help screen
    pub help_screen: HelpScreen,
    /// Confirmation dialog overlay (rendered on top of current screen)
    pub confirm_dialog: Option<crate::tui::components::ConfirmDialog>,
    /// Config directory path for persisting TUI settings
    config_dir: std::path::PathBuf,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    /// Create a new TUI application with default settings
    pub fn new() -> Self {
        Self::new_with_config_dir(None)
    }

    /// Create a new TUI application with a specific config directory
    pub fn new_with_config_dir(config_dir: Option<std::path::PathBuf>) -> Self {
        use std::path::PathBuf;

        // Determine config directory
        let config_dir = config_dir.unwrap_or_else(|| {
            dirs::config_dir()
                .map(|p| p.join("open-keyring"))
                .unwrap_or_else(|| PathBuf::from(".open-keyring"))
        });

        // Load TUI config from disk
        let config = crate::tui::config::TuiConfig::load(&config_dir).unwrap_or_default();

        // Initialize app state with loaded config
        let mut app_state = AppState::new();
        app_state.config = config;
        app_state.apply_filter();

        Self {
            running: true,
            keybinding_manager: KeyBindingManager::new(),
            locked: false,
            record_count: 0,
            sync_status: SyncStatus::Unsynced,
            version: env!("CARGO_PKG_VERSION").to_string(),
            current_screen: Screen::Main,
            wizard_state: None,
            welcome_screen: WelcomeScreen::new(),
            passkey_generate_screen: PasskeyGenerateScreen::new(),
            passkey_import_screen: PasskeyImportScreen::new(),
            passkey_verify_screen: None,
            security_notice_screen: SecurityNoticeScreen::new(),
            password_policy_screen: PasswordPolicyScreen::new(),
            clipboard_timeout_screen: ClipboardTimeoutScreen::new(),
            trash_retention_screen: TrashRetentionScreen::new(),
            master_password_screen: MasterPasswordScreen::new(),
            unlock_screen: UnlockScreen::new(),
            sync_screen: Some(SyncScreen::new()),
            app_state,
            main_screen: MainScreen::new(),
            new_password_screen: NewPasswordScreen::new(),
            edit_password_screen: EditPasswordScreen::empty(),
            trash_screen: TrashScreen::new(),
            settings_screen: SettingsScreen::new(),
            help_screen: HelpScreen::new(),
            confirm_dialog: None,
            config_dir,
        }
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    /// Save current TUI configuration to disk
    pub fn save_config(&self) -> std::io::Result<()> {
        self.app_state.config.save(&self.config_dir)
    }

    /// Get the current screen
    pub fn current_screen(&self) -> Screen {
        self.current_screen
    }

    /// Navigate to a different screen
    pub fn navigate_to(&mut self, screen: Screen) {
        self.current_screen = screen;
    }

    /// Return to the main screen
    pub fn return_to_main(&mut self) {
        self.current_screen = Screen::Main;
    }

    /// Check if the app is still running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the application
    pub fn quit(&mut self) {
        // Save config before quitting
        if let Err(e) = self.save_config() {
            eprintln!("Warning: Failed to save TUI config: {}", e);
        }
        self.running = false;
    }

}

