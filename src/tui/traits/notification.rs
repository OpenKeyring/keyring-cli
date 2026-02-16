//! 通知 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.2 中完成。

/// 通知管理器 trait
pub trait NotificationManager: Send + Sync {
    /// 显示通知
    fn show(&mut self, notification: Notification);

    /// 清除所有通知
    fn clear(&mut self);
}

/// 通知
#[derive(Debug, Clone, Default)]
pub struct Notification {
    pub _level: NotificationLevel,
    pub _message: String,
}

/// 通知级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationLevel {
    /// 信息
    #[default]
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 成功
    Success,
}
