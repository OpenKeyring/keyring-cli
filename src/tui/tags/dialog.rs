//! Policy preview dialog for tag configuration
//!
//! This module provides a dialog that shows users what authorization policy
//! will be applied based on their tag configuration.

use crate::error::Error;
use crate::mcp::policy::policy::{AuthDecision, EnvTag, OperationType, PolicyEngine, RiskTag};

/// Policy preview dialog for tag configuration
pub struct PolicyPreviewDialog {
    decision: AuthDecision,
    env: Option<EnvTag>,
    risk: Option<RiskTag>,
}

impl PolicyPreviewDialog {
    /// Create a new policy preview dialog
    ///
    /// # Arguments
    /// * `env` - Optional environment tag
    /// * `risk` - Optional risk tag
    pub fn new(env: Option<EnvTag>, risk: Option<RiskTag>) -> Self {
        // Determine policy based on tags
        let decision = PolicyEngine::decide_from_config(env, risk, OperationType::Write);
        Self {
            decision,
            env,
            risk,
        }
    }

    /// Show the policy preview dialog and get user confirmation
    ///
    /// # Returns
    /// * `Ok(true)` - User confirmed the policy
    /// * `Ok(false)` - User rejected the policy
    /// * `Err(Error)` - Dialog interaction failed
    pub fn show(&self) -> Result<bool, Error> {
        let policy_text = self.get_policy_text();

        let confirmed = dialoguer::Confirm::new()
            .with_prompt(&policy_text)
            .default(false)
            .interact()
            .map_err(|e| Error::IoError(format!("Failed to show policy preview dialog: {}", e)))?;

        Ok(confirmed)
    }

    /// Get the formatted policy text for display
    fn get_policy_text(&self) -> String {
        format!(
            "═══════════════════════════════════════\n\
             Authorization Policy Preview\n\
            ══════════════════════════════════════\n\
             \n\
             Tag Configuration:\n\
             {}\n\
             {}\n\
             \n\
             {}\n\
             \n\
             Confirm saving this configuration?",
            self.format_tag("Environment", self.env.as_ref().map(|e| e.to_string())),
            self.format_tag("Risk", self.risk.as_ref().map(|r| r.to_string())),
            self.format_decision()
        )
    }

    /// Format a tag label and value
    fn format_tag(&self, label: &str, value: Option<String>) -> String {
        match value {
            Some(v) => format!("  {}: {}", label, v),
            None => format!("  {}: (not set)", label),
        }
    }

    /// Format the authorization decision for display
    fn format_decision(&self) -> String {
        match self.decision {
            AuthDecision::AutoApprove => "  ✓ Auto Approve\n\
                  \n\
                  Operations using this credential will be executed automatically without any user confirmation."
                .to_string(),
            AuthDecision::SessionApprove => "  ✓ Session-level Approval\n\
                  \n\
                  • User confirmation required on first AI call\n\
                  • Auto-approved within 1 hour after confirmation\n\
                  • Re-confirmation required after 1 hour"
                .to_string(),
            AuthDecision::AlwaysConfirm => "  ⚠ Always Confirm\n\
                  \n\
                  • User confirmation required for every AI call\n\
                  • Suitable for production environments or high-risk operations"
                .to_string(),
            AuthDecision::Deny => "  ⊘ Deny\n\
                  \n\
                  • AI will not be able to use this credential for any operations"
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_decision_auto_approve() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::AutoApprove,
            env: Some(EnvTag::Dev),
            risk: Some(RiskTag::Low),
        };

        let text = dialog.format_decision();
        assert!(text.contains("Auto Approve"));
        assert!(text.contains("without any user confirmation"));
    }

    #[test]
    fn test_format_decision_session_approve() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::SessionApprove,
            env: Some(EnvTag::Dev),
            risk: Some(RiskTag::Medium),
        };

        let text = dialog.format_decision();
        assert!(text.contains("Session-level Approval"));
        assert!(text.contains("User confirmation required on first AI call"));
        assert!(text.contains("Auto-approved within 1 hour"));
    }

    #[test]
    fn test_format_decision_always_confirm() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::AlwaysConfirm,
            env: Some(EnvTag::Prod),
            risk: Some(RiskTag::Low),
        };

        let text = dialog.format_decision();
        assert!(text.contains("Always Confirm"));
        assert!(text.contains("User confirmation required for every AI call"));
    }

    #[test]
    fn test_format_decision_deny() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::Deny,
            env: Some(EnvTag::Dev),
            risk: Some(RiskTag::High),
        };

        let text = dialog.format_decision();
        assert!(text.contains("Deny"));
        assert!(text.contains("AI will not be able to use"));
    }

    #[test]
    fn test_format_tag_with_value() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::AutoApprove,
            env: Some(EnvTag::Dev),
            risk: Some(RiskTag::Low),
        };

        let text = dialog.format_tag("Environment", Some("env:dev".to_string()));
        assert_eq!(text, "  Environment: env:dev");
    }

    #[test]
    fn test_format_tag_without_value() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::SessionApprove,
            env: None,
            risk: None,
        };

        let text = dialog.format_tag("Environment", None);
        assert_eq!(text, "  Environment: (not set)");
    }

    #[test]
    fn test_new_with_env_and_risk() {
        let dialog = PolicyPreviewDialog::new(Some(EnvTag::Dev), Some(RiskTag::Low));

        // dev + low should be AutoApprove for Write operations
        assert_eq!(dialog.decision, AuthDecision::AutoApprove);
        assert_eq!(dialog.env, Some(EnvTag::Dev));
        assert_eq!(dialog.risk, Some(RiskTag::Low));
    }

    #[test]
    fn test_new_with_prod() {
        let dialog = PolicyPreviewDialog::new(Some(EnvTag::Prod), Some(RiskTag::Low));

        // prod should always be AlwaysConfirm
        assert_eq!(dialog.decision, AuthDecision::AlwaysConfirm);
        assert_eq!(dialog.env, Some(EnvTag::Prod));
        assert_eq!(dialog.risk, Some(RiskTag::Low));
    }

    #[test]
    fn test_new_with_dev_high() {
        let dialog = PolicyPreviewDialog::new(Some(EnvTag::Dev), Some(RiskTag::High));

        // dev + high should be Deny
        assert_eq!(dialog.decision, AuthDecision::Deny);
        assert_eq!(dialog.env, Some(EnvTag::Dev));
        assert_eq!(dialog.risk, Some(RiskTag::High));
    }

    #[test]
    fn test_new_with_no_tags() {
        let dialog = PolicyPreviewDialog::new(None, None);

        // no tags should default to SessionApprove
        assert_eq!(dialog.decision, AuthDecision::SessionApprove);
        assert_eq!(dialog.env, None);
        assert_eq!(dialog.risk, None);
    }
}
