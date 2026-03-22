//! Notification renderer
//!
//! Defines notification rendering interface.

use super::types::{Notification, NotificationLevel};

/// Notification renderer trait
///
/// Defines notification rendering interface.
pub trait NotificationRenderer: Send + Sync {
    /// Render notification
    fn render(&self, notification: &Notification) -> String;

    /// Get notification width
    #[must_use]
    fn width(&self, notification: &Notification) -> usize;

    /// Get notification height
    #[must_use]
    fn height(&self, notification: &Notification) -> usize;
}

/// Default notification renderer
pub struct DefaultNotificationRenderer {
    /// Show timestamp
    show_timestamp: bool,
    /// Show source
    show_source: bool,
}

impl DefaultNotificationRenderer {
    /// Create new renderer
    #[must_use]
    pub const fn new() -> Self {
        Self {
            show_timestamp: false,
            show_source: false,
        }
    }

    /// Set show timestamp
    #[must_use]
    pub const fn with_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    /// Set show source
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

        // Add level icon
        let icon = match notification.level {
            NotificationLevel::Info => "ℹ",
            NotificationLevel::Success => "✓",
            NotificationLevel::Warning => "⚠",
            NotificationLevel::Error => "✖",
        };
        result.push_str(icon);
        result.push(' ');
        result.push_str(&notification.message);

        // Add progress bar
        if let Some(progress) = notification.progress {
            let filled = progress as usize / 10;
            let bar = "█".repeat(filled);
            let empty = "░".repeat(10 - filled);
            result.push_str(&format!(" [{}{}{}]", bar, empty, progress));
        }

        // Add source
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
