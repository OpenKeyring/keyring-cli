//! Render implementation for MainScreen
//!
//! Contains rendering logic for the main screen layout.

use super::MainScreen;
use crate::tui::state::{AppState, FocusedPanel};
use crate::tui::traits::Component;
use ratatui::{
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render main screen to frame
pub fn render_frame(screen: &mut MainScreen, frame: &mut Frame, area: Rect, state: &AppState) {
    // Check minimum terminal size
    if area.width < MainScreen::MIN_WIDTH || area.height < MainScreen::MIN_HEIGHT {
        render_size_warning(frame, area);
        return;
    }

    let layout = screen.calculate_layout(area);

    // Update panel focus states based on AppState
    sync_panel_focus_states(screen, state);

    // Render panels
    let has_active_filters = state.filter.has_active_filters();
    screen.tree_panel.render_frame_with_context(
        frame,
        layout.tree_area,
        &state.tree,
        has_active_filters,
    );
    screen
        .filter_panel
        .render_frame(frame, layout.filter_area, &state.filter);
    screen
        .detail_panel
        .render_frame(frame, layout.detail_area, state);
    render_status_panel(frame, layout.status_area, state);
    render_status_bar(frame, layout.status_bar_area, state);

    // Render search bar (overlay at top, before notifications)
    if screen.search_bar.is_visible() {
        let search_area = Rect::new(
            area.x + area.width.saturating_sub(60) / 2,
            area.y + 1,
            60.min(area.width),
            3,
        );
        screen.search_bar.render_frame(frame, search_area);
    }

    // Render toast notifications on top (after all other panels)
    render_notifications(frame, area, state);
}

/// Sync panel focus states with AppState
fn sync_panel_focus_states(screen: &mut MainScreen, state: &AppState) {
    // Sync tree panel focus state
    let tree_should_be_focused = state.focused_panel == FocusedPanel::Tree;
    if screen.tree_panel.is_focused() != tree_should_be_focused {
        if tree_should_be_focused {
            let _ = screen.tree_panel.on_focus_gain();
        } else {
            let _ = screen.tree_panel.on_focus_loss();
        }
    }

    // Sync filter panel focus state
    let filter_should_be_focused = state.focused_panel == FocusedPanel::Filter;
    if screen.filter_panel.is_focused() != filter_should_be_focused {
        if filter_should_be_focused {
            let _ = screen.filter_panel.on_focus_gain();
        } else {
            let _ = screen.filter_panel.on_focus_loss();
        }
    }

    // Sync detail panel focus state
    let detail_should_be_focused = state.focused_panel == FocusedPanel::Detail;
    if screen.detail_panel.is_focused() != detail_should_be_focused {
        if detail_should_be_focused {
            let _ = screen.detail_panel.on_focus_gain();
        } else {
            let _ = screen.detail_panel.on_focus_loss();
        }
    }
}

/// Render terminal size warning
fn render_size_warning(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠ Terminal too small",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Current: {}x{}", area.width, area.height),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(
                "Required: {}x{}",
                MainScreen::MIN_WIDTH,
                MainScreen::MIN_HEIGHT
            ),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Please resize your terminal",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Render status panel with real statistics
fn render_status_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let border_style = Style::default().fg(Color::Rgb(70, 70, 90));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Stats ",
            Style::default().fg(Color::Rgb(120, 140, 170)),
        ))
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, frame.buffer_mut());

    let passwords = state.all_passwords();
    let total = passwords.iter().filter(|p| !p.is_deleted).count();
    let favorites = passwords.iter().filter(|p| p.is_favorite && !p.is_deleted).count();
    let trash = passwords.iter().filter(|p| p.is_deleted).count();

    let stat_style = Style::default().fg(Color::Rgb(180, 180, 200));
    let label_style = Style::default().fg(Color::Rgb(120, 140, 170));

    let lines = vec![
        Line::from(vec![
            Span::styled("Total: ", label_style),
            Span::styled(total.to_string(), stat_style),
            Span::styled("  Favorites: ", label_style),
            Span::styled(favorites.to_string(), stat_style),
            Span::styled("  Trash: ", label_style),
            Span::styled(trash.to_string(), stat_style),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Render context-aware status bar
fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let shortcuts = match state.focused_panel {
        FocusedPanel::Tree => {
            "[n] New  [e] Edit  [d] Delete  [/] Search  [Tab] Switch  [?] Help  [q] Quit"
        }
        FocusedPanel::Filter => {
            "[Enter] Apply  [j/k] Navigate  [Tab] Switch  [?] Help  [q] Quit"
        }
        FocusedPanel::Detail => {
            "[c] Copy User  [C] Copy Pwd  [Space] Toggle  [e] Edit  [d] Delete  [?] Help  [q] Quit"
        }
    };

    let style = Style::default()
        .bg(Color::Rgb(25, 25, 40))
        .fg(Color::Rgb(180, 180, 200));

    // Fill background
    let bg = ratatui::widgets::Block::default().style(style);
    frame.render_widget(bg, area);

    let paragraph = Paragraph::new(format!(" {}", shortcuts)).style(style);
    frame.render_widget(paragraph, area);
}

/// Render toast notifications
fn render_notifications(frame: &mut Frame, area: Rect, state: &AppState) {
    use crate::tui::traits::NotificationLevel;

    if state.notifications.is_empty() {
        return;
    }

    // Only render the most recent notification (as per Task 5 requirement)
    if let Some(notification) = state.notifications.back() {
        // Determine style based on notification level
        let style = match notification.level {
            NotificationLevel::Info => Style::default().fg(Color::Blue).bg(Color::Reset),
            NotificationLevel::Success => Style::default().fg(Color::Green).bg(Color::Reset),
            NotificationLevel::Warning => Style::default().fg(Color::Yellow).bg(Color::Reset),
            NotificationLevel::Error => Style::default().fg(Color::Red).bg(Color::Reset),
        };

        // Add icon prefix based on level
        let icon = match notification.level {
            NotificationLevel::Info => "ℹ ",
            NotificationLevel::Success => "✓ ",
            NotificationLevel::Warning => "⚠ ",
            NotificationLevel::Error => "✖ ",
        };

        // Create the notification text with padding
        let text = format!("  {}{}  ", icon, notification.message);
        let paragraph = Paragraph::new(text).style(style);

        // Render at the bottom of the content area, above status bar
        // Use a fixed height of 1 line for the toast
        let toast_area = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(2),
            area.width,
            1,
        );

        // Only render if there's enough space
        if toast_area.height > 0 && toast_area.width > 4 {
            frame.render_widget(paragraph, toast_area);
        }
    }
}
