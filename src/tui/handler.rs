//! Global keyboard event handler for TUI
//!
//! This module provides the event handler that maps keyboard events to AppActions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions that can be triggered by keyboard events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    /// Open settings screen (F2)
    OpenSettings,
    /// Trigger sync now (F5)
    SyncNow,
    /// Show help screen (F1, ?)
    ShowHelp,
    /// Refresh current view (Ctrl+R)
    RefreshView,
    /// Save configuration (Ctrl+S)
    SaveConfig,
    /// Disable sync (Ctrl+D)
    DisableSync,
    /// Quit the application (q, Esc)
    Quit,
    /// No action mapped to this key
    None,
}

/// Global keyboard event handler for TUI
///
/// Maps crossterm key events to application actions based on predefined keybindings.
#[derive(Debug, Clone, Copy, Default)]
pub struct TuiEventHandler;

impl TuiEventHandler {
    /// Create a new event handler
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Handle a key event and return the corresponding action
    ///
    /// # Keybindings
    ///
    /// | Key | Action |
    /// |-----|--------|
    /// | F1 or ? | ShowHelp |
    /// | F2 | OpenSettings |
    /// | F5 | SyncNow |
    /// | Ctrl+R | RefreshView |
    /// | Ctrl+S | SaveConfig |
    /// | Ctrl+D | DisableSync |
    /// | q or Esc | Quit |
    /// | other | None |
    #[must_use]
    pub const fn handle_key_event(&self, event: KeyEvent) -> AppAction {
        match event.code {
            // Function keys
            KeyCode::F(1) => AppAction::ShowHelp,
            KeyCode::F(2) => AppAction::OpenSettings,
            KeyCode::F(5) => AppAction::SyncNow,

            // Character keys with modifiers
            KeyCode::Char('r') if event.modifiers.contains(KeyModifiers::CONTROL) => AppAction::RefreshView,
            KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => AppAction::SaveConfig,
            KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => AppAction::DisableSync,

            // Regular character keys
            KeyCode::Char('?') => AppAction::ShowHelp,
            KeyCode::Char('q') => AppAction::Quit,

            // Special keys
            KeyCode::Esc => AppAction::Quit,

            // Everything else
            _ => AppAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_trait() {
        let handler = TuiEventHandler::default();
        let event = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());

        let action = handler.handle_key_event(event);
        assert!(matches!(action, AppAction::OpenSettings));
    }
}
