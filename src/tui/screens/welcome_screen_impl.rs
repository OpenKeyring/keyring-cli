//! Screen Implementations for Wizard Flow
//!
//! Individual screen implementations that integrate with the wizard flow and state management.

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::screens::wizard::{WizardState, WizardStep};
use crate::tui::screens::WelcomeScreen;
use crate::tui::traits::{Component, Render, Interactive, HandleResult, Screen, ScreenType, ComponentId, AppEvent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::buffer::Buffer;

impl Render for WelcomeScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // We need to wrap the original render method, but the original screen is implemented for Frame
        // For now, let's create a minimal render
        let block = Block::default()
            .title("OpenKeyring Setup Wizard - Welcome")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Draw the main content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Length(2), // Spacer
                    Constraint::Length(2), // Welcome message
                    Constraint::Length(2), // Spacer
                    Constraint::Length(2), // Prompt
                    Constraint::Min(0),    // Choices
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(inner_area);

        // Title
        let title = Paragraph::new(vec![Line::from(Span::styled(
            "OpenKeyring Setup Wizard",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(ratatui::layout::Alignment::Center);

        ratatui::widgets::Widget::render(title, chunks[0], buf);

        // Welcome message
        let welcome = Paragraph::new(vec![Line::from(Span::styled(
            "Welcome to OpenKeyring!",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(ratatui::layout::Alignment::Center);

        ratatui::widgets::Widget::render(welcome, chunks[2], buf);

        // Prompt
        let prompt = Paragraph::new(vec![Line::from(Span::styled(
            "Choose setup method:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(ratatui::layout::Alignment::Left);

        ratatui::widgets::Widget::render(prompt, chunks[4], buf);

        // Choices
        let choices_content = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    if self.selected == crate::tui::screens::WelcomeChoice::GenerateNew {
                        "●"
                    } else {
                        "○"
                    },
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    crate::tui::screens::WelcomeChoice::GenerateNew.display_text(),
                    Style::default()
                        .fg(if self.selected == crate::tui::screens::WelcomeChoice::GenerateNew {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    crate::tui::screens::WelcomeChoice::GenerateNew.description(),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    if self.selected == crate::tui::screens::WelcomeChoice::ImportExisting {
                        "●"
                    } else {
                        "○"
                    },
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    crate::tui::screens::WelcomeChoice::ImportExisting.display_text(),
                    Style::default()
                        .fg(if self.selected == crate::tui::screens::WelcomeChoice::ImportExisting {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    crate::tui::screens::WelcomeChoice::ImportExisting.description(),
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ];

        let choices = Paragraph::new(choices_content)
            .wrap(ratatui::widgets::Wrap { trim: false });

        ratatui::widgets::Widget::render(choices, chunks[5], buf);

        // Footer with keyboard hints
        let footer = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Next    "),
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Choose    "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Exit"),
        ])])
        .alignment(ratatui::layout::Alignment::Center);

        ratatui::widgets::Widget::render(footer, chunks[6], buf);
    }
}

impl Interactive for WelcomeScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Enter => HandleResult::Action(crate::tui::traits::Action::None),
            KeyCode::Down | KeyCode::Char('j') => {
                self.toggle();
                HandleResult::NeedsRender
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.toggle();
                HandleResult::NeedsRender
            }
            KeyCode::Tab => {
                self.toggle();
                HandleResult::NeedsRender
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for WelcomeScreen {
    fn id(&self) -> ComponentId {
        ComponentId::new(1001) // Unique ID for WelcomeScreen
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_event(&mut self, _event: &AppEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

impl Screen for WelcomeScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::Wizard
    }

    fn close(&mut self) -> TuiResult<()> {
        Ok(())
    }

    fn is_modal(&self) -> bool {
        true
    }

    fn show_overlay(&self) -> bool {
        true
    }

    fn size(&self, terminal: Rect) -> Rect {
        let width = (terminal.width as f32 * 0.8) as u16;
        let height = (terminal.height as f32 * 0.8) as u16;
        let x = (terminal.width.saturating_sub(width)) / 2;
        let y = (terminal.height.saturating_sub(height)) / 2;
        Rect::new(x, y, width, height)
    }
}