//! Password strength types for master password screen
//!
//! Contains types for representing password strength levels.

use ratatui::style::Color;

/// Password strength indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// Weak password
    Weak,
    /// Medium password
    Medium,
    /// Strong password
    Strong,
}

impl PasswordStrength {
    /// Get display text for this strength level
    pub fn display(&self) -> &str {
        match self {
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Medium => "Medium",
            PasswordStrength::Strong => "Strong",
        }
    }

    /// Get color for this strength level
    pub fn color(&self) -> Color {
        match self {
            PasswordStrength::Weak => Color::Red,
            PasswordStrength::Medium => Color::Yellow,
            PasswordStrength::Strong => Color::Green,
        }
    }

    /// Get icon for this strength level
    pub fn icon(&self) -> &str {
        match self {
            PasswordStrength::Weak => "⚠️",
            PasswordStrength::Medium => "🔒",
            PasswordStrength::Strong => "🔐",
        }
    }
}
