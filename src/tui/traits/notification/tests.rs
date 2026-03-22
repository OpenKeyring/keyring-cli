//! Tests for notification module
//!
//! Unit tests for notification system.

use super::*;
use std::time::Duration;

#[test]
fn test_notification_levels() {
    assert_eq!(
        NotificationLevel::Info.default_duration(),
        Duration::from_secs(3)
    );
    assert_eq!(
        NotificationLevel::Error.default_duration(),
        Duration::from_secs(0)
    );
    assert_eq!(
        NotificationLevel::Warning.default_duration(),
        Duration::from_secs(5)
    );
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
    queue.push(Notification::info("4")); // Exceeds capacity
    assert_eq!(queue.len(), 3);
    assert_eq!(queue.all()[0].message, "2"); // Oldest removed
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
