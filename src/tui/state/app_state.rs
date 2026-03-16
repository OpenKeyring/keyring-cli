//! Application global state management

use super::{FilterState, TreeState, SelectionState};
use crate::tui::traits::{Notification, NotificationLevel, NotificationId};
use crate::tui::models::password::PasswordRecord;
use crate::tui::services::{TuiDatabaseService, TuiClipboardService, TuiCryptoService};
use crate::tui::config::TuiConfig;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
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

/// Data source mode for AppState
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataSourceMode {
    /// Use mock data (for development/testing)
    #[default]
    Mock,
    /// Use real vault (for production)
    Vault,
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

    // === Phase 3: Password Cache ===
    /// Password cache for UI display (id -> record)
    password_cache: HashMap<String, PasswordRecord>,
    /// All passwords list (for list view)
    password_list: Vec<PasswordRecord>,

    // === Phase 3: Real Services ===
    /// Data source mode (mock or vault)
    pub data_source: DataSourceMode,
    /// Database service (optional, for vault mode)
    db_service: Option<Arc<Mutex<TuiDatabaseService>>>,
    /// Clipboard service
    pub clipboard_service: Option<TuiClipboardService>,
    /// Crypto service
    pub crypto_service: Option<TuiCryptoService>,
    /// TUI configuration
    pub config: TuiConfig,
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
            password_cache: HashMap::new(),
            password_list: Vec::new(),
            data_source: DataSourceMode::default(),
            db_service: None,
            clipboard_service: None,
            crypto_service: None,
            config: TuiConfig::default(),
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

    /// Switch to previous panel
    pub fn prev_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Tree => FocusedPanel::Detail,
            FocusedPanel::Filter => FocusedPanel::Tree,
            FocusedPanel::Detail => FocusedPanel::Filter,
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

    /// Apply current filter and update visible nodes
    pub fn apply_filter(&mut self) {
        use crate::tui::state::tree_state::{VisibleNode, TreeNodeId, NodeType};

        // Build visible nodes from password cache
        let nodes: Vec<VisibleNode> = self.password_list
            .iter()
            .filter(|p| {
                // Apply search filter if set
                if let Some(ref query) = self.filter.search_query {
                    if !query.is_empty() {
                        let query_lower = query.to_lowercase();
                        return p.name.to_lowercase().contains(&query_lower) ||
                               p.username.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false);
                    }
                }
                true
            })
            .map(|p| VisibleNode {
                id: TreeNodeId::Password(Uuid::parse_str(&p.id).unwrap_or_else(|_| Uuid::nil())),
                level: 1,
                node_type: NodeType::Password,
                label: p.name.clone(),
                child_count: 0,
            })
            .collect();

        self.tree.set_visible_nodes(nodes);
    }

    /// Get password name by ID (for UI display)
    pub fn get_password_name(&self, id: Uuid) -> String {
        self.password_cache
            .get(&id.to_string())
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Get password record by ID
    pub fn get_password(&self, id: Uuid) -> Option<&PasswordRecord> {
        self.password_cache.get(&id.to_string())
    }

    /// Get password record by ID string
    pub fn get_password_by_str(&self, id: &str) -> Option<&PasswordRecord> {
        self.password_cache.get(id)
    }

    // === Phase 3: Cache Management ===

    /// Refresh password cache from loaded data
    pub fn refresh_password_cache(&mut self, passwords: Vec<PasswordRecord>) {
        // Clear and rebuild cache
        self.password_cache.clear();
        for p in &passwords {
            self.password_cache.insert(p.id.clone(), p.clone());
        }
        self.password_list = passwords;

        // Reapply filter to update visible nodes
        self.apply_filter();
    }

    /// Set database service
    pub fn set_db_service(&mut self, service: Arc<Mutex<TuiDatabaseService>>) {
        self.db_service = Some(service);
    }

    /// Set clipboard service
    pub fn set_clipboard_service(&mut self, service: TuiClipboardService) {
        self.clipboard_service = Some(service);
    }

    /// Set crypto service
    pub fn set_crypto_service(&mut self, service: TuiCryptoService) {
        self.crypto_service = Some(service);
    }

    /// Get database service
    pub fn db_service(&self) -> Option<&Arc<Mutex<TuiDatabaseService>>> {
        self.db_service.as_ref()
    }

    /// Get mutable reference to clipboard service
    pub fn clipboard_service_mut(&mut self) -> Option<&mut TuiClipboardService> {
        self.clipboard_service.as_mut()
    }

    /// Update a password in cache (after successful db update)
    pub fn update_password_in_cache(&mut self, password: PasswordRecord) {
        if let Some(existing) = self.password_cache.get_mut(&password.id) {
            *existing = password.clone();
        } else {
            self.password_cache.insert(password.id.clone(), password.clone());
        }

        // Update in list
        if let Some(pos) = self.password_list.iter().position(|p| p.id == password.id) {
            self.password_list[pos] = password;
        }
    }

    /// Remove a password from cache (after successful db delete)
    pub fn remove_password_from_cache(&mut self, id: &str) {
        self.password_cache.remove(id);
        self.password_list.retain(|p| p.id != id);
        self.apply_filter();
    }

    /// Add a password to cache (after successful db create)
    pub fn add_password_to_cache(&mut self, password: PasswordRecord) {
        self.password_cache.insert(password.id.clone(), password.clone());
        self.password_list.push(password);
        self.apply_filter();
    }

    /// Get all passwords (for iteration)
    pub fn all_passwords(&self) -> &[PasswordRecord] {
        &self.password_list
    }

    /// Permanently delete a password from cache
    /// This removes the password completely (cannot be restored)
    pub fn permanent_delete_password(&mut self, id: &str) -> bool {
        if self.password_cache.contains_key(id) {
            self.password_cache.remove(id);
            self.password_list.retain(|p| p.id != id);
            self.apply_filter();
            true
        } else {
            false
        }
    }

    /// Empty trash - permanently delete all passwords marked as deleted
    /// Returns the number of passwords deleted
    pub fn empty_trash(&mut self) -> usize {
        let deleted_ids: Vec<String> = self.password_list
            .iter()
            .filter(|p| p.is_deleted)
            .map(|p| p.id.clone())
            .collect();

        let count = deleted_ids.len();

        for id in &deleted_ids {
            self.password_cache.remove(id);
        }
        self.password_list.retain(|p| !p.is_deleted);
        self.apply_filter();

        count
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
    fn test_prev_panel() {
        let mut state = AppState::default();

        assert_eq!(state.focused_panel, FocusedPanel::Tree);

        state.prev_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Detail);

        state.prev_panel();
        assert_eq!(state.focused_panel, FocusedPanel::Filter);

        state.prev_panel();
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

    #[test]
    fn test_permanent_delete_password() {
        let mut state = AppState::default();
        let mut p1 = PasswordRecord::new("id1", "Password 1", "pass1");
        p1.is_deleted = true;

        state.refresh_password_cache(vec![p1.clone()]);

        assert_eq!(state.all_passwords().len(), 1);

        // Permanent delete
        let result = state.permanent_delete_password("id1");
        assert!(result);
        assert!(state.all_passwords().is_empty());

        // Delete non-existent
        let result = state.permanent_delete_password("nonexistent");
        assert!(!result);
    }

    #[test]
    fn test_empty_trash() {
        let mut state = AppState::default();

        let mut p1 = PasswordRecord::new("id1", "Password 1", "pass1");
        p1.is_deleted = true;

        let p2 = PasswordRecord::new("id2", "Password 2", "pass2"); // not deleted

        let mut p3 = PasswordRecord::new("id3", "Password 3", "pass3");
        p3.is_deleted = true;

        state.refresh_password_cache(vec![p1, p2, p3]);

        assert_eq!(state.all_passwords().len(), 3);

        // Empty trash
        let count = state.empty_trash();
        assert_eq!(count, 2);
        assert_eq!(state.all_passwords().len(), 1);
        assert_eq!(state.all_passwords()[0].id, "id2");

        // Empty again (should do nothing)
        let count = state.empty_trash();
        assert_eq!(count, 0);
        assert_eq!(state.all_passwords().len(), 1);
    }
}
