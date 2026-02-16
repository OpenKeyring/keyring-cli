//! TUI Snapshot Tests
//!
//! This module contains snapshot tests for TUI components using `insta`.
//!
//! Tests are organized into separate modules for different components:
//! - `error_tests` - Error type tests
//! - `event_tests` - Event type tests
//! - `secure_tests` - Secure string tests
//! - `wizard_snapshot_tests` - Wizard state machine snapshots
//! - `app_snapshot_tests` - TuiApp state and rendering snapshots
//! - `visual_regression_tests` - Visual rendering snapshots
//! - `screen_snapshot_tests` - Individual screen component snapshots
//! - `integration_tests` - Cross-component integration tests

// ============ Phase 1.2 新测试 ============
mod error_tests;
mod event_tests;
mod secure_tests;

// ============ 现有测试 ============
mod app_snapshot_tests;
mod integration_tests;
mod screen_snapshot_tests;
mod visual_regression_tests;
mod wizard_snapshot_tests;
