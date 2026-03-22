//! Unlock handling for TUI application
//!
//! Contains methods related to vault unlocking and authentication.

use super::TuiApp;
use crate::tui::screens::UnlockState;

impl TuiApp {
    /// Handle unlock screen interactions
    pub fn handle_unlock_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        // Don't process input if currently unlocking
        if self.unlock_screen.state() == UnlockState::Unlocking {
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
    pub(crate) fn attempt_unlock(&mut self) {
        use crate::cli::config::ConfigManager;
        use crate::crypto::CryptoManager;
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
                self.unlock_screen
                    .set_failed(&format!("Database config error: {}", e));
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
        let vault = match crate::db::Vault::open(&db_path, &password) {
            Ok(v) => v,
            Err(e) => {
                self.unlock_screen
                    .set_failed(&format!("Failed to open vault: {}", e));
                return;
            }
        };

        // Create and inject services
        let vault_arc = Arc::new(Mutex::new(vault));
        let crypto_arc = Arc::new(Mutex::new(crypto));

        let db_service = crate::tui::services::TuiDatabaseService::with_vault(vault_arc)
            .with_dek(dek_array);

        // Set services in app state
        self.app_state
            .set_db_service(Arc::new(Mutex::new(db_service)));

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
        self.current_screen = super::types::Screen::Main;
        self.app_state.add_notification(
            "Vault unlocked successfully",
            crate::tui::traits::NotificationLevel::Success,
        );
    }

    /// Load all passwords from vault into cache
    /// Note: MutexGuard across await is safe here because we're in block_in_place
    #[allow(clippy::await_holding_lock)]
    pub(crate) fn load_passwords_from_vault(&mut self) {
        // Use block_in_place to run async code in sync context
        let passwords = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(db_service) = self.app_state.db_service() {
                    match db_service.lock() {
                        Ok(service) => match service.list_passwords().await {
                            Ok(passwords) => passwords,
                            Err(e) => {
                                eprintln!("Failed to load passwords: {}", e);
                                vec![]
                            }
                        },
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

        // Load groups from vault
        let groups = if let Some(db_service) = self.app_state.db_service() {
            let db = db_service.clone();
            db.lock().ok().and_then(|service| service.list_groups().ok())
        } else {
            None
        };
        if let Some(groups) = groups {
            self.app_state.load_groups(groups);
        }

        self.app_state.apply_filter();
    }
}
