//! 事件调度器
//!
//! 占位符模块，完整实现将在后续任务中完成。

/// 事件调度器
///
/// 负责将事件分发到相应的处理器。
pub struct EventDispatcher {
    _private: (),
}

impl EventDispatcher {
    /// 创建新的事件调度器
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

