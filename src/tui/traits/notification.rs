//! 通知/Toast 系统 Trait 定义
//!
//! 定义 TUI 通知显示和管理的接口。

use std::time::{Duration, Instant};

/// 通知 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotificationId(u64);

impl NotificationId {
    /// 创建新的通知 ID
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

/// 通知级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    /// 信息
    Info,
    /// 成功
    Success,
    /// 警告
    Warning,
    /// 错误
    Error,
}

impl Default for NotificationLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl NotificationLevel {
    /// 获取默认显示时长
    #[must_use]
    pub fn default_duration(&self) -> Duration {
        match self {
            Self::Success | Self::Info => Duration::from_secs(3),
            Self::Warning => Duration::from_secs(5),
            Self::Error => Duration::from_secs(0), // 错误不自动消失
        }
    }

    /// 获取级别名称
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Info => "信息",
            Self::Success => "成功",
            Self::Warning => "警告",
            Self::Error => "错误",
        }
    }

    /// 是否需要用户确认
    #[must_use]
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Error)
    }
}

/// 通知位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPosition {
    /// 顶部
    Top,
    /// 底部
    Bottom,
    /// 右上角
    TopRight,
    /// 右下角
    BottomRight,
}

impl Default for NotificationPosition {
    fn default() -> Self {
        Self::Top
    }
}

/// 通知
#[derive(Debug, Clone)]
pub struct Notification {
    /// 通知 ID（由管理器分配）
    pub id: Option<NotificationId>,
    /// 消息内容
    pub message: String,
    /// 通知级别
    pub level: NotificationLevel,
    /// 显示时长（None 表示使用默认值）
    pub duration: Option<Duration>,
    /// 是否可手动关闭
    pub dismissible: bool,
    /// 创建时间
    pub created_at: Instant,
    /// 来源组件/操作
    pub source: Option<String>,
    /// 进度（0-100，用于带进度的通知）
    pub progress: Option<u8>,
}

impl Notification {
    /// 创建信息通知
    #[must_use]
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: None,
            message: message.into(),
            level: NotificationLevel::Info,
            duration: None,
            dismissible: true,
            created_at: Instant::now(),
            source: None,
            progress: None,
        }
    }

    /// 创建成功通知
    #[must_use]
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            id: None,
            message: message.into(),
            level: NotificationLevel::Success,
            duration: None,
            dismissible: true,
            created_at: Instant::now(),
            source: None,
            progress: None,
        }
    }

    /// 创建警告通知
    #[must_use]
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            id: None,
            message: message.into(),
            level: NotificationLevel::Warning,
            duration: None,
            dismissible: true,
            created_at: Instant::now(),
            source: None,
            progress: None,
        }
    }

    /// 创建错误通知
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: None,
            message: message.into(),
            level: NotificationLevel::Error,
            duration: Some(Duration::from_secs(0)), // 不自动消失
            dismissible: true,
            created_at: Instant::now(),
            source: None,
            progress: None,
        }
    }

    /// 设置显示时长
    #[must_use]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// 设置来源
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 设置进度
    #[must_use]
    pub fn with_progress(mut self, progress: u8) -> Self {
        self.progress = Some(progress.min(100));
        self
    }

    /// 设置不可关闭
    #[must_use]
    pub fn not_dismissible(mut self) -> Self {
        self.dismissible = false;
        self
    }

    /// 获取实际显示时长
    #[must_use]
    pub fn effective_duration(&self) -> Duration {
        self.duration.unwrap_or_else(|| self.level.default_duration())
    }

    /// 检查是否已过期
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let duration = self.effective_duration();
        duration.as_secs() > 0 && self.created_at.elapsed() >= duration
    }

    /// 检查是否应该自动关闭
    #[must_use]
    pub fn should_auto_close(&self) -> bool {
        self.effective_duration().as_secs() > 0
    }

    /// 获取已存在时间
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// 获取剩余时间
    #[must_use]
    pub fn remaining(&self) -> Option<Duration> {
        let duration = self.effective_duration();
        if duration.as_secs() == 0 {
            return None;
        }
        let elapsed = self.elapsed();
        if elapsed >= duration {
            None
        } else {
            Some(duration - elapsed)
        }
    }
}

/// 通知管理器 Trait
///
/// 定义通知显示和管理的接口。
pub trait NotificationManager: Send + Sync {
    /// 显示通知
    ///
    /// 返回通知 ID，可用于关闭通知。
    fn show(&mut self, notification: Notification) -> NotificationId;

    /// 关闭通知
    fn dismiss(&mut self, id: NotificationId) -> bool;

    /// 关闭所有通知
    fn dismiss_all(&mut self);

    /// 获取当前活动通知（用于渲染）
    fn active_notifications(&self) -> Vec<&Notification>;

    /// 获取活动通知数量
    #[must_use]
    fn active_count(&self) -> usize {
        self.active_notifications().len()
    }

    /// 更新状态（清理过期通知）
    fn tick(&mut self);

    /// 设置最大显示数量
    fn set_max_visible(&mut self, max: usize);

    /// 获取最大显示数量
    #[must_use]
    fn max_visible(&self) -> usize;

    /// 设置显示位置
    fn set_position(&mut self, position: NotificationPosition);

    /// 获取显示位置
    #[must_use]
    fn position(&self) -> NotificationPosition;
}

/// 通知管理器扩展 Trait
///
/// 提供额外的功能方法。
pub trait NotificationManagerExt: NotificationManager {
    /// 快捷显示信息通知
    fn info(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::info(message))
    }

    /// 快捷显示成功通知
    fn success(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::success(message))
    }

    /// 快捷显示警告通知
    fn warning(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::warning(message))
    }

    /// 快捷显示错误通知
    fn error(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::error(message))
    }

    /// 显示带进度的通知
    fn progress(&mut self, message: impl Into<String>, progress: u8) -> NotificationId {
        let mut notification = Notification::info(message);
        notification.progress = Some(progress.min(100));
        self.show(notification)
    }

    /// 检查是否有活动通知
    #[must_use]
    fn has_active(&self) -> bool {
        !self.active_notifications().is_empty()
    }

    /// 检查是否有错误级别的通知
    #[must_use]
    fn has_errors(&self) -> bool {
        self.active_notifications()
            .iter()
            .any(|n| n.level == NotificationLevel::Error)
    }
}

/// 为所有 NotificationManager 实现提供扩展方法
impl<T: NotificationManager + ?Sized> NotificationManagerExt for T {}

/// 通知配置
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// 最大显示数量
    pub max_visible: usize,
    /// 显示位置
    pub position: NotificationPosition,
    /// 默认是否可关闭
    pub default_dismissible: bool,
    /// 是否启用声音（如果终端支持）
    pub enable_sound: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            max_visible: 5,
            position: NotificationPosition::Top,
            default_dismissible: true,
            enable_sound: false,
        }
    }
}

impl NotificationConfig {
    /// 创建新的配置
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_visible: 5,
            position: NotificationPosition::Top,
            default_dismissible: true,
            enable_sound: false,
        }
    }

    /// 设置最大显示数量
    #[must_use]
    pub const fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// 设置显示位置
    #[must_use]
    pub const fn with_position(mut self, position: NotificationPosition) -> Self {
        self.position = position;
        self
    }

    /// 设置是否启用声音
    #[must_use]
    pub const fn with_sound(mut self, enable: bool) -> Self {
        self.enable_sound = enable;
        self
    }
}

/// 通知队列
///
/// 用于管理待显示的通知。
#[derive(Debug, Clone, Default)]
pub struct NotificationQueue {
    notifications: Vec<Notification>,
    max_size: usize,
}

impl NotificationQueue {
    /// 创建新的队列
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            notifications: Vec::new(),
            max_size,
        }
    }

    /// 添加通知
    pub fn push(&mut self, notification: Notification) -> Option<Notification> {
        if self.notifications.len() >= self.max_size {
            // 移除最旧的通知
            self.notifications.remove(0);
        }
        self.notifications.push(notification);
        None
    }

    /// 移除通知
    pub fn remove(&mut self, id: NotificationId) -> bool {
        if let Some(pos) = self.notifications.iter().position(|n| n.id == Some(id)) {
            self.notifications.remove(pos);
            true
        } else {
            false
        }
    }

    /// 清空队列
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    /// 获取所有通知
    #[must_use]
    pub fn all(&self) -> &[Notification] {
        &self.notifications
    }

    /// 获取通知数量
    #[must_use]
    pub fn len(&self) -> usize {
        self.notifications.len()
    }

    /// 是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    /// 清理过期通知
    pub fn cleanup_expired(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    /// 迭代通知
    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.iter()
    }
}

/// 通知渲染器 Trait
///
/// 定义通知的渲染接口。
pub trait NotificationRenderer: Send + Sync {
    /// 渲染通知
    fn render(&self, notification: &Notification) -> String;

    /// 获取通知宽度
    #[must_use]
    fn width(&self, notification: &Notification) -> usize;

    /// 获取通知高度
    #[must_use]
    fn height(&self, notification: &Notification) -> usize;
}

/// 默认通知渲染器
pub struct DefaultNotificationRenderer {
    /// 是否显示时间戳
    show_timestamp: bool,
    /// 是否显示来源
    show_source: bool,
}

impl DefaultNotificationRenderer {
    /// 创建新的渲染器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            show_timestamp: false,
            show_source: false,
        }
    }

    /// 设置是否显示时间戳
    #[must_use]
    pub const fn with_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    /// 设置是否显示来源
    #[must_use]
    pub const fn with_source(mut self, show: bool) -> Self {
        self.show_source = show;
        self
    }
}

impl Default for DefaultNotificationRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationRenderer for DefaultNotificationRenderer {
    fn render(&self, notification: &Notification) -> String {
        let mut result = String::new();

        // 添加级别标签
        let icon = match notification.level {
            NotificationLevel::Info => "ℹ",
            NotificationLevel::Success => "✓",
            NotificationLevel::Warning => "⚠",
            NotificationLevel::Error => "✖",
        };
        result.push_str(icon);
        result.push(' ');
        result.push_str(&notification.message);

        // 添加进度条
        if let Some(progress) = notification.progress {
            let filled = progress as usize / 10;
            let bar = "█".repeat(filled);
            let empty = "░".repeat(10 - filled);
            result.push_str(&format!(" [{}{}{}]", bar, empty, progress));
        }

        // 添加来源
        if self.show_source {
            if let Some(ref source) = notification.source {
                result.push_str(&format!(" (@{})", source));
            }
        }

        result
    }

    fn width(&self, notification: &Notification) -> usize {
        self.render(notification).len()
    }

    fn height(&self, _notification: &Notification) -> usize {
        1
    }
}

/// 通知过滤器 Trait
///
/// 用于过滤通知。
pub trait NotificationFilter: Send + Sync {
    /// 检查通知是否应该显示
    fn should_show(&self, notification: &Notification) -> bool;
}

/// 级别过滤器
pub struct LevelFilter {
    /// 最小显示级别
    min_level: NotificationLevel,
}

impl LevelFilter {
    /// 创建新的级别过滤器
    #[must_use]
    pub const fn new(min_level: NotificationLevel) -> Self {
        Self { min_level }
    }
}

impl NotificationFilter for LevelFilter {
    fn should_show(&self, notification: &Notification) -> bool {
        // 错误和警告总是显示，其他根据级别判断
        match notification.level {
            NotificationLevel::Error | NotificationLevel::Warning => true,
            NotificationLevel::Success if matches!(self.min_level, NotificationLevel::Info) => true,
            NotificationLevel::Success => false,
            NotificationLevel::Info => matches!(self.min_level, NotificationLevel::Info),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_levels() {
        assert_eq!(NotificationLevel::Info.default_duration(), Duration::from_secs(3));
        assert_eq!(NotificationLevel::Error.default_duration(), Duration::from_secs(0));
        assert_eq!(NotificationLevel::Warning.default_duration(), Duration::from_secs(5));
    }

    #[test]
    fn test_notification_creation() {
        let info = Notification::info("Test message");
        assert_eq!(info.level, NotificationLevel::Info);
        assert_eq!(info.message, "Test message");
        assert!(info.dismissible);

        let error = Notification::error("Error occurred");
        assert_eq!(error.level, NotificationLevel::Error);
        assert_eq!(error.effective_duration(), Duration::from_secs(0));
    }

    #[test]
    fn test_notification_builder() {
        let notification = Notification::warning("Warning")
            .with_duration(Duration::from_secs(10))
            .with_source("test_module")
            .with_progress(50);

        assert_eq!(notification.level, NotificationLevel::Warning);
        assert_eq!(notification.duration, Some(Duration::from_secs(10)));
        assert_eq!(notification.source.as_deref(), Some("test_module"));
        assert_eq!(notification.progress, Some(50));
    }

    #[test]
    fn test_notification_queue() {
        let mut queue = NotificationQueue::new(3);

        queue.push(Notification::info("1"));
        queue.push(Notification::info("2"));
        assert_eq!(queue.len(), 2);

        queue.push(Notification::info("3"));
        queue.push(Notification::info("4")); // 超过容量
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.all()[0].message, "2"); // 最旧的被移除
    }

    #[test]
    fn test_notification_renderer() {
        let renderer = DefaultNotificationRenderer::new();

        let notification = Notification::success("Operation completed");
        let rendered = renderer.render(&notification);

        assert!(rendered.contains("✓"));
        assert!(rendered.contains("Operation completed"));
    }

    #[test]
    fn test_level_filter() {
        let filter = LevelFilter::new(NotificationLevel::Warning);

        let error = Notification::error("Error");
        let warning = Notification::warning("Warning");
        let info = Notification::info("Info");

        assert!(filter.should_show(&error));
        assert!(filter.should_show(&warning));
        assert!(!filter.should_show(&info));
    }

    #[test]
    fn test_notification_id() {
        let id1 = NotificationId::new(1);
        let id2 = NotificationId::new(2);

        assert_eq!(id1.value(), 1);
        assert_ne!(id1, id2);
    }
}
