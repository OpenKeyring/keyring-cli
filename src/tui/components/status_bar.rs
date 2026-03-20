//! StatusBar 组件
//!
//! 底部状态栏，显示当前状态信息和快捷键提示。

use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

/// 状态栏组件
///
/// 显示在应用底部，用于展示：
/// - 当前操作状态
/// - 快捷键提示
/// - 错误/警告信息
pub struct StatusBar {
    /// 组件 ID
    id: ComponentId,
    /// 状态消息
    message: String,
    /// 消息类型
    message_type: StatusMessageType,
    /// 左侧快捷键提示
    shortcuts: Vec<(String, String)>,
}

/// 状态消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusMessageType {
    /// 普通信息
    #[default]
    Info,
    /// 成功
    Success,
    /// 警告
    Warning,
    /// 错误
    Error,
}

impl StatusBar {
    /// 创建新的状态栏
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            id: ComponentId::new(0),
            message: message.into(),
            message_type: StatusMessageType::default(),
            shortcuts: Vec::new(),
        }
    }

    /// 设置组件 ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// 设置状态消息
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// 获取状态消息
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 设置消息类型
    pub fn set_message_type(&mut self, msg_type: StatusMessageType) {
        self.message_type = msg_type;
    }

    /// 添加快捷键提示
    pub fn add_shortcut(&mut self, key: impl Into<String>, desc: impl Into<String>) {
        self.shortcuts.push((key.into(), desc.into()));
    }

    /// 清除快捷键提示
    pub fn clear_shortcuts(&mut self) {
        self.shortcuts.clear();
    }
}

impl Render for StatusBar {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        // 背景样式
        let bg_style = Style::default().bg(Color::DarkGray).fg(Color::White);

        // 消息样式
        let msg_style = bg_style.add_modifier(Modifier::BOLD);

        // 渲染背景
        for x in 0..area.width {
            buf[(area.x + x, area.y)].set_style(bg_style);
        }

        // 渲染消息
        for (i, ch) in self.message.chars().enumerate() {
            if i as u16 >= area.width {
                break;
            }
            buf[(area.x + i as u16, area.y)]
                .set_symbol(&ch.to_string())
                .set_style(msg_style);
        }

        // 渲染快捷键提示（右侧）
        if !self.shortcuts.is_empty() && area.width > 20 {
            let shortcuts_text: String = self
                .shortcuts
                .iter()
                .map(|(k, d)| format!("[{}]{}", k, d))
                .collect::<Vec<_>>()
                .join(" ");

            let start_x = area.width.saturating_sub(shortcuts_text.len() as u16 + 2);
            for (i, ch) in shortcuts_text.chars().enumerate() {
                let x = area.x + start_x + i as u16;
                if x >= area.x + area.width {
                    break;
                }
                buf[(x, area.y)]
                    .set_symbol(&ch.to_string())
                    .set_style(bg_style);
            }
        }
    }
}

impl Interactive for StatusBar {
    fn handle_key(&mut self, _key: KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }

    fn handle_mouse(&mut self, _event: MouseEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

impl Component for StatusBar {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        false
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new("Ready")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::traits::{Component, Render};
    use ratatui::{buffer::Buffer, layout::Rect};

    #[test]
    fn test_status_bar_creation() {
        let bar = StatusBar::new("Test Message");
        assert_eq!(bar.message(), "Test Message");
    }

    #[test]
    fn test_status_bar_component_trait() {
        fn assert_component<T: Component + Send + Sync>() {}
        assert_component::<StatusBar>();
    }

    #[test]
    fn test_status_bar_cannot_focus() {
        let bar = StatusBar::new("Test");
        assert!(!bar.can_focus());
    }

    #[test]
    fn test_status_bar_render() {
        let bar = StatusBar::new("Hello");
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);

        bar.render(area, &mut buf);

        // 验证缓冲区有内容
        assert!(buf.content().iter().any(|c| c.symbol() != " "));
    }

    #[test]
    fn test_status_bar_set_message() {
        let mut bar = StatusBar::new("Initial");
        bar.set_message("Updated");
        assert_eq!(bar.message(), "Updated");
    }

    #[test]
    fn test_status_bar_shortcuts() {
        let mut bar = StatusBar::new("Test");
        bar.add_shortcut("q", "Quit");
        bar.add_shortcut("h", "Help");

        // Clear shortcuts
        bar.clear_shortcuts();
    }

    #[test]
    fn test_status_bar_message_types() {
        let mut bar = StatusBar::new("Test");

        bar.set_message_type(StatusMessageType::Info);
        bar.set_message_type(StatusMessageType::Success);
        bar.set_message_type(StatusMessageType::Warning);
        bar.set_message_type(StatusMessageType::Error);
    }

    #[test]
    fn test_status_bar_default() {
        let bar = StatusBar::default();
        assert_eq!(bar.message(), "Ready");
    }
}
