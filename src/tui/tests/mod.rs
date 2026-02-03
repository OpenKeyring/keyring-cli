//! TUI Snapshot Tests
//!
//! This module contains snapshot tests for TUI components using `insta`.
//!
//! Tests are organized into separate modules for different components:
//! - `wizard_snapshot_tests` - Wizard state machine snapshots
//! - `app_snapshot_tests` - TuiApp state and rendering snapshots
//! - `visual_regression_tests` - Visual rendering snapshots
//! - `screen_snapshot_tests` - Individual screen component snapshots
//! - `integration_tests` - Cross-component integration tests

mod app_snapshot_tests;
mod integration_tests;
mod screen_snapshot_tests;
mod visual_regression_tests;
mod wizard_snapshot_tests;
