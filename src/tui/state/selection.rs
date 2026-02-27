//! Selection state management (stub)

use uuid::Uuid;

/// Selection state (stub)
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Selected password ID
    pub selected_password: Option<Uuid>,
    /// Selected group ID
    pub selected_group: Option<Uuid>,
}
