//! Render implementation for MasterPasswordScreen
//!
//! Contains rendering logic for the master password setup screen.

use super::MasterPasswordScreen;
use crate::health::strength::calculate_strength;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the master password screen
pub fn render(screen: &MasterPasswordScreen, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(2), // Spacer
                Constraint::Length(5), // Password input
                Constraint::Length(5), // Confirm input
                Constraint::Length(2), // Status/Error
                Constraint::Min(0),    // Spacer
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(area);

    // Title
    let title = Paragraph::new(vec![Line::from(Span::styled(
        "Set Master Password for This Device",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, chunks[0]);

    render_password_field(screen, frame, chunks[2]);
    render_confirm_field(screen, frame, chunks[3]);
    render_status(screen, frame, chunks[4]);
    render_hint(frame, chunks[5]);
    render_footer(screen, frame, chunks[6]);
}

/// Render the password input field
fn render_password_field(screen: &MasterPasswordScreen, frame: &mut Frame, area: Rect) {
    let password_display = "•".repeat(screen.password_input().len());
    let show_first = screen.is_showing_first();

    let password_field = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Master Password: ",
                Style::default().fg(if show_first { Color::Cyan } else { Color::Gray }),
            ),
            Span::styled(
                if password_display.is_empty() {
                    if show_first { "Type here..." } else { "" }
                } else {
                    password_display.as_str()
                },
                Style::default().fg(if show_first { Color::White } else { Color::Gray }),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} Strength: {}", screen.strength().icon(), screen.strength().display()),
                Style::default()
                    .fg(screen.strength().color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ("),
            Span::styled(
                format!("{}", calculate_strength(screen.password_input())),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("/100)"),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if show_first {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .title(" Master Password "),
    )
    .wrap(Wrap { trim: false });

    frame.render_widget(password_field, area);
}

/// Render the confirmation input field
fn render_confirm_field(screen: &MasterPasswordScreen, frame: &mut Frame, area: Rect) {
    let confirm_display = "•".repeat(screen.confirm_input().len());
    let show_first = screen.is_showing_first();
    let confirm_input = screen.confirm_input();

    let confirm_field = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "Confirm Password: ",
            Style::default().fg(if !show_first { Color::Cyan } else { Color::Gray }),
        ),
        Span::styled(
            if confirm_display.is_empty() {
                if !show_first { "Type here..." } else { "" }
            } else {
                confirm_display.as_str()
            },
            Style::default().fg(if !show_first { Color::White } else { Color::Gray }),
        ),
        Span::raw(if !confirm_input.is_empty() && screen.passwords_match() {
            " ✓"
        } else if !confirm_input.is_empty() {
            " ✗"
        } else {
            ""
        }),
        Span::styled(
            if !confirm_input.is_empty() && screen.passwords_match() {
                " Match"
            } else if !confirm_input.is_empty() {
                " Mismatch"
            } else {
                ""
            },
            Style::default().fg(if screen.passwords_match() { Color::Green } else { Color::Red }),
        ),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if !show_first {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .title(" Confirm Password "),
    )
    .wrap(Wrap { trim: false });

    frame.render_widget(confirm_field, area);
}

/// Render status/error message
fn render_status(screen: &MasterPasswordScreen, frame: &mut Frame, area: Rect) {
    let status = if let Some(error) = screen.validation_error() {
        Paragraph::new(Line::from(vec![
            Span::styled("✗ ", Style::default().fg(Color::Red)),
            Span::styled(error, Style::default().fg(Color::Red)),
        ]))
    } else if screen.can_complete() {
        Paragraph::new(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::styled("Password setup complete", Style::default().fg(Color::Green)),
        ]))
    } else if screen.is_showing_first() {
        Paragraph::new(Line::from(Span::styled(
            "Hint: Password must be at least 8 characters",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )))
    } else {
        Paragraph::new(Line::from(""))
    };

    frame.render_widget(status, area);
}

/// Render hint section
fn render_hint(frame: &mut Frame, area: Rect) {
    let hint = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "This password is only used to encrypt the Passkey",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::raw("   "),
            Span::styled(
                "Can be different from other devices",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
    ])
    .wrap(Wrap { trim: true });

    frame.render_widget(hint, area);
}

/// Render footer with key bindings
fn render_footer(screen: &MasterPasswordScreen, frame: &mut Frame, area: Rect) {
    let footer_spans = vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(if screen.can_complete() {
            ": Done    "
        } else if screen.is_showing_first() && !screen.password_input().is_empty() {
            ": Continue    "
        } else {
            "         "
        }),
        Span::styled(
            "Tab",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Switch    "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Back"),
    ];

    let footer = Paragraph::new(Line::from(footer_spans))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
