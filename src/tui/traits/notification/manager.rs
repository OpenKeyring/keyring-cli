//! Notification manager trait
//!
//! Defines the interface for notification management.

use super::types::{Notification, NotificationId, NotificationLevel, NotificationPosition};

/// Notification manager trait
///
/// Defines the interface for notification display and management.
pub trait NotificationManager: Send + Sync {
    /// Show notification
    ///
    /// Returns notification ID, can be used to dismiss.
    fn show(&mut self, notification: Notification) -> NotificationId;

    /// Dismiss notification
    fn dismiss(&mut self, id: NotificationId) -> bool;

    /// Dismiss all notifications
    fn dismiss_all(&mut self);

    /// Get active notifications (for rendering)
    fn active_notifications(&self) -> Vec<&Notification>;

    /// Get active notification count
    #[must_use]
    fn active_count(&self) -> usize {
        self.active_notifications().len()
    }

    /// Update state (cleanup expired notifications)
    fn tick(&mut self);

    /// Set max visible count
    fn set_max_visible(&mut self, max: usize);

    /// Get max visible count
    #[must_use]
    fn max_visible(&self) -> usize;

    /// Set display position
    fn set_position(&mut self, position: NotificationPosition);

    /// Get display position
    #[must_use]
    fn position(&self) -> NotificationPosition;
}

/// Notification manager extension trait
///
/// Provides convenience methods.
pub trait NotificationManagerExt: NotificationManager {
    /// Quick show info notification
    fn info(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::info(message))
    }

    /// Quick show success notification
    fn success(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::success(message))
    }

    /// Quick show warning notification
    fn warning(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::warning(message))
    }

    /// Quick show error notification
    fn error(&mut self, message: impl Into<String>) -> NotificationId {
        self.show(Notification::error(message))
    }

    /// Show notification with progress
    fn progress(&mut self, message: impl Into<String>, progress: u8) -> NotificationId {
        let mut notification = Notification::info(message);
        notification.progress = Some(progress.min(100));
        self.show(notification)
    }

    /// Check if has active notifications
    #[must_use]
    fn has_active(&self) -> bool {
        !self.active_notifications().is_empty()
    }

    /// Check if has error level notifications
    #[must_use]
    fn has_errors(&self) -> bool {
        self.active_notifications()
            .iter()
            .any(|n| n.level == NotificationLevel::Error)
    }
}

/// Implement extension methods for all NotificationManager
impl<T: NotificationManager + ?Sized> NotificationManagerExt for T {}
