//! TUI Tag Configuration Widget
//!
//! Interactive widget for selecting credential tags in the terminal UI.

mod events;
mod render;
mod tests;
mod types;

// Re-export public types
pub use types::TagFocus;

use crate::tui::tags::config::{EnvTag, RiskTag, TagConfig};

/// Tag configuration widget for TUI
pub struct TagConfigWidget {
    /// Credential name being configured
    pub credential_name: String,
    /// Tag configuration state
    pub config: TagConfig,
    /// Selected environment tag index (0=dev, 1=test, 2=staging, 3=prod)
    pub selected_env: Option<usize>,
    /// Selected risk tag index (0=low, 1=medium, 2=high)
    pub selected_risk: Option<usize>,
    /// Whether to show advanced options
    pub show_advanced: bool,
    /// Current focus area
    pub focus: TagFocus,
    /// Selected custom tag index (for advanced section)
    pub selected_custom: Option<usize>,
}

impl TagConfigWidget {
    /// Create a new tag configuration widget
    ///
    /// # Arguments
    /// * `credential_name` - Name of the credential being configured
    pub fn new(credential_name: String) -> Self {
        Self {
            credential_name,
            config: TagConfig {
                env: None,
                risk: None,
                custom: Vec::new(),
            },
            selected_env: None,
            selected_risk: None,
            show_advanced: false,
            focus: TagFocus::Env,
            selected_custom: None,
        }
    }

    /// Create a new widget with existing tag configuration
    ///
    /// # Arguments
    /// * `credential_name` - Name of the credential being configured
    /// * `config` - Existing tag configuration to load
    pub fn with_config(credential_name: String, config: TagConfig) -> Self {
        let selected_env = config.env.map(|env| match env {
            EnvTag::Dev => 0,
            EnvTag::Test => 1,
            EnvTag::Staging => 2,
            EnvTag::Prod => 3,
        });

        let selected_risk = config.risk.map(|risk| match risk {
            RiskTag::Low => 0,
            RiskTag::Medium => 1,
            RiskTag::High => 2,
        });

        Self {
            credential_name,
            config,
            selected_env,
            selected_risk,
            show_advanced: false,
            focus: TagFocus::Env,
            selected_custom: None,
        }
    }

    /// Get the current tag configuration
    pub fn config(&self) -> &TagConfig {
        &self.config
    }

    /// Take the tag configuration (consuming self)
    pub fn into_config(self) -> TagConfig {
        self.config
    }

    /// Get the current focus area
    pub fn focus(&self) -> TagFocus {
        self.focus
    }

    /// Set the focus area
    pub fn set_focus(&mut self, focus: TagFocus) {
        self.focus = focus;
    }

    /// Check if configuration is ready to save
    pub fn can_save(&self) -> bool {
        // Require at least env tag to be set
        self.config.env.is_some()
    }

    /// Update the internal config from selections
    pub fn update_config(&mut self) {
        self.config.env = self.selected_env.and_then(|idx| match idx {
            0 => Some(EnvTag::Dev),
            1 => Some(EnvTag::Test),
            2 => Some(EnvTag::Staging),
            3 => Some(EnvTag::Prod),
            _ => None,
        });

        self.config.risk = self.selected_risk.and_then(|idx| match idx {
            0 => Some(RiskTag::Low),
            1 => Some(RiskTag::Medium),
            2 => Some(RiskTag::High),
            _ => None,
        });
    }
}

impl Default for TagConfigWidget {
    fn default() -> Self {
        Self::new("Unnamed Credential".to_string())
    }
}
