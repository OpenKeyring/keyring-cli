//! Tests for DetailPanel component
//!
//! Unit tests for the detail panel component.

use super::*;
use crate::tui::models::password::PasswordRecord;
use crate::tui::state::{AppState, DetailMode};
use crate::tui::traits::{Component, Interactive};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

#[test]
fn test_detail_panel_creation() {
    let panel = DetailPanel::new();
    assert!(!panel.focused);
    assert!(!panel.password_visible);
}

#[test]
fn test_password_visibility_toggle() {
    let mut panel = DetailPanel::new();
    assert!(!panel.is_password_visible());

    panel.toggle_password_visibility();
    assert!(panel.is_password_visible());

    panel.toggle_password_visibility();
    assert!(!panel.is_password_visible());
}

#[test]
fn test_focus_state() {
    let mut panel = DetailPanel::new();
    assert!(!panel.is_focused());

    panel.on_focus_gain().unwrap();
    assert!(panel.is_focused());

    panel.on_focus_loss().unwrap();
    assert!(!panel.is_focused());
}

#[test]
fn test_password_hides_on_focus_loss() {
    let mut panel = DetailPanel::new();
    panel.toggle_password_visibility();
    assert!(panel.is_password_visible());

    panel.on_focus_loss().unwrap();
    assert!(!panel.is_password_visible());
}

#[test]
fn test_component_trait() {
    let panel = DetailPanel::new();
    assert!(panel.can_focus());
}

#[test]
fn test_handle_key_toggle_password() {
    let mut panel = DetailPanel::new();
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());

    let result = panel.handle_key(key);
    assert!(matches!(result, HandleResult::Consumed));
    assert!(panel.is_password_visible());
}

#[test]
fn test_handle_key_with_state_copy_username() {
    let mut panel = DetailPanel::new();
    let mut state = AppState::new();

    let id = Uuid::new_v4();
    let test_password = PasswordRecord::new(id.to_string(), "Test Password", "secret123")
        .with_username("testuser@example.com".to_string());

    state.refresh_password_cache(vec![test_password]);
    state.detail_mode = DetailMode::PasswordDetail(id);

    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty());
    let result = panel.handle_key_with_state(key, &mut state, None);

    assert!(matches!(result, HandleResult::Consumed));
}

#[test]
fn test_handle_key_with_state_copy_password() {
    let mut panel = DetailPanel::new();
    let mut state = AppState::new();

    let id = Uuid::new_v4();
    let test_password = PasswordRecord::new(id.to_string(), "Test Password", "secret123");

    state.refresh_password_cache(vec![test_password]);
    state.detail_mode = DetailMode::PasswordDetail(id);

    // Terminals emit Shift+C as the uppercase character Char('C') with no modifier bit.
    // The SHIFT modifier is consumed to produce the uppercase character, so SHIFT is never set.
    let key = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::empty());
    let result = panel.handle_key_with_state(key, &mut state, None);

    assert!(matches!(result, HandleResult::Consumed));
}

#[test]
fn test_handle_key_with_state_toggle_password() {
    let mut panel = DetailPanel::new();
    let mut state = AppState::new();

    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
    let result = panel.handle_key_with_state(key, &mut state, None);

    assert!(matches!(result, HandleResult::Consumed));
    assert!(panel.is_password_visible());
}

#[test]
fn test_data_binding_password_detail() {
    let mut state = AppState::new();

    let password_id = Uuid::new_v4();
    let test_password = PasswordRecord::new(password_id.to_string(), "Gmail Work", "secret123")
        .with_username("user@gmail.com".to_string())
        .with_url("https://gmail.com".to_string())
        .with_favorite(true);

    state.refresh_password_cache(vec![test_password.clone()]);

    assert!(matches!(state.detail_mode, DetailMode::ProjectInfo));

    let password = state.get_password(password_id);
    assert!(password.is_some(), "Password should exist in cache");

    let password = password.unwrap();
    assert_eq!(password.name, "Gmail Work");
    assert!(password.is_favorite);
    assert!(password.username.is_some());
    assert!(password.url.is_some());
}

#[test]
fn test_render_password_from_state() {
    let mut state = AppState::new();
    let _panel = DetailPanel::new();

    let password_id = Uuid::new_v4();
    let test_password = PasswordRecord::new(password_id.to_string(), "Test Password", "secret123");

    state.refresh_password_cache(vec![test_password.clone()]);
    state.select_password(password_id);

    assert!(matches!(
        state.detail_mode,
        DetailMode::PasswordDetail(id) if id == password_id
    ));

    let password = state.get_password(password_id);
    assert!(password.is_some());
}

#[test]
fn test_full_password_detail_flow() {
    let mut state = AppState::new();

    let password_id = Uuid::new_v4();
    let test_password = PasswordRecord::new(password_id.to_string(), "Gmail Work", "secret123")
        .with_username("user@gmail.com".to_string())
        .with_url("https://gmail.com".to_string())
        .with_tags(vec!["email".to_string(), "work".to_string()])
        .with_favorite(true);

    state.refresh_password_cache(vec![test_password.clone()]);
    state.apply_filter();
    state.select_password(password_id);

    let password = state.get_password(password_id);
    assert!(password.is_some());

    let password = password.unwrap();
    assert_eq!(password.name, "Gmail Work");
    assert!(password.username.is_some());
    assert!(password.url.is_some());
    assert!(!password.tags.is_empty());
    assert!(password.is_favorite);

    assert!(password.created_at.timestamp_micros() > 0);
    assert!(password.modified_at.timestamp_micros() > 0);
}

#[test]
fn test_esc_clears_detail_panel() {
    let mut panel = DetailPanel::new();
    let mut state = AppState::default();
    let id = Uuid::new_v4();

    // Select a password
    state.select_password(id);
    assert!(matches!(state.detail_mode, DetailMode::PasswordDetail(_)));

    // Press Esc
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let result = panel.handle_key_with_state(key, &mut state, None);

    assert!(matches!(result, HandleResult::Consumed));
    assert!(matches!(state.detail_mode, DetailMode::ProjectInfo));
    assert_eq!(state.focused_panel, crate::tui::state::FocusedPanel::Tree);
}
