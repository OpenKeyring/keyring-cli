//! 默认状态管理器实现
//!
//! 占位符模块，完整实现将在 Task C.2 中完成。

use crate::tui::traits::{StateValue, StateManager, StateError};
use std::collections::HashMap;

/// 默认状态管理器
#[derive(Debug, Default)]
pub struct DefaultStateManager {
    _states: HashMap<String, StateValue>,
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
    fn get(&self, key: &str) -> Option<&StateValue> {
        self._states.get(key)
    }

    fn set(&mut self, key: &str, value: StateValue) -> Result<(), StateError> {
        self._states.insert(key.to_string(), value);
        Ok(())
    }

    fn remove(&mut self, key: &str) -> Option<StateValue> {
        self._states.remove(key)
    }

    fn keys(&self) -> Vec<String> {
        self._states.keys().cloned().collect()
    }

    fn clear(&mut self) {
        self._states.clear();
    }
}
