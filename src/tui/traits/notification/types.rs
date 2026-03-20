//! Notification types
//!
//! Basic types for the notification system.

use std::time::{Duration, Instant};

/// Notification ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotificationId(u64);

impl NotificationId {
    /// Create a new notification ID
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the ID value
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.0
    }
}

/// Notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    /// Information
    Info,
    /// Success
    Success,
    /// Warning
    Warning,
    /// Error
    Error,
}

impl Default for NotificationLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl NotificationLevel {
    /// Get default display duration
    #[must_use]
    pub fn default_duration(&self) -> Duration {
        match self {
            Self::Success | Self::Info => Duration::from_secs(3),
            Self::Warning => Duration::from_secs(5),
            Self::Error => Duration::from_secs(0), // Errors don't auto-dismiss
        }
    }

    /// Get level name
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Info => "信息",
            Self::Success => "成功",
            Self::Warning => "警告",
            Self::Error => "错误",
        }
    }

    /// Check if user confirmation is required
    #[must_use]
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Error)
    }
}

/// Notification position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPosition {
    /// Top
    Top,
    /// Bottom
    Bottom,
    /// Top right
    TopRight,
    /// Bottom right
    BottomRight,
}

impl Default for NotificationPosition {
    fn default() -> Self {
        Self::Top
    }
}

/// Notification
#[derive(Debug, Clone)]
pub struct Notification {
    /// Notification ID (assigned by manager)
    pub id: Option<NotificationId>,
    /// Message content
    pub message: String,
    /// Notification level
    pub level: NotificationLevel,
    /// Display duration (None means use default)
    pub duration: Option<Duration>,
    /// Whether can be manually dismissed
    pub dismissible: bool,
    /// Creation time
    pub created_at: Instant,
    /// Source component/operation
    pub source: Option<String>,
    /// Progress (0-100, for notifications with progress)
    pub progress: Option<u8>,
}

impl Notification {
    /// Create info notification
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

    /// Create success notification
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

    /// Create warning notification
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

    /// Create error notification
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: None,
            message: message.into(),
            level: NotificationLevel::Error,
            duration: Some(Duration::from_secs(0)), // Don't auto-dismiss
            dismissible: true,
            created_at: Instant::now(),
            source: None,
            progress: None,
        }
    }

    /// Set display duration
    #[must_use]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set source
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set progress
    #[must_use]
    pub fn with_progress(mut self, progress: u8) -> Self {
        self.progress = Some(progress.min(100));
        self
    }

    /// Set not dismissible
    #[must_use]
    pub fn not_dismissible(mut self) -> Self {
        self.dismissible = false;
        self
    }

    /// Get effective display duration
    #[must_use]
    pub fn effective_duration(&self) -> Duration {
        self.duration.unwrap_or_else(|| self.level.default_duration())
    }

    /// Check if expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let duration = self.effective_duration();
        duration.as_secs() > 0 && self.created_at.elapsed() >= duration
    }

    /// Check if should auto-close
    #[must_use]
    pub fn should_auto_close(&self) -> bool {
        self.effective_duration().as_secs() > 0
    }

    /// Get elapsed time
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get remaining time
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

/// Notification configuration
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// Maximum visible count
    pub max_visible: usize,
    /// Display position
    pub position: NotificationPosition,
    /// Default dismissible
    pub default_dismissible: bool,
    /// Enable sound (if terminal supports)
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
    /// Create new configuration
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_visible: 5,
            position: NotificationPosition::Top,
            default_dismissible: true,
            enable_sound: false,
        }
    }

    /// Set max visible count
    #[must_use]
    pub const fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Set display position
    #[must_use]
    pub const fn with_position(mut self, position: NotificationPosition) -> Self {
        self.position = position;
        self
    }

    /// Set enable sound
    #[must_use]
    pub const fn with_sound(mut self, enable: bool) -> Self {
        self.enable_sound = enable;
        self
    }
}
