//! Event handling for TUI application
//!
//! Contains screen-specific result handlers for NewPassword and EditPassword screens.

use super::TuiApp;
use crate::tui::components::ConfirmAction;
use crate::tui::traits::{
    Action as TraitAction, ClipboardService, DatabaseService, HandleResult,
};

impl TuiApp {
    /// Handle the result of a key event on the NewPassword screen
    pub(crate) fn handle_new_password_result(&mut self, result: HandleResult) {
        match result {
            HandleResult::Action(TraitAction::CloseScreen) => {
                if let Some(record) = self.new_password_screen.get_password_record() {
                    let mut password = crate::tui::models::password::PasswordRecord::new(
                        record.id.to_string(),
                        record.name.clone(),
                        record.password.clone(),
                    );
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

                    #[allow(clippy::await_holding_lock)]
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
                        self.app_state.add_password_to_cache(password);
                        self.app_state.add_notification(
                            &format!("Password '{}' created", record.name),
                            crate::tui::traits::NotificationLevel::Success,
                        );
                    } else {
                        self.app_state.add_notification(
                            &format!("Failed to save password '{}'", record.name),
                            crate::tui::traits::NotificationLevel::Error,
                        );
                    }
                }
                self.new_password_screen = crate::tui::screens::NewPasswordScreen::new();
                self.navigate_to(super::types::Screen::Main);
            }
            HandleResult::NeedsRender | HandleResult::Consumed => {}
            _ => {}
        }
    }

    /// Handle the result of a key event on the EditPassword screen
    pub(crate) fn handle_edit_password_result(&mut self, result: HandleResult) {
        match result {
            HandleResult::Action(TraitAction::CloseScreen) => {
                let fields = self.edit_password_screen.get_edited_fields();
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
                    created_at: existing
                        .map(|p| p.created_at)
                        .unwrap_or_else(chrono::Utc::now),
                    modified_at: chrono::Utc::now(),
                    expires_at: existing.and_then(|p| p.expires_at),
                    is_favorite: existing.map(|p| p.is_favorite).unwrap_or(false),
                    is_deleted: false,
                    deleted_at: None,
                };

                #[allow(clippy::await_holding_lock)]
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
                    self.app_state.update_password_in_cache(updated_record);
                    self.app_state.add_notification(
                        &format!("Password '{}' updated", fields.name),
                        crate::tui::traits::NotificationLevel::Success,
                    );
                } else {
                    self.app_state.add_notification(
                        &format!("Failed to update password '{}'", fields.name),
                        crate::tui::traits::NotificationLevel::Error,
                    );
                }

                self.edit_password_screen = crate::tui::screens::EditPasswordScreen::empty();
                self.navigate_to(super::types::Screen::Main);
            }
            HandleResult::NeedsRender | HandleResult::Consumed => {}
            _ => {}
        }
    }

    /// Handle actions from the main screen
    pub(crate) fn handle_main_screen_action(&mut self, action: TraitAction) {
        match action {
            TraitAction::Quit => {
                self.quit();
            }
            TraitAction::OpenScreen(screen_type) => {
                use crate::tui::traits::ScreenType;
                match screen_type {
                    ScreenType::Help => {
                        self.navigate_to(super::types::Screen::Help);
                    }
                    ScreenType::Settings => {
                        self.navigate_to(super::types::Screen::Settings);
                    }
                    ScreenType::NewPassword => {
                        self.navigate_to(super::types::Screen::NewPassword);
                    }
                    ScreenType::ConfirmDialog(action) => {
                        self.show_confirm_dialog(action);
                    }
                    ScreenType::EditPassword(id_str) => {
                        if let Some(record) = self.app_state.get_password_by_str(&id_str).cloned() {
                            if let Ok(uuid) = uuid::Uuid::parse_str(&record.id) {
                                self.edit_password_screen =
                                    crate::tui::screens::EditPasswordScreen::new(
                                        uuid,
                                        &record.name,
                                        record.username.as_deref(),
                                        &record.password,
                                        record.url.as_deref(),
                                        record.notes.as_deref(),
                                        &record.tags,
                                        record.group_id.as_deref(),
                                    );
                                self.navigate_to(super::types::Screen::EditPassword);
                            }
                        }
                    }
                    _ => {}
                }
            }
            TraitAction::CopyToClipboard(content) => {
                // Default to Password type; the MainScreen action carries
                // the content but not the type. For username copies, the
                // MainScreen should use a different action or we detect by context.
                self.handle_clipboard_copy(
                    &content,
                    crate::tui::traits::ClipboardContentType::Password,
                );
            }
            TraitAction::CloseScreen => {
                self.navigate_to(super::types::Screen::Main);
            }
            TraitAction::Refresh => {
                self.load_passwords_from_vault();
            }
            TraitAction::ConfirmDialog(action) => {
                self.show_confirm_dialog(action);
            }
            TraitAction::ShowToast(message) => {
                self.handle_toast_signal(message);
            }
            TraitAction::None => {}
        }
    }

    /// Handle clipboard copy action
    pub(crate) fn handle_clipboard_copy(
        &mut self,
        content: &str,
        content_type: crate::tui::traits::ClipboardContentType,
    ) {
        if let Some(clipboard) = &mut self.app_state.clipboard_service {
            match clipboard.copy_str(content, content_type) {
                Ok(()) => {
                    let timeout = self.app_state.config.clipboard_timeout_seconds;
                    let label = match content_type {
                        crate::tui::traits::ClipboardContentType::Password => "Password copied",
                        crate::tui::traits::ClipboardContentType::Username => "Username copied",
                        _ => "Copied",
                    };
                    self.app_state.add_notification(
                        &format!("{} (clears in {}s)", label, timeout),
                        crate::tui::traits::NotificationLevel::Success,
                    );
                }
                Err(e) => {
                    self.app_state.add_notification(
                        &format!("Clipboard error: {}", e),
                        crate::tui::traits::NotificationLevel::Error,
                    );
                }
            }
        } else {
            self.app_state.add_notification(
                "Clipboard not available",
                crate::tui::traits::NotificationLevel::Warning,
            );
        }
    }

    /// Show a confirmation dialog overlay
    pub fn show_confirm_dialog(&mut self, action: ConfirmAction) {
        use crate::tui::components::ConfirmDialog;
        self.confirm_dialog = Some(match &action {
            ConfirmAction::DeletePassword {
                password_name,
                password_id,
            } => ConfirmDialog::delete_confirmation(password_name, password_id),
            ConfirmAction::PermanentDelete(id) => {
                let name = self
                    .app_state
                    .get_password_by_str(id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| id.clone());
                ConfirmDialog::permanent_delete_confirmation(&name, id)
            }
            ConfirmAction::EmptyTrash => {
                let count = self
                    .app_state
                    .all_passwords()
                    .iter()
                    .filter(|p| p.is_deleted)
                    .count();
                ConfirmDialog::empty_trash_confirmation(count)
            }
            ConfirmAction::Generic => ConfirmDialog::new(),
            ConfirmAction::DeleteGroup { group_id, group_name } => {
                ConfirmDialog::for_delete_group(group_name, group_id)
            }
        });
    }

    /// Handle a confirmed action from the dialog
    #[allow(clippy::await_holding_lock)]
    pub(crate) fn handle_confirmed_action(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DeletePassword {
                password_id,
                password_name,
            } => {
                let deleted = if let Some(db_service) = self.app_state.db_service() {
                    let db = db_service.clone();
                    let id = password_id.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            if let Ok(service) = db.lock() {
                                service.delete_password(&id, true).await.is_ok()
                            } else {
                                false
                            }
                        })
                    })
                } else {
                    false
                };
                if deleted {
                    self.app_state.remove_password_from_cache(&password_id);
                    self.app_state.add_notification(
                        &format!("\"{}\" moved to trash", password_name),
                        crate::tui::traits::NotificationLevel::Success,
                    );
                } else {
                    self.app_state.add_notification(
                        &format!("Failed to delete \"{}\"", password_name),
                        crate::tui::traits::NotificationLevel::Error,
                    );
                }
            }
            ConfirmAction::PermanentDelete(id) => {
                let name = self
                    .app_state
                    .get_password_by_str(&id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| id.clone());
                let deleted = if let Some(db_service) = self.app_state.db_service() {
                    let db = db_service.clone();
                    let id_clone = id.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            if let Ok(service) = db.lock() {
                                service.permanently_delete(&id_clone).await.is_ok()
                            } else {
                                false
                            }
                        })
                    })
                } else {
                    false
                };
                if deleted {
                    self.app_state.permanent_delete_password(&id);
                    self.app_state.add_notification(
                        &format!("\"{}\" permanently deleted", name),
                        crate::tui::traits::NotificationLevel::Success,
                    );
                }
            }
            ConfirmAction::EmptyTrash => {
                let emptied = if let Some(db_service) = self.app_state.db_service() {
                    let db = db_service.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            if let Ok(service) = db.lock() {
                                service.empty_trash().await.unwrap_or(0)
                            } else {
                                0
                            }
                        })
                    })
                } else {
                    0
                };
                self.app_state.empty_trash();
                self.app_state.add_notification(
                    &format!("Trash emptied ({} deleted)", emptied),
                    crate::tui::traits::NotificationLevel::Success,
                );
            }
            ConfirmAction::Generic => {}
            ConfirmAction::DeleteGroup { group_id, group_name } => {
                let db = self.app_state.db_service().cloned();
                if let Some(db) = db {
                    let result = db.lock().ok().map(|service| {
                        service.delete_group(&group_id)
                    });
                    match result {
                        Some(Ok(())) => {
                            self.app_state.remove_group(&group_id);
                            self.app_state.apply_filter();
                            self.app_state.add_notification(
                                &format!("Group '{}' deleted", group_name),
                                crate::tui::traits::NotificationLevel::Success,
                            );
                        }
                        Some(Err(e)) => {
                            self.app_state.add_notification(
                                &format!("Failed to delete group: {}", e),
                                crate::tui::traits::NotificationLevel::Error,
                            );
                        }
                        None => {
                            self.app_state.add_notification(
                                "Database locked",
                                crate::tui::traits::NotificationLevel::Error,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Handle toast signals from tree panel and other components.
    ///
    /// Signals prefixed with `__` are intercepted and dispatched to database
    /// operations. All other messages are shown as normal info notifications.
    pub(crate) fn handle_toast_signal(&mut self, message: String) {
        use crate::tui::traits::NotificationLevel;

        if let Some(name) = message.strip_prefix("__create_group:") {
            let name = name.to_string();
            let db = self.app_state.db_service().cloned();
            if let Some(db) = db {
                let result = db.lock().ok().map(|service| service.create_group(&name));
                match result {
                    Some(Ok(group)) => {
                        let gname = group.name.clone();
                        self.app_state.add_group(group);
                        self.app_state.apply_filter();
                        self.app_state.add_notification(
                            &format!("Group '{}' created", gname),
                            NotificationLevel::Success,
                        );
                    }
                    Some(Err(e)) => {
                        self.app_state.add_notification(
                            &format!("Failed to create group: {}", e),
                            NotificationLevel::Error,
                        );
                    }
                    None => {
                        self.app_state.add_notification(
                            "Database locked",
                            NotificationLevel::Error,
                        );
                    }
                }
            }
        } else if let Some(rest) = message.strip_prefix("__rename_group:") {
            if let Some((id, new_name)) = rest.split_once(':') {
                let id = id.to_string();
                let new_name = new_name.to_string();
                let db = self.app_state.db_service().cloned();
                if let Some(db) = db {
                    let result = db.lock().ok().map(|service| service.rename_group(&id, &new_name));
                    match result {
                        Some(Ok(())) => {
                            self.app_state.rename_group_in_cache(&id, &new_name);
                            self.app_state.apply_filter();
                            self.app_state.add_notification(
                                "Group renamed",
                                NotificationLevel::Success,
                            );
                        }
                        Some(Err(e)) => {
                            self.app_state.add_notification(
                                &format!("Failed to rename: {}", e),
                                NotificationLevel::Error,
                            );
                        }
                        None => {
                            self.app_state.add_notification(
                                "Database locked",
                                NotificationLevel::Error,
                            );
                        }
                    }
                }
            }
        } else if let Some(rest) = message.strip_prefix("__move_password:") {
            if let Some((pw_id, group_id)) = rest.split_once(':') {
                let pw_id = pw_id.to_string();
                let group_id = group_id.to_string();
                let gid_opt = if group_id.is_empty() { None } else { Some(group_id.as_str()) };
                let db = self.app_state.db_service().cloned();
                if let Some(db) = db {
                    let result = db.lock().ok().map(|service| {
                        service.move_password_to_group(&pw_id, gid_opt)
                    });
                    match result {
                        Some(Ok(())) => {
                            let gid_string = if group_id.is_empty() { None } else { Some(group_id) };
                            self.app_state.update_password_group(&pw_id, gid_string);
                            self.app_state.apply_filter();
                            self.app_state.add_notification(
                                "Password moved",
                                NotificationLevel::Success,
                            );
                        }
                        Some(Err(e)) => {
                            self.app_state.add_notification(
                                &format!("Failed to move: {}", e),
                                NotificationLevel::Error,
                            );
                        }
                        None => {
                            self.app_state.add_notification(
                                "Database locked",
                                NotificationLevel::Error,
                            );
                        }
                    }
                }
            }
        } else if message == "__show_group_picker" {
            let password_id = self.app_state.tree.current_node().and_then(|node| {
                if let crate::tui::state::tree_state::TreeNodeId::Password(pid) = node.id {
                    Some(pid.to_string())
                } else {
                    None
                }
            });
            if let Some(pid) = password_id {
                let mut groups: Vec<(Option<String>, String)> = self
                    .app_state
                    .groups
                    .iter()
                    .map(|g| (Some(g.id.clone()), g.name.clone()))
                    .collect();
                groups.push((None, "Ungrouped".to_string()));
                self.main_screen.group_picker.show(pid, groups);
            }
        } else {
            // Normal toast notification
            self.app_state.add_notification(&message, NotificationLevel::Info);
        }
    }
}
