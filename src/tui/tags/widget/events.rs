//! Event handling for TagConfigWidget
//!
//! Contains all keyboard event handling methods for the tag configuration widget.

use super::types::TagFocus;
use super::TagConfigWidget;

impl TagConfigWidget {
    /// Handle key up event
    pub fn on_key_up(&mut self) {
        match self.focus {
            TagFocus::Env => {
                if let Some(ref mut idx) = self.selected_env {
                    *idx = if *idx == 0 { 3 } else { *idx - 1 };
                } else {
                    self.selected_env = Some(0);
                }
                self.update_config();
            }
            TagFocus::Risk => {
                if let Some(ref mut idx) = self.selected_risk {
                    *idx = if *idx == 0 { 2 } else { *idx - 1 };
                } else {
                    self.selected_risk = Some(0);
                }
                self.update_config();
            }
            TagFocus::Advanced => {
                if !self.config.custom.is_empty() {
                    if let Some(ref mut idx) = self.selected_custom {
                        *idx = if *idx == 0 {
                            self.config.custom.len() - 1
                        } else {
                            *idx - 1
                        };
                    } else {
                        self.selected_custom = Some(0);
                    }
                }
            }
            TagFocus::Buttons => {}
        }
    }

    /// Handle key down event
    pub fn on_key_down(&mut self) {
        match self.focus {
            TagFocus::Env => {
                if let Some(ref mut idx) = self.selected_env {
                    *idx = (*idx + 1) % 4;
                } else {
                    self.selected_env = Some(0);
                }
                self.update_config();
            }
            TagFocus::Risk => {
                if let Some(ref mut idx) = self.selected_risk {
                    *idx = (*idx + 1) % 3;
                } else {
                    self.selected_risk = Some(0);
                }
                self.update_config();
            }
            TagFocus::Advanced => {
                if !self.config.custom.is_empty() {
                    if let Some(ref mut idx) = self.selected_custom {
                        *idx = (*idx + 1) % self.config.custom.len();
                    } else {
                        self.selected_custom = Some(0);
                    }
                }
            }
            TagFocus::Buttons => {}
        }
    }

    /// Handle key left event
    pub fn on_key_left(&mut self) {
        match self.focus {
            TagFocus::Risk => {
                self.focus = TagFocus::Env;
            }
            TagFocus::Advanced => {
                self.focus = TagFocus::Risk;
            }
            TagFocus::Buttons => {
                if self.show_advanced {
                    self.focus = TagFocus::Advanced;
                } else {
                    self.focus = TagFocus::Risk;
                }
            }
            TagFocus::Env => {}
        }
    }

    /// Handle key right event
    pub fn on_key_right(&mut self) {
        match self.focus {
            TagFocus::Env => {
                self.focus = TagFocus::Risk;
            }
            TagFocus::Risk => {
                if self.show_advanced {
                    self.focus = TagFocus::Advanced;
                } else {
                    self.focus = TagFocus::Buttons;
                }
            }
            TagFocus::Advanced => {
                self.focus = TagFocus::Buttons;
            }
            TagFocus::Buttons => {}
        }
    }

    /// Handle select/toggle event (Enter or Space)
    pub fn on_select(&mut self) {
        match self.focus {
            TagFocus::Env => {
                // Toggle selection
                if self.selected_env.is_some() {
                    // Already selected, could deselect or keep
                    // For now, keep selection
                } else {
                    self.selected_env = Some(0);
                }
                self.update_config();
            }
            TagFocus::Risk => {
                if self.selected_risk.is_some() {
                    // Already selected
                } else {
                    self.selected_risk = Some(0);
                }
                self.update_config();
            }
            TagFocus::Advanced => {
                // Select a custom tag (for deletion)
                if self.selected_custom.is_none() && !self.config.custom.is_empty() {
                    self.selected_custom = Some(0);
                }
            }
            TagFocus::Buttons => {
                // Trigger save action (handled by caller)
            }
        }
    }

    /// Toggle advanced options visibility
    pub fn toggle_advanced(&mut self) {
        self.show_advanced = !self.show_advanced;
        if self.show_advanced {
            self.focus = TagFocus::Advanced;
        } else {
            self.focus = TagFocus::Risk;
        }
    }

    /// Add a custom tag
    pub fn add_custom_tag(&mut self, tag: String) {
        if !tag.is_empty() && !self.config.custom.contains(&tag) {
            self.config.custom.push(tag);
            self.selected_custom = Some(self.config.custom.len() - 1);
        }
    }

    /// Remove the selected custom tag
    pub fn remove_selected_custom_tag(&mut self) {
        if let Some(idx) = self.selected_custom {
            if idx < self.config.custom.len() {
                self.config.custom.remove(idx);
                if self.config.custom.is_empty() {
                    self.selected_custom = None;
                } else if idx >= self.config.custom.len() {
                    self.selected_custom = Some(self.config.custom.len() - 1);
                }
            }
        }
    }
}
