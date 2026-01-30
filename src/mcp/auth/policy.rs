use std::collections::HashSet;

/// Authorization decision based on credential tags and operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthDecision {
    /// No confirmation needed - automatically approved
    AutoApprove,
    /// First time confirms, then cached for 1 hour
    SessionApprove,
    /// Every call requires confirmation
    AlwaysConfirm,
    /// Reject the operation
    Deny,
}

/// Operation type for policy decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// List credentials, check connection
    Read,
    /// Exec, push, delete, etc.
    Write,
}

/// Environment tag extracted from credential tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvTag {
    Dev,
    Test,
    Staging,
    Prod,
}

/// Risk level tag extracted from credential tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskTag {
    Low,
    Medium,
    High,
}

/// Policy engine for making authorization decisions
#[derive(Debug, Clone)]
pub struct PolicyEngine;

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self
    }

    /// Make an authorization decision based on credential tags, operation type, and tool
    ///
    /// # Arguments
    /// * `tags` - Set of tags associated with the credential
    /// * `operation_type` - Type of operation (Read or Write)
    /// * `_tool` - Tool being used (for future use)
    ///
    /// # Returns
    /// * `AuthDecision` - The authorization decision
    pub fn decide(
        &self,
        tags: &HashSet<String>,
        operation_type: OperationType,
        _tool: &str,
    ) -> AuthDecision {
        // Extract env and risk tags
        let env_tags = Self::extract_env_tags(tags);
        let risk_tags = Self::extract_risk_tags(tags);

        // Handle contradictory tags
        if Self::has_contradictory_tags(&env_tags, &risk_tags) {
            return AuthDecision::Deny;
        }

        // Get the most restrictive env and risk tags
        let env_tag = Self::get_most_restrictive_env(&env_tags);
        let risk_tag = Self::get_most_restrictive_risk(&risk_tags);

        // Default behavior when no tags present
        let env_tag = env_tag.unwrap_or(EnvTag::Dev);
        let risk_tag = risk_tag.unwrap_or(RiskTag::Medium);

        // Apply policy rules
        Self::apply_policy_rules(env_tag, risk_tag, operation_type)
    }

    /// Apply the core policy rules based on env, risk, and operation type
    fn apply_policy_rules(env: EnvTag, risk: RiskTag, _operation: OperationType) -> AuthDecision {
        match (env, risk) {
            // env:dev + risk:low → AutoApprove
            (EnvTag::Dev, RiskTag::Low) => AuthDecision::AutoApprove,

            // env:dev + risk:medium → SessionApprove
            (EnvTag::Dev, RiskTag::Medium) => AuthDecision::SessionApprove,

            // env:dev + risk:high → Deny (contradictory: dev environment shouldn't be high risk)
            (EnvTag::Dev, RiskTag::High) => AuthDecision::Deny,

            // env:test + risk:low → AutoApprove
            (EnvTag::Test, RiskTag::Low) => AuthDecision::AutoApprove,

            // env:test + risk:medium → SessionApprove
            (EnvTag::Test, RiskTag::Medium) => AuthDecision::SessionApprove,

            // env:test + risk:high → SessionApprove (allow but require confirmation)
            (EnvTag::Test, RiskTag::High) => AuthDecision::SessionApprove,

            // env:staging + risk:low → SessionApprove
            (EnvTag::Staging, RiskTag::Low) => AuthDecision::SessionApprove,

            // env:staging + risk:medium → AlwaysConfirm
            (EnvTag::Staging, RiskTag::Medium) => AuthDecision::AlwaysConfirm,

            // env:staging + risk:high → AlwaysConfirm
            (EnvTag::Staging, RiskTag::High) => AuthDecision::AlwaysConfirm,

            // env:prod + any risk → AlwaysConfirm (production always requires confirmation)
            (EnvTag::Prod, _) => AuthDecision::AlwaysConfirm,
        }
    }

    /// Extract all environment tags from the tag set
    fn extract_env_tags(tags: &HashSet<String>) -> Vec<EnvTag> {
        tags.iter()
            .filter_map(|tag| {
                if tag == "env:dev" {
                    Some(EnvTag::Dev)
                } else if tag == "env:test" {
                    Some(EnvTag::Test)
                } else if tag == "env:staging" {
                    Some(EnvTag::Staging)
                } else if tag == "env:prod" {
                    Some(EnvTag::Prod)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract all risk tags from the tag set
    fn extract_risk_tags(tags: &HashSet<String>) -> Vec<RiskTag> {
        tags.iter()
            .filter_map(|tag| {
                if tag == "risk:low" {
                    Some(RiskTag::Low)
                } else if tag == "risk:medium" {
                    Some(RiskTag::Medium)
                } else if tag == "risk:high" {
                    Some(RiskTag::High)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the most restrictive environment tag
    /// Order: Prod > Staging > Test > Dev
    fn get_most_restrictive_env(env_tags: &[EnvTag]) -> Option<EnvTag> {
        if env_tags.is_empty() {
            return None;
        }

        // Check for prod first
        if env_tags.contains(&EnvTag::Prod) {
            return Some(EnvTag::Prod);
        }

        // Then staging
        if env_tags.contains(&EnvTag::Staging) {
            return Some(EnvTag::Staging);
        }

        // Then test
        if env_tags.contains(&EnvTag::Test) {
            return Some(EnvTag::Test);
        }

        // Finally dev
        if env_tags.contains(&EnvTag::Dev) {
            return Some(EnvTag::Dev);
        }

        None
    }

    /// Get the most restrictive risk tag
    /// Order: High > Medium > Low
    fn get_most_restrictive_risk(risk_tags: &[RiskTag]) -> Option<RiskTag> {
        if risk_tags.is_empty() {
            return None;
        }

        // Check for high first
        if risk_tags.contains(&RiskTag::High) {
            return Some(RiskTag::High);
        }

        // Then medium
        if risk_tags.contains(&RiskTag::Medium) {
            return Some(RiskTag::Medium);
        }

        // Then low
        if risk_tags.contains(&RiskTag::Low) {
            return Some(RiskTag::Low);
        }

        None
    }

    /// Check for contradictory tags
    /// Current contradiction: env:dev + risk:high
    fn has_contradictory_tags(env_tags: &[EnvTag], risk_tags: &[RiskTag]) -> bool {
        // dev environment with high risk is contradictory
        env_tags.contains(&EnvTag::Dev) && risk_tags.contains(&RiskTag::High)
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tags(tags: &[&str]) -> HashSet<String> {
        tags.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_auto_approve_dev_low() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:dev", "risk:low"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::AutoApprove);
    }

    #[test]
    fn test_session_approve_dev_medium() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:dev", "risk:medium"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::SessionApprove);
    }

    #[test]
    fn test_deny_dev_high() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:dev", "risk:high"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::Deny);
    }

    #[test]
    fn test_auto_approve_test_low() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:test", "risk:low"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::AutoApprove);
    }

    #[test]
    fn test_session_approve_test_medium() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:test", "risk:medium"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::SessionApprove);
    }

    #[test]
    fn test_session_approve_staging_low() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:staging", "risk:low"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::SessionApprove);
    }

    #[test]
    fn test_always_confirm_staging_high() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:staging", "risk:high"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::AlwaysConfirm);
    }

    #[test]
    fn test_always_confirm_prod_low() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:prod", "risk:low"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::AlwaysConfirm);
    }

    #[test]
    fn test_always_confirm_prod_medium() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:prod", "risk:medium"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::AlwaysConfirm);
    }

    #[test]
    fn test_always_confirm_prod_high() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:prod", "risk:high"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::AlwaysConfirm);
    }

    #[test]
    fn test_default_no_tags() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&[]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::SessionApprove);
    }

    #[test]
    fn test_most_restrictive_env_multiple_env_tags() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:dev", "env:prod", "risk:low"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        // Should use prod (most restrictive)
        assert_eq!(decision, AuthDecision::AlwaysConfirm);
    }

    #[test]
    fn test_most_restrictive_risk_multiple_risk_tags() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:dev", "risk:low", "risk:high"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        // Should use high (most restrictive) → Deny
        assert_eq!(decision, AuthDecision::Deny);
    }

    #[test]
    fn test_partial_tags_only_env() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:dev"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        // Default risk:medium
        assert_eq!(decision, AuthDecision::SessionApprove);
    }

    #[test]
    fn test_partial_tags_only_risk() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["risk:low"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        // Default env:dev
        assert_eq!(decision, AuthDecision::AutoApprove);
    }

    #[test]
    fn test_write_operation_same_as_read() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:prod", "risk:low"]);
        let decision = engine.decide(&tags, OperationType::Write, "exec_tool");
        assert_eq!(decision, AuthDecision::AlwaysConfirm);
    }

    #[test]
    fn test_non_policy_tags_ignored() {
        let engine = PolicyEngine::new();
        let tags = make_tags(&["env:dev", "risk:low", "category:database", "owner:team-a"]);
        let decision = engine.decide(&tags, OperationType::Read, "test_tool");
        assert_eq!(decision, AuthDecision::AutoApprove);
    }
}
