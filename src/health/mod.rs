//! Password health check module
//!
//! Provides comprehensive password health analysis including:
//! - Weak password detection using strength scoring
//! - Duplicate password detection across accounts
//! - Compromised password detection via HIBP API

pub mod checker;
pub mod hibp;
pub mod report;
pub mod strength;

pub use checker::HealthChecker;
pub use report::{HealthIssue, HealthIssueType, HealthReport};
