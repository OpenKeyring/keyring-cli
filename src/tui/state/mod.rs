//! TUI State Management Module
//!
//! Provides unified application state management, including filtering,
//! tree navigation, and selection states.

pub mod app_state;
pub mod filter_state;
pub mod tree_state;
pub mod selection;

pub use app_state::AppState;
pub use filter_state::{FilterState, FilterType};
pub use tree_state::{TreeState, TreeNodeId, NodeType, VisibleNode};
pub use selection::SelectionState;
