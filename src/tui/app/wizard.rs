//! Wizard handling for TUI application
//!
//! Contains all methods related to the onboarding wizard flow.

use super::TuiApp;
use crate::tui::screens::wizard::WizardStep;
use crate::tui::traits::{Action as TraitAction, HandleResult, Interactive, WizardStepValidator};

impl TuiApp {
    /// Handle wizard screen interactions using delegation pattern
    pub fn handle_wizard_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        // ========== Global Navigation Keys ==========

        // Esc: Go back or quit (on first step)
        if event.code == KeyCode::Esc {
            let should_go_back = self
                .wizard_state
                .as_ref()
                .map(|s| s.can_go_back())
                .unwrap_or(false);
            if should_go_back {
                self.clear_current_step_state();
                if let Some(state) = self.wizard_state.as_mut() {
                    state.back();
                }
            } else {
                // Can't go back (first step) — quit the application
                self.quit();
            }
            return;
        }

        // Enter: Finalize on Complete step, otherwise try to proceed
        if event.code == KeyCode::Enter {
            // Check if wizard is on the Complete step
            let is_complete = self
                .wizard_state
                .as_ref()
                .map(|s| s.step == WizardStep::Complete)
                .unwrap_or(false);

            if is_complete {
                // Finalize: create keystore, open vault, inject services
                self.finalize_wizard();
                return;
            }

            let can_proceed = self.try_proceed_current_step();
            if can_proceed {
                // Sync data before proceeding
                self.sync_current_step_to_state();
                if let Some(state) = self.wizard_state.as_mut() {
                    state.next();
                }
                self.initialize_next_step_screen();
            }
            return;
        }

        // ========== Delegate Other Keys to Current Screen ==========

        let Some(state) = self.wizard_state.as_ref() else {
            return;
        };
        let current_step = state.step;
        let _ = state; // Release borrow (drop reference)

        let result = match current_step {
            WizardStep::Welcome => {
                // Up/Down to toggle choice
                match event.code {
                    KeyCode::Up | KeyCode::Down => {
                        self.welcome_screen.toggle();
                        if let Some(state) = self.wizard_state.as_mut() {
                            state.set_passkey_choice(self.welcome_screen.selected());
                        }
                        HandleResult::NeedsRender
                    }
                    _ => HandleResult::Ignored,
                }
            }
            WizardStep::MasterPassword => {
                let result = self.master_password_screen.handle_key(event);
                // Sync password to state after any input using unified interface
                self.sync_current_step_to_state();
                result
            }
            WizardStep::PasskeyImport => {
                let result = self.passkey_import_screen.handle_key(event);
                // Sync using unified interface
                self.sync_current_step_to_state();
                result
            }
            WizardStep::PasskeyGenerate => {
                let result = self.passkey_generate_screen.handle_key(event);
                // Sync using unified interface
                self.sync_current_step_to_state();
                result
            }
            WizardStep::PasskeyVerify => {
                if let Some(screen) = &mut self.passkey_verify_screen {
                    let result = screen.handle_key(event);
                    if let Some(state) = self.wizard_state.as_mut() {
                        state.verify_answers = Some(screen.inputs().clone());
                    }
                    result
                } else {
                    HandleResult::Ignored
                }
            }
            WizardStep::SecurityNotice => self.security_notice_screen.handle_key(event),
            WizardStep::PasswordPolicy => self.password_policy_screen.handle_key(event),
            WizardStep::ClipboardTimeout => self.clipboard_timeout_screen.handle_key(event),
            WizardStep::TrashRetention => self.trash_retention_screen.handle_key(event),
            _ => HandleResult::Ignored,
        };

        // Handle action results
        if let HandleResult::Action(action) = result {
            match action {
                TraitAction::Quit => self.quit(),
                TraitAction::CloseScreen => {
                    self.wizard_state = None;
                    self.current_screen = super::types::Screen::Main;
                }
                TraitAction::ShowToast(msg) => {
                    self.app_state.add_notification(
                        &msg,
                        crate::tui::traits::NotificationLevel::Info,
                    );
                }
                _ => {}
            }
        }
    }

    /// Clear the current step's screen state using unified interface
    pub(crate) fn clear_current_step_state(&mut self) {
        let Some(state) = &mut self.wizard_state else {
            return;
        };

        match state.step {
            WizardStep::MasterPassword => {
                self.master_password_screen.clear_input();
                state.master_password = None;
                state.master_password_confirm = None;
            }
            WizardStep::PasskeyImport => {
                self.passkey_import_screen.clear_input();
                state.passkey_words = None;
            }
            WizardStep::PasskeyVerify => {
                self.passkey_verify_screen = None;
                state.verify_answers = None;
            }
            WizardStep::PasskeyGenerate => {
                self.passkey_generate_screen.clear_input();
                state.confirmed = false;
            }
            _ => {}
        }
    }

    /// Sync current step data to state using unified interface
    pub(crate) fn sync_current_step_to_state(&mut self) {
        let Some(state) = &mut self.wizard_state else {
            return;
        };

        match state.step {
            WizardStep::MasterPassword => {
                self.master_password_screen.sync_to_state(state);
            }
            WizardStep::PasskeyImport => {
                self.passkey_import_screen.sync_to_state(state);
            }
            WizardStep::PasskeyGenerate => {
                self.passkey_generate_screen.sync_to_state(state);
            }
            _ => {}
        }
    }

    /// Try to proceed from the current step, returns true if successful
    pub(crate) fn try_proceed_current_step(&mut self) -> bool {
        let Some(state) = &self.wizard_state else {
            return false;
        };

        match state.step {
            WizardStep::MasterPassword => self.master_password_screen.validate_step(),
            WizardStep::PasskeyVerify => {
                if let Some(screen) = &self.passkey_verify_screen {
                    if screen.verify() {
                        true
                    } else {
                        // Show error on the screen
                        if let Some(screen) = &mut self.passkey_verify_screen {
                            screen.set_error(
                                "One or more words are incorrect. Please try again.".to_string(),
                            );
                        }
                        false
                    }
                } else {
                    false
                }
            }
            WizardStep::PasskeyImport => {
                if !self.passkey_import_screen.is_validated() {
                    let _ = self.passkey_import_screen.validate();
                }
                self.passkey_import_screen.validate_step()
            }
            WizardStep::PasskeyGenerate => self.passkey_generate_screen.validate_step(),
            _ => state.can_proceed(),
        }
    }

    /// Initialize the screen for the next step after transition
    pub(crate) fn initialize_next_step_screen(&mut self) {
        let Some(state) = &self.wizard_state else {
            return;
        };

        if state.step == WizardStep::PasskeyVerify {
            if let Some(words) = state.passkey_words.clone() {
                self.passkey_verify_screen =
                    Some(crate::tui::screens::PasskeyVerifyScreen::new(words));
            }
        }
    }

    /// Finalize the wizard: create keystore, open vault, inject services.
    ///
    /// Returns `true` on success, `false` on failure (error notification shown,
    /// wizard state preserved so the user can retry).
    pub(crate) fn finalize_wizard(&mut self) -> bool {
        use crate::cli::config::ConfigManager;
        use crate::crypto::CryptoManager;
        use std::sync::{Arc, Mutex};

        // Extract all needed values from wizard state upfront to avoid borrow conflicts.
        let (keystore_path, password, clipboard_seconds, trash_days, policy_length,
             policy_min_digits, policy_min_special, policy_type) = {
            let Some(state) = &self.wizard_state else {
                return false;
            };
            if !state.is_complete() {
                return false;
            }
            let Some(keystore_path) = state.require_keystore_path().cloned() else {
                self.app_state.add_notification(
                    "Keystore path not set",
                    crate::tui::traits::NotificationLevel::Error,
                );
                return false;
            };
            let Some(password) = state.require_master_password().map(|s| s.to_string()) else {
                self.app_state.add_notification(
                    "Master password not set",
                    crate::tui::traits::NotificationLevel::Error,
                );
                return false;
            };

            let clipboard_seconds = state.clipboard_timeout.seconds();
            let trash_days = state.trash_retention.days();
            let policy_length = state.password_policy.default_length;
            let policy_min_digits = state.password_policy.min_digits;
            let policy_min_special = state.password_policy.min_special;
            let policy_type = state.password_policy.default_type;

            (keystore_path, password, clipboard_seconds, trash_days,
             policy_length, policy_min_digits, policy_min_special, policy_type)
        };
        // wizard_state borrow is now dropped; self is free for &mut calls.

        // 1. Initialize keystore
        let keystore = match crate::onboarding::initialize_keystore(&keystore_path, &password) {
            Ok(ks) => ks,
            Err(e) => {
                self.app_state.add_notification(
                    &format!("Failed to create keystore: {}", e),
                    crate::tui::traits::NotificationLevel::Error,
                );
                return false;
            }
        };

        // 2. Get DEK
        let dek_bytes = keystore.get_dek();
        let mut dek_array = [0u8; 32];
        dek_array.copy_from_slice(dek_bytes);

        // 3. Initialize CryptoManager
        let mut crypto = CryptoManager::new();
        crypto.initialize_with_key(dek_array);

        // 4. Open vault (get DB path from config)
        let db_path = ConfigManager::new()
            .ok()
            .and_then(|cm| cm.get_database_config().ok())
            .map(|c| std::path::PathBuf::from(c.path))
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("open-keyring")
                    .join("passwords.db")
            });

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let vault = match crate::db::Vault::open(&db_path, &password) {
            Ok(v) => v,
            Err(e) => {
                self.app_state.add_notification(
                    &format!("Failed to open database: {}", e),
                    crate::tui::traits::NotificationLevel::Error,
                );
                return false;
            }
        };

        // 5. Inject services
        let vault_arc = Arc::new(Mutex::new(vault));
        let crypto_arc = Arc::new(Mutex::new(crypto));

        let db_service = crate::tui::services::TuiDatabaseService::with_vault(vault_arc)
            .with_dek(dek_array);
        self.app_state
            .set_db_service(Arc::new(Mutex::new(db_service)));

        let clipboard = crate::tui::services::TuiClipboardService::new();
        self.app_state.set_clipboard_service(clipboard);

        let crypto_service =
            crate::tui::services::TuiCryptoService::with_crypto_manager(crypto_arc);
        self.app_state.set_crypto_service(crypto_service);

        // 6. Apply wizard config
        self.app_state.config.clipboard_timeout_seconds = clipboard_seconds;
        self.app_state.config.trash_retention_days = trash_days;
        self.app_state.config.default_password_policy =
            crate::tui::config::PasswordPolicyConfig {
                length: policy_length,
                min_digits: policy_min_digits,
                min_special: policy_min_special,
                min_lowercase: 1,
                min_uppercase: 1,
                password_type: match policy_type {
                    crate::tui::screens::wizard::PasswordType::Random => {
                        crate::tui::config::PasswordTypeConfig::Random
                    }
                    crate::tui::screens::wizard::PasswordType::Memorable => {
                        crate::tui::config::PasswordTypeConfig::Memorable
                    }
                    crate::tui::screens::wizard::PasswordType::Pin => {
                        crate::tui::config::PasswordTypeConfig::Pin
                    }
                },
            };

        if let Err(e) = self.save_config() {
            eprintln!("Warning: Failed to save config: {}", e);
        }

        // 7. Load passwords into cache (empty for fresh setup, but consistent with unlock flow)
        self.load_passwords_from_vault();

        // 8. Clear wizard state and transition
        self.wizard_state = None;
        self.passkey_verify_screen = None;
        self.current_screen = super::types::Screen::Main;

        self.app_state.add_notification(
            "Setup complete! Press [n] to create your first password.",
            crate::tui::traits::NotificationLevel::Success,
        );

        true
    }
}
