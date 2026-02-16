//! 默认屏幕管理器实现
//!
//! 占位符模块，完整实现将在 Task C.3 中完成。

use crate::tui::traits::{ComponentId, ScreenManager, Screen};
use std::collections::VecDeque;

/// 默认屏幕管理器
#[derive(Default)]
pub struct DefaultScreenManager {
    _stack: VecDeque<Box<dyn Screen>>,
    _next_id: ComponentId,
}

impl std::fmt::Debug for DefaultScreenManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultScreenManager")
            .field("stack_size", &self._stack.len())
            .field("next_id", &self._next_id)
            .finish()
    }
}

impl DefaultScreenManager {
    /// 创建新的屏幕管理器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _stack: VecDeque::new(),
            _next_id: ComponentId(0),
        }
    }
}

impl ScreenManager for DefaultScreenManager {
    fn push(&mut self, screen: Box<dyn Screen>) -> ComponentId {
        let id = self._next_id;
        self._next_id = ComponentId(id.0 + 1);
        self._stack.push_back(screen);
        id
    }

    fn pop(&mut self) -> Option<Box<dyn Screen>> {
        self._stack.pop_back()
    }

    fn current(&self) -> Option<&dyn Screen> {
        self._stack.back().map(|s| s.as_ref())
    }
}
