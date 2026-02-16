//! 默认状态管理器实现
//!
//! 占位符模块，完整实现将在 Task C.2 中完成。

use crate::tui::traits::StateManager;
use std::collections::HashMap;

/// 默认状态管理器
#[derive(Debug, Default)]
pub struct DefaultStateManager {
    _states: HashMap<String, String>,
}

impl DefaultStateManager {
    /// 创建新的状态管理器
    #[must_use]
    pub fn new() -> Self {
        Self {
            _states: HashMap::new(),
        }
    }
}

impl StateManager for DefaultStateManager {
    fn get_state(&self, key: &str) -> Option<String> {
        self._states.get(key).cloned()
    }

    fn set_state(&mut self, key: &str, value: String) {
        self._states.insert(key.to_string(), value);
    }
}
