//! Render implementation for TreePanel
//!
//! Contains rendering logic for the tree panel.

use crate::tui::state::{NodeType, TreeNodeId, TreeState};
use ratatui::{
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Pre-computed indentation strings for tree levels
const INDENTS: [&str; 10] = [
    "",
    "  ",
    "    ",
    "      ",
    "        ",
    "          ",
    "            ",
    "              ",
    "                ",
    "                  ",
];

/// Render context for tree panel
pub struct RenderContext {
    pub focused: bool,
}

/// Render to frame (preferred method)
pub fn render_frame(frame: &mut Frame, area: Rect, state: &TreeState, ctx: &RenderContext) {
    render_frame_with_context(frame, area, state, ctx, false)
}

/// Render to frame with filter context for better empty state messages
pub fn render_frame_with_context(
    frame: &mut Frame,
    area: Rect,
    state: &TreeState,
    ctx: &RenderContext,
    has_active_filters: bool,
) {
    if area.height < 3 {
        return;
    }

    let border_style = if ctx.focused {
        Style::default().fg(Color::Rgb(100, 200, 255)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(70, 70, 90))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" [1] Groups ");

    let inner_area = block.inner(area);
    block.render(area, frame.buffer_mut());

    render_nodes(frame, inner_area, state, ctx, has_active_filters);
}

/// Render the list of visible nodes
fn render_nodes(
    frame: &mut Frame,
    area: Rect,
    state: &TreeState,
    ctx: &RenderContext,
    has_active_filters: bool,
) {
    if state.visible_nodes.is_empty() {
        let lines = if has_active_filters {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No matching entries",
                    Style::default().fg(Color::Rgb(100, 200, 255)),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Try adjusting your filters",
                    Style::default().fg(Color::Rgb(100, 100, 120)),
                )),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No passwords yet",
                    Style::default().fg(Color::Rgb(100, 100, 120)),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press [n] to create one",
                    Style::default().fg(Color::Rgb(100, 200, 255)),
                )),
            ]
        };
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
        return;
    }

    let max_rows = area.height as usize;
    let start_row = calculate_scroll_offset(state, max_rows);

    for (i, node) in state.visible_nodes.iter().skip(start_row).enumerate() {
        if i >= max_rows {
            break;
        }

        let y = area.y + i as u16;
        let row_area = Rect::new(area.x, y, area.width, 1);

        let is_highlighted = (start_row + i) == state.highlighted_index && ctx.focused;
        let is_expanded = if let TreeNodeId::Group(id) = node.id {
            state.is_expanded(&id)
        } else {
            false
        };

        let line = format_node_line(node, is_highlighted, is_expanded);
        let paragraph = Paragraph::new(line);
        paragraph.render(row_area, frame.buffer_mut());
    }
}

/// Calculate scroll offset to keep highlighted item visible
fn calculate_scroll_offset(state: &TreeState, max_rows: usize) -> usize {
    if state.highlighted_index < max_rows.saturating_sub(1) {
        return 0;
    }
    state
        .highlighted_index
        .saturating_sub(max_rows.saturating_sub(2))
}

/// Format a single node line for display
fn format_node_line(
    node: &crate::tui::state::VisibleNode,
    is_highlighted: bool,
    is_expanded: bool,
) -> Line<'static> {
    let indent = INDENTS.get(node.level as usize).unwrap_or(&"");

    let icon = match node.node_type {
        NodeType::Folder => {
            if is_expanded {
                "[-]"
            } else {
                "[+]"
            }
        }
        NodeType::Password => " • ",
    };

    let style = if is_highlighted {
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(40, 60, 100))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(200, 200, 215))
    };

    let mut spans = Vec::with_capacity(6);
    spans.push(Span::styled(*indent, style));
    spans.push(Span::styled(icon, style));
    spans.push(Span::styled(" ", style));
    spans.push(Span::styled(node.label.clone(), style));

    // Show favorite icon for password nodes
    if node.is_favorite && matches!(node.node_type, NodeType::Password) {
        spans.push(Span::styled(" ★", Style::default().fg(Color::Rgb(220, 180, 50))));
    }

    if node.child_count > 0 && matches!(node.node_type, NodeType::Folder) {
        spans.push(Span::styled(format!(" ({})", node.child_count), style));
    }

    Line::from(spans)
}
