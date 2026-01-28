//! Keyboard shortcut string parser
//!
//! Parses shortcut strings like "Ctrl+N", "F5", "Ctrl+Shift+N" into crossterm KeyEvent.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fmt;

/// Error type for shortcut parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Empty input
    EmptyInput,
    /// Unknown modifier
    UnknownModifier(String),
    /// Unknown key
    UnknownKey(String),
    /// Invalid format
    InvalidFormat(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Empty input"),
            ParseError::UnknownModifier(m) => write!(f, "Unknown modifier: {}", m),
            ParseError::UnknownKey(k) => write!(f, "Unknown key: {}", k),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a shortcut string into a KeyEvent
///
/// # Examples
///
/// ```
/// use keyring_cli::keybindings::parser::parseShortcut;
///
/// // Simple Ctrl+Char
/// let event = parseShortcut("Ctrl+N").unwrap();
/// assert_eq!(event.code, KeyCode::Char('n'));
///
/// // Function key
/// let event = parseShortcut("F5").unwrap();
/// assert_eq!(event.code, KeyCode::F(5));
///
/// // Multiple modifiers
/// let event = parseShortcut("Ctrl+Shift+N").unwrap();
/// assert_eq!(event.code, KeyCode::Char('N'));
/// ```
pub fn parseShortcut(input: &str) -> Result<KeyEvent, ParseError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let parts: Vec<&str> = input.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Last part is always the key
    let key_part = parts.last().unwrap();
    let modifier_parts = &parts[..parts.len() - 1];

    // Parse modifiers
    let mut modifiers = KeyModifiers::empty();
    for modifier in modifier_parts {
        match modifier.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= KeyModifiers::CONTROL,
            "SHIFT" => modifiers |= KeyModifiers::SHIFT,
            "ALT" => modifiers |= KeyModifiers::ALT,
            "SUPER" | "CMD" | "COMMAND" => {
                // These are not directly supported by crossterm's KeyModifiers
                // We'll ignore them for now
            }
            _ => {
                return Err(ParseError::UnknownModifier(modifier.to_string()));
            }
        }
    }

    // Parse key
    let code = parseKeyCode(key_part, modifiers.contains(KeyModifiers::SHIFT))?;

    Ok(KeyEvent::new(code, modifiers))
}

/// Parse the key part of a shortcut string
fn parseKeyCode(key_str: &str, has_shift: bool) -> Result<KeyCode, ParseError> {
    let key_upper = key_str.to_uppercase();

    // Special keys
    match key_upper.as_str() {
        "ENTER" | "RETURN" => return Ok(KeyCode::Enter),
        "TAB" => return Ok(KeyCode::Tab),
        "BACKSPACE" => return Ok(KeyCode::Backspace),
        "ESC" | "ESCAPE" => return Ok(KeyCode::Esc),
        "SPACE" => return Ok(KeyCode::Char(' ')),
        "UP" => return Ok(KeyCode::Up),
        "DOWN" => return Ok(KeyCode::Down),
        "LEFT" => return Ok(KeyCode::Left),
        "RIGHT" => return Ok(KeyCode::Right),
        "INSERT" => return Ok(KeyCode::Insert),
        "DELETE" => return Ok(KeyCode::Delete),
        "HOME" => return Ok(KeyCode::Home),
        "END" => return Ok(KeyCode::End),
        "PAGEUP" => return Ok(KeyCode::PageUp),
        "PAGEDOWN" => return Ok(KeyCode::PageDown),
        _ => {}
    }

    // Function keys F1-F12
    if key_upper.starts_with('F') {
        let num_str = &key_upper[1..];
        if let Ok(num) = num_str.parse::<u8>() {
            if (1..=12).contains(&num) {
                return Ok(KeyCode::F(num));
            }
        }
    }

    // Single character
    if key_str.len() == 1 {
        let c = key_str.chars().next().unwrap();
        if has_shift {
            // When shift is pressed, use the uppercase version
            return Ok(KeyCode::Char(c.to_ascii_uppercase()));
        } else {
            return Ok(KeyCode::Char(c.to_ascii_lowercase()));
        }
    }

    Err(ParseError::UnknownKey(key_str.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ctrl_char() {
        let result = parseShortcut("Ctrl+N").unwrap();
        assert_eq!(result.code, KeyCode::Char('n'));
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
        assert!(!result.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_parse_ctrl_uppercase() {
        let result = parseShortcut("CTRL+N").unwrap();
        assert_eq!(result.code, KeyCode::Char('n'));
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_function_key() {
        let result = parseShortcut("F5").unwrap();
        assert_eq!(result.code, KeyCode::F(5));
        assert!(!result.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_ctrl_shift_char() {
        let result = parseShortcut("Ctrl+Shift+N").unwrap();
        assert_eq!(result.code, KeyCode::Char('N'));
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
        assert!(result.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_parse_ctrl_alt_char() {
        let result = parseShortcut("Ctrl+Alt+T").unwrap();
        assert_eq!(result.code, KeyCode::Char('t'));
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
        assert!(result.modifiers.contains(KeyModifiers::ALT));
    }

    #[test]
    fn test_parse_special_keys() {
        assert_eq!(parseShortcut("Enter").unwrap().code, KeyCode::Enter);
        assert_eq!(parseShortcut("Tab").unwrap().code, KeyCode::Tab);
        assert_eq!(parseShortcut("Esc").unwrap().code, KeyCode::Esc);
        assert_eq!(parseShortcut("Backspace").unwrap().code, KeyCode::Backspace);
        assert_eq!(parseShortcut("Space").unwrap().code, KeyCode::Char(' '));
    }

    #[test]
    fn test_parse_navigation_keys() {
        assert_eq!(parseShortcut("Up").unwrap().code, KeyCode::Up);
        assert_eq!(parseShortcut("Down").unwrap().code, KeyCode::Down);
        assert_eq!(parseShortcut("Left").unwrap().code, KeyCode::Left);
        assert_eq!(parseShortcut("Right").unwrap().code, KeyCode::Right);
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parseShortcut("");
        assert_eq!(result, Err(ParseError::EmptyInput));
    }

    #[test]
    fn test_parse_invalid_shortcut() {
        let result = parseShortcut("Invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_modifier() {
        let result = parseShortcut("Win+N");
        assert!(matches!(result, Err(ParseError::UnknownModifier(_))));
    }

    #[test]
    fn test_parse_ctrl_plus_enter() {
        let result = parseShortcut("Ctrl+Enter").unwrap();
        assert_eq!(result.code, KeyCode::Enter);
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_function_key_with_modifier() {
        let result = parseShortcut("Ctrl+F5").unwrap();
        assert_eq!(result.code, KeyCode::F(5));
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
    }
}
