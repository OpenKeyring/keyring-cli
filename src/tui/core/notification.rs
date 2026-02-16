//! 默认通知管理器实现
//!
//! 占位符模块，完整实现将在 Task C.4 中完成。

use crate::tui::traits::{NotificationManager, Notification, NotificationLevel};
use std::collections::VecDeque;

/// 默认通知管理器
#[derive(Debug, Default)]
pub struct DefaultNotificationManager {
    _notifications: VecDeque<Notification>,
}

impl DefaultNotificationManager {
    /// 创建新的通知管理器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _notifications: VecDeque::new(),
        }
    }
}

impl NotificationManager for DefaultNotificationManager {
    fn show(&mut self, notification: Notification) {
        self._notifications.push_back(notification);
    }

    fn clear(&mut self) {
        self._notifications.clear();
    }
}
