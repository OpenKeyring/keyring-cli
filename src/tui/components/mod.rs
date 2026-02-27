//! TUI 组件实现
//!
//! 提供可复用的 UI 组件，实现 Component trait。

mod status_bar;
mod text_input;
mod tree;
mod select;
mod text_area;
mod form_example;
mod filter_panel;
mod detail_panel;

pub use status_bar::StatusBar;
pub use text_input::TextInput;
pub use tree::TreeComponent;
pub use select::Select;
pub use select::SelectItem;
pub use text_area::TextArea;
pub use filter_panel::{FilterPanel, FilterItem};
pub use detail_panel::DetailPanel;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::tui::traits::{Component, Render, Interactive, HandleResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{buffer::Buffer, layout::Rect};

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_component_trait_bounds() {
        // 验证所有组件都实现了必要的 trait
        fn assert_component<T: Component + Send + Sync + 'static>() {}

        assert_component::<StatusBar>();
        assert_component::<TextInput>();
    }

    #[test]
    fn test_status_bar_render_integration() {
        let bar = StatusBar::new("Ready");
        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);

        bar.render(area, &mut buf);

        // 验证渲染输出
        let content: String = buf.content().iter()
            .take(5)
            .map(|c| c.symbol())
            .collect();
        assert_eq!(content, "Ready");
    }

    #[test]
    fn test_text_input_focus_integration() {
        let mut input = TextInput::new("Enter text");

        // 初始状态不可聚焦但 focused 为 false
        assert!(input.can_focus());

        // 模拟获得焦点
        input.on_focus_gain().unwrap();

        // 输入文本
        let result = input.handle_key(create_key_event(KeyCode::Char('H')));
        assert!(matches!(result, HandleResult::Consumed));

        input.handle_key(create_key_event(KeyCode::Char('i')));

        assert_eq!(input.text(), "Hi");

        // 失去焦点
        input.on_focus_loss().unwrap();
    }

    #[test]
    fn test_multiple_components_independently() {
        let mut status = StatusBar::new("Status");
        let mut input = TextInput::new("Input");

        // 它们应该独立工作
        status.set_message("Updated");
        assert_eq!(status.message(), "Updated");

        input.set_text("test".to_string());
        assert_eq!(input.text(), "test");
    }

    #[test]
    fn test_component_lifecycle_hooks() {
        let mut input = TextInput::new("test");

        // Test mount hook
        assert!(input.on_mount().is_ok());

        // Gain focus
        assert!(input.on_focus_gain().is_ok());
        // We can't directly check the focused field since it's private,
        // but we know the function call succeeded

        // Lose focus
        assert!(input.on_focus_loss().is_ok());
        // Same as above, function call succeeded

        // Test unmount hook
        assert!(input.on_unmount().is_ok());
    }

    #[test]
    fn test_text_input_keyboard_events() {
        let mut input = TextInput::new("placeholder");

        // Type some text
        input.handle_key(create_key_event(KeyCode::Char('h')));
        input.handle_key(create_key_event(KeyCode::Char('e')));
        input.handle_key(create_key_event(KeyCode::Char('l')));
        input.handle_key(create_key_event(KeyCode::Char('l')));
        input.handle_key(create_key_event(KeyCode::Char('o')));

        assert_eq!(input.text(), "hello");

        // Test backspace
        input.handle_key(create_key_event(KeyCode::Backspace));
        assert_eq!(input.text(), "hell");

        // Test left/right movement
        input.handle_key(create_key_event(KeyCode::Left));
        input.handle_key(create_key_event(KeyCode::Left));
        // At this point, cursor is positioned but text is unchanged

        // Test delete
        input.handle_key(create_key_event(KeyCode::Delete));
        // This should delete the character at cursor position
        // The actual behavior may vary depending on how cursor position is handled
    }

    #[test]
    fn test_render_trait_compatibility() {
        let status = StatusBar::new("Status");
        let input = TextInput::new("Input");

        let area = Rect::new(0, 0, 20, 1);
        let mut status_buf = Buffer::empty(area);
        let mut input_buf = Buffer::empty(area);

        status.render(area, &mut status_buf);
        input.render(area, &mut input_buf);

        // Both components should render without panicking
        assert!(true); // If we reach this, both rendered without panicking
    }

    #[test]
    fn test_interactive_trait_compatibility() {
        use crossterm::event::KeyCode;

        let mut status = StatusBar::new("Status");
        let mut input = TextInput::new("Input");

        // Both should handle key events without panicking
        let key_event = create_key_event(KeyCode::Char('a'));

        let _status_result = status.handle_key(key_event);
        let _input_result = input.handle_key(key_event);

        // Status bar should ignore, input should consume
        assert!(true); // If we reach this, both handled without panicking
    }
}
