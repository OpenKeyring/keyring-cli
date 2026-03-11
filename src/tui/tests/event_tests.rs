//! TUI 事件类型测试
//!
//! 测试 AppEvent、HandleResult、Action、ScreenType 等事件相关类型。

use crate::tui::components::ConfirmAction;
use crate::tui::traits::{Action, AppEvent, FilterType, HandleResult, ScreenType, ComponentId};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseButton, MouseEventKind};

// ============================================================================
// AppEvent 测试
// ============================================================================

#[test]
fn test_app_event_key() {
    let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    let app_event = AppEvent::Key(key_event);
    assert!(matches!(app_event, AppEvent::Key(_)));
}

#[test]
fn test_app_event_mouse() {
    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 20,
        modifiers: KeyModifiers::NONE,
    };
    let app_event = AppEvent::Mouse(mouse_event);
    assert!(matches!(app_event, AppEvent::Mouse(_)));
}

#[test]
fn test_app_event_quit() {
    let app_event = AppEvent::Quit;
    assert!(matches!(app_event, AppEvent::Quit));
}

#[test]
fn test_app_event_tick() {
    let app_event = AppEvent::Tick;
    assert!(matches!(app_event, AppEvent::Tick));
}

#[test]
fn test_app_event_refresh() {
    let app_event = AppEvent::Refresh;
    assert!(matches!(app_event, AppEvent::Refresh));
}

#[test]
fn test_app_event_focus_changed() {
    let id = ComponentId::new(42);
    let app_event = AppEvent::FocusChanged(id);
    assert!(matches!(app_event, AppEvent::FocusChanged(_)));
    if let AppEvent::FocusChanged(component_id) = app_event {
        assert_eq!(component_id.value(), 42);
    }
}

#[test]
fn test_app_event_screen_opened() {
    let screen_type = ScreenType::Wizard;
    let app_event = AppEvent::ScreenOpened(screen_type);
    assert!(matches!(app_event, AppEvent::ScreenOpened(_)));
}

#[test]
fn test_app_event_screen_closed() {
    let app_event = AppEvent::ScreenClosed;
    assert!(matches!(app_event, AppEvent::ScreenClosed));
}

#[test]
fn test_app_event_screen_dismissed() {
    let app_event = AppEvent::ScreenDismissed;
    assert!(matches!(app_event, AppEvent::ScreenDismissed));
}

// ============================================================================
// FilterType 测试
// ============================================================================

#[test]
fn test_filter_type_none() {
    let filter = FilterType::None;
    assert_eq!(filter, FilterType::None);
}

#[test]
fn test_filter_type_all() {
    let filter = FilterType::All;
    assert_eq!(filter, FilterType::All);
    assert_ne!(filter, FilterType::None);
}

#[test]
fn test_filter_type_favorites() {
    let filter = FilterType::Favorites;
    assert_eq!(filter, FilterType::Favorites);
}

#[test]
fn test_filter_type_recent() {
    let filter = FilterType::Recent;
    assert_eq!(filter, FilterType::Recent);
}

#[test]
fn test_filter_type_group() {
    let filter = FilterType::Group("work".to_string());
    assert_eq!(filter, FilterType::Group("work".to_string()));
    assert_ne!(filter, FilterType::Group("personal".to_string()));
}

#[test]
fn test_filter_type_tag() {
    let filter = FilterType::Tag("social".to_string());
    assert_eq!(filter, FilterType::Tag("social".to_string()));
}

#[test]
fn test_filter_type_search() {
    let filter = FilterType::Search("password".to_string());
    assert_eq!(filter, FilterType::Search("password".to_string()));
}

// ============================================================================
// ScreenType 测试
// ============================================================================

#[test]
fn test_screen_type_wizard() {
    let screen = ScreenType::Wizard;
    assert_eq!(screen, ScreenType::Wizard);
}

#[test]
fn test_screen_type_new_password() {
    let screen = ScreenType::NewPassword;
    assert_eq!(screen, ScreenType::NewPassword);
}

#[test]
fn test_screen_type_edit_password() {
    let screen = ScreenType::EditPassword("test-id".to_string());
    assert_eq!(screen, ScreenType::EditPassword("test-id".to_string()));
    assert_ne!(screen, ScreenType::EditPassword("other-id".to_string()));
}

#[test]
fn test_screen_type_confirm_dialog() {
    let screen = ScreenType::ConfirmDialog(ConfirmAction::Generic);
    assert_eq!(screen, ScreenType::ConfirmDialog(ConfirmAction::Generic));
}

#[test]
fn test_screen_type_trash_bin() {
    let screen = ScreenType::TrashBin;
    assert_eq!(screen, ScreenType::TrashBin);
}

#[test]
fn test_screen_type_settings() {
    let screen = ScreenType::Settings;
    assert_eq!(screen, ScreenType::Settings);
}

#[test]
fn test_screen_type_help() {
    let screen = ScreenType::Help;
    assert_eq!(screen, ScreenType::Help);
}

#[test]
fn test_screen_type_main() {
    let screen = ScreenType::Main;
    assert_eq!(screen, ScreenType::Main);
}

// ============================================================================
// HandleResult 测试
// ============================================================================

#[test]
fn test_handle_result_consumed() {
    let result = HandleResult::Consumed;
    assert_eq!(result, HandleResult::Consumed);
    assert_ne!(result, HandleResult::Ignored);
}

#[test]
fn test_handle_result_ignored() {
    let result = HandleResult::Ignored;
    assert_eq!(result, HandleResult::Ignored);
}

#[test]
fn test_handle_result_needs_render() {
    let result = HandleResult::NeedsRender;
    assert_eq!(result, HandleResult::NeedsRender);
}

#[test]
fn test_handle_result_action() {
    let action = Action::Quit;
    let result = HandleResult::Action(action.clone());
    assert!(matches!(result, HandleResult::Action(_)));
    if let HandleResult::Action(a) = result {
        assert!(matches!(a, Action::Quit));
    }
}

#[test]
fn test_handle_result_default() {
    let result = HandleResult::default();
    assert_eq!(result, HandleResult::Ignored);
}

// ============================================================================
// Action 测试
// ============================================================================

#[test]
fn test_action_quit() {
    let action = Action::Quit;
    assert!(matches!(action, Action::Quit));
}

#[test]
fn test_action_open_screen() {
    let screen = ScreenType::Settings;
    let action = Action::OpenScreen(screen);
    assert!(matches!(action, Action::OpenScreen(_)));
}

#[test]
fn test_action_close_screen() {
    let action = Action::CloseScreen;
    assert!(matches!(action, Action::CloseScreen));
}

#[test]
fn test_action_show_toast() {
    let action = Action::ShowToast("Test message".to_string());
    assert!(matches!(action, Action::ShowToast(_)));
    if let Action::ShowToast(msg) = action {
        assert_eq!(msg, "Test message");
    }
}

#[test]
fn test_action_copy_to_clipboard() {
    let action = Action::CopyToClipboard("copied text".to_string());
    assert!(matches!(action, Action::CopyToClipboard(_)));
    if let Action::CopyToClipboard(text) = action {
        assert_eq!(text, "copied text");
    }
}

#[test]
fn test_action_refresh() {
    let action = Action::Refresh;
    assert!(matches!(action, Action::Refresh));
}

#[test]
fn test_action_none() {
    let action = Action::None;
    assert!(matches!(action, Action::None));
}

#[test]
fn test_action_default() {
    let action = Action::default();
    assert_eq!(action, Action::None);
}

// ============================================================================
// Clone 测试
// ============================================================================

#[test]
fn test_app_event_clone() {
    let event1 = AppEvent::Quit;
    let event2 = event1.clone();
    assert!(matches!(event2, AppEvent::Quit));
}

#[test]
fn test_action_clone() {
    let action1 = Action::ShowToast("test".to_string());
    let action2 = action1.clone();
    assert_eq!(action1, action2);
}

#[test]
fn test_filter_type_clone() {
    let filter1 = FilterType::Group("work".to_string());
    let filter2 = filter1.clone();
    assert_eq!(filter1, filter2);
}

#[test]
fn test_screen_type_clone() {
    let screen1 = ScreenType::EditPassword("id".to_string());
    let screen2 = screen1.clone();
    assert_eq!(screen1, screen2);
}

// ============================================================================
// Debug 测试
// ============================================================================

#[test]
fn test_app_event_debug() {
    let event = AppEvent::Quit;
    assert_eq!(format!("{:?}", event), "Quit");
}

#[test]
fn test_handle_result_debug() {
    let result = HandleResult::Consumed;
    assert_eq!(format!("{:?}", result), "Consumed");
}

#[test]
fn test_action_debug() {
    let action = Action::Quit;
    assert_eq!(format!("{:?}", action), "Quit");
}

#[test]
fn test_filter_type_debug() {
    let filter = FilterType::Search("query".to_string());
    let debug_str = format!("{:?}", filter);
    assert!(debug_str.contains("Search"));
}
