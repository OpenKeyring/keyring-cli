//! TUI State Management Module
//!
//! Provides unified application state management, including filtering,
//! tree navigation, and selection states.

pub mod app_state;
pub mod filter_state;
pub mod selection;
pub mod tree_state;

pub use app_state::{AppState, DetailMode, FocusedPanel};
pub use filter_state::{FilterState, FilterType};
pub use selection::SelectionState;
pub use tree_state::{NodeType, TreeNodeId, TreeState, VisibleNode};
