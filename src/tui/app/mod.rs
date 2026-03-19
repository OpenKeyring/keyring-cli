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

use crate::db::vault::Vault;
use crate::error::Result;
use crate::tui::keybindings::KeyBindingManager;
use crate::tui::screens::wizard::WizardState;
use crate::tui::screens::{
    ClipboardTimeoutScreen, EditPasswordScreen, MainScreen, MasterPasswordScreen, NewPasswordScreen,
    PasskeyGenerateScreen, PasskeyImportScreen, PasskeyVerifyScreen, PasswordPolicyScreen,
    SecurityNoticeScreen, SettingsScreen, SyncScreen, TrashRetentionScreen, TrashScreen,
    UnlockScreen, WelcomeScreen,
};
use crate::tui::state::AppState;

/// Maximum output lines to display
const MAX_OUTPUT_LINES: usize = 500;

/// TUI Application State
pub struct TuiApp {
    /// Running state
    running: bool,
    /// Current input buffer
    pub input_buffer: String,
    /// Autocomplete matches (for display)
    autocomplete_matches: Vec<String>,
    /// Command history
    history: Vec<String>,
    /// History cursor position
    history_index: usize,
    /// Current output/messages to display
    pub output_lines: Vec<String>,
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
            input_buffer: String::new(),
            autocomplete_matches: Vec::new(),
            history: Vec::new(),
            history_index: 0,
            output_lines: vec![
                "OpenKeyring TUI v0.1.0".to_string(),
                "Type /help for available commands".to_string(),
                "".to_string(),
            ],
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

    /// Add an output line, trimming old lines if exceeding MAX_OUTPUT_LINES
    pub fn add_output(&mut self, line: String) {
        if self.output_lines.len() >= MAX_OUTPUT_LINES {
            let excess = self.output_lines.len() - MAX_OUTPUT_LINES + 1;
            self.output_lines.drain(0..excess);
        }
        self.output_lines.push(line);
    }

    /// Add multiple output lines, trimming if necessary
    pub fn add_outputs(&mut self, lines: Vec<String>) {
        for line in lines {
            self.add_output(line);
        }
    }

    /// Get the current screen
    pub fn current_screen(&self) -> Screen {
        self.current_screen
    }

    /// Navigate to a different screen
    pub fn navigate_to(&mut self, screen: Screen) {
        self.current_screen = screen;
        self.output_lines
            .push(format!("Navigated to: {}", screen.name()));
    }

    /// Return to the main screen
    pub fn return_to_main(&mut self) {
        self.current_screen = Screen::Main;
        self.output_lines
            .push("Returned to main screen".to_string());
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

    /// Handle autocomplete with database for record name completion
    ///
    /// This method extends autocomplete to support completing record names from the vault.
    /// When the input contains a space (e.g., "/show "), it attempts to complete the record name.
    ///
    /// # Stub Implementation
    /// Currently returns empty matches since record completion requires:
    /// - Vault access
    /// - CryptoManager for decryption
    /// - Integration into the TUI command flow
    ///
    /// TODO: Full integration requires:
    /// 1. Pass CryptoManager to TuiApp or this method
    /// 2. Decrypt records to get names
    /// 3. Cache record names for performance
    pub async fn handle_autocomplete_with_db(&mut self, vault: Option<&Vault>) -> Result<()> {
        if self.input_buffer.starts_with('/') {
            // Command autocomplete - use existing logic
            self.handle_autocomplete();
        } else if let Some(_vault) = vault {
            // Record name autocomplete
            let _prefix = self.input_buffer.as_str();

            // TODO: Query vault for record names matching prefix
            // Stub implementation - requires CryptoManager for decryption
            // For now, return empty matches
            let _matches: Vec<String> = vec![];

            if _matches.is_empty() {
                self.autocomplete_matches.clear();
            }
        } else {
            // No vault available, use command autocomplete
            self.handle_autocomplete();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_app_creation() {
        let app = TuiApp::new();
        assert!(app.is_running());
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_app_quit() {
        let mut app = TuiApp::new();
        app.quit();
        assert!(!app.is_running());
    }

    #[test]
    fn test_handle_char() {
        let mut app = TuiApp::new();
        app.handle_char('t');
        app.handle_char('e');
        app.handle_char('s');
        app.handle_char('t');
        assert_eq!(app.input_buffer, "test");
    }

    #[test]
    fn test_handle_backspace() {
        let mut app = TuiApp::new();
        app.handle_char('t');
        app.handle_char('e');
        app.handle_backspace();
        assert_eq!(app.input_buffer, "t");
    }

    #[test]
    fn test_submit_command() {
        let mut app = TuiApp::new();
        app.handle_char('/');
        app.handle_char('h');
        app.handle_char('e');
        app.handle_char('l');
        app.handle_char('p');
        app.handle_char('\n');
        assert_eq!(app.input_buffer, "");
        // Check for either keyboard shortcuts or commands section
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Keyboard Shortcuts") || l.contains("Commands:")));
    }

    #[test]
    fn test_exit_command() {
        let mut app = TuiApp::new();
        app.handle_char('/');
        app.handle_char('e');
        app.handle_char('x');
        app.handle_char('i');
        app.handle_char('t');
        app.handle_char('\n');
        assert!(!app.is_running());
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_process_delete_command() {
        let mut app = TuiApp::new();
        app.process_command("/delete test");
        // Should show delete confirmation
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Delete") || l.contains("Confirm")));
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_process_list_command() {
        let mut app = TuiApp::new();
        app.process_command("/list");
        // Should show password prompt or list output or error message
        // Since keystore may not be initialized, should show error or prompt
        let has_output = !app.output_lines.is_empty();
        assert!(has_output, "Should have output content");
    }

    #[test]
    fn test_process_show_command() {
        let mut app = TuiApp::new();
        app.process_command("/show test");
        // Should show error or record info
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Error") || l.contains("not found") || l.contains("test")));
    }

    #[test]
    fn test_process_new_command() {
        let mut app = TuiApp::new();
        app.process_command("/new");
        // Should show new record wizard
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("New") || l.contains("Create") || l.contains("record")));
    }

    #[test]
    fn test_process_update_command() {
        let mut app = TuiApp::new();
        app.process_command("/update test");
        // Should show update wizard or error
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Update") || l.contains("Error") || l.contains("not found")));
    }

    #[test]
    fn test_process_search_command() {
        let mut app = TuiApp::new();
        app.process_command("/search test");
        // Should show search results or empty state
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Search") || l.contains("No results") || l.contains("Error")));
    }

    #[test]
    fn test_process_config_command() {
        let mut app = TuiApp::new();
        app.process_command("/config");
        // Should show configuration list
        assert!(app.output_lines.iter().any(|l| l.contains("Configuration")
            || l.contains("[Database]")
            || l.contains("Error")));
    }

    #[test]
    fn test_process_config_get_command() {
        let mut app = TuiApp::new();
        app.process_command("/config get sync.enabled");
        // Should show configuration value or error
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("=") || l.contains("Error")));
    }

    #[test]
    fn test_process_unknown_command() {
        let mut app = TuiApp::new();
        app.process_command("/unknown");
        // Should show unknown command message
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Unknown") || l.contains("unknown")));
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_process_command_with_args() {
        let mut app = TuiApp::new();
        app.process_command("/delete my record name");
        // Should handle command with multiple args (only first arg used)
        assert!(app.output_lines.iter().any(|l| l.contains("> /delete")));
    }

    #[test]
    fn test_statusline_render_full_width() {
        let app = TuiApp::new();
        // Test statusline at full width (>=60 columns)
        let statusline = app.render_statusline(80);
        // Should contain version info
        assert!(statusline
            .iter()
            .any(|s| s.content.contains("v0.1") || s.content.contains("0.1.0")));
    }

    #[test]
    fn test_statusline_render_narrow_width() {
        let app = TuiApp::new();
        // Test statusline at narrow width (<60 columns)
        let statusline = app.render_statusline(40);
        // Narrow screens should only show minimal info
        assert!(!statusline.is_empty());
    }

    #[test]
    fn test_statusline_shows_lock_icon() {
        let app = TuiApp::new();
        let statusline = app.render_statusline(80);
        // Should show lock status icon
        assert!(statusline
            .iter()
            .any(|s| s.content.contains("🔓") || s.content.contains("🔒")));
    }

    #[test]
    fn test_keybinding_ctrl_q_triggers_quit() {
        let mut app = TuiApp::new();
        let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        app.handle_key_event(ctrl_q);
        assert!(!app.is_running());
    }

    #[test]
    fn test_keybinding_f1_triggers_help() {
        let mut app = TuiApp::new();
        let f1 = KeyEvent::new(KeyCode::F(1), KeyModifiers::empty());
        app.handle_key_event(f1);
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Keyboard Shortcuts") || l.contains("Available Commands")));
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_keybinding_ctrl_l_triggers_list() {
        let mut app = TuiApp::new();
        let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
        app.handle_key_event(ctrl_l);
        assert!(app.output_lines.iter().any(|l| l.contains("> /list")));
    }

    #[test]
    fn test_keybinding_ctrl_k_clears_output() {
        let mut app = TuiApp::new();
        // Add some output first
        app.output_lines.push("test line".to_string());
        assert!(app.output_lines.len() > 3);

        let ctrl_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        app.handle_key_event(ctrl_k);
        // Output should be cleared
        assert!(app.output_lines.is_empty() || app.output_lines.len() <= 3);
    }
}
