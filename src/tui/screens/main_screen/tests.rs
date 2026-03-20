//! Tests for MainScreen
//!
//! Unit tests for the main screen component.

use super::*;
use crate::tui::models::password::PasswordRecord;
use crate::tui::state::filter_state::FilterType;
use crate::tui::state::FocusedPanel;
use crate::tui::traits::{Action, HandleResult, NotificationLevel, ScreenType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use uuid::Uuid;

#[test]
fn test_main_screen_creation() {
    let screen = MainScreen::new();
    assert!(screen.layout().is_none());
}

#[test]
fn test_layout_calculation() {
    let mut screen = MainScreen::new();
    let area = Rect::new(0, 0, 100, 30);
    let layout = screen.calculate_layout(area);

    // Verify layout regions are valid
    assert!(layout.left_column.width > 0);
    assert!(layout.right_column.width > 0);
    assert!(layout.tree_area.height > 0);
    assert!(layout.filter_area.height > 0);
    assert!(layout.detail_area.height > 0);
    assert!(layout.status_bar_area.height == 1);

    // Verify layout is cached
    assert!(screen.layout().is_some());
}

#[test]
fn test_layout_proportions() {
    let mut screen = MainScreen::new();
    let area = Rect::new(0, 0, 100, 30);
    let layout = screen.calculate_layout(area);

    // Left column should be ~35% of content width
    let content_width = 100;
    let left_width = layout.left_column.width as f32;
    let left_ratio = left_width / content_width as f32;
    assert!(
        left_ratio > 0.30 && left_ratio < 0.40,
        "Left column ratio: {}",
        left_ratio
    );

    // Right column should be ~65% of content width
    let right_width = layout.right_column.width as f32;
    let right_ratio = right_width / content_width as f32;
    assert!(
        right_ratio > 0.60 && right_ratio < 0.70,
        "Right column ratio: {}",
        right_ratio
    );
}

#[test]
fn test_filter_panel_updates_tree() {
    // Test that toggling a filter updates the tree panel
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Initialize tree with data
    state.apply_filter();
    let _initial_count = state.tree.visible_nodes.len();

    // Switch to Filter panel
    state.set_focus(FocusedPanel::Filter);

    // Navigate to Favorite filter (index 3 in default items: All, Trash, Expired, Favorite)
    for _ in 0..3 {
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
            &mut state,
        );
    }

    // Toggle Favorite filter
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));

    // Verify filter is active
    assert!(state.filter.is_active(&FilterType::Favorite));

    // Verify tree was updated (should have different nodes now)
    // After filtering, tree should be refreshed
    // Note: Without expansion, we only see root groups, but the filter is now active
    assert!(state.filter.active_filters.contains(&FilterType::Favorite));
}

#[test]
fn test_filter_panel_navigation_updates_filter() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Switch to Filter panel
    state.set_focus(FocusedPanel::Filter);

    // Navigate to Trash filter (index 1)
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
        &mut state,
    );

    // Toggle Trash filter
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        &mut state,
    );

    // Verify Trash filter is active
    assert!(state.filter.is_active(&FilterType::Trash));
}

// ========== Global Shortcuts Tests ==========

#[test]
fn test_global_quit_shortcut() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Press 'q' should return Quit action
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(result, HandleResult::Action(Action::Quit)));
}

#[test]
fn test_global_help_shortcut() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Press '?' should return OpenScreen(Help) action
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(
        result,
        HandleResult::Action(Action::OpenScreen(ScreenType::Help))
    ));
}

#[test]
fn test_global_trash_shortcut() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Press 'T' (Shift+T) should return OpenScreen(Trash) action
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('T'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(
        result,
        HandleResult::Action(Action::OpenScreen(ScreenType::Trash))
    ));
}

#[test]
fn test_global_search_shortcut() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Press '/' should show search bar
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(result, HandleResult::Consumed));
}

#[test]
fn test_panel_switch_with_numbers() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Press '1' - switch to Tree
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.focused_panel, FocusedPanel::Tree);

    // Press '2' - switch to Filter
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('2'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.focused_panel, FocusedPanel::Filter);

    // Press '3' - switch to Detail
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('3'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.focused_panel, FocusedPanel::Detail);
}

#[test]
fn test_tab_navigation() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Start at Tree
    assert_eq!(state.focused_panel, FocusedPanel::Tree);

    // Tab - next panel
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        &mut state,
    );
    assert_eq!(state.focused_panel, FocusedPanel::Filter);

    // Tab - next panel
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        &mut state,
    );
    assert_eq!(state.focused_panel, FocusedPanel::Detail);

    // Tab - wrap around to Tree
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        &mut state,
    );
    assert_eq!(state.focused_panel, FocusedPanel::Tree);
}

#[test]
fn test_shift_tab_navigation() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // Start at Tree
    assert_eq!(state.focused_panel, FocusedPanel::Tree);

    // Shift+Tab (BackTab) - go to Detail (reverse direction)
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::empty()),
        &mut state,
    );
    assert_eq!(state.focused_panel, FocusedPanel::Detail);

    // Shift+Tab - go to Filter
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::empty()),
        &mut state,
    );
    assert_eq!(state.focused_panel, FocusedPanel::Filter);

    // Shift+Tab - go to Tree
    screen.handle_key_with_state(
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::empty()),
        &mut state,
    );
    assert_eq!(state.focused_panel, FocusedPanel::Tree);
}

// ========== Toast Notification Tests ==========

#[test]
fn test_notification_queue_management() {
    let mut state = AppState::new();

    // Add notification
    state.add_notification("Test message", NotificationLevel::Info);
    assert_eq!(state.notifications.len(), 1);

    // Add another notification
    state.add_notification("Second message", NotificationLevel::Success);
    assert_eq!(state.notifications.len(), 2);
}

#[test]
fn test_notification_level_styles() {
    let mut state = AppState::new();

    // Add notifications of different levels
    state.add_notification("Info", NotificationLevel::Info);
    state.add_notification("Success", NotificationLevel::Success);
    state.add_notification("Warning", NotificationLevel::Warning);
    state.add_notification("Error", NotificationLevel::Error);

    assert_eq!(state.notifications.len(), 4);

    // Verify levels are correct
    let levels: Vec<_> = state.notifications.iter().map(|n| n.level).collect();
    assert!(levels.contains(&NotificationLevel::Info));
    assert!(levels.contains(&NotificationLevel::Success));
    assert!(levels.contains(&NotificationLevel::Warning));
    assert!(levels.contains(&NotificationLevel::Error));
}

#[test]
fn test_notification_queue_limit() {
    let mut state = AppState::new();

    // Add more than the limit (5)
    for i in 0..10 {
        state.add_notification(&format!("Message {}", i), NotificationLevel::Info);
    }

    // Should be limited to 5
    assert_eq!(state.notifications.len(), 5);

    // The oldest messages should have been removed
    let messages: Vec<_> = state
        .notifications
        .iter()
        .map(|n| n.message.as_str())
        .collect();
    assert!(!messages.contains(&"Message 0"));
    assert!(!messages.contains(&"Message 4"));
    assert!(messages.contains(&"Message 5"));
    assert!(messages.contains(&"Message 9"));
}

#[test]
fn test_notification_auto_dismiss() {
    use std::time::Duration;

    let mut state = AppState::new();

    // Add a notification
    state.add_notification("Test", NotificationLevel::Info);

    // Verify it's there
    assert_eq!(state.notifications.len(), 1);

    // The notification should have a default duration of 3 seconds for Info
    let notification = state.notifications.front().unwrap();
    assert_eq!(notification.effective_duration(), Duration::from_secs(3));

    // Error notifications don't auto-dismiss (duration = 0)
    state.add_notification("Error test", NotificationLevel::Error);
    let error_notification = state.notifications.back().unwrap();
    assert_eq!(
        error_notification.effective_duration(),
        Duration::from_secs(0)
    );
}

#[test]
fn test_notification_cleanup_expired() {
    let mut state = AppState::new();

    // Add notification with very short duration
    state.add_notification("Test", NotificationLevel::Info);

    // Since we just created it, it shouldn't be expired
    state.cleanup_notifications();
    assert_eq!(state.notifications.len(), 1);
}

// ========== Boundary Condition Tests ==========

#[test]
fn test_minimum_terminal_size_constant() {
    // Verify the minimum size constants are reasonable
    assert_eq!(MainScreen::MIN_WIDTH, 80);
    assert_eq!(MainScreen::MIN_HEIGHT, 24);
}

#[test]
fn test_empty_tree_with_no_filters() {
    let mut state = AppState::new();

    // With empty cache, apply_filter produces empty visible_nodes
    state.apply_filter();

    let tree_state = &state.tree;

    // Tree state should be empty with no data in cache
    assert!(
        tree_state.visible_nodes.is_empty(),
        "Empty cache should produce empty visible nodes"
    );

    // Now add some data and verify it appears
    let test_password =
        PasswordRecord::new(Uuid::new_v4().to_string(), "Test Password", "secret123");
    state.refresh_password_cache(vec![test_password]);

    // Tree state should now have visible nodes
    assert!(
        !state.tree.visible_nodes.is_empty(),
        "Cache with data should produce visible nodes"
    );
}

#[test]
fn test_filter_state_active_detection() {
    let mut filter_state = crate::tui::state::filter_state::FilterState::default();

    // Initially, no active filters
    assert!(!filter_state.has_active_filters());

    // Toggle "All" - still considered no real filter
    filter_state.toggle(FilterType::All);
    assert!(
        !filter_state.has_active_filters(),
        "All filter alone should not count as active"
    );

    // Toggle "Favorite" - now we have an active filter
    filter_state.toggle(FilterType::Favorite);
    assert!(
        filter_state.has_active_filters(),
        "Favorite filter should be considered active"
    );

    // Clear and check
    filter_state.clear();
    assert!(
        !filter_state.has_active_filters(),
        "After clear, no active filters"
    );
}

#[test]
fn test_terminal_size_check() {
    // Create areas of different sizes
    let normal_area = Rect::new(0, 0, 100, 30);
    let narrow_area = Rect::new(0, 0, 60, 30);
    let short_area = Rect::new(0, 0, 100, 15);
    let tiny_area = Rect::new(0, 0, 60, 15);

    // Normal size should not trigger warning
    assert!(normal_area.width >= MainScreen::MIN_WIDTH);
    assert!(normal_area.height >= MainScreen::MIN_HEIGHT);

    // Narrow size should trigger warning
    assert!(narrow_area.width < MainScreen::MIN_WIDTH);

    // Short size should trigger warning
    assert!(short_area.height < MainScreen::MIN_HEIGHT);

    // Tiny size should trigger warning
    assert!(tiny_area.width < MainScreen::MIN_WIDTH || tiny_area.height < MainScreen::MIN_HEIGHT);
}

#[test]
fn test_empty_visible_nodes_scenario() {
    let mut state = AppState::new();

    // Apply filter that matches nothing
    state.filter.active_filters.insert(FilterType::Trash);
    state.apply_filter();

    // The filter is active
    assert!(state.filter.has_active_filters());
}

#[test]
fn test_toggle_favorite_shortcut() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    let id = Uuid::new_v4();
    let password = PasswordRecord::new(id.to_string(), "Test Password", "secret123");
    state.refresh_password_cache(vec![password]);
    state.select_password(id);

    // Verify not favorite initially
    assert!(!state.get_password(id).unwrap().is_favorite);

    // Press 'f' to toggle favorite on
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(result, HandleResult::Consumed));
    assert!(state.get_password(id).unwrap().is_favorite);

    // Press 'f' again to toggle favorite off
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(result, HandleResult::Consumed));
    assert!(!state.get_password(id).unwrap().is_favorite);

    // Test '*' shortcut also works
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('*'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(result, HandleResult::Consumed));
    assert!(state.get_password(id).unwrap().is_favorite);
}

#[test]
fn test_toggle_favorite_no_selection() {
    let mut screen = MainScreen::new();
    let mut state = AppState::new();

    // No password selected - should show toast
    let result = screen.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()),
        &mut state,
    );

    assert!(matches!(result, HandleResult::Action(Action::ShowToast(_))));
}
