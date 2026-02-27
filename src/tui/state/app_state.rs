//! Application global state (stub)

use super::{FilterState, TreeState, SelectionState};

/// Focused panel (stub)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Tree,
    Filter,
    Detail,
}

/// Detail mode (stub)
#[derive(Debug, Clone, Default)]
pub enum DetailMode {
    #[default]
    Empty,
    PasswordDetail(uuid::Uuid),
}

/// Application state (stub)
#[derive(Debug)]
pub struct AppState {
    pub filter: FilterState,
    pub tree: TreeState,
    pub selection: SelectionState,
    pub detail_mode: DetailMode,
    pub focused_panel: FocusedPanel,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            filter: FilterState::default(),
            tree: TreeState::default(),
            selection: SelectionState::default(),
            detail_mode: DetailMode::default(),
            focused_panel: FocusedPanel::default(),
        }
    }
}
