//! Select 组件
//!
//! 下拉选择框组件，支持选项选择和导航。

use crate::tui::error::TuiResult;
use crate::tui::traits::{
    Component, ComponentId, FieldValidation, HandleResult, Interactive, Render, ValidationResult,
    ValidationTrigger,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// 选择项
#[derive(Debug, Clone)]
pub struct SelectItem {
    /// 显示文本
    pub label: String,
    /// 值
    pub value: String,
    /// 是否禁用
    pub disabled: bool,
}

impl SelectItem {
    /// 创建新的选择项
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            disabled: false,
        }
    }

    /// 设置禁用状态
    #[must_use]
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 选择组件（下拉框）
///
/// 支持功能：
/// - 选项选择
/// - 键盘导航
/// - 展开/收起
/// - 焦点状态
/// - 验证功能
pub struct Select {
    /// 组件 ID
    id: ComponentId,
    /// 选项列表
    items: Vec<SelectItem>,
    /// 当前选中索引
    selected_index: Option<usize>,
    /// 是否展开
    expanded: bool,
    /// 是否有焦点
    focused: bool,
    /// 验证配置
    validation: Option<FieldValidation>,
    /// 验证结果
    validation_result: Option<ValidationResult>,
    /// 边框标题
    title: String,
}

impl Select {
    /// 创建新的选择组件
    pub fn new(items: Vec<SelectItem>) -> Self {
        Self {
            id: ComponentId::new(0),
            items,
            selected_index: None,
            expanded: false,
            focused: false,
            validation: None,
            validation_result: Some(ValidationResult::valid()),
            title: String::new(),
        }
    }

    /// 设置组件 ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// 设置边框标题
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// 设置验证配置
    #[must_use]
    pub fn with_validation(mut self, validation: FieldValidation) -> Self {
        self.validation = Some(validation);
        self
    }

    /// 获取选中的值
    pub fn selected_value(&self) -> Option<&str> {
        if let Some(index) = self.selected_index {
            if index < self.items.len() {
                return Some(&self.items[index].value);
            }
        }
        None
    }

    /// 获取选中的标签
    pub fn selected_label(&self) -> Option<&str> {
        if let Some(index) = self.selected_index {
            if index < self.items.len() {
                return Some(&self.items[index].label);
            }
        }
        None
    }

    /// 设置选中项（通过索引）
    pub fn set_selected_by_index(&mut self, index: usize) {
        if index < self.items.len() && !self.items[index].disabled {
            self.selected_index = Some(index);
        }
    }

    /// 设置选中项（通过值）
    pub fn set_selected_by_value(&mut self, value: &str) {
        if let Some(index) = self.items.iter().position(|item| item.value == value) {
            if !self.items[index].disabled {
                self.selected_index = Some(index);
            }
        }
    }

    /// 移动选中项向上
    fn move_up(&mut self) {
        if let Some(current) = self.selected_index {
            let mut new_index = current;
            // 查找下一个非禁用的项
            loop {
                if new_index == 0 {
                    break;
                }
                new_index -= 1;
                if !self.items[new_index].disabled {
                    self.selected_index = Some(new_index);
                    break;
                }
            }
        } else if !self.items.is_empty() {
            // 如果没有选中项，选中最后一个非禁用的项
            for i in (0..self.items.len()).rev() {
                if !self.items[i].disabled {
                    self.selected_index = Some(i);
                    break;
                }
            }
        }
    }

    /// 移动选中项向下
    fn move_down(&mut self) {
        if let Some(current) = self.selected_index {
            let mut new_index = current;
            // 查找下一个非禁用的项
            while new_index < self.items.len() - 1 {
                new_index += 1;
                if !self.items[new_index].disabled {
                    self.selected_index = Some(new_index);
                    break;
                }
            }
        } else if !self.items.is_empty() {
            // 如果没有选中项，选中第一个非禁用的项
            for (i, item) in self.items.iter().enumerate() {
                if !item.disabled {
                    self.selected_index = Some(i);
                    break;
                }
            }
        }
    }

    /// 验证当前值
    fn validate(&self) -> ValidationResult {
        if let Some(ref validation) = self.validation {
            let value = self.selected_value().unwrap_or("");
            validation.validate(value)
        } else {
            ValidationResult::valid()
        }
    }

    /// 获取选中的选项文本
    fn selected_text(&self) -> String {
        if let Some(index) = self.selected_index {
            if index < self.items.len() {
                return self.items[index].label.clone();
            }
        }

        // 如果没有选中项，显示提示文字
        if self.items.is_empty() {
            "无选项".to_string()
        } else {
            "请选择...".to_string()
        }
    }
}

impl Render for Select {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        // 创建边框
        let block_style = if self.focused {
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Black)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_style)
            .title(self.title.as_str());

        // 渲染主选择框
        let inner_area = block.inner(area);
        block.render(area, buf);

        // 渲染选中的文本
        let selected_text = self.selected_text();
        let paragraph = Paragraph::new(selected_text).style(if self.focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        });
        paragraph.render(inner_area, buf);

        // 如果展开且有焦点，渲染选项列表
        if self.expanded && self.focused && !self.items.is_empty() {
            // 计算选项列表的区域
            let list_height = (self.items.len().min(8)) as u16; // 最多显示8个项目
            let list_area = Rect {
                x: area.x,
                y: area.y + area.height,
                width: area.width,
                height: list_height,
            };

            // 准备列表项
            let items: Vec<ListItem> = self
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let mut style = if Some(i) == self.selected_index {
                        Style::default().fg(Color::Yellow).bg(Color::Blue)
                    } else if item.disabled {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    if item.disabled {
                        style = style.add_modifier(Modifier::DIM);
                    }

                    let content = Line::from(item.label.as_str());
                    ListItem::new(content).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("选项"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                );

            list.render(list_area, buf);
        }

        // 渲染验证错误
        if let Some(ref result) = self.validation_result {
            if result.has_errors() {
                let error_line = Line::from(result.first_error().unwrap_or("验证错误"));
                let error_area = Rect {
                    x: area.x,
                    y: area.y + area.height + (if self.expanded { 8 } else { 0 }),
                    width: area.width,
                    height: 1,
                };

                let error_paragraph =
                    Paragraph::new(error_line).style(Style::default().fg(Color::Red));
                error_paragraph.render(error_area, buf);
            }
        }
    }
}

impl Interactive for Select {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        // 只处理按下事件
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                // 展开/收起下拉列表
                self.expanded = !self.expanded;
                HandleResult::Consumed
            }
            KeyCode::Up => {
                if self.expanded {
                    self.move_up();
                } else {
                    // 如果未展开，在收起状态下也可以导航
                    self.move_up();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if self.expanded {
                    self.move_down();
                } else {
                    // 如果未展开，在收起状态下也可以导航
                    self.move_down();
                }
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                // 收起下拉列表
                self.expanded = false;
                HandleResult::Consumed
            }
            KeyCode::Tab => {
                // 收起下拉列表并让父组件处理焦点转移
                self.expanded = false;
                HandleResult::Ignored
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for Select {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        self.focused = true;
        // 验证当前值
        if self.validation.is_some() {
            self.validation_result = Some(self.validate());
        }
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        self.focused = false;
        self.expanded = false;
        // 失焦时验证，如果配置了 onBlur
        if let Some(ref validation) = self.validation {
            if matches!(validation.trigger, ValidationTrigger::OnBlur) {
                self.validation_result = Some(self.validate());
            }
        }
        Ok(())
    }

    fn on_mount(&mut self) -> TuiResult<()> {
        // 初始验证
        if self.validation.is_some() {
            self.validation_result = Some(self.validate());
        }
        Ok(())
    }
}

impl Default for Select {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::traits::{BuiltinValidator, ValidationTrigger};

    #[test]
    fn test_select_creation() {
        let items = vec![
            SelectItem::new("选项1", "value1"),
            SelectItem::new("选项2", "value2"),
        ];
        let select = Select::new(items);
        assert_eq!(select.items.len(), 2);
    }

    #[test]
    fn test_select_selection_by_index() {
        let items = vec![
            SelectItem::new("选项1", "value1"),
            SelectItem::new("选项2", "value2"),
        ];
        let mut select = Select::new(items);

        select.set_selected_by_index(0);
        assert_eq!(select.selected_value(), Some("value1"));
        assert_eq!(select.selected_label(), Some("选项1"));
    }

    #[test]
    fn test_select_selection_by_value() {
        let items = vec![
            SelectItem::new("选项1", "value1"),
            SelectItem::new("选项2", "value2"),
        ];
        let mut select = Select::new(items);

        select.set_selected_by_value("value2");
        assert_eq!(select.selected_value(), Some("value2"));
        assert_eq!(select.selected_label(), Some("选项2"));
    }

    #[test]
    fn test_select_navigation() {
        let items = vec![
            SelectItem::new("选项1", "value1"),
            SelectItem::new("选项2", "value2"),
            SelectItem::new("选项3", "value3"),
        ];
        let mut select = Select::new(items);

        // 初始没有选中项
        assert_eq!(select.selected_index, None);

        // 向下移动，应该选中第一项
        select.move_down();
        assert_eq!(select.selected_index, Some(0));

        // 再次向下移动，应该选中第二项
        select.move_down();
        assert_eq!(select.selected_index, Some(1));

        // 向上移动，应该回到第一项
        select.move_up();
        assert_eq!(select.selected_index, Some(0));
    }

    #[test]
    fn test_select_with_disabled_item() {
        let items = vec![
            SelectItem::new("选项1", "value1"),
            SelectItem::new("选项2", "value2").with_disabled(true),
            SelectItem::new("选项3", "value3"),
        ];
        let mut select = Select::new(items);

        // 移动应该跳过禁用的项
        select.move_down(); // 会选中选项1 (index 0)
        assert_eq!(select.selected_index, Some(0));

        select.move_down(); // 会跳过禁用的选项2，选中选项3 (index 2)
        assert_eq!(select.selected_index, Some(2));
    }

    #[test]
    fn test_select_expansion() {
        let items = vec![
            SelectItem::new("选项1", "value1"),
            SelectItem::new("选项2", "value2"),
        ];
        let mut select = Select::new(items);

        assert!(!select.expanded);
        select.handle_key(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert!(select.expanded);
    }

    #[test]
    fn test_select_validation() {
        let items = vec![
            SelectItem::new("选项1", "value1"),
            SelectItem::new("选项2", "value2"),
        ];
        let validation = FieldValidation::new()
            .with_validator(BuiltinValidator::Required)
            .with_trigger(ValidationTrigger::OnBlur);

        let mut select = Select::new(items).with_validation(validation);

        // Set selection first
        select.set_selected_by_index(0);
        assert_eq!(select.selected_value(), Some("value1"));

        // Test validation with selection (should pass)
        let result = select.validate();
        assert!(result.is_valid);

        // Change selection to none manually to test required validation
        select.selected_index = None;

        // Test validation without selection (should fail because of Required validator)
        let result = select.validate();
        assert!(!result.is_valid);
    }
}
