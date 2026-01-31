use std::collections::HashSet;
use std::fmt;

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

impl EnvTag {
    /// Get the display name of this environment tag
    pub fn name(&self) -> &str {
        match self {
            EnvTag::Dev => "dev",
            EnvTag::Test => "test",
            EnvTag::Staging => "staging",
            EnvTag::Prod => "prod",
        }
    }

    /// Get the description of this environment tag
    pub fn description(&self) -> &str {
        match self {
            EnvTag::Dev => "开发环境 - 开发和测试",
            EnvTag::Test => "测试环境 - 集成测试",
            EnvTag::Staging => "预发布环境 - 生产前验证",
            EnvTag::Prod => "生产环境 - 线上环境",
        }
    }

    /// Get the tag string format
    pub fn tag_str(&self) -> String {
        format!("env:{}", self.name())
    }
}

impl fmt::Display for EnvTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "env:{}", self.name())
    }
}

impl RiskTag {
    /// Get the display name of this risk tag
    pub fn name(&self) -> &str {
        match self {
            RiskTag::Low => "low",
            RiskTag::Medium => "medium",
            RiskTag::High => "high",
        }
    }

    /// Get the description of this risk tag
    pub fn description(&self) -> &str {
        match self {
            RiskTag::Low => "低风险 - 开发/测试数据",
            RiskTag::Medium => "中风险 - 非关键生产数据",
            RiskTag::High => "高风险 - 关键生产数据",
        }
    }

    /// Get the tag string format
    pub fn tag_str(&self) -> String {
        format!("risk:{}", self.name())
    }
}

impl fmt::Display for RiskTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "risk:{}", self.name())
    }
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

        // Default behavior when no tags present
        if env_tags.is_empty() && risk_tags.is_empty() {
            return AuthDecision::SessionApprove;
        }

        // Check for strict contradiction: ONLY dev env with high risk
        // (if there are other env tags besides dev, we can use those instead)
        if env_tags.contains(&EnvTag::Dev)
            && risk_tags.contains(&RiskTag::High)
            && env_tags.len() == 1
        {
            return AuthDecision::Deny;
        }

        // If we have tags, evaluate all combinations and pick the most restrictive
        let envs_to_eval = if env_tags.is_empty() {
            vec![EnvTag::Dev]
        } else {
            env_tags.clone()
        };

        let risks_to_eval = if risk_tags.is_empty() {
            vec![RiskTag::Medium]
        } else {
            risk_tags.clone()
        };

        // Evaluate all valid combinations and return the most restrictive decision
        // Skip contradictory combinations (dev+high)
        let mut decisions = Vec::new();

        for env in &envs_to_eval {
            for risk in &risks_to_eval {
                // Skip contradictory combinations
                if *env == EnvTag::Dev && *risk == RiskTag::High {
                    continue;
                }

                let decision = Self::apply_policy_rules(*env, *risk, operation_type);
                decisions.push(decision);
            }
        }

        // If no valid decisions found (all were contradictions), deny
        if decisions.is_empty() {
            return AuthDecision::Deny;
        }

        // Return the most restrictive decision
        decisions
            .into_iter()
            .reduce(|a, b| Self::most_restrictive_decision(a, b))
            .unwrap_or(AuthDecision::SessionApprove)
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

    /// Get the most restrictive of two authorization decisions
    /// Order: Deny > AlwaysConfirm > SessionApprove > AutoApprove
    fn most_restrictive_decision(a: AuthDecision, b: AuthDecision) -> AuthDecision {
        match (a, b) {
            (AuthDecision::Deny, _) | (_, AuthDecision::Deny) => AuthDecision::Deny,
            (AuthDecision::AlwaysConfirm, _) | (_, AuthDecision::AlwaysConfirm) => {
                AuthDecision::AlwaysConfirm
            }
            (AuthDecision::SessionApprove, _) | (_, AuthDecision::SessionApprove) => {
                AuthDecision::SessionApprove
            }
            (AuthDecision::AutoApprove, AuthDecision::AutoApprove) => AuthDecision::AutoApprove,
        }
    }
}

impl PolicyEngine {
    /// Make an authorization decision directly from env and risk tags
    ///
    /// This is a convenience method for the tag configuration dialog to preview
    /// what policy will be applied based on the selected tags.
    ///
    /// # Arguments
    /// * `env` - Optional environment tag
    /// * `risk` - Optional risk tag
    /// * `operation` - Type of operation (Read or Write)
    ///
    /// # Returns
    /// * `AuthDecision` - The authorization decision
    pub fn decide_from_config(
        env: Option<EnvTag>,
        risk: Option<RiskTag>,
        operation: OperationType,
    ) -> AuthDecision {
        // Convert env/risk to tag strings
        let mut tags = HashSet::new();

        if let Some(env) = env {
            tags.insert(env.to_string());
        }

        if let Some(risk) = risk {
            tags.insert(risk.to_string());
        }

        let engine = Self::new();
        engine.decide(&tags, operation, "tool")
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
