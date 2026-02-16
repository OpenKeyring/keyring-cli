//! 默认通知管理器实现
//!
//! 占位符模块，完整实现将在 Task C.4 中完成。

use crate::tui::traits::{
    NotificationManager, Notification, NotificationId, NotificationLevel, NotificationPosition,
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

/// 默认通知管理器
#[derive(Debug)]
pub struct DefaultNotificationManager {
    _notifications: VecDeque<Notification>,
    _next_id: AtomicU64,
    _max_visible: usize,
    _position: NotificationPosition,
}

impl Default for DefaultNotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultNotificationManager {
    /// 创建新的通知管理器
    #[must_use]
    pub fn new() -> Self {
        Self {
            _notifications: VecDeque::new(),
            _next_id: AtomicU64::new(1),
            _max_visible: 5,
            _position: NotificationPosition::Top,
        }
    }
}

impl NotificationManager for DefaultNotificationManager {
    fn show(&mut self, mut notification: Notification) -> NotificationId {
        let id = NotificationId::new(self._next_id.fetch_add(1, Ordering::SeqCst));
        notification.id = Some(id);
        self._notifications.push_back(notification);
        id
    }

    fn dismiss(&mut self, id: NotificationId) -> bool {
        if let Some(pos) = self._notifications.iter().position(|n| n.id == Some(id)) {
            self._notifications.remove(pos);
            true
        } else {
            false
        }
    }

    fn dismiss_all(&mut self) {
        self._notifications.clear();
    }

    fn active_notifications(&self) -> Vec<&Notification> {
        self._notifications.iter().collect()
    }

    fn tick(&mut self) {
        self._notifications.retain(|n| !n.is_expired());
    }

    fn set_max_visible(&mut self, max: usize) {
        self._max_visible = max;
    }

    fn max_visible(&self) -> usize {
        self._max_visible
    }

    fn set_position(&mut self, position: NotificationPosition) {
        self._position = position;
    }

    fn position(&self) -> NotificationPosition {
        self._position
    }
}
