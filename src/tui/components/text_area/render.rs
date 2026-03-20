//! Render implementation for TextArea
//!
//! Contains the rendering logic for the text area component.

use super::TextArea;
use crate::tui::traits::Render;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::cmp;

impl Render for TextArea {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        // Create border
        let block_style = if self.focused {
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Black)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_style);

        let block = if self.title.is_empty() {
            block
        } else {
            block.title(self.title.as_str())
        };

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Prepare text to display
        let display_lines =
            if self.lines.is_empty() || (self.lines.len() == 1 && self.lines[0].is_empty()) {
                if self.placeholder.is_empty() {
                    vec![String::new()]
                } else {
                    vec![self.placeholder.clone()]
                }
            } else {
                self.lines.clone()
            };

        // Slice visible portion
        let start_idx = self.scroll_offset;
        let end_idx = cmp::min(start_idx + inner_area.height as usize, display_lines.len());
        let visible_lines: Vec<&str> = display_lines[start_idx..end_idx]
            .iter()
            .map(|s| s.as_str())
            .collect();

        // Create text with cursor marker
        let spans: Vec<Line> = visible_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let line_idx = start_idx + idx;
                if self.focused && line_idx == self.cursor_row {
                    // Current line, need to mark cursor position
                    let mut spans = Vec::new();

                    // Text before cursor
                    if self.cursor_col > 0 {
                        let before_cursor = &line[..self.cursor_col.min(line.len())];
                        spans.push(Span::raw(before_cursor));
                    }

                    // Cursor character or next character
                    if self.cursor_col < line.len() {
                        let cursor_char = &line[self.cursor_col
                            ..self.cursor_col
                                + line[self.cursor_col..]
                                    .chars()
                                    .next()
                                    .map(|c| c.len_utf8())
                                    .unwrap_or(1)];
                        spans.push(Span::styled(
                            cursor_char,
                            Style::default().add_modifier(Modifier::REVERSED),
                        ));

                        // Text after cursor
                        if self.cursor_col + cursor_char.len() < line.len() {
                            spans.push(Span::raw(&line[self.cursor_col + cursor_char.len()..]));
                        }
                    } else {
                        // Insert cursor at end of line
                        spans.push(Span::styled(
                            " ",
                            Style::default().add_modifier(Modifier::REVERSED),
                        ));
                    }

                    Line::from(spans)
                } else {
                    // Non-current line, display directly
                    Line::from(*line)
                }
            })
            .collect();

        let text_widget = Paragraph::new(spans)
            .wrap(Wrap { trim: false })
            .scroll((0, 0)); // Using custom scroll logic

        text_widget.render(inner_area, buf);

        // Render validation error
        if let Some(ref result) = self.validation_result {
            if result.has_errors() {
                let error_line = Line::from(result.first_error().unwrap_or("Validation error"));
                let error_area = Rect {
                    x: area.x,
                    y: area.y + area.height,
                    width: area.width,
                    height: 1,
                };

                let error_paragraph =
                    Paragraph::new(error_line).style(Style::default().fg(Color::Red));
                error_paragraph.render(error_area, buf);
            }
        }
    }
}
