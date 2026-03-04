//! Mock data layer for TUI development and testing
//!
//! This module provides a mock vault implementation that can be used
//! during UI development without connecting to the real database.

pub mod vault;
pub use vault::MockVault;
