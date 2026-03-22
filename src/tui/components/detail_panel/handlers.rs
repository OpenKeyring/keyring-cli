//! Key event handlers for DetailPanel
//!
//! Contains keyboard handling logic including clipboard operations.

use super::DetailPanel;
use crate::tui::state::AppState;
use crate::tui::traits::{HandleResult, NotificationLevel};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

/// Handle key event with state mutation
pub fn handle_key_with_state(
    panel: &mut DetailPanel,
    key: KeyEvent,
    state: &mut AppState,
) -> HandleResult {
    if key.kind == KeyEventKind::Release {
        return HandleResult::Ignored;
    }

    match key.code {
        KeyCode::Char(' ') => {
            panel.toggle_password_visibility();
            HandleResult::Consumed
        }
        KeyCode::Char('C') => {
            copy_password_to_clipboard(state);
            HandleResult::Consumed
        }
        KeyCode::Char('c') => {
            copy_username_to_clipboard(state);
            HandleResult::Consumed
        }
        KeyCode::Char('o') => {
            state.add_notification("Opening URL...", NotificationLevel::Info);
            HandleResult::Consumed
        }
        _ => HandleResult::Ignored,
    }
}

/// Copy password to clipboard
fn copy_password_to_clipboard(state: &mut AppState) {
    if let crate::tui::state::DetailMode::PasswordDetail(id) = state.detail_mode {
        if let Some(record) = state.get_password_by_str(&id.to_string()).cloned() {
            let password = record.password.clone();
            match Clipboard::new() {
                Ok(mut clipboard) => match clipboard.set_text(&password) {
                    Ok(_) => {
                        state.add_notification(
                            "Password copied to clipboard (30s)",
                            NotificationLevel::Info,
                        );
                    }
                    Err(e) => {
                        state.add_notification(
                            &format!("Failed to copy password: {}", e),
                            NotificationLevel::Error,
                        );
                    }
                },
                Err(e) => {
                    state.add_notification(
                        &format!("Clipboard not available: {}", e),
                        NotificationLevel::Warning,
                    );
                }
            }
        }
    }
}

/// Copy username to clipboard
fn copy_username_to_clipboard(state: &mut AppState) {
    if let crate::tui::state::DetailMode::PasswordDetail(id) = state.detail_mode {
        if let Some(record) = state.get_password_by_str(&id.to_string()).cloned() {
            if let Some(ref username) = record.username {
                match Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(username) {
                        Ok(_) => {
                            state.add_notification(
                                "Username copied to clipboard",
                                NotificationLevel::Info,
                            );
                        }
                        Err(e) => {
                            state.add_notification(
                                &format!("Failed to copy username: {}", e),
                                NotificationLevel::Error,
                            );
                        }
                    },
                    Err(e) => {
                        state.add_notification(
                            &format!("Clipboard not available: {}", e),
                            NotificationLevel::Warning,
                        );
                    }
                }
            } else {
                state.add_notification("No username to copy", NotificationLevel::Warning);
            }
        }
    }
}
