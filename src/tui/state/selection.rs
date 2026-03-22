//! Selection state management

use uuid::Uuid;

/// Selection state
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Currently selected password entry ID
    pub selected_password: Option<Uuid>,
    /// Currently selected group ID (for filter context)
    pub selected_group: Option<Uuid>,
}

impl SelectionState {
    /// Create new selection state
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a password entry
    pub fn select_password(&mut self, id: Uuid) {
        self.selected_password = Some(id);
    }

    /// Select a group
    pub fn select_group(&mut self, id: Option<Uuid>) {
        self.selected_group = id;
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.selected_password = None;
        self.selected_group = None;
    }

    /// Check if there is a selected password
    pub fn has_selection(&self) -> bool {
        self.selected_password.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_default() {
        let state = SelectionState::default();
        assert!(state.selected_password.is_none());
        assert!(state.selected_group.is_none());
    }

    #[test]
    fn test_select_password() {
        let mut state = SelectionState::default();
        let id = Uuid::new_v4();

        state.select_password(id);
        assert_eq!(state.selected_password, Some(id));

        state.clear();
        assert!(state.selected_password.is_none());
    }

    #[test]
    fn test_select_group() {
        let mut state = SelectionState::default();
        let id = Uuid::new_v4();

        state.select_group(Some(id));
        assert_eq!(state.selected_group, Some(id));

        state.select_group(None);
        assert!(state.selected_group.is_none());
    }

    #[test]
    fn test_has_selection() {
        let mut state = SelectionState::default();
        assert!(!state.has_selection());

        state.select_password(Uuid::new_v4());
        assert!(state.has_selection());

        state.clear();
        assert!(!state.has_selection());
    }
}
