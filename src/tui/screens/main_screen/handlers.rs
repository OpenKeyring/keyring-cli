//! Key event handlers for MainScreen
//!
//! Contains keyboard handling logic for the main screen.

use super::MainScreen;
use crate::tui::components::ConfirmAction;
use crate::tui::state::{AppState, FocusedPanel};
use crate::tui::traits::{Action, HandleResult, ScreenType};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

/// Handle key event with state mutation
pub fn handle_key_with_state(screen: &mut MainScreen, key: KeyEvent, state: &mut AppState) -> HandleResult {
    // Only handle press events
    if key.kind == KeyEventKind::Release {
        return HandleResult::Ignored;
    }

    // Global shortcuts (take priority over panel-specific handling)
    match key.code {
        // Quit application
        KeyCode::Char('q') => {
            return HandleResult::Action(Action::Quit);
        }
        // Show help
        KeyCode::Char('?') => {
            return HandleResult::Action(Action::OpenScreen(ScreenType::Help));
        }
        // Open trash screen (Shift+T)
        KeyCode::Char('T') => {
            return HandleResult::Action(Action::OpenScreen(ScreenType::Trash));
        }
        // Start search
        KeyCode::Char('/') => {
            screen.search_bar.show();
            return HandleResult::Consumed;
        }
        // Create new password
        KeyCode::Char('n') => {
            return HandleResult::Action(Action::OpenScreen(ScreenType::NewPassword));
        }
        // Edit selected password
        KeyCode::Char('e') => {
            if let Some(password_id) = state.selection.selected_password {
                let id_str = password_id.to_string();
                return HandleResult::Action(Action::OpenScreen(ScreenType::EditPassword(id_str)));
            } else {
                return HandleResult::Action(Action::ShowToast("No password selected".to_string()));
            }
        }
        // Delete selected password (move to trash)
        KeyCode::Char('d') => {
            if let Some(password_id) = state.selection.selected_password {
                let id_str = password_id.to_string();
                // Get password name for confirmation dialog
                let password_name = state.get_password_by_str(&id_str)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                // Show confirmation dialog
                return HandleResult::Action(Action::OpenScreen(ScreenType::ConfirmDialog(
                    ConfirmAction::DeletePassword { password_id: id_str, password_name }
                )));
            } else {
                return HandleResult::Action(Action::ShowToast("No password selected".to_string()));
            }
        }
        // Toggle favorite
        KeyCode::Char('f') | KeyCode::Char('*') => {
            if let Some(password_id) = state.selection.selected_password {
                if let Some(password) = state.get_password(password_id).cloned() {
                    let mut updated = password.clone();
                    updated.is_favorite = !updated.is_favorite;

                    // Store the new favorite state before moving
                    let is_now_favorite = updated.is_favorite;

                    // Update in cache
                    state.update_password_in_cache(updated);

                    // Show notification
                    let message = if is_now_favorite {
                        "Added to favorites"
                    } else {
                        "Removed from favorites"
                    };
                    state.add_notification(message, crate::tui::traits::NotificationLevel::Success);
                    return HandleResult::Consumed;
                }
            }
            return HandleResult::Action(Action::ShowToast("No password selected".to_string()));
        }
        // Panel switching with number keys
        KeyCode::Char('1') => {
            state.set_focus(FocusedPanel::Tree);
            return HandleResult::Consumed;
        }
        KeyCode::Char('2') => {
            state.set_focus(FocusedPanel::Filter);
            return HandleResult::Consumed;
        }
        KeyCode::Char('3') => {
            state.set_focus(FocusedPanel::Detail);
            return HandleResult::Consumed;
        }
        // Tab navigation
        KeyCode::Tab => {
            state.next_panel();
            return HandleResult::Consumed;
        }
        KeyCode::BackTab => {
            state.prev_panel();
            return HandleResult::Consumed;
        }
        _ => {}
    }

    // Route to search bar if visible (takes priority over panel handling)
    if screen.search_bar.is_visible() {
        let result = screen.search_bar.handle_key(key);
        if matches!(result, HandleResult::NeedsRender) {
            // Update filter with search query and reapply
            state.filter.set_search_query(screen.search_bar.query().to_string());
            state.apply_filter();
        }
        return result;
    }

    // Route to focused panel
    match state.focused_panel {
        FocusedPanel::Tree => {
            screen.tree_panel.handle_key_with_state(key, state)
        }
        FocusedPanel::Filter => {
            let result = screen.filter_panel.handle_key_with_state(key, &mut state.filter);
            // If filter changed, update tree panel
            if matches!(result, HandleResult::Consumed) {
                state.apply_filter();
            }
            result
        }
        FocusedPanel::Detail => {
            screen.detail_panel.handle_key_with_state(key, state, None)
        }
    }
}
