//! Main Screen Component
//!
//! Dual-column layout for password management main interface.
//! Left column (35%): Tree panel + Filter panel
//! Right column (65%): Detail panel + Status area

use crate::tui::components::{DetailPanel, FilterPanel, GroupPicker, SearchBar, TreePanel};
use crate::tui::state::AppState;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget, Frame};

mod handlers;
mod render;
#[cfg(test)]
mod tests;
mod types;

pub use types::MainLayout;

/// Main screen component
///
/// Implements the primary password management interface with dual-column layout.
pub struct MainScreen {
    /// Component ID
    id: ComponentId,
    /// Cached layout
    layout: Option<MainLayout>,
    /// Tree panel component (password groups tree)
    tree_panel: TreePanel,
    /// Filter panel component
    filter_panel: FilterPanel,
    /// Detail panel component
    detail_panel: DetailPanel,
    /// Search bar component
    search_bar: SearchBar,
    /// Group picker overlay component
    pub group_picker: GroupPicker,
}

impl MainScreen {
    /// Create a new main screen
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(100),
            layout: None,
            tree_panel: TreePanel::new(),
            filter_panel: FilterPanel::new(),
            detail_panel: DetailPanel::new(),
            search_bar: SearchBar::new(),
            group_picker: GroupPicker::new(),
        }
    }

    /// Calculate layout for given area
    pub fn calculate_layout(&mut self, area: Rect) -> MainLayout {
        use ratatui::layout::{Constraint, Direction, Layout};

        // Bottom status bar: 1 line fixed
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Main content area
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        let content_area = main_chunks[0];
        let status_bar_area = main_chunks[1];

        // Left/right columns: 35%/65%
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35), // Left column
                Constraint::Percentage(65), // Right column
            ])
            .split(content_area);

        let left_column = columns[0];
        let right_column = columns[1];

        // Left column: tree (70%) + filter (30%)
        let left_panes = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70), // Tree
                Constraint::Percentage(30), // Filter
            ])
            .split(left_column);

        // Right column: detail (80%) + status (20%)
        let right_panes = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80), // Detail
                Constraint::Percentage(20), // Reserved
            ])
            .split(right_column);

        let layout = MainLayout {
            left_column,
            right_column,
            tree_area: left_panes[0],
            filter_area: left_panes[1],
            detail_area: right_panes[0],
            status_area: right_panes[1],
            status_bar_area,
        };

        self.layout = Some(layout);
        layout
    }

    /// Minimum terminal size for proper UI display
    pub const MIN_WIDTH: u16 = 80;
    pub const MIN_HEIGHT: u16 = 24;

    /// Render main screen to frame
    pub fn render_frame(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        render::render_frame(self, frame, area, state);
    }

    /// Get current layout (if calculated)
    pub fn layout(&self) -> Option<MainLayout> {
        self.layout
    }

    /// Handle key event with state mutation
    pub fn handle_key_with_state(&mut self, key: KeyEvent, state: &mut AppState) -> HandleResult {
        handlers::handle_key_with_state(self, key, state)
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for MainScreen {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }
}

impl Render for MainScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{Block, Borders};

        // This is a simplified render for Buffer interface
        // Use render_frame for Frame-based rendering
        let block = Block::default().borders(Borders::ALL).title("Main Screen");
        block.render(area, buf);
    }
}

impl Interactive for MainScreen {
    fn handle_key(&mut self, _key: KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
}
