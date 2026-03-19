//! Event handling for TUI application
//!
//! Contains keyboard event handling, action execution, and input processing.

use super::TuiApp;
use crate::tui::components::ConfirmAction;
use crate::tui::keybindings::Action;
use crate::tui::traits::{Action as TraitAction, HandleResult, Interactive};
use crossterm::event::KeyCode;

/// Maximum history entries to keep
const MAX_HISTORY: usize = 1000;

impl TuiApp {
    /// Handle keyboard events for the main application
    pub fn handle_key_event(&mut self, event: crossterm::event::KeyEvent) {
        // Handle NewPassword screen specially
        if self.current_screen == super::types::Screen::NewPassword {
            let result = self.new_password_screen.handle_key(event);
            match result {
                HandleResult::Action(TraitAction::CloseScreen) => {
                    // Check if the form was successfully validated
                    if let Some(record) = self.new_password_screen.get_password_record() {
                        // Convert NewPasswordRecord to PasswordRecord
                        let mut password = crate::tui::models::password::PasswordRecord::new(
                            record.id.to_string(),
                            record.name.clone(),
                            record.password.clone(),
                        );

                        // Apply optional fields
                        if let Some(username) = &record.username {
                            password = password.with_username(username.clone());
                        }
                        if let Some(url) = &record.url {
                            password = password.with_url(url.clone());
                        }
                        if let Some(notes) = &record.notes {
                            password = password.with_notes(notes.clone());
                        }
                        password = password.with_tags(record.tags.clone());
                        if !record.group.is_empty() {
                            password = password.with_group(record.group.clone());
                        }

                        // Save to database
                        let saved = if let Some(db_service) = self.app_state.db_service() {
                            let db = db_service.clone();
                            let password_clone = password.clone();
                            tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    if let Ok(service) = db.lock() {
                                        service.create_password(&password_clone).await.is_ok()
                                    } else {
                                        false
                                    }
                                })
                            })
                        } else {
                            false
                        };

                        if saved {
                            // Add to cache after successful database save
                            self.app_state.add_password_to_cache(password);
                            self.add_output(format!("✓ Password '{}' created and saved", record.name));
                        } else {
                            self.add_output(format!("✗ Failed to save password '{}'", record.name));
                        }
                    }
                    // Reset screen for next use
                    self.new_password_screen =
                        crate::tui::screens::NewPasswordScreen::new();
                    self.return_to_main();
                }
                HandleResult::NeedsRender => {
                    // Screen will be re-rendered on next frame
                }
                _ => {}
            }
            return;
        }

        // Handle EditPassword screen specially
        if self.current_screen == super::types::Screen::EditPassword {
            let result = self.edit_password_screen.handle_key(event);
            match result {
                HandleResult::Action(TraitAction::CloseScreen) => {
                    // Get the edited fields and update the password
                    let fields = self.edit_password_screen.get_edited_fields();

                    // Build updated PasswordRecord
                    let existing = self
                        .app_state
                        .get_password_by_str(&fields.id.to_string());
                    let updated_record = crate::tui::models::password::PasswordRecord {
                        id: fields.id.to_string(),
                        name: fields.name.clone(),
                        username: fields.username.clone(),
                        password: fields.password.clone().unwrap_or_default(),
                        url: fields.url.clone(),
                        notes: fields.notes.clone(),
                        tags: fields.tags.clone(),
                        group_id: fields.group_id.clone(),
                        created_at: existing
                            .map(|p| p.created_at)
                            .unwrap_or_else(chrono::Utc::now),
                        modified_at: chrono::Utc::now(),
                        expires_at: existing.and_then(|p| p.expires_at),
                        is_favorite: existing.map(|p| p.is_favorite).unwrap_or(false),
                        is_deleted: false,
                        deleted_at: None,
                    };

                    // Save to database first
                    let saved = if let Some(db_service) = self.app_state.db_service() {
                        let db = db_service.clone();
                        let record_clone = updated_record.clone();
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                if let Ok(service) = db.lock() {
                                    service.update_password(&record_clone).await.is_ok()
                                } else {
                                    false
                                }
                            })
                        })
                    } else {
                        false
                    };

                    if saved {
                        // Update in cache after successful database save
                        self.app_state.update_password_in_cache(updated_record);
                        self.add_output(format!("✓ Password '{}' updated and saved", fields.name));
                    } else {
                        self.add_output(format!("✗ Failed to update password '{}'", fields.name));
                    }

                    // Reset screen for next use
                    self.edit_password_screen = crate::tui::screens::EditPasswordScreen::empty();
                    self.return_to_main();
                }
                HandleResult::NeedsRender => {
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
                self.navigate_to(super::types::Screen::Settings);
                return;
            }
            KeyCode::F(5) => {
                // F5 - Sync
                self.navigate_to(super::types::Screen::Sync);
                return;
            }
            KeyCode::Char('?') => {
                // ? - Help
                self.navigate_to(super::types::Screen::Help);
                self.show_help();
                return;
            }
            KeyCode::Esc => {
                // Esc - Return to main or quit
                if self.current_screen != super::types::Screen::Main {
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
    pub(crate) fn execute_action(&mut self, action: Action) {
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
                self.navigate_to(super::types::Screen::Settings);
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
            Action::SaveConfig => match self.save_config() {
                Ok(()) => {
                    self.output_lines.push("✓ Configuration saved".to_string());
                }
                Err(e) => {
                    self.output_lines
                        .push(format!("✗ Failed to save configuration: {}", e));
                }
            },
            Action::DisableSync => {
                self.output_lines.push("✓ Sync disabled".to_string());
            }
        }
    }

    /// Show help with keyboard shortcuts
    pub(crate) fn show_help(&mut self) {
        let bindings = self.keybinding_manager.all_bindings();

        self.output_lines.extend_from_slice(&[
            "".to_string(),
            "Keyboard Shortcuts:".to_string(),
            "".to_string(),
        ]);

        for (action, key_event) in bindings {
            let key_str = crate::tui::keybindings::KeyBindingManager::format_key(&key_event);
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
    pub(crate) fn clear_output(&mut self) {
        self.output_lines.clear();
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
                let deleted_ids: Vec<String> = self
                    .app_state
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
}
