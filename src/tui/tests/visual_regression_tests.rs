//! Visual Regression Tests for TUI
//!
//! These tests render TUI components using ratatui's TestBackend
//! and snapshot the visual output to catch layout and rendering changes.

use crate::tui::testing::{render_snapshot, SnapshotSequence};
use crate::tui::{Screen, TuiApp};

#[test]
fn test_tuiapp_initial_render() {
    let mut app = TuiApp::new();
    let output = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_initial_render_narrow() {
    let mut app = TuiApp::new();
    let output = render_snapshot(40, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_render_with_output() {
    let mut app = TuiApp::new();
    app.process_command("/help");

    let output = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_render_with_input_buffer() {
    let mut app = TuiApp::new();
    for c in "/show mypassword".chars() {
        app.handle_char(c);
    }

    let output = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_render_input_sequence() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("input_render_sequence");

    // Initial render
    let initial = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("initial", initial);

    // Type some input
    for c in "/help".chars() {
        app.handle_char(c);
    }

    let with_input = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("with_input", with_input);

    // Submit command
    app.handle_char('\n');

    let after_submit = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("after_submit", after_submit);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_render_command_execution_sequence() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("command_execution_render");

    // Initial state
    let initial = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("initial", initial);

    // Type /help
    for c in "/help".chars() {
        app.handle_char(c);
    }

    let typed_help = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("typed_help", typed_help);

    // Submit
    app.handle_char('\n');

    let submitted_help = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("submitted_help", submitted_help);

    // Type /config
    for c in "/config".chars() {
        app.handle_char(c);
    }

    let typed_config = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("typed_config", typed_config);

    // Submit
    app.handle_char('\n');

    let submitted_config = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("submitted_config", submitted_config);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_render_screen_navigation() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("screen_navigation_render");

    // Main screen
    let main_screen = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("main_screen", main_screen);

    // Navigate to Settings
    app.navigate_to(Screen::Settings);
    let settings_screen = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("settings_screen", settings_screen);

    // Navigate to Help
    app.navigate_to(Screen::Help);
    let help_screen = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("help_screen", help_screen);

    // Return to main
    app.return_to_main();
    let back_to_main = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("back_to_main", back_to_main);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_multiple_outputs_render() {
    let mut app = TuiApp::new();
    app.process_command("/help");
    app.process_command("/config");
    app.process_command("/list");

    let output = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_long_command_render() {
    let mut app = TuiApp::new();
    let long_command = "/show this-is-a-very-long-record-name-that-might-wrap";
    for c in long_command.chars() {
        app.handle_char(c);
    }

    let output = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_very_narrow_render() {
    let mut app = TuiApp::new();
    app.process_command("/help");

    let output = render_snapshot(30, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_very_wide_render() {
    let mut app = TuiApp::new();

    let output = render_snapshot(120, 24, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_short_terminal_render() {
    let mut app = TuiApp::new();

    let output = render_snapshot(80, 10, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_tall_terminal_render() {
    let mut app = TuiApp::new();
    app.process_command("/help");

    let output = render_snapshot(80, 40, |frame| {
        app.render(frame);
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_tuiapp_statusline_at_different_widths() {
    let app = TuiApp::new();
    let mut seq = SnapshotSequence::new("statusline_widths");

    // Very narrow (< 60 columns)
    let narrow = render_snapshot(40, 1, |frame| {
        // Render minimal statusline test
        use ratatui::text::{Line, Text};
        use ratatui::widgets::Paragraph;
        let spans = app.render_statusline(40);
        frame.render_widget(Paragraph::new(Text::from(Line::from(spans))), frame.area());
    });
    seq.step("narrow_statusline", narrow);

    // Full width (>= 60 columns)
    let full = render_snapshot(80, 1, |frame| {
        use ratatui::text::{Line, Text};
        use ratatui::widgets::Paragraph;
        let spans = app.render_statusline(80);
        frame.render_widget(Paragraph::new(Text::from(Line::from(spans))), frame.area());
    });
    seq.step("full_statusline", full);

    // Wide screen (>= 100 columns)
    let wide = render_snapshot(120, 1, |frame| {
        use ratatui::text::{Line, Text};
        use ratatui::widgets::Paragraph;
        let spans = app.render_statusline(120);
        frame.render_widget(Paragraph::new(Text::from(Line::from(spans))), frame.area());
    });
    seq.step("wide_statusline", wide);

    insta::assert_snapshot!(seq.to_string());
}
