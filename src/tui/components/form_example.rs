//! 示例表单实现
//!
//! 演示如何使用新实现的表单组件创建复合表单。

use crate::tui::components::{Select, SelectItem, TextArea, TextInput};
use crate::tui::error::TuiResult;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Widget,
    widgets::{Block, Borders, Paragraph},
};

/// 示例表单组件
pub struct SampleForm {
    /// 组件 ID
    id: ComponentId,
    /// 用户名输入框
    username_input: TextInput,
    /// 描述文本区域
    description_area: TextArea,
    /// 角色选择下拉框
    role_select: Select,
    /// 是否有焦点
    focused: bool,
}

impl SampleForm {
    /// 创建新的示例表单
    pub fn new() -> Self {
        // 创建用户名输入框
        let username_input = TextInput::new("输入用户名...")
            .with_id(ComponentId::new(1))
            .with_max_length(50);

        // 创建描述文本区域
        let description_area = TextArea::new("输入描述...")
            .with_id(ComponentId::new(2))
            .with_title("描述");

        // 创建角色选择下拉框
        let role_items = vec![
            SelectItem::new("管理员", "admin"),
            SelectItem::new("普通用户", "user"),
            SelectItem::new("访客", "guest"),
        ];
        let role_select = Select::new(role_items)
            .with_id(ComponentId::new(3))
            .with_title("用户角色");

        Self {
            id: ComponentId::new(0),
            username_input,
            description_area,
            role_select,
            focused: false,
        }
    }

    /// 获取用户名
    pub fn username(&self) -> &str {
        self.username_input.text()
    }

    /// 获取描述
    pub fn description(&self) -> String {
        self.description_area.text()
    }

    /// 获取角色
    pub fn role(&self) -> Option<&str> {
        self.role_select.selected_value()
    }
}

impl Render for SampleForm {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // 创建布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 用户名输入
                Constraint::Min(5),    // 描述文本区域
                Constraint::Length(5), // 角色选择
                Constraint::Length(3), // 说明文本
            ])
            .split(area);

        // 渲染用户名输入框
        self.username_input.render(chunks[0], buf);

        // 渲染描述文本区域
        self.description_area.render(chunks[1], buf);

        // 渲染角色选择下拉框
        self.role_select.render(chunks[2], buf);

        // 渲染说明文本
        let help_text = if self.focused {
            "使用 Tab 切换焦点，Enter 确认选择"
        } else {
            "按 Tab 进入表单"
        };

        let help = Paragraph::new(help_text).block(Block::default().borders(Borders::NONE));
        help.render(chunks[3], buf);
    }
}

impl Interactive for SampleForm {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        // 根据当前焦点决定处理方式
        if self.username_input.can_focus() {
            self.username_input.handle_key(key)
        } else if self.description_area.can_focus() {
            self.description_area.handle_key(key)
        } else if self.role_select.can_focus() {
            self.role_select.handle_key(key)
        } else {
            HandleResult::Ignored
        }
    }

    fn handle_mouse(&mut self, _event: MouseEvent) -> HandleResult {
        // Handle mouse events
        HandleResult::Ignored
    }
}

impl Component for SampleForm {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_form_creation() {
        let form = SampleForm::new();
        assert_eq!(form.id().value(), 0);
        assert!(form.can_focus());
    }

    #[test]
    fn test_sample_form_getters() {
        let form = SampleForm::new();
        assert_eq!(form.username(), "");
        assert_eq!(form.description(), "");
        assert_eq!(form.role(), None);
    }
}
