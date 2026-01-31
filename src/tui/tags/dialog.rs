//! Policy preview dialog for tag configuration
//!
//! This module provides a dialog that shows users what authorization policy
//! will be applied based on their tag configuration.

use crate::mcp::policy::policy::{AuthDecision, EnvTag, PolicyEngine, RiskTag, OperationType};
use crate::error::Error;

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
        Self { decision, env, risk }
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
             授权策略预览\n\
            ══════════════════════════════════════\n\
             \n\
             标签配置:\n\
             {}\n\
             {}\n\
             \n\
             {}\n\
             \n\
             确认保存此配置？",
            self.format_tag("环境", self.env.as_ref().map(|e| e.to_string())),
            self.format_tag("风险", self.risk.as_ref().map(|r| r.to_string())),
            self.format_decision()
        )
    }

    /// Format a tag label and value
    fn format_tag(&self, label: &str, value: Option<String>) -> String {
        match value {
            Some(v) => format!("  {}: {}", label, v),
            None => format!("  {}: (未设置)", label),
        }
    }

    /// Format the authorization decision for display
    fn format_decision(&self) -> String {
        match self.decision {
            AuthDecision::AutoApprove => {
                "  ✓ 自动授权\n\
                  \n\
                  AI 调用此凭证时将自动执行操作，无需任何用户确认。".to_string()
            }
            AuthDecision::SessionApprove => {
                "  ✓ 会话级授权\n\
                  \n\
                  • 首次 AI 调用时需要用户确认\n\
                  • 确认后 1 小时内自动授权\n\
                  • 1 小时后需要重新确认".to_string()
            }
            AuthDecision::AlwaysConfirm => {
                "  ⚠ 每次确认\n\
                  \n\
                  • 每次 AI 调用都需要用户确认\n\
                  • 适用于生产环境或高风险操作".to_string()
            }
            AuthDecision::Deny => {
                "  ⊘ 拒绝执行\n\
                  \n\
                  • AI 将无法使用此凭证执行任何操作".to_string()
            }
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
        assert!(text.contains("自动授权"));
        assert!(text.contains("无需任何用户确认"));
    }

    #[test]
    fn test_format_decision_session_approve() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::SessionApprove,
            env: Some(EnvTag::Dev),
            risk: Some(RiskTag::Medium),
        };

        let text = dialog.format_decision();
        assert!(text.contains("会话级授权"));
        assert!(text.contains("首次 AI 调用时需要用户确认"));
        assert!(text.contains("1 小时内自动授权"));
    }

    #[test]
    fn test_format_decision_always_confirm() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::AlwaysConfirm,
            env: Some(EnvTag::Prod),
            risk: Some(RiskTag::Low),
        };

        let text = dialog.format_decision();
        assert!(text.contains("每次确认"));
        assert!(text.contains("每次 AI 调用都需要用户确认"));
    }

    #[test]
    fn test_format_decision_deny() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::Deny,
            env: Some(EnvTag::Dev),
            risk: Some(RiskTag::High),
        };

        let text = dialog.format_decision();
        assert!(text.contains("拒绝执行"));
        assert!(text.contains("AI 将无法使用此凭证"));
    }

    #[test]
    fn test_format_tag_with_value() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::AutoApprove,
            env: Some(EnvTag::Dev),
            risk: Some(RiskTag::Low),
        };

        let text = dialog.format_tag("环境", Some("env:dev".to_string()));
        assert_eq!(text, "  环境: env:dev");
    }

    #[test]
    fn test_format_tag_without_value() {
        let dialog = PolicyPreviewDialog {
            decision: AuthDecision::SessionApprove,
            env: None,
            risk: None,
        };

        let text = dialog.format_tag("环境", None);
        assert_eq!(text, "  环境: (未设置)");
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
