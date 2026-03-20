//! Notification filter
//!
//! Filters notifications based on criteria.

use super::types::{Notification, NotificationLevel};

/// Notification filter trait
///
/// Used to filter notifications.
pub trait NotificationFilter: Send + Sync {
    /// Check if notification should be shown
    fn should_show(&self, notification: &Notification) -> bool;
}

/// Level filter
pub struct LevelFilter {
    /// Minimum level to show
    min_level: NotificationLevel,
}

impl LevelFilter {
    /// Create new level filter
    #[must_use]
    pub const fn new(min_level: NotificationLevel) -> Self {
        Self { min_level }
    }
}

impl NotificationFilter for LevelFilter {
    fn should_show(&self, notification: &Notification) -> bool {
        // Errors and warnings always show, others based on level
        match notification.level {
            NotificationLevel::Error | NotificationLevel::Warning => true,
            NotificationLevel::Success if matches!(self.min_level, NotificationLevel::Info) => true,
            NotificationLevel::Success => false,
            NotificationLevel::Info => matches!(self.min_level, NotificationLevel::Info),
        }
    }
}
