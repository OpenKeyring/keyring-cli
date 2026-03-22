//! Render implementation for DetailPanel
//!
//! Contains rendering logic for the detail panel.

use crate::tui::models::password::PasswordRecord;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// Render project information when no password is selected
pub fn render_project_info(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "OpenKeyring",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Privacy-first Password Manager",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Version: v0.1.0",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "License: MIT License",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Website: github.com/open-keyring",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [n] to create your first password",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

/// Render password details
pub fn render_password(
    frame: &mut Frame,
    area: Rect,
    password: &PasswordRecord,
    password_visible: bool,
) {
    let mut lines: Vec<Line<'_>> = Vec::new();

    // Title (name)
    lines.push(Line::from(Span::styled(
        password.name.clone(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Username
    if let Some(ref username) = password.username {
        lines.push(create_field_line("Username:", username, "[c] copy"));
    }

    // Password
    let password_display = if password_visible {
        password.password.clone()
    } else {
        "*".repeat(password.password.len().min(20))
    };
    lines.push(create_field_line(
        "Password:",
        &password_display,
        "[C] copy",
    ));

    // URL
    if let Some(ref url) = password.url {
        lines.push(create_field_line("URL:", url, "[o] open"));
    }

    // Tags
    if !password.tags.is_empty() {
        let tags_str = password.tags.join(", ");
        lines.push(create_field_line("Tags:", &tags_str, ""));
    }

    // Notes
    if let Some(ref notes) = password.notes {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Notes:",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(Span::raw(notes.clone())));
    }

    // Timestamps
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "Created: {}  |  Modified: {}",
            password.created_at.format("%Y-%m-%d %H:%M"),
            password.modified_at.format("%Y-%m-%d %H:%M")
        ),
        Style::default().fg(Color::DarkGray),
    )));

    // Status indicators
    if password.is_favorite {
        lines.push(Line::from(Span::styled(
            "⭐ Favorite",
            Style::default().fg(Color::Yellow),
        )));
    }
    if password.is_deleted {
        lines.push(Line::from(Span::styled(
            "🗑 In Trash",
            Style::default().fg(Color::Red),
        )));
    }

    // Action hints at bottom
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[e]", Style::default().fg(Color::Rgb(100, 200, 255))),
        Span::styled(" Edit  ", Style::default().fg(Color::Rgb(100, 100, 120))),
        Span::styled("[d]", Style::default().fg(Color::Rgb(100, 200, 255))),
        Span::styled(" Delete  ", Style::default().fg(Color::Rgb(100, 100, 120))),
        Span::styled("[Space]", Style::default().fg(Color::Rgb(100, 200, 255))),
        Span::styled(" Toggle  ", Style::default().fg(Color::Rgb(100, 100, 120))),
        Span::styled("[c]", Style::default().fg(Color::Rgb(100, 200, 255))),
        Span::styled(" Copy User  ", Style::default().fg(Color::Rgb(100, 100, 120))),
        Span::styled("[C]", Style::default().fg(Color::Rgb(100, 200, 255))),
        Span::styled(" Copy Pwd", Style::default().fg(Color::Rgb(100, 100, 120))),
    ]));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

/// Create a field line with label, value, and hint
fn create_field_line(label: &str, value: &str, hint: &str) -> Line<'static> {
    let mut spans = vec![
        Span::styled(label.to_string(), Style::default().fg(Color::Rgb(120, 140, 170))),
        Span::raw(" "),
        Span::styled(value.to_string(), Style::default().fg(Color::Rgb(220, 220, 240))),
    ];

    if !hint.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            hint.to_string(),
            Style::default().fg(Color::Rgb(100, 100, 120)),
        ));
    }

    Line::from(spans)
}
