//! 状态管理 trait 定义
//!
//! 占位符模块，完整实现将在 Task A.3 中完成。

/// 状态管理器 trait
pub trait StateManager: Send + Sync {
    /// 获取状态
    fn get_state(&self, key: &str) -> Option<String>;

    /// 设置状态
    fn set_state(&mut self, key: &str, value: String);
}

/// 响应式状态 trait
pub trait ReactiveState: StateManager {
    /// 订阅状态变化
    fn subscribe(&mut self, key: &str);
}
