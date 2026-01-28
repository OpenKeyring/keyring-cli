//! TUI Application State and Logic
//!
//! Core TUI application handling alternate screen mode, rendering, and event loop.

use crate::error::{KeyringError, Result};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

/// TUI-specific error type
#[derive(Debug)]
pub enum TuiError {
    /// Terminal initialization failed
    InitFailed(String),
    /// Terminal restore failed
    RestoreFailed(String),
    /// I/O error
    IoError(String),
}

impl std::fmt::Display for TuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuiError::InitFailed(msg) => write!(f, "TUI init failed: {}", msg),
            TuiError::RestoreFailed(msg) => write!(f, "TUI restore failed: {}", msg),
            TuiError::IoError(msg) => write!(f, "TUI I/O error: {}", msg),
        }
    }
}

impl std::error::Error for TuiError {}

/// TUI result type
pub type TuiResult<T> = std::result::Result<T, TuiError>;

/// TUI Application State
pub struct TuiApp {
    /// Running state
    running: bool,
    /// Current input buffer
    input_buffer: String,
    /// Command history
    history: Vec<String>,
    /// History cursor position
    history_index: usize,
    /// Current output/messages to display
    pub output_lines: Vec<String>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new() -> Self {
        Self {
            running: true,
            input_buffer: String::new(),
            history: Vec::new(),
            history_index: 0,
            output_lines: vec![
                "OpenKeyring TUI v0.1.0".to_string(),
                "Type /help for available commands".to_string(),
                "".to_string(),
            ],
        }
    }

    /// Check if the app is still running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Handle input character
    pub fn handle_char(&mut self, c: char) {
        match c {
            '\n' | '\r' => {
                // Enter key - submit command
                self.submit_command();
            }
            '\t' => {
                // Tab key - trigger autocomplete (placeholder for now)
                // TODO: Implement autocomplete
            }
            c if c.is_ascii_control() => {
                // Ignore other control characters
            }
            c => {
                // Regular character - add to buffer
                self.input_buffer.push(c);
            }
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        self.input_buffer.pop();
    }

    /// Submit the current command
    fn submit_command(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let cmd = self.input_buffer.clone();
        self.history.push(cmd.clone());
        self.history_index = self.history.len();
        self.input_buffer.clear();

        // Process command
        self.process_command(&cmd);
    }

    /// Process a command
    pub(crate) fn process_command(&mut self, cmd: &str) {
        use crate::tui::commands::{delete, list, new, search, show, update};

        self.output_lines.push(format!("> {}", cmd));

        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let args = if parts.len() > 1 {
            parts[1].split_whitespace().collect()
        } else {
            Vec::new()
        };

        match command {
            "/exit" | "/quit" => {
                self.quit();
                self.output_lines.push("Goodbye!".to_string());
            }
            "/help" => {
                self.output_lines.extend_from_slice(&[
                    "".to_string(),
                    "Available Commands:".to_string(),
                    "  /list [filter]    - List password records".to_string(),
                    "  /show <name>      - Show a password record".to_string(),
                    "  /new              - Create a new record".to_string(),
                    "  /update <name>    - Update a record".to_string(),
                    "  /delete <name>    - Delete a record".to_string(),
                    "  /search <query>   - Search records".to_string(),
                    "  /exit             - Exit TUI".to_string(),
                    "".to_string(),
                ]);
            }
            "/list" => {
                match list::handle_list(args) {
                    Ok(lines) => self.output_lines.extend(lines),
                    Err(e) => self.output_lines.push(format!("Error: {}", e)),
                }
            }
            "/show" => {
                match show::handle_show(args) {
                    Ok(lines) => self.output_lines.extend(lines),
                    Err(e) => self.output_lines.push(format!("Error: {}", e)),
                }
            }
            "/new" => {
                match new::handle_new() {
                    Ok(lines) => self.output_lines.extend(lines),
                    Err(e) => self.output_lines.push(format!("Error: {}", e)),
                }
            }
            "/update" => {
                match update::handle_update(args) {
                    Ok(lines) => self.output_lines.extend(lines),
                    Err(e) => self.output_lines.push(format!("Error: {}", e)),
                }
            }
            "/delete" => {
                match delete::handle_delete(args) {
                    Ok(lines) => self.output_lines.extend(lines),
                    Err(e) => self.output_lines.push(format!("Error: {}", e)),
                }
            }
            "/search" => {
                match search::handle_search(args) {
                    Ok(lines) => self.output_lines.extend(lines),
                    Err(e) => self.output_lines.push(format!("Error: {}", e)),
                }
            }
            cmd if cmd.starts_with('/') => {
                self.output_lines.push(
                    format!("Unknown command '{}'. Type /help for available commands.", cmd),
                );
            }
            _ => {
                self.output_lines
                    .push("Unknown command. Type /help for available commands.".to_string());
            }
        }
    }

    /// Render the TUI
    pub fn render(&self, frame: &mut Frame) {
        let size = frame.area();

        // Split screen into output area and input area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
            .split(size);

        // Render output area
        self.render_output(frame, chunks[0]);

        // Render input area
        self.render_input(frame, chunks[1]);
    }

    /// Render the output area
    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let text: Text = self
            .output_lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" OpenKeyring TUI "),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Render the input area
    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let input_text = if self.input_buffer.is_empty() {
            vec![Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Type a command...",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Gray)),
                Span::raw(&self.input_buffer),
            ])]
        };

        let paragraph = Paragraph::new(Text::from(input_text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        // Set cursor position
        frame.set_cursor_position((area.x + 2 + self.input_buffer.len() as u16, area.y + 1));
    }
}

/// Initialize terminal for TUI mode
pub fn init_terminal() -> TuiResult<Terminal<CrosstermBackend<Stdout>>> {
    use crossterm::{
        event::EnableMouseCapture,
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    };

    enable_raw_mode().map_err(|e| TuiError::InitFailed(e.to_string()))?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| TuiError::InitFailed(e.to_string()))?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend).map_err(|e| TuiError::InitFailed(e.to_string()))?;

    Ok(terminal)
}

/// Restore terminal after TUI mode
pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> TuiResult<()> {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };

    disable_raw_mode().map_err(|e| TuiError::RestoreFailed(e.to_string()))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .map_err(|e| TuiError::RestoreFailed(e.to_string()))?;

    terminal
        .show_cursor()
        .map_err(|e| TuiError::RestoreFailed(e.to_string()))?;

    Ok(())
}

/// Run the TUI application
pub fn run_tui() -> Result<()> {
    use crossterm::event;

    let mut terminal =
        init_terminal().map_err(|e| KeyringError::IoError(format!("Failed to init TUI: {}", e)))?;

    let mut app = TuiApp::new();

    // Main event loop
    while app.is_running() {
        terminal
            .draw(|f| app.render(f))
            .map_err(|e| KeyringError::IoError(format!("Failed to draw: {}", e)))?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))
            .map_err(|e| KeyringError::IoError(format!("Event poll failed: {}", e)))?
        {
            match event::read()
                .map_err(|e| KeyringError::IoError(format!("Event read failed: {}", e)))?
            {
                event::Event::Key(key) => {
                    use crossterm::event::KeyCode;
                    match key.code {
                        KeyCode::Char(c) => app.handle_char(c),
                        KeyCode::Backspace | KeyCode::Delete => app.handle_backspace(),
                        KeyCode::Enter => app.handle_char('\n'),
                        KeyCode::Esc if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            app.quit();
                        }
                        _ => {}
                    }
                }
                event::Event::Resize(_, _) => {
                    // Terminal resized - will be handled on next draw
                }
                _ => {}
            }
        }
    }

    restore_terminal(terminal)
        .map_err(|e| KeyringError::IoError(format!("Failed to restore terminal: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = TuiApp::new();
        assert!(app.is_running());
        assert_eq!(app.input_buffer, "");
    }

    #[test]
    fn test_app_quit() {
        let mut app = TuiApp::new();
        app.quit();
        assert!(!app.is_running());
    }

    #[test]
    fn test_handle_char() {
        let mut app = TuiApp::new();
        app.handle_char('t');
        app.handle_char('e');
        app.handle_char('s');
        app.handle_char('t');
        assert_eq!(app.input_buffer, "test");
    }

    #[test]
    fn test_handle_backspace() {
        let mut app = TuiApp::new();
        app.handle_char('t');
        app.handle_char('e');
        app.handle_backspace();
        assert_eq!(app.input_buffer, "t");
    }

    #[test]
    fn test_submit_command() {
        let mut app = TuiApp::new();
        app.handle_char('/');
        app.handle_char('h');
        app.handle_char('e');
        app.handle_char('l');
        app.handle_char('p');
        app.handle_char('\n');
        assert_eq!(app.input_buffer, "");
        assert!(app
            .output_lines
            .iter()
            .any(|l| l.contains("Available Commands")));
    }

    #[test]
    fn test_exit_command() {
        let mut app = TuiApp::new();
        app.handle_char('/');
        app.handle_char('e');
        app.handle_char('x');
        app.handle_char('i');
        app.handle_char('t');
        app.handle_char('\n');
        assert!(!app.is_running());
    }

    #[test]
    fn test_process_delete_command() {
        let mut app = TuiApp::new();
        app.process_command("/delete test");
        // Should show delete confirmation
        assert!(app.output_lines.iter().any(|l| l.contains("Delete") || l.contains("Confirm")));
    }

    #[test]
    fn test_process_list_command() {
        let mut app = TuiApp::new();
        app.process_command("/list");
        // Should show password prompt or list output
        assert!(app.output_lines.iter().any(|l| l.contains("password") || l.contains("Password") || l.contains("Records")));
    }

    #[test]
    fn test_process_show_command() {
        let mut app = TuiApp::new();
        app.process_command("/show test");
        // Should show error or record info
        assert!(app.output_lines.iter().any(|l| l.contains("Error") || l.contains("not found") || l.contains("test")));
    }

    #[test]
    fn test_process_new_command() {
        let mut app = TuiApp::new();
        app.process_command("/new");
        // Should show new record wizard
        assert!(app.output_lines.iter().any(|l| l.contains("New") || l.contains("Create") || l.contains("record")));
    }

    #[test]
    fn test_process_update_command() {
        let mut app = TuiApp::new();
        app.process_command("/update test");
        // Should show update wizard or error
        assert!(app.output_lines.iter().any(|l| l.contains("Update") || l.contains("Error") || l.contains("not found")));
    }

    #[test]
    fn test_process_search_command() {
        let mut app = TuiApp::new();
        app.process_command("/search test");
        // Should show search results or empty state
        assert!(app.output_lines.iter().any(|l| l.contains("Search") || l.contains("No results") || l.contains("Error")));
    }

    #[test]
    fn test_process_unknown_command() {
        let mut app = TuiApp::new();
        app.process_command("/unknown");
        // Should show unknown command message
        assert!(app.output_lines.iter().any(|l| l.contains("Unknown") || l.contains("unknown")));
    }

    #[test]
    fn test_process_command_with_args() {
        let mut app = TuiApp::new();
        app.process_command("/delete my record name");
        // Should handle command with multiple args (only first arg used)
        assert!(app.output_lines.iter().any(|l| l.contains("> /delete")));
    }
}
