//! Application global state management

use super::{FilterState, TreeState, SelectionState};
use crate::tui::traits::{Notification, NotificationLevel, NotificationId};
use std::collections::VecDeque;
use std::time::Instant;
use uuid::Uuid;

/// Currently focused panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    /// Group tree
    #[default]
    Tree,
    /// Filter conditions
    Filter,
    /// Detail panel
    Detail,
}

/// Detail display mode
#[derive(Debug, Clone, Default)]
pub enum DetailMode {
    /// No selection, show project information
    #[default]
    ProjectInfo,
    /// Show password detail
    PasswordDetail(Uuid),
}

/// Application global state
#[derive(Debug)]
pub struct AppState {
    /// Filter condition state
    pub filter: FilterState,
    /// Group tree state
    pub tree: TreeState,
    /// Current selection
    pub selection: SelectionState,
    /// Detail panel display mode
    pub detail_mode: DetailMode,
    /// Notification message queue
    pub notifications: VecDeque<Notification>,
    /// Currently focused panel
    pub focused_panel: FocusedPanel,
    /// Notification ID counter
    notification_counter: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            filter: FilterState::default(),
            tree: TreeState::default(),
            selection: SelectionState::default(),
            detail_mode: DetailMode::default(),
            notifications: VecDeque::new(),
            focused_panel: FocusedPanel::default(),
            notification_counter: 0,
        }
    }
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set focused panel
    pub fn set_focus(&mut self, panel: FocusedPanel) {
        self.focused_panel = panel;
    }

    /// Switch to next panel
    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Tree => FocusedPanel::Filter,
            FocusedPanel::Filter => FocusedPanel::Detail,
            FocusedPanel::Detail => FocusedPanel::Tree,
        };
    }

    /// Add a notification
    pub fn add_notification(&mut self, message: &str, level: NotificationLevel) {
        self.notification_counter += 1;
        let notification = Notification {
            id: Some(NotificationId::new(self.notification_counter)),
            message: message.to_string(),
            level,
            duration: Some(level.default_duration()),
            dismissible: true,
            created_at: Instant::now(),
            source: None,
            progress: None,
        };
        self.notifications.push_back(notification);

        // Limit queue length
        while self.notifications.len() > 5 {
            self.notifications.pop_front();
        }
    }

    /// Clear expired notifications
    pub fn cleanup_notifications(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|n| {
            let duration = n.effective_duration();
            duration.as_secs() == 0 || now.duration_since(n.created_at) < duration
        });
    }

    /// Select a password and update detail mode
    pub fn select_password(&mut self, id: Uuid) {
        self.selection.select_password(id);
        self.detail_mode = DetailMode::PasswordDetail(id);
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection.clear();
        self.detail_mode = DetailMode::ProjectInfo;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.selection.selected_password.is_none());
        assert_eq!(state.focused_panel, FocusedPanel::Tree);
        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_focus_panel() {
        let mut state = AppState::default();

        state.set_focus(FocusedPanel::Filter);
        assert_eq!(state.focused_panel, FocusedPanel::Filter);

        state.set_focus(FocusedPanel::Detail);
        assert_eq!(state.focused_panel, FocusedPanel::Detail);
    }

    #[test]
    fn test_next_panel() {
        let mut state = AppState::default();

        assert_eq!(state.focused_panel, FocusedPanel::Tree);

        state.next_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Filter);

        state.next_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Detail);

        state.next_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Tree);
    }

    #[test]
    fn test_notification_queue() {
        let mut state = AppState::default();

        state.add_notification("Test message", NotificationLevel::Info);
        assert_eq!(state.notifications.len(), 1);
    }

    #[test]
    fn test_notification_queue_limit() {
        let mut state = AppState::default();

        for i in 0..10 {
            state.add_notification(&format!("Message {}", i), NotificationLevel::Info);
        }

        assert_eq!(state.notifications.len(), 5);
    }

    #[test]
    fn test_select_password() {
        let mut state = AppState::default();
        let id = Uuid::new_v4();

        state.select_password(id);

        assert_eq!(state.selection.selected_password, Some(id));
        assert!(matches!(state.detail_mode, DetailMode::PasswordDetail(_)));
    }

    #[test]
    fn test_clear_selection() {
        let mut state = AppState::default();
        let id = Uuid::new_v4();

        state.select_password(id);
        state.clear_selection();

        assert!(state.selection.selected_password.is_none());
        assert!(matches!(state.detail_mode, DetailMode::ProjectInfo));
    }
}
