//! Render implementation for EditPasswordScreen
//!
//! Contains rendering logic for the edit password form.

use super::{EditFormField, EditPasswordScreen};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget},
};

impl Render for EditPasswordScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("  Edit Password  ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        let inner = block.inner(area);
        block.render(area, buf);

        // Display name (read-only) at the top
        let name_y = inner.y + 1;
        let name_style = Style::default().fg(Color::DarkGray);
        buf.set_string(
            inner.x + 2,
            name_y,
            &format!("Name: {} (read-only)", self.password_name),
            name_style,
        );

        let start_y = inner.y + 3;
        let row_height = 3;

        // Render each editable field
        for i in 0..8 {
            let y = start_y + (i as u16) * row_height;
            if y >= inner.y + inner.height {
                break;
            }

            let field = match EditFormField::from_index(i) {
                Some(f) => f,
                None => continue,
            };

            let is_focused = i == self.focused_field;
            let label_style = if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Field label
            let label = format!("{}:", field.label());
            buf.set_string(inner.x + 2, y, &label, label_style);

            // Render field content
            match field {
                EditFormField::Username => {
                    self.render_text_field(buf, inner, y, &self.username, label_style);
                }
                EditFormField::PasswordType => {
                    let type_label = self.password_type.label();
                    let display = format!("[{}]  ", type_label);
                    buf.set_string(
                        inner.x + 2,
                        y + 1,
                        &display,
                        Style::default().fg(if is_focused {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                    );
                }
                EditFormField::PasswordLength => {
                    let display = format!("[{}]  ", self.password_length);
                    buf.set_string(
                        inner.x + 2,
                        y + 1,
                        &display,
                        Style::default().fg(if is_focused {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                    );
                }
                EditFormField::Password => {
                    let display = if self.password_visible {
                        Span::raw(self.get_current_password())
                    } else {
                        Span::raw("•".repeat(self.get_current_password().len().max(16)))
                    };
                    let input = Paragraph::new(display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));

                    input.render(
                        Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2),
                        buf,
                    );

                    // Show hints
                    let hint = if self.new_password.is_some() {
                        "[r] Regenerate  [Space] Show/Hide  (modified)"
                    } else {
                        "[r] Regenerate  [Space] Show/Hide  (original)"
                    };
                    buf.set_string(
                        inner.x + 20,
                        y + 1,
                        hint,
                        Style::default().fg(if self.new_password.is_some() {
                            Color::Yellow
                        } else {
                            Color::DarkGray
                        }),
                    );
                }
                EditFormField::Url => {
                    self.render_text_field(buf, inner, y, &self.url, label_style);
                }
                EditFormField::Notes => {
                    let notes_display = if self.notes.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.notes)
                    };
                    let input = Paragraph::new(notes_display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));
                    input.render(
                        Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 3),
                        buf,
                    );
                }
                EditFormField::Tags => {
                    let content = if self.tags.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.tags)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));
                    input.render(
                        Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2),
                        buf,
                    );

                    let hint = "(comma separated)";
                    buf.set_string(
                        inner.x + 20,
                        y + 1,
                        hint,
                        Style::default().fg(Color::DarkGray),
                    );
                }
                EditFormField::Group => {
                    let display = format!("[{}]", self.group);
                    buf.set_string(
                        inner.x + 2,
                        y + 1,
                        &display,
                        Style::default().fg(if is_focused {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                    );
                }
            }
        }

        // Help text at bottom
        let help_y = inner.y + inner.height - 2;
        let help = "[Tab] Next  [Esc] Cancel  [Enter] Save";
        buf.set_string(
            inner.x + 2,
            help_y,
            help,
            Style::default().fg(Color::DarkGray),
        );
    }
}

impl EditPasswordScreen {
    /// Render a text field
    fn render_text_field(&self, buf: &mut Buffer, inner: Rect, y: u16, value: &str, style: Style) {
        let content = if value.is_empty() {
            Span::raw("")
        } else {
            Span::raw(value)
        };
        let input = Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::NONE));
        input.render(
            Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2),
            buf,
        );
    }
}

use crate::tui::traits::Render;
