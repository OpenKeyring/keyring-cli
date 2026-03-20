//! Wizard Step Traits
//!
//! Traits for wizard step validation and state synchronization.

use crate::tui::screens::wizard::WizardState;

/// Trait for validating wizard step input
///
/// Screens that are part of a wizard flow should implement this trait
/// to provide validation and state synchronization capabilities.
pub trait WizardStepValidator {
    /// Check if the current step is valid and can proceed
    fn validate_step(&self) -> bool;

    /// Get the validation error message, if any
    fn validation_error(&self) -> Option<String>;

    /// Sync the screen's data to the wizard state
    fn sync_to_state(&self, state: &mut WizardState);

    /// Load the screen's data from the wizard state
    fn load_from_state(&mut self, state: &WizardState);

    /// Clear the screen's input fields
    fn clear_input(&mut self);
}

/// Trait for wizard step screen metadata
///
/// Provides basic information about a wizard step screen.
pub trait WizardStepScreen {
    /// Get the name of this wizard step
    fn step_name(&self) -> &'static str;

    /// Check if the screen has any user changes
    fn has_changes(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test struct to verify trait definitions compile
    struct TestStep {
        value: String,
    }

    impl WizardStepValidator for TestStep {
        fn validate_step(&self) -> bool {
            !self.value.is_empty()
        }

        fn validation_error(&self) -> Option<String> {
            if self.value.is_empty() {
                Some("Value is required".to_string())
            } else {
                None
            }
        }

        fn sync_to_state(&self, _state: &mut WizardState) {
            // Test implementation - no-op
        }

        fn load_from_state(&mut self, _state: &WizardState) {
            // Test implementation - no-op
        }

        fn clear_input(&mut self) {
            self.value.clear();
        }
    }

    impl WizardStepScreen for TestStep {
        fn step_name(&self) -> &'static str {
            "Test Step"
        }

        fn has_changes(&self) -> bool {
            !self.value.is_empty()
        }
    }

    #[test]
    fn test_wizard_step_validator() {
        let step = TestStep {
            value: String::new(),
        };
        assert!(!step.validate_step());
        assert!(step.validation_error().is_some());

        let step = TestStep {
            value: "test".to_string(),
        };
        assert!(step.validate_step());
        assert!(step.validation_error().is_none());
    }

    #[test]
    fn test_wizard_step_screen() {
        let step = TestStep {
            value: String::new(),
        };
        assert_eq!(step.step_name(), "Test Step");
        assert!(!step.has_changes());

        let step = TestStep {
            value: "test".to_string(),
        };
        assert!(step.has_changes());
    }

    #[test]
    fn test_clear_input() {
        let mut step = TestStep {
            value: "test".to_string(),
        };
        step.clear_input();
        assert!(step.value.is_empty());
    }
}
