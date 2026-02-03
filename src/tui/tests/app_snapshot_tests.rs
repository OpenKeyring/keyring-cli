//! TuiApp Snapshot Tests
//!
//! These tests use `insta` to snapshot TuiApp state at various stages
//! of command execution, navigation, and screen transitions.

use crate::tui::{Screen, TuiApp};

#[test]
fn test_tuiapp_initial_output() {
    let app = TuiApp::new();
    // Snapshot initial output lines
    insta::assert_debug_snapshot!(&app.output_lines);
}

#[test]
fn test_tuiapp_after_help_command() {
    let mut app = TuiApp::new();
    app.process_command("/help");

    // Snapshot state after help command
    insta::assert_debug_snapshot!(&app.output_lines);
}

#[test]
fn test_tuiapp_after_config_command() {
    let mut app = TuiApp::new();
    app.process_command("/config");

    // Snapshot state after config command
    insta::assert_debug_snapshot!(&app.output_lines);
}

#[test]
fn test_tuiapp_command_sequence() {
    use crate::tui::testing::SnapshotSequence;

    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("command_execution_flow");

    // Initial state
    let output_lines_snapshot = format!("{:?}", app.output_lines);
    seq.step("initial", output_lines_snapshot);

    // After /help command
    app.process_command("/help");
    let help_snapshot = format!("{:?}", app.output_lines);
    seq.step("after_help", help_snapshot);

    // After /config command
    app.process_command("/config");
    let config_snapshot = format!("{:?}", app.output_lines);
    seq.step("after_config", config_snapshot);

    // After /list command
    app.process_command("/list");
    let list_snapshot = format!("{:?}", app.output_lines);
    seq.step("after_list", list_snapshot);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_input_buffer_sequence() {
    use crate::tui::testing::SnapshotSequence;

    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("input_buffer_sequence");

    // Type characters
    app.handle_char('/');
    seq.step("typed_slash", format!("buffer: '{}'", app.input_buffer));

    app.handle_char('h');
    seq.step("typed_h", format!("buffer: '{}'", app.input_buffer));

    app.handle_char('e');
    seq.step("typed_e", format!("buffer: '{}'", app.input_buffer));

    app.handle_char('l');
    seq.step("typed_l", format!("buffer: '{}'", app.input_buffer));

    app.handle_char('p');
    seq.step("typed_p", format!("buffer: '{}'", app.input_buffer));

    // Submit
    app.handle_char('\n');
    seq.step("after_submit", format!("buffer: '{}'", app.input_buffer));

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_backspace_sequence() {
    use crate::tui::testing::SnapshotSequence;

    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("backspace_sequence");

    // Type "test"
    for c in "test".chars() {
        app.handle_char(c);
    }
    seq.step("typed_test", format!("buffer: '{}'", app.input_buffer));

    // Backspace
    app.handle_backspace();
    seq.step("after_backspace", format!("buffer: '{}'", app.input_buffer));

    // Backspace again
    app.handle_backspace();
    seq.step(
        "after_second_backspace",
        format!("buffer: '{}'", app.input_buffer),
    );

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_navigation_snapshots() {
    let mut app = TuiApp::new();

    // Initial screen - snapshot screen name
    insta::assert_snapshot!(app.current_screen().name());

    // Navigate to Settings
    app.navigate_to(Screen::Settings);
    insta::assert_snapshot!(app.current_screen().name());

    // Navigate to Help
    app.navigate_to(Screen::Help);
    insta::assert_snapshot!(app.current_screen().name());

    // Navigate to Sync
    app.navigate_to(Screen::Sync);
    insta::assert_snapshot!(app.current_screen().name());

    // Return to main
    app.return_to_main();
    insta::assert_snapshot!(app.current_screen().name());
}

#[test]
fn test_tuiapp_screen_navigation_sequence() {
    use crate::tui::testing::SnapshotSequence;

    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("screen_navigation");

    seq.step("initial_screen", format!("{:?}", app.current_screen()));

    app.navigate_to(Screen::Settings);
    seq.step("to_settings", format!("{:?}", app.current_screen()));

    app.navigate_to(Screen::Help);
    seq.step("to_help", format!("{:?}", app.current_screen()));

    app.navigate_to(Screen::Sync);
    seq.step("to_sync", format!("{:?}", app.current_screen()));

    app.return_to_main();
    seq.step("back_to_main", format!("{:?}", app.current_screen()));

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_unknown_command() {
    let mut app = TuiApp::new();
    app.process_command("/unknown_command");

    insta::assert_debug_snapshot!(&app.output_lines);
}

#[test]
fn test_tuiapp_autocomplete_sequence() {
    use crate::tui::testing::SnapshotSequence;

    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("autocomplete_sequence");

    // Type "/h" - should match /help
    for c in "/h".chars() {
        app.handle_char(c);
    }
    app.handle_autocomplete();
    seq.step("autocomplete_h", format!("buffer: '{}'", app.input_buffer));

    // Type "/l" - should match /list
    app.input_buffer.clear();
    for c in "/l".chars() {
        app.handle_char(c);
    }
    app.handle_autocomplete();
    seq.step("autocomplete_l", format!("buffer: '{}'", app.input_buffer));

    // Type "/s" - should match /show, /search
    app.input_buffer.clear();
    for c in "/s".chars() {
        app.handle_char(c);
    }
    app.handle_autocomplete();
    seq.step("autocomplete_s", format!("buffer: '{}'", app.input_buffer));

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_quit_sequence() {
    let mut app = TuiApp::new();

    // Initially running
    assert!(app.is_running());
    insta::assert_snapshot!(format!("is_running: {}", app.is_running()));

    // Execute /quit
    app.process_command("/quit");

    // No longer running
    assert!(!app.is_running());
    insta::assert_snapshot!(format!("is_running: {}", app.is_running()));
}

#[test]
fn test_tuiapp_multiple_commands_sequence() {
    use crate::tui::testing::SnapshotSequence;

    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("multiple_commands");

    // Clear initial output
    app.output_lines.clear();

    // Execute multiple commands in sequence
    app.process_command("/help");
    seq.step("after_help", format!("count: {}", app.output_lines.len()));

    app.process_command("/config");
    seq.step("after_config", format!("count: {}", app.output_lines.len()));

    app.process_command("/list");
    seq.step("after_list", format!("count: {}", app.output_lines.len()));

    app.process_command("/new");
    seq.step("after_new", format!("count: {}", app.output_lines.len()));

    insta::assert_snapshot!(seq.to_string());
}
