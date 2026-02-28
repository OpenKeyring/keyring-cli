//! Passkey Verification Screen
//!
//! Verify user has correctly saved their passkey by asking for 3 random positions

use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Passkey verification screen
pub struct PasskeyVerifyScreen {
    /// The 24-word passkey
    passkey_words: Vec<String>,
    /// 3 random positions to verify (1-indexed)
    positions: [usize; 3],
    /// User input for each position
    inputs: [String; 3],
    /// Currently focused input (0, 1, or 2)
    focused: usize,
    /// Error message
    error: Option<String>,
    /// Component ID
    id: ComponentId,
}

impl PasskeyVerifyScreen {
    /// Create new verification screen with random positions
    pub fn new(passkey_words: Vec<String>) -> Self {
        // Generate 3 unique random positions (1-24)
        let mut rng = rand::rng();
        let mut positions = [0usize; 3];

        for i in 0..3 {
            loop {
                let pos = rng.random_range(1..=24);
                if !positions.contains(&pos) {
                    positions[i] = pos;
                    break;
                }
            }
        }

        Self {
            passkey_words,
            positions,
            inputs: [String::new(), String::new(), String::new()],
            focused: 0,
            error: None,
            id: ComponentId::new(3012),
        }
    }

    /// Create with specific positions (for testing)
    pub fn with_positions(passkey_words: Vec<String>, positions: [usize; 3]) -> Self {
        Self {
            passkey_words,
            positions,
            inputs: [String::new(), String::new(), String::new()],
            focused: 0,
            error: None,
            id: ComponentId::new(3012),
        }
    }

    /// Get the random positions being verified
    pub fn positions(&self) -> [usize; 3] {
        self.positions
    }

    /// Verify if all inputs match
    pub fn verify(&self) -> bool {
        for (i, &pos) in self.positions.iter().enumerate() {
            if self.inputs[i].to_lowercase().trim()
                != self.passkey_words[pos - 1].to_lowercase().trim()
            {
                return false;
            }
        }
        true
    }

    /// Get the expected word at position
    pub fn expected_word(&self, pos: usize) -> &str {
        &self.passkey_words[self.positions[pos] - 1]
    }

    /// Get user inputs
    pub fn inputs(&self) -> &[String; 3] {
        &self.inputs
    }

    /// Set error message
    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Clear all inputs
    pub fn clear_inputs(&mut self) {
        self.inputs = [String::new(), String::new(), String::new()];
        self.error = None;
    }
}

impl Render for PasskeyVerifyScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Widget;

        let block = Block::default()
            .title("🔐 Verify Your Recovery Phrase")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = vec![
            Line::from(""),
            Line::from("Please enter the words at the following positions to confirm"),
            Line::from("you have correctly saved your PassKey:"),
            Line::from(""),
        ];

        // Render 3 input fields
        for i in 0..3 {
            let is_focused = i == self.focused;
            let style = if is_focused {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  Word #{}: ", self.positions[i]),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    if self.inputs[i].is_empty() {
                        "___________".to_string()
                    } else {
                        self.inputs[i].clone()
                    },
                    style,
                ),
                if is_focused {
                    Span::raw(" ◀")
                } else {
                    Span::raw("")
                },
            ]));
        }

        lines.push(Line::from(""));
        lines.push(
            Line::from("Tip: These words should match what you recorded earlier")
                .style(Style::default().fg(Color::DarkGray)),
        );

        if let Some(err) = &self.error {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("❌ {}", err),
                Style::default().fg(Color::Red),
            )));
        }

        lines.push(Line::from(""));
        lines.push(
            Line::from("[Tab] Switch field   [Enter] Verify   [Esc] Back")
                .style(Style::default().fg(Color::DarkGray)),
        );

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

        paragraph.render(inner, buf);
    }
}

impl Interactive for PasskeyVerifyScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Tab => {
                self.focused = (self.focused + 1) % 3;
                HandleResult::NeedsRender
            }
            KeyCode::BackTab => {
                self.focused = (self.focused + 2) % 3;
                HandleResult::NeedsRender
            }
            KeyCode::Char(c) => {
                if self.inputs[self.focused].len() < 32 {
                    self.inputs[self.focused].push(c);
                    self.error = None;
                }
                HandleResult::NeedsRender
            }
            KeyCode::Backspace => {
                self.inputs[self.focused].pop();
                HandleResult::NeedsRender
            }
            KeyCode::Enter => {
                if self.verify() {
                    HandleResult::Consumed
                } else {
                    self.error =
                        Some("One or more words are incorrect. Please try again.".to_string());
                    HandleResult::NeedsRender
                }
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for PasskeyVerifyScreen {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_words() -> Vec<String> {
        (1..=24)
            .map(|i| format!("word{}", i))
            .collect()
    }

    #[test]
    fn test_new_screen() {
        let words = get_test_words();
        let screen = PasskeyVerifyScreen::new(words);

        // Check positions are unique and in range
        assert!(screen.positions.iter().all(|&p| p >= 1 && p <= 24));
        assert_eq!(screen.positions.len(), 3);

        // Check all inputs empty
        assert!(screen.inputs.iter().all(|i| i.is_empty()));
    }

    #[test]
    fn test_with_positions() {
        let words = get_test_words();
        let screen = PasskeyVerifyScreen::with_positions(words, [1, 12, 24]);

        assert_eq!(screen.positions(), [1, 12, 24]);
    }

    #[test]
    fn test_verify_correct() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 12, 24]);

        screen.inputs = [
            "word1".to_string(),
            "word12".to_string(),
            "word24".to_string(),
        ];

        assert!(screen.verify());
    }

    #[test]
    fn test_verify_incorrect() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 12, 24]);

        screen.inputs = [
            "wrong1".to_string(),
            "word12".to_string(),
            "word24".to_string(),
        ];

        assert!(!screen.verify());
    }

    #[test]
    fn test_verify_case_insensitive() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 12, 24]);

        screen.inputs = [
            "WORD1".to_string(),
            "Word12".to_string(),
            "word24".to_string(),
        ];

        assert!(screen.verify());
    }

    #[test]
    fn test_handle_tab_navigation() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 2, 3]);

        assert_eq!(screen.focused, 0);

        screen.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(screen.focused, 1);

        screen.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(screen.focused, 2);

        screen.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(screen.focused, 0); // Wraps around
    }

    #[test]
    fn test_handle_backtab_navigation() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 2, 3]);

        assert_eq!(screen.focused, 0);

        screen.handle_key(KeyEvent::from(KeyCode::BackTab));
        assert_eq!(screen.focused, 2); // Goes backwards
    }

    #[test]
    fn test_handle_char_input() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 2, 3]);

        let result = screen.handle_key(KeyEvent::from(KeyCode::Char('a')));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert_eq!(screen.inputs[0], "a");
    }

    #[test]
    fn test_handle_backspace() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 2, 3]);
        screen.inputs[0] = "abc".to_string();

        let result = screen.handle_key(KeyEvent::from(KeyCode::Backspace));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert_eq!(screen.inputs[0], "ab");
    }

    #[test]
    fn test_enter_with_correct_inputs() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 2, 3]);
        screen.inputs = [
            "word1".to_string(),
            "word2".to_string(),
            "word3".to_string(),
        ];

        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::Consumed));
    }

    #[test]
    fn test_enter_with_incorrect_inputs() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 2, 3]);
        screen.inputs = [
            "wrong1".to_string(),
            "word2".to_string(),
            "word3".to_string(),
        ];

        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert!(screen.error.is_some());
    }

    #[test]
    fn test_clear_inputs() {
        let words = get_test_words();
        let mut screen = PasskeyVerifyScreen::with_positions(words, [1, 2, 3]);
        screen.inputs = ["a".to_string(), "b".to_string(), "c".to_string()];
        screen.error = Some("error".to_string());

        screen.clear_inputs();

        assert!(screen.inputs.iter().all(|i| i.is_empty()));
        assert!(screen.error.is_none());
    }

    #[test]
    fn test_expected_word() {
        let words = get_test_words();
        let screen = PasskeyVerifyScreen::with_positions(words, [1, 12, 24]);

        assert_eq!(screen.expected_word(0), "word1");
        assert_eq!(screen.expected_word(1), "word12");
        assert_eq!(screen.expected_word(2), "word24");
    }
}
