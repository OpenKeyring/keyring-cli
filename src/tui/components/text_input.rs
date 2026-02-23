//! TextInput 组件
//!
//! 可聚焦的文本输入框，支持基本编辑操作。

use crate::tui::error::TuiResult;
use crate::tui::traits::{
    Component, ComponentId, HandleResult, Interactive, Render,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

/// 文本输入组件
///
/// 支持功能：
/// - 文本输入和编辑
/// - 光标移动
/// - 占位符显示
/// - 密码模式（隐藏字符）
/// - 焦点状态样式
pub struct TextInput {
    /// 组件 ID
    id: ComponentId,
    /// 当前文本
    text: String,
    /// 光标位置
    cursor: usize,
    /// 占位符文本
    placeholder: String,
    /// 是否有焦点
    focused: bool,
    /// 是否为密码模式
    password_mode: bool,
    /// 最大长度限制
    max_length: Option<usize>,
}

impl TextInput {
    /// 创建新的文本输入框
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            id: ComponentId::new(0),
            text: String::new(),
            cursor: 0,
            placeholder: placeholder.into(),
            focused: false,
            password_mode: false,
            max_length: None,
        }
    }

    /// 设置组件 ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// 设置密码模式
    #[must_use]
    pub fn with_password_mode(mut self, enabled: bool) -> Self {
        self.password_mode = enabled;
        self
    }

    /// 设置最大长度
    #[must_use]
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// 获取文本内容
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 设置文本内容
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor = self.text.len();
    }

    /// 获取占位符
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// 清除文本
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// 插入字符
    fn insert_char(&mut self, ch: char) {
        if let Some(max) = self.max_length {
            if self.text.len() >= max {
                return;
            }
        }
        self.text.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    /// 删除光标前的字符
    fn backspace(&mut self) {
        if self.cursor > 0 {
            // 找到前一个字符的位置
            let prev_cursor = self.text[..self.cursor]
                .char_indices()
                .rev()
                .next()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.remove(prev_cursor);
            self.cursor = prev_cursor;
        }
    }

    /// 删除光标后的字符
    fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    /// 移动光标到左侧
    fn move_left(&mut self) {
        if self.cursor > 0 {
            let prev_cursor = self.text[..self.cursor]
                .char_indices()
                .rev()
                .next()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.cursor = prev_cursor;
        }
    }

    /// 移动光标到右侧
    fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            let next_cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
            self.cursor = next_cursor;
        }
    }

    /// 移动光标到开头
    fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// 移动光标到结尾
    fn move_end(&mut self) {
        self.cursor = self.text.len();
    }

    /// 获取显示文本（考虑密码模式）
    fn display_text(&self) -> String {
        if self.password_mode {
            "*".repeat(self.text.chars().count())
        } else {
            self.text.clone()
        }
    }
}

impl Render for TextInput {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        // 确定样式
        let style = if self.focused {
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::Gray)
                .bg(Color::Black)
        };

        let placeholder_style = Style::default()
            .fg(Color::DarkGray)
            .bg(if self.focused { Color::DarkGray } else { Color::Black });

        // 渲染背景
        for x in 0..area.width {
            buf[(area.x + x, area.y)].set_style(style);
        }

        // 渲染文本或占位符
        if self.text.is_empty() && !self.placeholder.is_empty() {
            // 显示占位符
            for (i, ch) in self.placeholder.chars().enumerate() {
                if i as u16 >= area.width {
                    break;
                }
                buf[(area.x + i as u16, area.y)]
                    .set_symbol(&ch.to_string())
                    .set_style(placeholder_style);
            }
        } else {
            // 显示文本
            let display = self.display_text();
            for (i, ch) in display.chars().enumerate() {
                if i as u16 >= area.width {
                    break;
                }
                buf[(area.x + i as u16, area.y)]
                    .set_symbol(&ch.to_string())
                    .set_style(style);
            }
        }

        // 如果有焦点，显示光标
        if self.focused && area.width > 0 {
            let cursor_x = self.cursor.min(area.width as usize - 1);
            // 光标位置用下划线表示
            if self.cursor < self.text.len() {
                buf[(area.x + cursor_x as u16, area.y)]
                    .set_style(style.add_modifier(Modifier::UNDERLINED));
            }
        }
    }
}

impl Interactive for TextInput {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        // 只处理按下事件
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Char(ch) => {
                self.insert_char(ch);
                HandleResult::Consumed
            }
            KeyCode::Backspace => {
                self.backspace();
                HandleResult::Consumed
            }
            KeyCode::Delete => {
                self.delete();
                HandleResult::Consumed
            }
            KeyCode::Left => {
                self.move_left();
                HandleResult::Consumed
            }
            KeyCode::Right => {
                self.move_right();
                HandleResult::Consumed
            }
            KeyCode::Home => {
                self.move_home();
                HandleResult::Consumed
            }
            KeyCode::End => {
                self.move_end();
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                HandleResult::Consumed // 可以触发提交事件
            }
            KeyCode::Esc => {
                HandleResult::Ignored // 让父组件处理
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for TextInput {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        self.focused = true;
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        self.focused = false;
        Ok(())
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::traits::{Component, ComponentId, Render, Interactive};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{buffer::Buffer, layout::Rect};

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new("placeholder");
        assert_eq!(input.placeholder(), "placeholder");
        assert!(input.text().is_empty());
    }

    #[test]
    fn test_text_input_can_focus() {
        let input = TextInput::new("test");
        assert!(input.can_focus());
    }

    #[test]
    fn test_text_input_type_chars() {
        let mut input = TextInput::new("test");

        let result = input.handle_key(create_key_event(KeyCode::Char('a')));
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(input.text(), "a");

        input.handle_key(create_key_event(KeyCode::Char('b')));
        input.handle_key(create_key_event(KeyCode::Char('c')));
        assert_eq!(input.text(), "abc");
    }

    #[test]
    fn test_text_input_backspace() {
        let mut input = TextInput::new("test");
        input.set_text("hello".to_string());

        input.handle_key(create_key_event(KeyCode::Backspace));
        assert_eq!(input.text(), "hell");
    }

    #[test]
    fn test_text_input_clear() {
        let mut input = TextInput::new("test");
        input.set_text("hello".to_string());

        input.clear();
        assert!(input.text().is_empty());
    }

    #[test]
    fn test_text_input_component_trait() {
        fn assert_component<T: Component + Send + Sync>() {}
        assert_component::<TextInput>();
    }

    #[test]
    fn test_text_input_render_with_text() {
        let mut input = TextInput::new("placeholder");
        input.set_text("hello".to_string());

        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);

        input.render(area, &mut buf);

        // 验证缓冲区包含 "hello"
        let content: String = buf.content().iter()
            .map(|c| c.symbol())
            .take(5)
            .collect();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_text_input_focus_handling() {
        let mut input = TextInput::new("test");

        // 模拟获得焦点
        input.on_focus_gain().unwrap();
        assert!(input.focused);

        // 输入内容
        input.handle_key(create_key_event(KeyCode::Char('X')));
        assert_eq!(input.text(), "X");

        // 模拟失去焦点
        input.on_focus_loss().unwrap();
        assert!(!input.focused);
    }

    #[test]
    fn test_text_input_password_mode() {
        let mut input = TextInput::new("password");
        input.set_text("secret123".to_string());
        input = input.with_password_mode(true); // Fix: assign the result back

        // In password mode, display text should be asterisks
        assert_eq!(input.display_text(), "*********");
    }

    #[test]
    fn test_text_input_cursor_movement() {
        let mut input = TextInput::new("test");
        input.set_text("hello".to_string());

        // Initially cursor is at end
        assert_eq!(input.cursor, 5);

        // Move left
        input.move_left();
        assert_eq!(input.cursor, 4);

        // Move right
        input.move_right();
        assert_eq!(input.cursor, 5);

        // Move home
        input.move_home();
        assert_eq!(input.cursor, 0);

        // Move end
        input.move_end();
        assert_eq!(input.cursor, 5);
    }
}