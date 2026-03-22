//! Main Screen Types
//!
//! Type definitions for the main screen layout.

use ratatui::layout::Rect;

/// Main interface layout regions
#[derive(Debug, Clone, Copy)]
pub struct MainLayout {
    /// Left column (35%)
    pub left_column: Rect,
    /// Right column (65%)
    pub right_column: Rect,
    /// Tree area (70% of left column)
    pub tree_area: Rect,
    /// Filter area (30% of left column)
    pub filter_area: Rect,
    /// Detail area (80% of right column)
    pub detail_area: Rect,
    /// Reserved status area (20% of right column)
    pub status_area: Rect,
    /// Bottom status bar
    pub status_bar_area: Rect,
}
