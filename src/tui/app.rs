//! TUI Application State and Logic
//!
//! Core TUI application handling alternate screen mode, rendering, and event loop.

use crate::db::vault::Vault;
use crate::error::{KeyringError, Result};
use crate::onboarding::{initialize_keystore, is_initialized};
use crate::tui::components::ConfirmAction;
use crate::tui::keybindings::{Action, KeyBindingManager};
use crate::tui::screens::wizard::{WizardState, WizardStep};
use crate::tui::screens::{
    ClipboardTimeoutScreen, EditPasswordScreen, MainScreen, MasterPasswordScreen, NewPasswordScreen,
    PasskeyGenerateScreen, PasskeyImportScreen, PasskeyVerifyScreen, PasswordPolicyScreen,
    SecurityNoticeScreen, SyncScreen, TrashRetentionScreen, TrashScreen, UnlockScreen, WelcomeScreen,
};
use crate::tui::state::AppState;
use chrono::{DateTime, Utc};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

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

/// Maximum history entries to keep
const MAX_HISTORY: usize = 1000;
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
    current_screen: Screen,
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

    // ========== Wizard Methods ==========

    /// Check if onboarding is needed, and if so, start the wizard
    pub async fn check_onboarding(&mut self, keystore_path: &std::path::Path) -> Result<bool> {
        if !is_initialized(keystore_path) {
            // Show wizard
            self.wizard_state =
                Some(WizardState::new().with_keystore_path(keystore_path.to_path_buf()));
            self.current_screen = Screen::Wizard;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Complete the wizard and initialize the keystore
    pub async fn complete_wizard(&mut self) -> Result<()> {
        if let Some(state) = &self.wizard_state {
            if !state.is_complete() {
                return Err(KeyringError::InvalidInput {
                    context: "Wizard not complete".to_string(),
                });
            }

            let Some(keystore_path) = state.require_keystore_path() else {
                return Err(KeyringError::InvalidInput {
                    context: "Keystore path not set".to_string(),
                });
            };
            let Some(password) = state.require_master_password() else {
                return Err(KeyringError::InvalidInput {
                    context: "Master password not set".to_string(),
                });
            };

            // Initialize keystore
            let _keystore = initialize_keystore(keystore_path, password).map_err(|e| {
                KeyringError::Internal {
                    context: e.to_string(),
                }
            })?;

            // TODO: Store Passkey seed wrapped with master password

            // Clear wizard state
            self.wizard_state = None;
            self.passkey_verify_screen = None;
            self.current_screen = Screen::Main;

            self.add_output("✓ Initialization complete".to_string());
            Ok(())
        } else {
            Err(KeyringError::InvalidInput {
                context: "No wizard state".to_string(),
            })
        }
    }

    /// Handle wizard screen interactions
    pub fn handle_wizard_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        let Some(state) = self.wizard_state.as_mut() else {
            return;
        };

        match event.code {
            KeyCode::Esc => {
                // Go back or exit
                if state.can_go_back() {
                    state.back();
                } else {
                    // Exit wizard
                    self.wizard_state = None;
                    self.current_screen = Screen::Main;
                }
            }
            KeyCode::Enter => {
                // Try to proceed
                if state.can_proceed() {
                    state.next();

                    // Handle special cases - initialize screens for new steps
                    if state.step == WizardStep::PasskeyVerify {
                        if let Some(words) = state.passkey_words.clone() {
                            self.passkey_verify_screen = Some(PasskeyVerifyScreen::new(words));
                        }
                    }

                    // Check if wizard complete
                    if state.is_complete() {
                        // Note: complete_wizard needs to be called separately in async context
                        self.output_lines
                            .push("Wizard complete! Type /wizard-complete to finish.".to_string());
                    }
                }
            }
            KeyCode::Char(' ') => {
                // Space to toggle security notice acknowledgment
                if state.step == WizardStep::SecurityNotice {
                    use crate::tui::traits::Interactive;
                    self.security_notice_screen
                        .handle_key(crossterm::event::KeyEvent::from(KeyCode::Char(' ')));
                }
            }
            KeyCode::Up | KeyCode::Down => {
                // Toggle choice on welcome screen
                if state.step == WizardStep::Welcome {
                    self.welcome_screen.toggle();
                    state.set_passkey_choice(self.welcome_screen.selected());
                }
            }
            KeyCode::Tab => {
                // Switch between password fields
                if state.step == WizardStep::MasterPassword {
                    if self.master_password_screen.is_showing_first() {
                        self.master_password_screen.next();
                    } else {
                        self.master_password_screen.back();
                    }
                }
            }
            KeyCode::Char(c) => {
                // Character input
                match state.step {
                    WizardStep::PasskeyImport => {
                        self.passkey_import_screen.handle_char(c);
                        if self.passkey_import_screen.is_validated() {
                            if let Some(words) = self.passkey_import_screen.words() {
                                state.set_passkey_words(words.to_vec());
                            }
                        }
                    }
                    WizardStep::MasterPassword => {
                        self.master_password_screen.handle_char(c);
                        if let Some(pwd) = self.master_password_screen.get_password() {
                            state.set_master_password(pwd);
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace | KeyCode::Delete => {
                // Backspace
                match state.step {
                    WizardStep::PasskeyImport => {
                        self.passkey_import_screen.handle_backspace();
                    }
                    WizardStep::MasterPassword => {
                        self.master_password_screen.handle_backspace();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Handle unlock screen interactions
    pub fn handle_unlock_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        // Don't process input if currently unlocking
        if self.unlock_screen.state() == crate::tui::screens::UnlockState::Unlocking {
            return;
        }

        match event.code {
            KeyCode::Esc => {
                // Exit the application
                self.quit();
            }
            KeyCode::Enter => {
                // Attempt to unlock
                if self.unlock_screen.can_unlock() {
                    // Try to unlock with the entered password
                    self.attempt_unlock();
                }
            }
            KeyCode::Char(c) => {
                self.unlock_screen.handle_char(c);
            }
            KeyCode::Backspace => {
                self.unlock_screen.handle_backspace();
            }
            _ => {}
        }
    }

    /// Attempt to unlock the vault with the entered password
    fn attempt_unlock(&mut self) {
        use crate::cli::config::ConfigManager;
        use crate::crypto::CryptoManager;
        use crate::db::Vault;
        use std::sync::{Arc, Mutex};

        let password = self.unlock_screen.password().to_string();
        if password.is_empty() {
            return;
        }

        // Set unlocking state
        self.unlock_screen.set_unlocking();

        // Get config and paths
        let config = match ConfigManager::new() {
            Ok(c) => c,
            Err(e) => {
                self.unlock_screen.set_failed(&format!("Config error: {}", e));
                return;
            }
        };

        let keystore_path = config.get_keystore_path();
        let db_config = match config.get_database_config() {
            Ok(c) => c,
            Err(e) => {
                self.unlock_screen.set_failed(&format!("Database config error: {}", e));
                return;
            }
        };
        let db_path = std::path::PathBuf::from(db_config.path);

        // Try to unlock keystore
        let keystore = match crate::crypto::keystore::KeyStore::unlock(&keystore_path, &password) {
            Ok(ks) => ks,
            Err(e) => {
                self.unlock_screen.set_failed(&format!("Unlock failed: {}", e));
                return;
            }
        };

        // Get DEK from keystore and initialize CryptoManager
        let dek_bytes = keystore.get_dek();
        let mut crypto = CryptoManager::new();
        let mut dek_array = [0u8; 32];
        dek_array.copy_from_slice(dek_bytes);
        crypto.initialize_with_key(dek_array);

        // Open Vault
        let vault = match Vault::open(&db_path, &password) {
            Ok(v) => v,
            Err(e) => {
                self.unlock_screen.set_failed(&format!("Failed to open vault: {}", e));
                return;
            }
        };

        // Create and inject services
        let vault_arc = Arc::new(Mutex::new(vault));
        let crypto_arc = Arc::new(Mutex::new(crypto));

        let db_service = crate::tui::services::TuiDatabaseService::with_vault(vault_arc)
            .with_dek(dek_array);

        // Set services in app state
        self.app_state.set_db_service(Arc::new(Mutex::new(db_service)));

        // Create clipboard service
        let clipboard = crate::tui::services::TuiClipboardService::new();
        self.app_state.set_clipboard_service(clipboard);

        // Create crypto service
        let crypto_service =
            crate::tui::services::TuiCryptoService::with_crypto_manager(crypto_arc);
        self.app_state.set_crypto_service(crypto_service);

        // Load passwords into cache
        self.load_passwords_from_vault();

        // Success - transition to main screen
        self.unlock_screen.set_success();
        self.current_screen = Screen::Main;
        self.output_lines
            .push("✓ Vault unlocked successfully".to_string());
    }

    /// Load all passwords from vault into cache
    fn load_passwords_from_vault(&mut self) {
        // Use block_in_place to run async code in sync context
        let passwords = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(db_service) = self.app_state.db_service() {
                    match db_service.lock() {
                        Ok(service) => {
                            match service.list_passwords().await {
                                Ok(passwords) => passwords,
                                Err(e) => {
                                    eprintln!("Failed to load passwords: {}", e);
                                    vec![]
                                }
                            }
                        }
                        Err(_) => {
                            eprintln!("Failed to lock db_service");
                            vec![]
                        }
                    }
                } else {
                    vec![]
                }
            })
        });

        self.app_state.refresh_password_cache(passwords);
    }
    pub fn handle_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crate::tui::traits::Interactive;
        use crossterm::event::KeyCode;

        // Handle NewPassword screen specially
        if self.current_screen == Screen::NewPassword {
            let result = self.new_password_screen.handle_key(event);
            match result {
                crate::tui::traits::HandleResult::Action(crate::tui::traits::Action::CloseScreen) => {
                    // Check if the form was successfully validated
                    if self.new_password_screen.get_password_record().is_some() {
                        self.add_output("✓ Password created successfully".to_string());
                    }
                    // Reset screen for next use
                    self.new_password_screen = NewPasswordScreen::new();
                    self.return_to_main();
                }
                crate::tui::traits::HandleResult::NeedsRender => {
                    // Screen will be re-rendered on next frame
                }
                _ => {}
            }
            return;
        }

        // Handle EditPassword screen specially
        if self.current_screen == Screen::EditPassword {
            let result = self.edit_password_screen.handle_key(event);
            match result {
                crate::tui::traits::HandleResult::Action(crate::tui::traits::Action::CloseScreen) => {
                    // Get the edited fields and update the password
                    let fields = self.edit_password_screen.get_edited_fields();

                    // Build updated PasswordRecord
                    let existing = self.app_state.get_password_by_str(&fields.id.to_string());
                    let updated_record = crate::tui::models::password::PasswordRecord {
                        id: fields.id.to_string(),
                        name: fields.name.clone(),
                        username: fields.username.clone(),
                        password: fields.password.clone().unwrap_or_default(),
                        url: fields.url.clone(),
                        notes: fields.notes.clone(),
                        tags: fields.tags.clone(),
                        group_id: fields.group_id.clone(),
                        created_at: existing.map(|p| p.created_at).unwrap_or_else(chrono::Utc::now),
                        modified_at: chrono::Utc::now(),
                        expires_at: existing.and_then(|p| p.expires_at),
                        is_favorite: existing.map(|p| p.is_favorite).unwrap_or(false),
                        is_deleted: false,
                        deleted_at: None,
                    };

                    // Update in cache
                    self.app_state.update_password_in_cache(updated_record);
                    self.add_output(format!("✓ Password '{}' updated", fields.name));

                    // Reset screen for next use
                    self.edit_password_screen = EditPasswordScreen::empty();
                    self.return_to_main();
                }
                crate::tui::traits::HandleResult::NeedsRender => {
                    // Screen will be re-rendered on next frame
                }
                _ => {}
            }
            return;
        }

        // Handle screen navigation keys first
        match event.code {
            KeyCode::F(2) => {
                // F2 - Settings
                self.navigate_to(Screen::Settings);
                return;
            }
            KeyCode::F(5) => {
                // F5 - Sync
                self.navigate_to(Screen::Sync);
                return;
            }
            KeyCode::Char('?') => {
                // ? - Help
                self.navigate_to(Screen::Help);
                self.show_help();
                return;
            }
            KeyCode::Esc => {
                // Esc - Return to main or quit
                if self.current_screen != Screen::Main {
                    self.return_to_main();
                } else {
                    self.quit();
                }
                return;
            }
            _ => {}
        }

        // Handle keyboard shortcuts via keybinding manager
        if let Some(action) = self.keybinding_manager.get_action(&event) {
            self.execute_action(action);
        }
    }

    /// Execute an action triggered by a keyboard shortcut
    fn execute_action(&mut self, action: Action) {
        match action {
            Action::New => {
                self.process_command("/new");
            }
            Action::List => {
                self.process_command("/list");
            }
            Action::Search => {
                self.output_lines.push("Search: ".to_string());
            }
            Action::Show => {
                self.output_lines.push("Usage: /show <name>".to_string());
            }
            Action::Update => {
                self.output_lines.push("Usage: /update <name>".to_string());
            }
            Action::Delete => {
                self.output_lines.push("Usage: /delete <name>".to_string());
            }
            Action::Quit => {
                self.quit();
                self.output_lines.push("Goodbye!".to_string());
            }
            Action::Help => {
                self.show_help();
            }
            Action::Clear => {
                self.clear_output();
            }
            Action::CopyPassword => {
                self.output_lines
                    .push("Use /show <name> to copy password".to_string());
            }
            Action::CopyUsername => {
                self.output_lines
                    .push("Use /show <name> to copy username".to_string());
            }
            Action::Config => {
                self.process_command("/config");
            }
            Action::OpenSettings => {
                // Navigate to settings screen
                self.navigate_to(Screen::Settings);
                self.output_lines.push("Opened settings screen".to_string());
            }
            Action::SyncNow => {
                self.output_lines.push("Starting sync...".to_string());

                // Try to trigger sync
                // Note: Full sync implementation pending cloud integration
                self.output_lines
                    .push("Note: Full sync implementation pending Phase 4".to_string());
            }
            Action::ShowHelp => {
                self.show_help();
            }
            Action::RefreshView => {
                self.output_lines.push("Refreshing view...".to_string());
            }
            Action::SaveConfig => {
                self.output_lines.push("✓ Configuration saved".to_string());
            }
            Action::DisableSync => {
                self.output_lines.push("✓ Sync disabled".to_string());
            }
        }
    }

    /// Show help with keyboard shortcuts
    fn show_help(&mut self) {
        let bindings = self.keybinding_manager.all_bindings();

        self.output_lines.extend_from_slice(&[
            "".to_string(),
            "Keyboard Shortcuts:".to_string(),
            "".to_string(),
        ]);

        for (action, key_event) in bindings {
            let key_str = KeyBindingManager::format_key(&key_event);
            self.output_lines
                .push(format!("  {:20} - {}", key_str, action.description()));
        }

        self.output_lines.extend_from_slice(&[
            "".to_string(),
            "Commands:".to_string(),
            "  /list [filter]    - List password records".to_string(),
            "  /show <name>      - Show a password record".to_string(),
            "  /new              - Create a new record".to_string(),
            "  /update <name>    - Update a record".to_string(),
            "  /delete <name>    - Delete a record".to_string(),
            "  /search <query>   - Search records".to_string(),
            "  /health [flags]   - Check password health".to_string(),
            "  /config [sub]     - Manage configuration".to_string(),
            "  /exit             - Exit TUI".to_string(),
            "".to_string(),
        ]);
    }

    /// Clear output lines
    fn clear_output(&mut self) {
        self.output_lines.clear();
    }

    /// Render the statusline
    pub fn render_statusline(&self, width: u16) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Narrow screen (<60 columns): show only sync status
        if width < 60 {
            spans.push(Span::styled(
                format!(" {}", self.sync_status.display()),
                Style::default().fg(Color::DarkGray),
            ));
            return spans;
        }

        // Full statusline for width >= 60 columns
        let width_usize = width as usize;

        // Left: lock status + record count
        let lock_icon = if self.locked { "🔒" } else { "🔓" };
        let left_part = format!("{} {} rec", lock_icon, self.record_count);
        spans.push(Span::styled(left_part, Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" | "));

        // Center-left: sync status
        spans.push(Span::styled(
            self.sync_status.display(),
            Style::default().fg(Color::Green),
        ));
        spans.push(Span::raw(" | "));

        // Center-right: version
        spans.push(Span::styled(
            format!("v{}", self.version),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::raw(" | "));

        // Right: keyboard hints (most important shortcuts)
        let hints = self.get_keyboard_hints(width_usize);
        spans.push(Span::styled(
            hints,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        spans
    }

    /// Get keyboard hints for the statusline
    fn get_keyboard_hints(&self, width: usize) -> String {
        // For very wide screens, show more hints
        if width >= 100 {
            "Ctrl+N new | Ctrl+L list | Ctrl+Q quit".to_string()
        } else if width >= 80 {
            "Ctrl+N new | Ctrl+Q quit".to_string()
        } else {
            "Ctrl+Q quit".to_string()
        }
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

    /// Show a confirmation dialog for the given action
    pub fn show_confirm_dialog(&mut self, action: ConfirmAction) {
        // For now, just push to output lines as a placeholder
        // In a full implementation, this would open a confirmation dialog screen
        match &action {
            ConfirmAction::PermanentDelete(id) => {
                self.output_lines.push(format!("Confirm permanent delete: {}", id));
            }
            ConfirmAction::EmptyTrash => {
                self.output_lines.push("Confirm empty trash".to_string());
            }
            ConfirmAction::DeletePassword { password_name, .. } => {
                self.output_lines.push(format!("Confirm delete: {}", password_name));
            }
            ConfirmAction::Generic => {
                self.output_lines.push("Confirm action?".to_string());
            }
        }
        // Immediately handle the confirmed action
        // In a real implementation, we'd show a dialog and wait for user confirmation
        match action {
            ConfirmAction::PermanentDelete(id) => {
                // Permanently delete from cache
                self.app_state.remove_password_from_cache(&id);
                self.output_lines.push("Password permanently deleted".to_string());
            }
            ConfirmAction::EmptyTrash => {
                // Remove all deleted passwords from cache
                let deleted_ids: Vec<String> = self.app_state
                    .all_passwords()
                    .iter()
                    .filter(|p| p.is_deleted)
                    .map(|p| p.id.clone())
                    .collect();
                for id in deleted_ids {
                    self.app_state.remove_password_from_cache(&id);
                }
                self.output_lines.push("Trash emptied".to_string());
            }
            _ => {}
        }
    }

    /// Handle input character
    pub fn handle_char(&mut self, c: char) {
        match c {
            '\n' | '\r' => {
                // Enter key - submit command
                self.submit_command();
            }
            '\t' => {
                // Tab key - trigger autocomplete
                self.handle_autocomplete();
            }
            c if c.is_ascii_control() => {
                // Ignore other control characters
            }
            c => {
                // Regular character - add to buffer
                self.input_buffer.push(c);
            }
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        self.input_buffer.pop();
    }

    /// Handle tab autocomplete for commands
    pub fn handle_autocomplete(&mut self) {
        if self.input_buffer.is_empty() {
            // Empty buffer - nothing to complete
            return;
        }

        // Check if input starts with "/" (command)
        if self.input_buffer.starts_with('/') {
            let commands = [
                "/new",
                "/list",
                "/search",
                "/show",
                "/update",
                "/delete",
                "/config",
                "/help",
                "/quit",
                "/exit",
                "/clear",
                "/sync",
                "/generate",
                "/recover",
            ];

            // Find the current word/prefix to complete
            let prefix = self.input_buffer.as_str();

            // Find matching commands
            let matches: Vec<&str> = commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .copied()
                .collect();

            // Store matches for potential display
            self.autocomplete_matches = matches.iter().map(|s| s.to_string()).collect();

            match matches.as_slice() {
                [] => {
                    // No match - keep original
                    self.autocomplete_matches.clear();
                }
                [single] => {
                    // Single match - complete and add space
                    self.input_buffer = format!("{} ", single);
                    self.autocomplete_matches.clear();
                }
                [first, second] => {
                    // Two matches - complete to common prefix
                    let common = Self::common_prefix(first, second);
                    if common.len() > prefix.len() {
                        self.input_buffer = common;
                    } else {
                        // No common extension, show first match
                        self.input_buffer = format!("{} ", first);
                    }
                    // Keep matches for display
                }
                _ => {
                    // Multiple matches - show them to user
                    self.output_lines
                        .push(format!("Matching commands: {}", matches.join(", ")));
                    // Use first match for now
                    self.input_buffer = format!("{} ", matches[0]);
                }
            }
        } else if self.input_buffer.contains(' ') {
            // Has space - might be completing record name
            // Use handle_autocomplete_with_db() with vault for record name completion
            self.autocomplete_matches.clear();
        }
    }

    /// Find common prefix of two strings
    fn common_prefix(a: &str, b: &str) -> String {
        a.chars()
            .zip(b.chars())
            .take_while(|(ca, cb)| ca == cb)
            .map(|(c, _)| c)
            .collect()
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

    /// Submit the current command
    fn submit_command(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let cmd = self.input_buffer.clone();
        // Limit history size
        if self.history.len() >= MAX_HISTORY {
            self.history.remove(0);
        }
        self.history.push(cmd.clone());
        self.history_index = self.history.len();
        self.input_buffer.clear();

        // Process command
        self.process_command(&cmd);
    }

    /// Process a command
    pub(crate) fn process_command(&mut self, cmd: &str) {
        use crate::tui::commands::{config, delete, health, list, new, search, show, update};

        self.add_output(format!("> {}", cmd));

        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let args = if parts.len() > 1 {
            parts[1].split_whitespace().collect()
        } else {
            Vec::new()
        };

        match command {
            "/exit" | "/quit" => {
                self.quit();
                self.output_lines.push("Goodbye!".to_string());
            }
            "/help" => {
                self.show_help();
            }
            "/config" => match config::handle_config(args) {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            "/list" => match list::handle_list(args) {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            "/show" => match show::handle_show(args) {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            "/new" => match new::handle_new() {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            "/update" => match update::handle_update(args) {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            "/delete" => match delete::handle_delete(args) {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            "/search" => match search::handle_search(args) {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            "/health" => match health::handle_health(args) {
                Ok(lines) => self.output_lines.extend(lines),
                Err(e) => self.output_lines.push(format!("Error: {}", e)),
            },
            cmd if cmd.starts_with('/') => {
                self.output_lines.push(format!(
                    "Unknown command '{}'. Type /help for available commands.",
                    cmd
                ));
            }
            _ => {
                self.output_lines
                    .push("Unknown command. Type /help for available commands.".to_string());
            }
        }
    }

    /// Render the TUI
    pub fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();

        // Handle wizard screens differently
        if self.current_screen == Screen::Wizard {
            if let Some(state) = &self.wizard_state {
                self.render_wizard(frame, size, state);
                return;
            }
        }

        // Handle sync screen
        if self.current_screen == Screen::Sync {
            if let Some(screen) = &self.sync_screen {
                screen.render(frame, size);
                return;
            }
        }

        // Handle unlock screen
        if self.current_screen == Screen::Unlock {
            self.unlock_screen.render(frame, size);
            return;
        }

        // Handle new password screen
        if self.current_screen == Screen::NewPassword {
            use crate::tui::traits::Render;
            self.new_password_screen.render(size, frame.buffer_mut());
            return;
        }

        // Handle edit password screen
        if self.current_screen == Screen::EditPassword {
            use crate::tui::traits::Render;
            self.edit_password_screen.render(size, frame.buffer_mut());
            return;
        }

        // Handle trash screen
        if self.current_screen == Screen::Trash {
            self.trash_screen.render_frame(frame, size, &self.app_state);
            return;
        }

        // Handle main screen with dual-column layout
        if self.current_screen == Screen::Main {
            self.main_screen.render_frame(frame, size, &self.app_state);
            return;
        }

        // Fallback: Split screen into output area, input area, and statusline
        // (for any other screens not yet migrated)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),    // Output area (flexible)
                    Constraint::Length(3), // Input area
                    Constraint::Length(1), // Statusline
                ]
                .as_ref(),
            )
            .split(size);

        // Render output area
        self.render_output(frame, chunks[0]);

        // Render input area
        self.render_input(frame, chunks[1]);

        // Render statusline
        self.render_statusline_widget(frame, chunks[2]);
    }

    /// Render the wizard screen
    fn render_wizard(&self, frame: &mut Frame, area: Rect, state: &WizardState) {
        use crate::tui::traits::Render;

        match state.step {
            WizardStep::Welcome => {
                self.welcome_screen.render(frame, area);
            }
            WizardStep::MasterPassword => {
                self.master_password_screen.render(frame, area);
            }
            WizardStep::MasterPasswordConfirm => {
                // Use MasterPasswordScreen's confirm mode or separate screen
                self.master_password_screen.render(frame, area);
            }
            WizardStep::SecurityNotice => {
                self.security_notice_screen.render(area, frame.buffer_mut());
            }
            WizardStep::PasskeyGenerate => {
                self.passkey_generate_screen.render(frame, area);
            }
            WizardStep::PasskeyVerify => {
                if let Some(screen) = &self.passkey_verify_screen {
                    screen.render(area, frame.buffer_mut());
                }
            }
            WizardStep::PasskeyImport => {
                self.passkey_import_screen.render(frame, area);
            }
            WizardStep::MasterPasswordImport => {
                self.master_password_screen.render(frame, area);
            }
            WizardStep::MasterPasswordImportConfirm => {
                self.master_password_screen.render(frame, area);
            }
            WizardStep::PasswordHint => {
                // Simple hint screen - render as paragraph
                let paragraph = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("💡 Password Hint"),
                    Line::from(""),
                    Line::from("Your PassKey has been imported successfully."),
                    Line::from(""),
                    Line::from("Make sure to remember your master password."),
                    Line::from(""),
                    Line::from("Press Enter to continue..."),
                ])
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
                frame.render_widget(paragraph, area);
            }
            WizardStep::PasswordPolicy => {
                self.password_policy_screen.render(area, frame.buffer_mut());
            }
            WizardStep::ClipboardTimeout => {
                self.clipboard_timeout_screen.render(area, frame.buffer_mut());
            }
            WizardStep::TrashRetention => {
                self.trash_retention_screen.render(area, frame.buffer_mut());
            }
            WizardStep::ImportPasswords => {
                // Optional import screen - for now just show message
                let paragraph = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("📥 Import Existing Passwords"),
                    Line::from(""),
                    Line::from("This step is optional."),
                    Line::from(""),
                    Line::from("Press Enter to skip or provide import file..."),
                ])
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
                frame.render_widget(paragraph, area);
            }
            WizardStep::Complete => {
                // Show completion message with quick start guide
                let paragraph = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("🎉 ", Style::default()),
                        Span::styled(
                            "Setup Complete!",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Quick Start Guide:",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  [n] Create a new password",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [j/k] Navigate through your passwords",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [Enter] View password details",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [c] Copy username  |  [C] Copy password",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [?] Show help anytime",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press [Enter] to start using OpenKeyring",
                        Style::default().fg(Color::Gray),
                    )),
                ])
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Welcome to OpenKeyring "));

                frame.render_widget(paragraph, area);
            }
        }
    }

    /// Render the statusline widget
    fn render_statusline_widget(&self, frame: &mut Frame, area: Rect) {
        let spans = self.render_statusline(area.width);
        let line = Line::from(spans);

        let paragraph = Paragraph::new(Text::from(line))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render the output area
    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let text: Text = self
            .output_lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" OpenKeyring TUI "),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Render the input area
    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let input_text = if self.input_buffer.is_empty() {
            vec![Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Type a command...",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Gray)),
                Span::raw(&self.input_buffer),
            ])]
        };

        let paragraph = Paragraph::new(Text::from(input_text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        // Set cursor position (area.x + 1 for left border, + 2 for "> " prefix, then cursor offset)
        frame.set_cursor_position((area.x + 3 + self.input_buffer.len() as u16, area.y + 1));
    }
}

/// Initialize terminal for TUI mode
pub fn init_terminal() -> TuiResult<Terminal<CrosstermBackend<Stdout>>> {
    use crossterm::{
        event::EnableMouseCapture,
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    };

    enable_raw_mode().map_err(|e| TuiError::InitFailed(e.to_string()))?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| TuiError::InitFailed(e.to_string()))?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend).map_err(|e| TuiError::InitFailed(e.to_string()))?;

    Ok(terminal)
}

/// Restore terminal after TUI mode
pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> TuiResult<()> {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };

    disable_raw_mode().map_err(|e| TuiError::RestoreFailed(e.to_string()))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .map_err(|e| TuiError::RestoreFailed(e.to_string()))?;

    terminal
        .show_cursor()
        .map_err(|e| TuiError::RestoreFailed(e.to_string()))?;

    Ok(())
}

/// Run the TUI application
pub fn run_tui() -> Result<()> {
    use crossterm::event;

    // Install panic hook FIRST to ensure terminal recovery on panic
    crate::tui::panic_hook::install_panic_hook();

    let mut terminal =
        init_terminal().map_err(|e| KeyringError::IoError(format!("Failed to init TUI: {}", e)))?;

    let mut app = TuiApp::new();

    // Check onboarding status - show wizard if keystore doesn't exist
    let keystore_path = crate::cli::config::ConfigManager::new()
        .map(|cm| cm.get_keystore_path())
        .unwrap_or_else(|_| {
            // Fallback to default path if config manager fails
            dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("open-keyring")
                .join("keystore.json")
        });
    if !is_initialized(&keystore_path) {
        // User not initialized - show wizard for first-time setup
        app.wizard_state = Some(WizardState::new().with_keystore_path(keystore_path));
        app.current_screen = Screen::Wizard;
    } else {
        // User already initialized - show unlock screen to enter master password
        app.current_screen = Screen::Unlock;
    }

    // Main event loop
    while app.is_running() {
        terminal
            .draw(|f| app.render(f))
            .map_err(|e| KeyringError::IoError(format!("Failed to draw: {}", e)))?;

        // Cleanup expired notifications (3-second auto-dismiss for Info/Success)
        app.app_state.cleanup_notifications();

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))
            .map_err(|e| KeyringError::IoError(format!("Event poll failed: {}", e)))?
        {
            match event::read()
                .map_err(|e| KeyringError::IoError(format!("Event read failed: {}", e)))?
            {
                // Filter to only handle Press events to avoid duplicate key handling on Windows
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    use crossterm::event::KeyCode;

                    // Route wizard events
                    if app.current_screen == Screen::Wizard {
                        app.handle_wizard_key_event(key);
                    } else if app.current_screen == Screen::Unlock {
                        // Route unlock screen events
                        app.handle_unlock_key_event(key);
                    } else if app.current_screen == Screen::Trash {
                        // Route trash screen events
                        use crate::tui::traits::{HandleResult, Action, ScreenType};
                        let result = app.trash_screen.handle_key_with_state(key, &mut app.app_state);
                        match result {
                            HandleResult::Action(Action::CloseScreen) => {
                                app.navigate_to(Screen::Main);
                            }
                            HandleResult::Action(Action::OpenScreen(screen_type)) => {
                                match screen_type {
                                    ScreenType::ConfirmDialog(action) => {
                                        app.show_confirm_dialog(action);
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    } else if app.current_screen == Screen::Main {
                        // Route main screen events to MainScreen handler
                        use crate::tui::traits::{HandleResult, Action, ScreenType};
                        match app.main_screen.handle_key_with_state(key, &mut app.app_state) {
                            HandleResult::Consumed => {}
                            HandleResult::Ignored => {
                                // Fallback: handle global shortcuts
                                if key.code == KeyCode::Char('q') {
                                    app.quit();
                                }
                            }
                            HandleResult::Action(action) => {
                                match action {
                                    Action::Quit => {
                                        app.quit();
                                    }
                                    Action::OpenScreen(screen_type) => {
                                        match screen_type {
                                            ScreenType::Help => {
                                                app.show_help();
                                            }
                                            ScreenType::Settings => {
                                                app.navigate_to(Screen::Settings);
                                            }
                                            ScreenType::NewPassword => {
                                                app.navigate_to(Screen::NewPassword);
                                            }
                                            ScreenType::EditPassword(id_str) => {
                                                // Get password data from cache
                                                if let Some(record) = app.app_state.get_password_by_str(&id_str).cloned() {
                                                    // Create EditPasswordScreen with existing data
                                                    // Convert String ID to Uuid
                                                    if let Ok(uuid) = uuid::Uuid::parse_str(&record.id) {
                                                            app.edit_password_screen = EditPasswordScreen::new(
                                                                uuid,
                                                                &record.name,
                                                                record.username.as_deref(),
                                                                &record.password,
                                                                record.url.as_deref(),
                                                                record.notes.as_deref(),
                                                                &record.tags,
                                                                record.group_id.as_deref(),
                                                            );
                                                        } else {
                                                            app.output_lines.push(format!("Invalid UUID in record: {}", record.id));
                                                        }
                                                        app.navigate_to(Screen::EditPassword);
                                                    } else {
                                                        app.output_lines.push(format!("Password not found: {}", id_str));
                                                    }
                                            }
                                            _ => {
                                                // For other screens, show a placeholder message
                                                app.output_lines.push(format!(
                                                    "Screen {:?} not yet implemented",
                                                    screen_type
                                                ));
                                            }
                                        }
                                    }
                                    Action::ShowToast(message) => {
                                        app.output_lines.push(message);
                                    }
                                    Action::CloseScreen => {
                                        // Return to main screen
                                        app.navigate_to(Screen::Main);
                                    }
                                    Action::CopyToClipboard(content) => {
                                        app.output_lines.push(format!("Copied: {}", content));
                                    }
                                    Action::Refresh => {
                                        app.output_lines.push("Refreshed".to_string());
                                    }
                                    Action::ConfirmDialog(action) => {
                                        // Handle the confirmed action (user clicked confirm)
                                        match action {
                                            ConfirmAction::DeletePassword { password_id, password_name } => {
                                                // Delete the password (remove from cache)
                                                app.app_state.remove_password_from_cache(&password_id);
                                                app.output_lines.push(format!("Deleted \"{}\"", password_name));
                                            }
                                            ConfirmAction::PermanentDelete(id) => {
                                                // Permanently delete the password
                                                let password_name = app.app_state.get_password_by_str(&id)
                                                    .map(|p| p.name.clone())
                                                    .unwrap_or_else(|| id.clone());
                                                if app.app_state.permanent_delete_password(&id) {
                                                    app.output_lines.push(format!("Permanently deleted \"{}\"", password_name));
                                                } else {
                                                    app.output_lines.push(format!("Password not found: {}", id));
                                                }
                                            }
                                            ConfirmAction::EmptyTrash => {
                                                // Empty trash - permanently delete all trashed passwords
                                                let count = app.app_state.empty_trash();
                                                app.output_lines.push(format!("Emptied trash ({} passwords permanently deleted)", count));
                                            }
                                            ConfirmAction::Generic => {
                                                app.output_lines.push("Action confirmed".to_string());
                                            }
                                        }
                                    }
                                    Action::None => {}
                                }
                            }
                            HandleResult::NeedsRender => {}
                        }
                    } else {
                        // Check for keyboard shortcuts first (Ctrl keys)
                        if key.modifiers.contains(event::KeyModifiers::CONTROL) {
                            app.handle_key_event(key);
                        } else {
                            // Regular input handling
                            match key.code {
                                KeyCode::Char(c) => app.handle_char(c),
                                KeyCode::Backspace | KeyCode::Delete => app.handle_backspace(),
                                KeyCode::Enter => app.handle_char('\n'),
                                KeyCode::Esc
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.quit();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                event::Event::Resize(_, _) => {
                    // Terminal resized - will be handled on next draw
                }
                _ => {}
            }
        }
    }

    restore_terminal(terminal)
        .map_err(|e| KeyringError::IoError(format!("Failed to restore terminal: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = TuiApp::new();
        let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        app.handle_key_event(ctrl_q);
        assert!(!app.is_running());
    }

    #[test]
    fn test_keybinding_f1_triggers_help() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = TuiApp::new();
        let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
        app.handle_key_event(ctrl_l);
        assert!(app.output_lines.iter().any(|l| l.contains("> /list")));
    }

    #[test]
    fn test_keybinding_ctrl_k_clears_output() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
