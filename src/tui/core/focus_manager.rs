//! 默认焦点管理器实现
//!
//! 占位符模块，完整实现将在 Task C.1 中完成。

use crate::tui::traits::{ComponentId, FocusManager, FocusState};

/// 默认焦点管理器
#[derive(Debug, Default)]
pub struct DefaultFocusManager {
    _current: Option<ComponentId>,
    _state: FocusState,
}

impl DefaultFocusManager {
    /// 创建新的焦点管理器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _current: None,
            _state: FocusState::None,
        }
    }
}

impl FocusManager for DefaultFocusManager {
    fn set_focus(&mut self, id: ComponentId) -> bool {
        self._current = Some(id);
        self._state = FocusState::Focused;
        true
    }

    fn current_focus(&self) -> Option<ComponentId> {
        self._current
    }
}
