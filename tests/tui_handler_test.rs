//! Tests for TUI event handler
//!
//! These tests verify that keyboard events are correctly mapped to AppActions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use keyring_cli::tui::handler::{AppAction, TuiEventHandler};

#[test]
fn test_f2_opens_settings() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::OpenSettings));
}

#[test]
fn test_f5_triggers_sync() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::F(5), KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::SyncNow));
}

#[test]
fn test_question_mark_shows_help() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::ShowHelp));
}

#[test]
fn test_ctrl_r_refreshes() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::RefreshView));
}

#[test]
fn test_f1_also_shows_help() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::F(1), KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::ShowHelp));
}

#[test]
fn test_ctrl_s_saves_config() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::SaveConfig));
}

#[test]
fn test_ctrl_d_disables_sync() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::DisableSync));
}

#[test]
fn test_q_quits() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::Quit));
}

#[test]
fn test_escape_quits() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::Quit));
}

#[test]
fn test_unknown_key_returns_none() {
    let handler = TuiEventHandler::new();
    let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::None));
}

#[test]
fn test_default_trait() {
    let handler = TuiEventHandler;
    let event = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());

    let action = handler.handle_key_event(event);
    assert!(matches!(action, AppAction::OpenSettings));
}
