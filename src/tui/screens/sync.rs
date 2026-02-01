//! Sync Screen
//!
//! TUI screen for displaying sync status and triggering manual sync.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

/// Sync status
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Success { uploaded: usize, downloaded: usize },
    Error { message: String },
    ConflictsDetected { count: usize },
}

/// Sync screen
#[derive(Debug, Clone)]
pub struct SyncScreen {
    /// Current sync status
    status: SyncStatus,
    /// Progress (0.0 to 1.0)
    progress: f32,
    /// Status message
    message: String,
}

impl SyncScreen {
    /// Create a new sync screen
    pub fn new() -> Self {
        Self {
            status: SyncStatus::Idle,
            progress: 0.0,
            message: "Ready to sync".to_string(),
        }
    }

    /// Get current sync status
    pub fn get_status(&self) -> &SyncStatus {
        &self.status
    }

    /// Set sync status
    pub fn set_status(&mut self, status: SyncStatus) {
        self.status = status;
        self.update_message();
    }

    /// Set progress (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Update message based on status
    fn update_message(&mut self) {
        self.message = match &self.status {
            SyncStatus::Idle => "Ready to sync. Press F5 to start.".to_string(),
            SyncStatus::Syncing => format!("Syncing... {:.0}%", self.progress * 100.0),
            SyncStatus::Success {
                uploaded,
                downloaded,
            } => {
                format!("✓ Sync complete (↑{} ↓{})", uploaded, downloaded)
            }
            SyncStatus::Error { message } => format!("✗ Sync failed: {}", message),
            SyncStatus::ConflictsDetected { count } => {
                format!("⚠ {} conflicts detected. Press Enter to resolve.", count)
            }
        };
    }

    /// Render the sync screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new(Text::from(vec![Line::from(Span::styled(
            "Sync / 同步",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(title, chunks[0]);

        // Content
        let mut content_lines = vec![];

        content_lines.push(Line::from(""));
        content_lines.push(Line::from(self.message.clone()));
        content_lines.push(Line::from(""));

        // Show progress bar if syncing
        if matches!(self.status, SyncStatus::Syncing) {
            content_lines.push(Line::from(""));
            content_lines.push(Line::from("Progress:"));
        }

        let content = Paragraph::new(Text::from(content_lines))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Status"));

        frame.render_widget(content, chunks[1]);

        // Progress bar
        if matches!(self.status, SyncStatus::Syncing) {
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent((self.progress * 100.0) as u16);

            let progress_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(1)].as_ref())
                .split(chunks[1]);

            frame.render_widget(gauge, progress_area[1]);
        }

        // Footer
        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::from("F5: Sync  "),
            Span::from("Esc: Back"),
        ])]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[2]);
    }
}

impl Default for SyncScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_screen_new() {
        let screen = SyncScreen::new();
        assert_eq!(screen.get_status(), &SyncStatus::Idle);
        assert_eq!(screen.progress, 0.0);
    }

    #[test]
    fn test_sync_screen_message_updates() {
        let mut screen = SyncScreen::new();

        screen.set_status(SyncStatus::Success {
            uploaded: 5,
            downloaded: 3,
        });

        assert!(screen.message.contains("5"));
        assert!(screen.message.contains("3"));
    }

    #[test]
    fn test_sync_screen_progress_clamping() {
        let mut screen = SyncScreen::new();

        screen.set_progress(1.5);
        assert_eq!(screen.progress, 1.0);

        screen.set_progress(-0.5);
        assert_eq!(screen.progress, 0.0);
    }
}
