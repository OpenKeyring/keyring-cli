//! 默认状态管理器实现
//!
//! 实现状态管理器和响应式状态。

use crate::tui::traits::{
    ReactiveState, StateCallback, StateChange, StateError, StateManager, StateValue,
    SubscriptionId, SubscriptionIdGenerator,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 默认状态管理器
///
/// 提供基础的状态存储和检索功能。
#[derive(Debug, Default)]
pub struct DefaultStateManager {
    states: HashMap<String, StateValue>,
}

impl DefaultStateManager {
    /// 创建新的状态管理器
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }
}

impl StateManager for DefaultStateManager {
    fn get(&self, key: &str) -> Option<&StateValue> {
        self.states.get(key)
    }

    fn set(&mut self, key: &str, value: StateValue) -> Result<(), StateError> {
        self.states.insert(key.to_string(), value);
        Ok(())
    }

    fn remove(&mut self, key: &str) -> Option<StateValue> {
        self.states.remove(key)
    }

    fn keys(&self) -> Vec<String> {
        self.states.keys().cloned().collect()
    }

    fn clear(&mut self) {
        self.states.clear();
    }
}

/// 响应式状态管理器
///
/// 扩展默认状态管理器，支持状态变化订阅和通知。
pub struct ReactiveStateManager {
    /// 内部状态
    states: HashMap<String, StateValue>,
    /// 状态变化历史
    history: HashMap<String, Vec<StateChange>>,
    /// 订阅者（按键分组）
    subscribers: HashMap<String, Vec<(SubscriptionId, StateCallback)>>,
    /// 全局订阅者
    global_subscribers: Vec<(SubscriptionId, StateCallback)>,
    /// ID 生成器
    id_generator: SubscriptionIdGenerator,
    /// 最大历史长度
    max_history: usize,
    /// 撤销栈
    undo_stack: HashMap<String, Vec<StateValue>>,
    /// 重做栈
    redo_stack: HashMap<String, Vec<StateValue>>,
}

impl std::fmt::Debug for ReactiveStateManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactiveStateManager")
            .field("state_count", &self.states.len())
            .field("subscriber_count", &self.subscribers.len())
            .field("global_subscriber_count", &self.global_subscribers.len())
            .field("max_history", &self.max_history)
            .finish()
    }
}

impl Default for ReactiveStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactiveStateManager {
    /// 创建新的响应式状态管理器
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            history: HashMap::new(),
            subscribers: HashMap::new(),
            global_subscribers: Vec::new(),
            id_generator: SubscriptionIdGenerator::new(),
            max_history: 100,
            undo_stack: HashMap::new(),
            redo_stack: HashMap::new(),
        }
    }

    /// 创建带指定历史长度的管理器
    #[must_use]
    pub fn with_history(max_history: usize) -> Self {
        Self {
            max_history,
            ..Self::new()
        }
    }

    /// 获取状态快照
    #[must_use]
    pub fn snapshot(&self) -> HashMap<String, StateValue> {
        self.states.clone()
    }

    /// 从快照恢复状态
    pub fn restore(&mut self, snapshot: HashMap<String, StateValue>) -> Result<(), StateError> {
        self.states = snapshot;
        Ok(())
    }

    /// 触发状态变化通知
    fn notify(&self, key: &str, change: &StateChange) {
        // 通知特定键的订阅者
        if let Some(subs) = self.subscribers.get(key) {
            for (_, callback) in subs {
                callback(change);
            }
        }

        // 通知全局订阅者
        for (_, callback) in &self.global_subscribers {
            callback(change);
        }
    }

    /// 记录状态变化
    fn record_change(&mut self, key: &str, change: StateChange) {
        let history = self.history.entry(key.to_string()).or_default();
        history.push(change);

        // 限制历史长度
        if history.len() > self.max_history {
            history.remove(0);
        }

        // 清空重做栈（新操作会清除重做历史）
        self.redo_stack.remove(key);
    }
}

impl StateManager for ReactiveStateManager {
    fn get(&self, key: &str) -> Option<&StateValue> {
        self.states.get(key)
    }

    fn set(&mut self, key: &str, value: StateValue) -> Result<(), StateError> {
        let old_value = self.states.get(key).cloned();

        // 保存到撤销栈
        if let Some(ref old) = old_value {
            self.undo_stack
                .entry(key.to_string())
                .or_default()
                .push(old.clone());
        }

        let change = StateChange::new(old_value, value.clone(), None);
        self.states.insert(key.to_string(), value);
        self.record_change(key, change.clone());
        self.notify(key, &change);

        Ok(())
    }

    fn remove(&mut self, key: &str) -> Option<StateValue> {
        let value = self.states.remove(key);
        if value.is_some() {
            self.history.remove(key);
            self.subscribers.remove(key);
        }
        value
    }

    fn keys(&self) -> Vec<String> {
        self.states.keys().cloned().collect()
    }

    fn clear(&mut self) {
        self.states.clear();
        self.history.clear();
        self.subscribers.clear();
        self.global_subscribers.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl ReactiveState for ReactiveStateManager {
    fn subscribe(&mut self, key: String, callback: StateCallback) -> SubscriptionId {
        let id = self.id_generator.generate();
        self.subscribers
            .entry(key)
            .or_default()
            .push((id, callback));
        id
    }

    fn unsubscribe(&mut self, id: SubscriptionId) -> bool {
        // 检查特定键的订阅者
        for subs in self.subscribers.values_mut() {
            if let Some(pos) = subs.iter().position(|(sub_id, _)| *sub_id == id) {
                let _ = subs.remove(pos);
                return true;
            }
        }

        // 检查全局订阅者
        if let Some(pos) = self
            .global_subscribers
            .iter()
            .position(|(sub_id, _)| *sub_id == id)
        {
            let _ = self.global_subscribers.remove(pos);
            return true;
        }

        false
    }

    fn subscribe_all(&mut self, callback: StateCallback) -> SubscriptionId {
        let id = self.id_generator.generate();
        self.global_subscribers.push((id, callback));
        id
    }

    fn history(&self, key: &str) -> Vec<&StateChange> {
        self.history
            .get(key)
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    fn undo(&mut self, key: &str) -> Result<(), StateError> {
        let stack = self
            .undo_stack
            .get_mut(key)
            .ok_or_else(|| StateError::UndoFailed(format!("No undo history for key: {}", key)))?;

        let old_value = stack
            .pop()
            .ok_or_else(|| StateError::UndoFailed(format!("Undo stack empty for key: {}", key)))?;

        // 保存当前值到重做栈
        let current = self.states.get(key).cloned();
        if let Some(ref curr) = current {
            self.redo_stack
                .entry(key.to_string())
                .or_default()
                .push(curr.clone());
        }

        let change = StateChange::new(
            Some(old_value.clone()),
            old_value.clone(),
            Some("undo".to_string()),
        );
        self.states.insert(key.to_string(), old_value);

        // 直接记录历史但不调用 record_change（避免清空重做栈）
        let history = self.history.entry(key.to_string()).or_default();
        history.push(change.clone());
        if history.len() > self.max_history {
            history.remove(0);
        }

        self.notify(key, &change);

        Ok(())
    }

    fn redo(&mut self, key: &str) -> Result<(), StateError> {
        let stack = self
            .redo_stack
            .get_mut(key)
            .ok_or_else(|| StateError::RedoFailed(format!("No redo history for key: {}", key)))?;

        let new_value = stack
            .pop()
            .ok_or_else(|| StateError::RedoFailed(format!("Redo stack empty for key: {}", key)))?;

        // 保存当前值到撤销栈
        let current = self.states.get(key).cloned();
        if let Some(ref curr) = current {
            self.undo_stack
                .entry(key.to_string())
                .or_default()
                .push(curr.clone());
        }

        let change = StateChange::new(
            Some(new_value.clone()),
            new_value.clone(),
            Some("redo".to_string()),
        );
        self.states.insert(key.to_string(), new_value);

        // 直接记录历史但不调用 record_change（避免清空重做栈）
        let history = self.history.entry(key.to_string()).or_default();
        history.push(change.clone());
        if history.len() > self.max_history {
            history.remove(0);
        }

        self.notify(key, &change);

        Ok(())
    }
}

/// 线程安全的状态管理器
///
/// 使用 RwLock 实现线程安全的状态访问。
#[derive(Debug, Clone)]
pub struct ThreadSafeStateManager {
    inner: Arc<RwLock<ReactiveStateManager>>,
}

impl Default for ThreadSafeStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadSafeStateManager {
    /// 创建新的线程安全状态管理器
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ReactiveStateManager::new())),
        }
    }

    /// 创建带指定历史长度的管理器
    #[must_use]
    pub fn with_history(max_history: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ReactiveStateManager::with_history(max_history))),
        }
    }

    /// 获取内部状态管理器的引用
    #[must_use]
    pub fn inner(&self) -> &Arc<RwLock<ReactiveStateManager>> {
        &self.inner
    }

    /// 获取状态值的克隆（线程安全版本）
    #[must_use]
    pub fn get_cloned(&self, key: &str) -> Option<StateValue> {
        self.inner.read().ok()?.get(key).cloned()
    }
}

impl StateManager for ThreadSafeStateManager {
    fn get(&self, _key: &str) -> Option<&StateValue> {
        // 注意：由于返回引用，这里无法正确实现
        // 这是一个已知限制，实际使用中应避免使用 ThreadSafeStateManager 的 get 方法
        // 或者直接使用 ReactiveStateManager
        // 作为临时解决方案，这里返回 None
        // 请使用 get_cloned 方法代替
        None
    }

    fn set(&mut self, key: &str, value: StateValue) -> Result<(), StateError> {
        if let Ok(mut guard) = self.inner.write() {
            guard.set(key, value)
        } else {
            Err(StateError::InvalidValue("Poison error".to_string()))
        }
    }

    fn remove(&mut self, key: &str) -> Option<StateValue> {
        if let Ok(mut guard) = self.inner.write() {
            guard.remove(key)
        } else {
            None
        }
    }

    fn keys(&self) -> Vec<String> {
        self.inner
            .read()
            .map(|guard| guard.keys())
            .unwrap_or_default()
    }

    fn clear(&mut self) {
        let _ = self.inner.write().map(|mut guard| guard.clear());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_manager() {
        let mut manager = DefaultStateManager::new();

        assert!(manager.get("test").is_none());

        manager.set("test", StateValue::from("hello")).unwrap();
        assert_eq!(manager.get("test"), Some(&StateValue::from("hello")));

        let keys = manager.keys();
        assert_eq!(keys, vec!["test".to_string()]);

        manager.remove("test");
        assert!(manager.get("test").is_none());
    }

    #[test]
    fn test_reactive_state_manager() {
        let mut manager = ReactiveStateManager::new();

        // 测试订阅
        let notified = std::sync::Arc::new(std::sync::Mutex::new(false));
        let notified_clone = notified.clone();

        manager.subscribe(
            "counter".to_string(),
            Box::new(move |_change| {
                *notified_clone.lock().unwrap() = true;
            }),
        );

        manager.set("counter", StateValue::from(42)).unwrap();
        assert!(*notified.lock().unwrap());
    }

    #[test]
    fn test_undo_redo() {
        let mut manager = ReactiveStateManager::new();

        manager.set("value", StateValue::from(1)).unwrap();
        manager.set("value", StateValue::from(2)).unwrap();
        manager.set("value", StateValue::from(3)).unwrap();

        assert_eq!(manager.get("value"), Some(&StateValue::from(3)));

        manager.undo("value").unwrap();
        assert_eq!(manager.get("value"), Some(&StateValue::from(2)));

        manager.undo("value").unwrap();
        assert_eq!(manager.get("value"), Some(&StateValue::from(1)));

        manager.redo("value").unwrap();
        assert_eq!(manager.get("value"), Some(&StateValue::from(2)));
    }

    #[test]
    fn test_history() {
        let mut manager = ReactiveStateManager::new();

        manager.set("value", StateValue::from(1)).unwrap();
        manager.set("value", StateValue::from(2)).unwrap();

        let history = manager.history("value");
        assert_eq!(history.len(), 2);
    }
}
