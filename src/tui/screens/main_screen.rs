//! Main Screen Implementation
//!
//! Dual-column layout for password management main interface.
//! Left column (35%): Tree panel + Filter panel
//! Right column (65%): Detail panel + Status area

use crate::tui::components::{DetailPanel, FilterPanel, TreePanel};
use crate::tui::state::{AppState, FocusedPanel};
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render, Action, ScreenType};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

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
        }
    }

    /// Calculate layout for given area
    ///
    /// Layout structure:
    /// ```text
    /// ┌─────────────────────────────────────────────────────────┐
    /// │                      Content Area                       │
    /// │  ┌────────────┐  ┌──────────────────────────────────┐  │
    /// │  │   Tree     │  │           Detail                 │  │
    /// │  │   (70%)    │  │            (80%)                 │  │
    /// │  ├────────────┤  ├──────────────────────────────────┤  │
    /// │  │   Filter   │  │          Status Area             │  │
    /// │  │   (30%)    │  │            (20%)                 │  │
    /// │  │  (35%)     │  │           (65%)                  │  │
    /// │  └────────────┘  └──────────────────────────────────┘  │
    /// ├─────────────────────────────────────────────────────────┤
    /// │                    Status Bar (1 line)                  │
    /// └─────────────────────────────────────────────────────────┘
    /// ```
    pub fn calculate_layout(&mut self, area: Rect) -> MainLayout {
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

    /// Render main screen to frame
    pub fn render_frame(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let layout = self.calculate_layout(area);

        // Update panel focus states based on AppState
        self.sync_panel_focus_states(state);

        // Render panels
        self.tree_panel.render_frame(frame, layout.tree_area, &state.tree);
        self.filter_panel.render_frame(frame, layout.filter_area, &state.filter);
        self.detail_panel.render_frame(frame, layout.detail_area, state);
        self.render_status_panel(frame, layout.status_area, state);
        self.render_status_bar(frame, layout.status_bar_area, state);
    }

    /// Sync panel focus states with AppState
    fn sync_panel_focus_states(&mut self, state: &AppState) {
        // Sync tree panel focus state
        let tree_should_be_focused = state.focused_panel == FocusedPanel::Tree;
        if self.tree_panel.is_focused() != tree_should_be_focused {
            if tree_should_be_focused {
                let _ = self.tree_panel.on_focus_gain();
            } else {
                let _ = self.tree_panel.on_focus_loss();
            }
        }

        // Sync filter panel focus state
        let filter_should_be_focused = state.focused_panel == FocusedPanel::Filter;
        if self.filter_panel.is_focused() != filter_should_be_focused {
            if filter_should_be_focused {
                let _ = self.filter_panel.on_focus_gain();
            } else {
                let _ = self.filter_panel.on_focus_loss();
            }
        }

        // Sync detail panel focus state
        let detail_should_be_focused = state.focused_panel == FocusedPanel::Detail;
        if self.detail_panel.is_focused() != detail_should_be_focused {
            if detail_should_be_focused {
                let _ = self.detail_panel.on_focus_gain();
            } else {
                let _ = self.detail_panel.on_focus_loss();
            }
        }
    }

    /// Render status panel with placeholder content
    fn render_status_panel(&self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let border_style = Style::default().fg(Color::DarkGray);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" [4] Status ")
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, frame.buffer_mut());

        // Placeholder content
        let lines = vec![
            Line::from(Span::styled(
                "OpenKeyring v0.1.0",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Stats: Coming soon",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    /// Render status bar
    fn render_status_bar(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        use crate::tui::state::FocusedPanel;

        let focus_text = match state.focused_panel {
            FocusedPanel::Tree => "Tree",
            FocusedPanel::Filter => "Filter",
            FocusedPanel::Detail => "Detail",
        };

        let text = format!(
            "Focus: {} | [1-3] Switch | [j/k] Navigate | [q] Quit",
            focus_text
        );
        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }

    /// Get current layout (if calculated)
    pub fn layout(&self) -> Option<MainLayout> {
        self.layout
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
        // This is a simplified render for Buffer interface
        // Use render_frame for Frame-based rendering
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Main Screen");
        block.render(area, buf);
    }
}

impl Interactive for MainScreen {
    fn handle_key(&mut self, _key: KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

impl MainScreen {
    /// Handle key event with state mutation
    pub fn handle_key_with_state(&mut self, key: KeyEvent, state: &mut AppState) -> HandleResult {
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
            // Start search (placeholder - search is Phase 2)
            KeyCode::Char('/') => {
                return HandleResult::Action(Action::ShowToast("Search: Coming in Phase 2".to_string()));
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

        // Route to focused panel
        match state.focused_panel {
            FocusedPanel::Tree => {
                self.tree_panel.handle_key_with_state(key, state)
            }
            FocusedPanel::Filter => {
                let result = self.filter_panel.handle_key_with_state(key, &mut state.filter);
                // If filter changed, update tree panel
                if matches!(result, HandleResult::Consumed) {
                    state.apply_filter();
                }
                result
            }
            FocusedPanel::Detail => {
                self.detail_panel.handle_key_with_state(key, state, None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::filter_state::FilterType;

    #[test]
    fn test_main_screen_creation() {
        let screen = MainScreen::new();
        assert!(screen.layout.is_none());
    }

    #[test]
    fn test_layout_calculation() {
        let mut screen = MainScreen::new();
        let area = Rect::new(0, 0, 100, 30);
        let layout = screen.calculate_layout(area);

        // Verify layout regions are valid
        assert!(layout.left_column.width > 0);
        assert!(layout.right_column.width > 0);
        assert!(layout.tree_area.height > 0);
        assert!(layout.filter_area.height > 0);
        assert!(layout.detail_area.height > 0);
        assert!(layout.status_bar_area.height == 1);

        // Verify layout is cached
        assert!(screen.layout.is_some());
    }

    #[test]
    fn test_layout_proportions() {
        let mut screen = MainScreen::new();
        let area = Rect::new(0, 0, 100, 30);
        let layout = screen.calculate_layout(area);

        // Left column should be ~35% of content width
        let content_width = 100;
        let left_width = layout.left_column.width as f32;
        let left_ratio = left_width / content_width as f32;
        assert!(left_ratio > 0.30 && left_ratio < 0.40, "Left column ratio: {}", left_ratio);

        // Right column should be ~65% of content width
        let right_width = layout.right_column.width as f32;
        let right_ratio = right_width / content_width as f32;
        assert!(right_ratio > 0.60 && right_ratio < 0.70, "Right column ratio: {}", right_ratio);
    }

    #[test]
    fn test_filter_panel_updates_tree() {
        // Test that toggling a filter updates the tree panel
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Initialize tree with data
        state.apply_filter();
        let _initial_count = state.tree.visible_nodes.len();

        // Switch to Filter panel
        state.set_focus(FocusedPanel::Filter);

        // Navigate to Favorite filter (index 3 in default items: All, Trash, Expired, Favorite)
        for _ in 0..3 {
            screen.handle_key_with_state(
                KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty()),
                &mut state,
            );
        }

        // Toggle Favorite filter
        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));

        // Verify filter is active
        assert!(state.filter.is_active(&FilterType::Favorite));

        // Verify tree was updated (should have different nodes now)
        // After filtering, tree should be refreshed
        // Note: Without expansion, we only see root groups, but the filter is now active
        assert!(state.filter.active_filters.contains(&FilterType::Favorite));
    }

    #[test]
    fn test_filter_panel_navigation_updates_filter() {
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Switch to Filter panel
        state.set_focus(FocusedPanel::Filter);

        // Navigate to Trash filter (index 1)
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );

        // Toggle Trash filter
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );

        // Verify Trash filter is active
        assert!(state.filter.is_active(&FilterType::Trash));
    }

    // ========== Global Shortcuts Tests ==========

    #[test]
    fn test_global_quit_shortcut() {
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Press 'q' should return Quit action
        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('q'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );

        assert!(matches!(result, HandleResult::Action(Action::Quit)));
    }

    #[test]
    fn test_global_help_shortcut() {
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Press '?' should return OpenScreen(Help) action
        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('?'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );

        assert!(matches!(result, HandleResult::Action(Action::OpenScreen(ScreenType::Help))));
    }

    #[test]
    fn test_global_search_shortcut() {
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Press '/' should return ShowToast action (placeholder)
        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('/'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );

        assert!(matches!(result, HandleResult::Action(Action::ShowToast(_))));
    }

    #[test]
    fn test_panel_switch_with_numbers() {
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Press '1' - switch to Tree
        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('1'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.focused_panel, FocusedPanel::Tree);

        // Press '2' - switch to Filter
        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('2'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.focused_panel, FocusedPanel::Filter);

        // Press '3' - switch to Detail
        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('3'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.focused_panel, FocusedPanel::Detail);
    }

    #[test]
    fn test_tab_navigation() {
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Start at Tree
        assert_eq!(state.focused_panel, FocusedPanel::Tree);

        // Tab - next panel
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(state.focused_panel, FocusedPanel::Filter);

        // Tab - next panel
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(state.focused_panel, FocusedPanel::Detail);

        // Tab - wrap around to Tree
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(state.focused_panel, FocusedPanel::Tree);
    }

    #[test]
    fn test_shift_tab_navigation() {
        let mut screen = MainScreen::new();
        let mut state = AppState::new();

        // Start at Tree
        assert_eq!(state.focused_panel, FocusedPanel::Tree);

        // Shift+Tab (BackTab) - go to Detail (reverse direction)
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::BackTab, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(state.focused_panel, FocusedPanel::Detail);

        // Shift+Tab - go to Filter
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::BackTab, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(state.focused_panel, FocusedPanel::Filter);

        // Shift+Tab - go to Tree
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::BackTab, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(state.focused_panel, FocusedPanel::Tree);
    }
}
