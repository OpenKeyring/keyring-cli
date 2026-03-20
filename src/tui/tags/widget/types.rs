//! Widget types
//!
//! Type definitions for the tag configuration widget.

/// Focus area for the tag configuration widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagFocus {
    /// Focus on environment tag selection
    Env,
    /// Focus on risk tag selection
    Risk,
    /// Focus on advanced options (custom tags)
    Advanced,
    /// Focus on buttons
    Buttons,
}
