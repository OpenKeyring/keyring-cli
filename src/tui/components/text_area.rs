//! TextArea 组件
//!
//! 多行文本输入框组件，支持滚动和编辑操作。

use crate::tui::error::TuiResult;
use crate::tui::traits::{
    Component, ComponentId, HandleResult, Interactive, Render, ValidationTrigger,
    ValidationResult, FieldValidation
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    prelude::Widget,
};
use std::cmp;

/// 文本区域组件
///
/// 支持功能：
/// - 多行文本输入和编辑
/// - 垂直滚动
/// - 光标移动
/// - 占位符显示
/// - 焦点状态样式
/// - 验证功能
pub struct TextArea {
    /// 组件 ID
    id: ComponentId,
    /// 当前行（多行文本）
    lines: Vec<String>,
    /// 光标位置（行，列）
    cursor_row: usize,
    /// 光标列
    cursor_col: usize,
    /// 垂直滚动偏移
    scroll_offset: usize,
    /// 最大行数限制
    max_lines: Option<usize>,
    /// 最大长度限制
    max_length: Option<usize>,
    /// 占位符文本
    placeholder: String,
    /// 是否有焦点
    focused: bool,
    /// 验证配置
    validation: Option<FieldValidation>,
    /// 验证结果
    validation_result: Option<ValidationResult>,
    /// 边框标题
    title: String,
}

impl TextArea {
    /// 创建新的文本区域
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            id: ComponentId::new(0),
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            max_lines: None,
            max_length: None,
            placeholder: placeholder.into(),
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

    /// 设置最大行数
    #[must_use]
    pub fn with_max_lines(mut self, max: usize) -> Self {
        self.max_lines = Some(max);
        self
    }

    /// 设置最大长度
    #[must_use]
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// 设置验证配置
    #[must_use]
    pub fn with_validation(mut self, validation: FieldValidation) -> Self {
        self.validation = Some(validation);
        self
    }

    /// 获取所有文本内容
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// 设置文本内容
    pub fn set_text(&mut self, text: String) {
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        // 将光标移到末尾
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.lines[self.cursor_row].len();

        // 确保不会超过最大行数限制
        if let Some(max_lines) = self.max_lines {
            if self.lines.len() > max_lines {
                self.lines.truncate(max_lines);
                if self.cursor_row >= max_lines {
                    self.cursor_row = max_lines - 1;
                    self.cursor_col = self.lines[self.cursor_row].len();
                }
            }
        }
    }

    /// 获取占位符
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// 清除文本
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    /// 获取当前行文本
    fn current_line(&self) -> &str {
        if self.cursor_row < self.lines.len() {
            &self.lines[self.cursor_row]
        } else {
            ""
        }
    }

    /// 获取当前行文本的可变引用
    fn current_line_mut(&mut self) -> &mut String {
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }
        &mut self.lines[self.cursor_row]
    }

    /// 插入字符
    fn insert_char(&mut self, ch: char) {
        // 检查总长度限制
        if let Some(max_length) = self.max_length {
            let total_len = self.text().len() + ch.len_utf8();
            if total_len > max_length {
                return;
            }
        }

        // Ensure cursor_row is valid
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }

        let line = &mut self.lines[self.cursor_row];
        if self.cursor_col <= line.len() {
            line.insert(self.cursor_col, ch);
            self.cursor_col += ch.len_utf8();
        }
    }

    /// 插入换行符
    fn insert_newline(&mut self) {
        if let Some(max_lines) = self.max_lines {
            if self.lines.len() >= max_lines {
                return; // 达到最大行数限制
            }
        }

        // Ensure cursor_row is valid
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }

        let current_line = &mut self.lines[self.cursor_row];
        let new_part = current_line.split_off(self.cursor_col);
        self.lines.insert(self.cursor_row + 1, new_part);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    /// 删除光标前的字符
    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            // 删除当前行中的字符
            // Ensure cursor_row is valid
            if self.cursor_row >= self.lines.len() {
                self.lines.resize(self.cursor_row + 1, String::new());
            }

            let line = &mut self.lines[self.cursor_row];
            if self.cursor_col <= line.len() {
                // 找到前一个字符的位置
                let prev_col = line[..self.cursor_col]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                line.remove(prev_col);
                self.cursor_col = prev_col;
            }
        } else if self.cursor_row > 0 {
            // 光标在行首，合并到上一行
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
        }
    }

    /// 删除光标后的字符
    fn delete(&mut self) {
        // Ensure cursor_row is valid
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }

        let current_line = &self.lines[self.cursor_row];
        if self.cursor_col < current_line.len() {
            // 删除当前行的字符
            let line = &mut self.lines[self.cursor_row];
            if self.cursor_col < line.len() {
                let next_col = line[self.cursor_col..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor_col + i)
                    .unwrap_or(line.len());

                line.remove(self.cursor_col);
            }
        } else if self.cursor_row < self.lines.len() - 1 {
            // 光标在行尾，合并下一行
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
    }

    /// 移动光标到上方行
    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // 保持列位置，但不超过新行的长度
            self.cursor_col = std::cmp::min(self.cursor_col, self.lines[self.cursor_row].len());
        }
    }

    /// 移动光标到下方行
    fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            // 保持列位置，但不超过新行的长度
            self.cursor_col = std::cmp::min(self.cursor_col, self.lines[self.cursor_row].len());
        }
    }

    /// 移动光标到左侧
    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            // 找到前一个字符的位置
            // Ensure cursor_row is valid
            if self.cursor_row >= self.lines.len() {
                self.lines.resize(self.cursor_row + 1, String::new());
            }

            let line = &self.lines[self.cursor_row];
            let prev_col = line[..self.cursor_col]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.cursor_col = prev_col;
        } else if self.cursor_row > 0 {
            // 移动到上一行的末尾
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// 移动光标到右侧
    fn move_right(&mut self) {
        // Ensure cursor_row is valid
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }

        let line = &self.lines[self.cursor_row];
        if self.cursor_col < line.len() {
            // 移动到下一个字符位置
            let next_col = line[self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_col + i)
                .unwrap_or(line.len());
            self.cursor_col = next_col;
        } else if self.cursor_row < self.lines.len() - 1 {
            // 移动到下一行的开头
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// 移动光标到行首
    fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// 移动光标到行尾
    fn move_end(&mut self) {
        // Ensure cursor_row is valid
        if self.cursor_row >= self.lines.len() {
            self.lines.resize(self.cursor_row + 1, String::new());
        }

        self.cursor_col = self.lines[self.cursor_row].len();
    }

    /// 移动光标到文档开始
    fn move_document_start(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    /// 移动光标到文档末尾
    fn move_document_end(&mut self) {
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.lines[self.cursor_row].len();

        // 调整滚动位置以显示最后一行
        // 注意：这里需要知道可用高度，我们将在渲染时处理
    }

    /// 验证当前文本
    fn validate(&self) -> ValidationResult {
        if let Some(ref validation) = self.validation {
            validation.validate(&self.text())
        } else {
            ValidationResult::valid()
        }
    }

    /// 更新滚动偏移以使光标可见
    fn update_scroll_if_needed(&mut self, area: Rect) {
        // 计算光标相对于显示区域的行位置
        let visible_start_row = self.scroll_offset;
        let visible_end_row = self.scroll_offset + area.height as usize - 2; // -2 for borders

        // 调整滚动偏移，使光标行始终可见
        if self.cursor_row < visible_start_row {
            self.scroll_offset = self.cursor_row;
        } else if self.cursor_row >= visible_end_row {
            self.scroll_offset = self.cursor_row - (area.height as usize - 3); // -3 to keep cursor in view
        }
    }
}

impl Render for TextArea {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        // 创建边框
        let block_style = if self.focused {
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Gray)
                .bg(Color::Black)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_style)
            .title(self.title.as_str());

        let inner_area = block.inner(area);
        block.render(area, buf);

        // 准备显示的文本
        let display_lines = if self.lines.is_empty() || (self.lines.len() == 1 && self.lines[0].is_empty()) {
            if self.placeholder.is_empty() {
                vec![String::new()]
            } else {
                vec![self.placeholder.clone()]
            }
        } else {
            self.lines.clone()
        };

        // 截取需要显示的部分
        let start_idx = self.scroll_offset;
        let end_idx = cmp::min(start_idx + inner_area.height as usize, display_lines.len());
        let visible_lines: Vec<&str> = display_lines[start_idx..end_idx]
            .iter()
            .map(|s| s.as_str())
            .collect();

        // 创建带有光标标记的文本
        let spans: Vec<Line> = visible_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let line_idx = start_idx + idx;
                if self.focused && line_idx == self.cursor_row {
                    // 当前行，需要标记光标位置
                    let mut spans = Vec::new();

                    // 前面的文本
                    if self.cursor_col > 0 {
                        let before_cursor = &line[..self.cursor_col.min(line.len())];
                        spans.push(Span::raw(before_cursor));
                    }

                    // 光标字符或下一个字符
                    if self.cursor_col < line.len() {
                        let cursor_char = &line[self.cursor_col..self.cursor_col + line[self.cursor_col..].chars().next().map(|c| c.len_utf8()).unwrap_or(1)];
                        spans.push(Span::styled(
                            cursor_char,
                            Style::default().add_modifier(Modifier::REVERSED)
                        ));

                        // 后面的文本
                        if self.cursor_col + cursor_char.len() < line.len() {
                            spans.push(Span::raw(&line[self.cursor_col + cursor_char.len()..]));
                        }
                    } else {
                        // 在行尾插入一个特殊光标
                        spans.push(Span::styled(" ", Style::default().add_modifier(Modifier::REVERSED)));
                    }

                    Line::from(spans)
                } else {
                    // 非当前行，直接显示
                    Line::from(*line)
                }
            })
            .collect();

        let text_widget = Paragraph::new(spans)
            .wrap(Wrap { trim: false })
            .scroll((0, 0)); // 使用自定义滚动逻辑

        text_widget.render(inner_area, buf);

        // 渲染验证错误
        if let Some(ref result) = self.validation_result {
            if result.has_errors() {
                let error_line = Line::from(result.first_error().unwrap_or("验证错误"));
                let error_area = Rect {
                    x: area.x,
                    y: area.y + area.height,
                    width: area.width,
                    height: 1,
                };

                let error_paragraph = Paragraph::new(error_line)
                    .style(Style::default().fg(Color::Red));
                error_paragraph.render(error_area, buf);
            }
        }
    }
}

impl Interactive for TextArea {
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
            KeyCode::Enter => {
                self.insert_newline();
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
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+左箭头：移动到行首
                    self.move_home();
                } else {
                    self.move_left();
                }
                HandleResult::Consumed
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+右箭头：移动到行尾
                    self.move_end();
                } else {
                    self.move_right();
                }
                HandleResult::Consumed
            }
            KeyCode::Up => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+上箭头：向上滚动（如果需要）
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                } else {
                    self.move_up();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+下箭头：向下滚动（如果需要）
                    let max_scroll = self.lines.len().saturating_sub(10); // 10是假设的可见行数
                    if self.scroll_offset < max_scroll {
                        self.scroll_offset += 1;
                    }
                } else {
                    self.move_down();
                }
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
            KeyCode::PageUp => {
                // 向上翻页
                let page_size = (self.lines.len() / 2).max(1);
                self.cursor_row = self.cursor_row.saturating_sub(page_size);
                self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
                self.cursor_col = 0;
                HandleResult::Consumed
            }
            KeyCode::PageDown => {
                // 向下翻页
                let page_size = (self.lines.len() / 2).max(1);
                self.cursor_row = cmp::min(self.cursor_row + page_size, self.lines.len() - 1);
                self.cursor_col = 0;
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for TextArea {
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

    fn before_render(&mut self) -> TuiResult<()> {
        // 在渲染之前更新滚动位置
        // 由于我们无法在这里获取 area，这个逻辑会在 render 中执行
        Ok(())
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::traits::{ValidationTrigger, BuiltinValidator};

    #[test]
    fn test_text_area_creation() {
        let textarea = TextArea::new("placeholder");
        assert_eq!(textarea.placeholder(), "placeholder");
        assert_eq!(textarea.text(), "");
    }

    #[test]
    fn test_text_area_set_text() {
        let mut textarea = TextArea::new("placeholder");
        textarea.set_text("Hello\nWorld\nTest".to_string());

        assert_eq!(textarea.text(), "Hello\nWorld\nTest");
        assert_eq!(textarea.lines.len(), 3);
    }

    #[test]
    fn test_text_area_insert_char() {
        let mut textarea = TextArea::new("placeholder");
        textarea.insert_char('H');
        textarea.insert_char('i');

        assert_eq!(textarea.text(), "Hi");
    }

    #[test]
    fn test_text_area_newline() {
        let mut textarea = TextArea::new("placeholder");
        textarea.insert_char('H');
        textarea.insert_char('i');
        textarea.insert_newline();
        textarea.insert_char('Y');
        textarea.insert_char('o');
        textarea.insert_char('u');

        assert_eq!(textarea.text(), "Hi\nYou");
    }

    #[test]
    fn test_text_area_cursor_movement() {
        let mut textarea = TextArea::new("placeholder");
        textarea.set_text("Line1\nLine2\nLine3".to_string());

        // Start at end of last line
        assert_eq!(textarea.cursor_row, 2);
        assert_eq!(textarea.cursor_col, 5); // length of "Line3"

        // Move up
        textarea.move_up();
        assert_eq!(textarea.cursor_row, 1);
        assert_eq!(textarea.cursor_col, 5); // Should keep column position

        // Move down
        textarea.move_down();
        assert_eq!(textarea.cursor_row, 2);
    }

    #[test]
    fn test_text_area_backspace() {
        let mut textarea = TextArea::new("placeholder");
        textarea.set_text("Hello World".to_string());
        textarea.cursor_col = 6; // Position at "W"

        textarea.backspace(); // Remove space

        assert_eq!(textarea.text(), "HelloWorld");
    }

    #[test]
    fn test_text_area_clear() {
        let mut textarea = TextArea::new("placeholder");
        textarea.set_text("Some text\nMore text".to_string());

        textarea.clear();

        assert_eq!(textarea.text(), "");
        assert_eq!(textarea.lines.len(), 1);
        assert_eq!(textarea.lines[0], "");
    }

    #[test]
    fn test_text_area_validation() {
        let validation = FieldValidation::new()
            .with_validator(BuiltinValidator::Required)
            .with_trigger(ValidationTrigger::OnBlur);

        let mut textarea = TextArea::new("placeholder")
            .with_validation(validation);

        textarea.set_text("".to_string()); // Empty text should fail validation
        let result = textarea.validate();

        assert!(!result.is_valid);
        assert!(result.has_errors());
    }

    #[test]
    fn test_text_area_max_lines() {
        let mut textarea = TextArea::new("placeholder")
            .with_max_lines(2);

        textarea.insert_char('A');
        textarea.insert_newline();
        textarea.insert_char('B');
        textarea.insert_newline(); // This should exceed max lines
        textarea.insert_char('C');

        // Get the text first, then check
        let text = textarea.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines.len() <= 2);
    }
}