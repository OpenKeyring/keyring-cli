//! Wizard handling for TUI application
//!
//! Contains all methods related to the onboarding wizard flow.

use super::TuiApp;
use crate::error::{KeyringError, Result};
use crate::onboarding::initialize_keystore;
use crate::tui::screens::wizard::WizardStep;
use crate::tui::traits::{Action as TraitAction, HandleResult, Interactive, WizardStepValidator};

impl TuiApp {
    /// Check if onboarding is needed, and if so, start the wizard
    pub async fn check_onboarding(&mut self, keystore_path: &std::path::Path) -> Result<bool> {
        if !crate::onboarding::is_initialized(keystore_path) {
            // Show wizard
            self.wizard_state = Some(
                crate::tui::screens::wizard::WizardState::new()
                    .with_keystore_path(keystore_path.to_path_buf()),
            );
            self.current_screen = super::types::Screen::Wizard;
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

            // Apply wizard configuration to TuiConfig
            let clipboard_seconds = state.clipboard_timeout.seconds();
            let trash_days = state.trash_retention.days();

            self.app_state.config.clipboard_timeout_seconds = clipboard_seconds;
            self.app_state.config.trash_retention_days = trash_days;

            // Apply password policy from wizard
            self.app_state.config.default_password_policy =
                crate::tui::config::PasswordPolicyConfig {
                    length: state.password_policy.default_length,
                    min_digits: state.password_policy.min_digits,
                    min_special: state.password_policy.min_special,
                    min_lowercase: 1, // Default
                    min_uppercase: 1, // Default
                    password_type: match state.password_policy.default_type {
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

            // Save configuration to disk
            if let Err(e) = self.save_config() {
                eprintln!("Warning: Failed to save TUI config: {}", e);
            }

            // Clear wizard state
            self.wizard_state = None;
            self.passkey_verify_screen = None;
            self.current_screen = super::types::Screen::Main;

            self.add_output("✓ Initialization complete".to_string());
            self.add_output(format!("  Clipboard timeout: {}s", clipboard_seconds));
            self.add_output(format!("  Trash retention: {} days", trash_days));
            Ok(())
        } else {
            Err(KeyringError::InvalidInput {
                context: "No wizard state".to_string(),
            })
        }
    }

    /// Handle wizard screen interactions using delegation pattern
    pub fn handle_wizard_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        // ========== Global Navigation Keys ==========

        // Esc: Go back or exit
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
                self.wizard_state = None;
                self.current_screen = super::types::Screen::Main;
            }
            return;
        }

        // Enter: Try to proceed to next step
        if event.code == KeyCode::Enter {
            let can_proceed = self.try_proceed_current_step();
            if can_proceed {
                // Sync data before proceeding
                self.sync_current_step_to_state();
                if let Some(state) = self.wizard_state.as_mut() {
                    state.next();
                }
                self.initialize_next_step_screen();

                if self
                    .wizard_state
                    .as_ref()
                    .map(|s| s.is_complete())
                    .unwrap_or(false)
                {
                    self.output_lines.push(
                        "Wizard complete! Type /wizard-complete to finish.".to_string(),
                    );
                }
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
                    self.output_lines.push(msg);
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
                        return true;
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
}
