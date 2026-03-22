//! Notification/Toast system trait definitions
//!
//! Defines interfaces for TUI notification display and management.

mod filter;
mod manager;
mod queue;
mod render;
#[cfg(test)]
mod tests;
mod types;

pub use filter::{LevelFilter, NotificationFilter};
pub use manager::{NotificationManager, NotificationManagerExt};
pub use queue::NotificationQueue;
pub use render::{DefaultNotificationRenderer, NotificationRenderer};
pub use types::{
    Notification, NotificationConfig, NotificationId, NotificationLevel, NotificationPosition,
};
