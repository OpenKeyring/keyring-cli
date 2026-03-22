//! 状态管理系统 Trait 定义
//!
//! 定义组件状态管理和响应式状态更新的接口。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 状态键
///
/// 用于标识和查找状态值。
pub type StateKey = String;

/// 状态值
///
/// 支持多种类型的状态值。
#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    /// 字符串值
    String(String),
    /// 整数值
    Integer(i64),
    /// 布尔值
    Boolean(bool),
    /// 浮点数值
    Float(f64),
    /// 无值
    Null,
}

impl StateValue {
    /// 获取字符串值
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// 获取整数值
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// 获取布尔值
    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// 获取浮点数值
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// 转换为字符串
    #[must_use]
    pub fn to_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Integer(i) => i.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Null => String::new(),
        }
    }
}

impl From<String> for StateValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for StateValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for StateValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<bool> for StateValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<f64> for StateValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

/// 状态变化记录
#[derive(Debug, Clone)]
pub struct StateChange {
    /// 变化前的值
    pub old_value: Option<StateValue>,
    /// 变化后的值
    pub new_value: StateValue,
    /// 变化时间
    pub timestamp: Instant,
    /// 变化来源（哪个组件/操作）
    pub source: Option<String>,
}

impl StateChange {
    /// 创建新的状态变化记录
    #[must_use]
    pub fn new(
        old_value: Option<StateValue>,
        new_value: StateValue,
        source: Option<String>,
    ) -> Self {
        Self {
            old_value,
            new_value,
            timestamp: Instant::now(),
            source,
        }
    }
}

/// 订阅 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// 创建新的订阅 ID
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取 ID 值
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.0
    }
}

/// 状态变化回调
pub type StateCallback = Box<dyn Fn(&StateChange) + Send + Sync>;

/// 状态管理器 Trait
///
/// 定义状态存储和检索的基本接口。
pub trait StateManager: Send + Sync {
    /// 获取状态值
    fn get(&self, key: &str) -> Option<&StateValue>;

    /// 设置状态值
    fn set(&mut self, key: &str, value: StateValue) -> Result<(), StateError>;

    /// 删除状态值
    fn remove(&mut self, key: &str) -> Option<StateValue>;

    /// 检查状态是否存在
    #[must_use]
    fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// 获取所有状态键
    #[must_use]
    fn keys(&self) -> Vec<StateKey>;

    /// 清除所有状态
    fn clear(&mut self);
}

/// 响应式状态 Trait
///
/// 扩展状态管理器，支持状态变化订阅和通知。
pub trait ReactiveState: StateManager {
    /// 订阅状态变化
    ///
    /// 返回订阅 ID，可用于取消订阅。
    fn subscribe(&mut self, key: String, callback: StateCallback) -> SubscriptionId;

    /// 取消订阅
    fn unsubscribe(&mut self, id: SubscriptionId) -> bool;

    /// 订阅所有状态变化
    fn subscribe_all(&mut self, callback: StateCallback) -> SubscriptionId;

    /// 获取状态变化历史
    #[must_use]
    fn history(&self, key: &str) -> Vec<&StateChange>;

    /// 撤销到指定版本
    fn undo(&mut self, key: &str) -> Result<(), StateError>;

    /// 重做
    fn redo(&mut self, key: &str) -> Result<(), StateError>;
}

/// 状态错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// 状态不存在
    NotFound(String),
    /// 类型不匹配
    TypeMismatch { expected: String, actual: String },
    /// 只读状态
    ReadOnly(String),
    /// 无效的状态值
    InvalidValue(String),
    /// 撤销失败
    UndoFailed(String),
    /// 重做失败
    RedoFailed(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(key) => write!(f, "状态不存在: {}", key),
            Self::TypeMismatch { expected, actual } => {
                write!(f, "类型不匹配: 期望 {}, 实际 {}", expected, actual)
            }
            Self::ReadOnly(key) => write!(f, "只读状态: {}", key),
            Self::InvalidValue(msg) => write!(f, "无效的状态值: {}", msg),
            Self::UndoFailed(msg) => write!(f, "撤销失败: {}", msg),
            Self::RedoFailed(msg) => write!(f, "重做失败: {}", msg),
        }
    }
}

impl std::error::Error for StateError {}

/// 计算状态 Trait
///
/// 支持从其他状态派生的计算状态。
pub trait ComputedState: Send + Sync {
    /// 计算状态值
    fn compute(&self, dependencies: &HashMap<String, StateValue>) -> StateValue;

    /// 获取依赖的状态键
    #[must_use]
    fn dependencies(&self) -> Vec<String>;
}

/// 可追踪状态 Trait
///
/// 支持状态变化历史和撤销/重做。
pub trait TrackableState {
    /// 获取变化历史
    #[must_use]
    fn history(&self) -> &[StateChange];

    /// 当前版本号
    #[must_use]
    fn current_version(&self) -> usize;

    /// 最大历史长度
    #[must_use]
    fn max_history(&self) -> usize {
        100
    }

    /// 撤销到指定版本
    fn undo_to(&mut self, version: usize) -> Result<(), StateError>;

    /// 重做到指定版本
    fn redo_to(&mut self, version: usize) -> Result<(), StateError>;
}

/// 状态快照
///
/// 用于保存和恢复状态。
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// 快照时间
    pub timestamp: Instant,
    /// 状态数据
    pub data: HashMap<StateKey, StateValue>,
}

impl StateSnapshot {
    /// 创建新的状态快照
    #[must_use]
    pub fn new(data: HashMap<StateKey, StateValue>) -> Self {
        Self {
            timestamp: Instant::now(),
            data,
        }
    }

    /// 创建空快照
    #[must_use]
    pub fn empty() -> Self {
        Self {
            timestamp: Instant::now(),
            data: HashMap::new(),
        }
    }
}

/// 状态路径
///
/// 支持嵌套状态的路径访问。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatePath {
    /// 根字段
    Field(String),
    /// 嵌套字段（如 "selected_password.id"）
    Nested(String, Box<StatePath>),
    /// 数组索引
    Index(usize, Box<StatePath>),
}

impl StatePath {
    /// 创建字段路径
    #[must_use]
    pub fn field(name: &str) -> Self {
        Self::Field(name.to_string())
    }

    /// 创建嵌套路径
    #[must_use]
    pub fn nested(parent: &str, child: StatePath) -> Self {
        Self::Nested(parent.to_string(), Box::new(child))
    }

    /// 解析路径字符串
    ///
    /// 支持点分隔的路径，如 "user.profile.name"
    #[must_use]
    pub fn parse(path: &str) -> Self {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Self::Field(String::new());
        }

        let mut result = Self::Field(parts.last().unwrap().to_string());
        for part in parts.iter().rev().skip(1) {
            result = Self::Nested(part.to_string(), Box::new(result));
        }
        result
    }

    /// 转换为字符串
    #[must_use]
    pub fn to_string(&self) -> String {
        match self {
            Self::Field(name) => name.clone(),
            Self::Nested(parent, child) => format!("{}.{}", parent, child.to_string()),
            Self::Index(idx, child) => format!("[{}].{}", idx, child.to_string()),
        }
    }
}

/// 状态选择器
///
/// 用于选择和转换状态数据。
pub trait StateSelector: Send + Sync {
    /// 从状态中选择值
    fn select(&self, state: &HashMap<StateKey, StateValue>) -> Option<StateValue>;
}

/// 简单键选择器
pub struct KeySelector {
    key: StateKey,
}

impl KeySelector {
    /// 创建新的键选择器
    #[must_use]
    pub fn new(key: impl Into<StateKey>) -> Self {
        Self { key: key.into() }
    }
}

impl StateSelector for KeySelector {
    fn select(&self, state: &HashMap<StateKey, StateValue>) -> Option<StateValue> {
        state.get(&self.key).cloned()
    }
}

/// 组合选择器
pub struct CombinedSelector {
    selectors: Vec<Box<dyn StateSelector>>,
    combiner: fn(Vec<Option<StateValue>>) -> Option<StateValue>,
}

impl CombinedSelector {
    /// 创建新的组合选择器
    #[must_use]
    pub fn new(
        selectors: Vec<Box<dyn StateSelector>>,
        combiner: fn(Vec<Option<StateValue>>) -> Option<StateValue>,
    ) -> Self {
        Self {
            selectors,
            combiner,
        }
    }
}

impl StateSelector for CombinedSelector {
    fn select(&self, state: &HashMap<StateKey, StateValue>) -> Option<StateValue> {
        let values: Vec<Option<StateValue>> =
            self.selectors.iter().map(|s| s.select(state)).collect();
        (self.combiner)(values)
    }
}

/// ID 生成器（用于订阅 ID）
#[derive(Debug)]
pub struct SubscriptionIdGenerator {
    counter: AtomicU64,
}

impl SubscriptionIdGenerator {
    /// 创建新的生成器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }

    /// 生成新 ID
    #[must_use]
    pub fn generate(&self) -> SubscriptionId {
        SubscriptionId(self.counter.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for SubscriptionIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 状态观察者
///
/// 监听状态变化并触发相应操作。
pub trait StateObserver: Send + Sync {
    /// 状态变化时调用
    fn on_change(&self, change: &StateChange);

    /// 获取观察的状态键
    #[must_use]
    fn watched_keys(&self) -> Vec<String>;
}

/// 简单观察者
pub struct SimpleObserver {
    keys: Vec<String>,
    callback: Box<dyn Fn(&StateChange) + Send + Sync>,
}

impl SimpleObserver {
    /// 创建新的观察者
    pub fn new<F>(keys: Vec<String>, callback: F) -> Self
    where
        F: Fn(&StateChange) + Send + Sync + 'static,
    {
        Self {
            keys,
            callback: Box::new(callback),
        }
    }
}

impl StateObserver for SimpleObserver {
    fn on_change(&self, change: &StateChange) {
        (self.callback)(change);
    }

    fn watched_keys(&self) -> Vec<String> {
        self.keys.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_state_value_conversions() {
        let s = StateValue::from("hello");
        assert_eq!(s.as_string(), Some("hello"));

        let i = StateValue::from(42i64);
        assert_eq!(i.as_integer(), Some(42));

        let b = StateValue::from(true);
        assert_eq!(b.as_boolean(), Some(true));

        let f = StateValue::from(3.14f64);
        assert_eq!(f.as_float(), Some(3.14));
    }

    #[test]
    fn test_state_path_parse() {
        let path = StatePath::parse("user.profile.name");
        assert_eq!(path.to_string(), "user.profile.name");

        let simple = StatePath::parse("username");
        assert_eq!(simple.to_string(), "username");
    }

    #[test]
    fn test_state_snapshot() {
        let mut data = HashMap::new();
        data.insert("key1".to_string(), StateValue::from("value1"));
        data.insert("key2".to_string(), StateValue::from(42));

        let snapshot = StateSnapshot::new(data);
        assert_eq!(snapshot.data.len(), 2);
        assert_eq!(snapshot.data.get("key2"), Some(&StateValue::Integer(42)));
    }

    #[test]
    fn test_key_selector() {
        let mut state = HashMap::new();
        state.insert("name".to_string(), StateValue::from("Alice"));
        state.insert("age".to_string(), StateValue::from(30));

        let selector = KeySelector::new("name");
        assert_eq!(
            selector.select(&state),
            Some(StateValue::String("Alice".to_string()))
        );

        let age_selector = KeySelector::new("age");
        assert_eq!(age_selector.select(&state), Some(StateValue::Integer(30)));
    }

    #[test]
    fn test_subscription_id_generator() {
        let gen = SubscriptionIdGenerator::new();
        let id1 = gen.generate();
        let id2 = gen.generate();
        assert_ne!(id1, id2);
        assert_eq!(id2.value(), id1.value() + 1);
    }
}
