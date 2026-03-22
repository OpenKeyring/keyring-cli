//! 默认屏幕管理器实现
//!
//! 占位符模块，完整实现将在 Task C.3 中完成。

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::traits::{Screen, ScreenManager, ScreenType};
use std::collections::VecDeque;

/// 默认屏幕管理器
#[derive(Default)]
pub struct DefaultScreenManager {
    stack: VecDeque<Box<dyn Screen>>,
}

impl std::fmt::Debug for DefaultScreenManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultScreenManager")
            .field("stack_size", &self.stack.len())
            .finish()
    }
}

impl DefaultScreenManager {
    /// 创建新的屏幕管理器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }
}

impl ScreenManager for DefaultScreenManager {
    fn push(&mut self, screen: Box<dyn Screen>) -> TuiResult<()> {
        self.stack.push_back(screen);
        Ok(())
    }

    fn pop(&mut self) -> TuiResult<Option<Box<dyn Screen>>> {
        Ok(self.stack.pop_back())
    }

    fn replace(&mut self, screen: Box<dyn Screen>) -> TuiResult<()> {
        self.stack.pop_back();
        self.stack.push_back(screen);
        Ok(())
    }

    fn current(&self) -> Option<&dyn Screen> {
        self.stack.back().map(|s| s.as_ref())
    }

    fn current_mut(&mut self) -> Option<&mut (dyn Screen + '_)> {
        if self.stack.is_empty() {
            None
        } else {
            // 获取最后一个元素的可变引用
            let len = self.stack.len();
            // SAFETY: len > 0 已经由 is_empty() 检查保证
            // 直接通过索引获取元素
            Some(self.stack[len - 1].as_mut())
        }
    }

    fn has_active_screen(&self) -> bool {
        !self.stack.is_empty()
    }

    fn clear(&mut self) -> TuiResult<()> {
        self.stack.clear();
        Ok(())
    }

    fn depth(&self) -> usize {
        self.stack.len()
    }

    fn navigate_to(&mut self, _screen_type: ScreenType) -> TuiResult<()> {
        // 占位符实现
        Err(TuiError::invalid_state("导航功能尚未实现"))
    }
}
