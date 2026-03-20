//! Render implementation for NewPasswordScreen
//!
//! Contains rendering logic for the new password form.

use super::{FormField, NewPasswordScreen};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget},
};

impl Render for NewPasswordScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("  New Password  ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        let inner = block.inner(area);
        block.render(area, buf);

        let field_count = 9;
        let row_height = 3;
        let start_y = inner.y + 1;

        // Render each field
        for i in 0..field_count {
            let y = start_y + (i as u16) * row_height;
            if y >= inner.y + inner.height {
                break;
            }

            let field = match FormField::from_index(i) {
                Some(f) => f,
                None => continue,
            };

            let is_focused = i == self.focused_field;
            let error = self.errors.get(&field);

            // Field label
            let label = if field.is_required() {
                format!("{}*:", field.label())
            } else {
                format!("{}:", field.label())
            };

            let label_style = if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Render based on field type
            match field {
                FormField::Name => {
                    self.render_text_field(buf, inner, y, &label, &self.name, label_style);
                }
                FormField::Username => {
                    self.render_text_field(buf, inner, y, &label, &self.username, label_style);
                }
                FormField::PasswordType => {
                    let type_label = self.password_type.label();
                    let display = format!("[{}]  ", type_label);
                    buf.set_string(inner.x + 2, y, &label, label_style);
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
                FormField::PasswordLength => {
                    let display = format!("[{}]  ", self.password_length);
                    buf.set_string(inner.x + 2, y, &label, label_style);
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
                FormField::Password => {
                    let display = if self.password_visible {
                        Span::raw(&self.password)
                    } else {
                        Span::raw("•".repeat(self.password.len().max(16)))
                    };
                    let input = Paragraph::new(display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE))
                        .wrap(ratatui::widgets::Wrap { trim: false });

                    buf.set_string(inner.x + 2, y, &label, label_style);
                    input.render(
                        Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2),
                        buf,
                    );

                    // Show regenerate hint
                    let hint = "[r] Regenerate  [Space] Show/Hide";
                    buf.set_string(
                        inner.x + 20,
                        y + 1,
                        hint,
                        Style::default().fg(Color::DarkGray),
                    );
                }
                FormField::Url => {
                    self.render_text_field(buf, inner, y, &label, &self.url, label_style);
                }
                FormField::Notes => {
                    buf.set_string(inner.x + 2, y, &label, label_style);
                    let notes_display = if self.notes.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.notes)
                    };
                    let input = Paragraph::new(notes_display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE))
                        .wrap(ratatui::widgets::Wrap { trim: false });
                    input.render(
                        Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 3),
                        buf,
                    );
                }
                FormField::Tags => {
                    self.render_text_field(buf, inner, y, &label, &self.tags, label_style);
                    let hint = "(comma separated)";
                    buf.set_string(
                        inner.x + 20,
                        y + 1,
                        hint,
                        Style::default().fg(Color::DarkGray),
                    );
                }
                FormField::Group => {
                    let display = format!("[{}]", self.group);
                    buf.set_string(inner.x + 2, y, &label, label_style);
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

            // Show error if any
            if let Some(err) = error {
                buf.set_string(
                    inner.x + 2,
                    y + row_height - 1,
                    err,
                    Style::default().fg(Color::Red),
                );
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

        // Show validation errors at bottom
        if !self.errors.is_empty() {
            let error_y = inner.y + inner.height - 4;
            for (i, (field, err)) in self.errors.iter().enumerate() {
                let msg = format!("{}: {}", field.label(), err);
                buf.set_string(
                    inner.x + 2,
                    error_y + i as u16,
                    msg,
                    Style::default().fg(Color::Red),
                );
            }
        }
    }
}

impl NewPasswordScreen {
    /// Render a text field with label
    fn render_text_field(
        &self,
        buf: &mut Buffer,
        inner: Rect,
        y: u16,
        label: &str,
        value: &str,
        style: Style,
    ) {
        let content = if value.is_empty() {
            Span::raw("")
        } else {
            Span::raw(value)
        };
        let input = Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::NONE))
            .wrap(ratatui::widgets::Wrap { trim: false });

        buf.set_string(inner.x + 2, y, label, style);
        input.render(
            Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2),
            buf,
        );
    }
}

use crate::tui::traits::Render;
