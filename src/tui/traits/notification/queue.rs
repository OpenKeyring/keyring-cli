//! Notification queue
//!
//! Manages pending notifications.

use super::types::{Notification, NotificationId};

/// Notification queue
///
/// Used to manage pending notifications.
#[derive(Debug, Clone, Default)]
pub struct NotificationQueue {
    notifications: Vec<Notification>,
    max_size: usize,
}

impl NotificationQueue {
    /// Create new queue
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            notifications: Vec::new(),
            max_size,
        }
    }

    /// Add notification
    pub fn push(&mut self, notification: Notification) -> Option<Notification> {
        if self.notifications.len() >= self.max_size {
            // Remove oldest notification
            self.notifications.remove(0);
        }
        self.notifications.push(notification);
        None
    }

    /// Remove notification
    pub fn remove(&mut self, id: NotificationId) -> bool {
        if let Some(pos) = self.notifications.iter().position(|n| n.id == Some(id)) {
            self.notifications.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear queue
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    /// Get all notifications
    #[must_use]
    pub fn all(&self) -> &[Notification] {
        &self.notifications
    }

    /// Get notification count
    #[must_use]
    pub fn len(&self) -> usize {
        self.notifications.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    /// Cleanup expired notifications
    pub fn cleanup_expired(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    /// Iterate notifications
    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.iter()
    }
}
