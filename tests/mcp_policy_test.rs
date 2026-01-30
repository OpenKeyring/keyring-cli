use keyring_cli::mcp::auth::{AuthDecision, EnvTag, OperationType, PolicyEngine, RiskTag};
use std::collections::HashSet;

/// Helper function to create a tag set from string slices
fn make_tags(tags: &[&str]) -> HashSet<String> {
    tags.iter().map(|s| s.to_string()).collect()
}

/// Helper function to create a policy engine
fn make_engine() -> PolicyEngine {
    PolicyEngine::new()
}

// ============================================================================
// Basic Policy Rule Tests
// ============================================================================

#[test]
fn test_auto_approve_dev_low_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AutoApprove,
        "dev+low should auto-approve"
    );
}

#[test]
fn test_session_approve_dev_medium_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "risk:medium"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "dev+medium should require session approval"
    );
}

#[test]
fn test_deny_dev_high_risk_contradiction() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "risk:high"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::Deny,
        "dev+high is contradictory and should deny"
    );
}

#[test]
fn test_auto_approve_test_low_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:test", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AutoApprove,
        "test+low should auto-approve"
    );
}

#[test]
fn test_session_approve_test_medium_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:test", "risk:medium"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "test+medium should require session approval"
    );
}

#[test]
fn test_session_approve_test_high_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:test", "risk:high"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "test+high should require session approval"
    );
}

#[test]
fn test_session_approve_staging_low_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:staging", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "staging+low should require session approval"
    );
}

#[test]
fn test_always_confirm_staging_high_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:staging", "risk:high"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AlwaysConfirm,
        "staging+high should always confirm"
    );
}

#[test]
fn test_always_confirm_prod_low_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:prod", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AlwaysConfirm,
        "prod+low should always confirm"
    );
}

#[test]
fn test_always_confirm_prod_medium_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:prod", "risk:medium"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AlwaysConfirm,
        "prod+medium should always confirm"
    );
}

#[test]
fn test_always_confirm_prod_high_risk() {
    let engine = make_engine();
    let tags = make_tags(&["env:prod", "risk:high"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AlwaysConfirm,
        "prod+high should always confirm"
    );
}

// ============================================================================
// Default Behavior Tests
// ============================================================================

#[test]
fn test_default_no_tags_session_approve() {
    let engine = make_engine();
    let tags = make_tags(&[]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "no tags should default to session approve"
    );
}

#[test]
fn test_default_only_env_tag() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "only env:dev with default risk:medium should be session approve"
    );
}

#[test]
fn test_default_only_risk_low_tag() {
    let engine = make_engine();
    let tags = make_tags(&["risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AutoApprove,
        "only risk:low with default env:dev should auto-approve"
    );
}

#[test]
fn test_default_only_risk_high_tag() {
    let engine = make_engine();
    let tags = make_tags(&["risk:high"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::Deny,
        "only risk:high with default env:dev should deny"
    );
}

// ============================================================================
// Multiple Tags (Most Restrictive) Tests
// ============================================================================

#[test]
fn test_multiple_env_tags_uses_most_restrictive() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "env:test", "env:staging", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "multiple env tags should use staging (most restrictive)"
    );
}

#[test]
fn test_multiple_env_tags_with_prod() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "env:prod", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AlwaysConfirm,
        "multiple env tags with prod should use prod"
    );
}

#[test]
fn test_multiple_risk_tags_uses_most_restrictive() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "risk:low", "risk:medium"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "multiple risk tags should use medium (most restrictive)"
    );
}

#[test]
fn test_multiple_risk_tags_with_high() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "risk:low", "risk:high"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::Deny,
        "multiple risk tags with high should use high and deny"
    );
}

#[test]
fn test_multiple_both_env_and_risk_tags() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "env:test", "risk:low", "risk:high"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "test+high should session approve (not dev+high which would deny)"
    );
}

// ============================================================================
// Operation Type Tests
// ============================================================================

#[test]
fn test_read_operation() {
    let engine = make_engine();
    let tags = make_tags(&["env:prod", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "list_tool");
    assert_eq!(
        decision,
        AuthDecision::AlwaysConfirm,
        "read operation on prod should always confirm"
    );
}

#[test]
fn test_write_operation() {
    let engine = make_engine();
    let tags = make_tags(&["env:prod", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Write, "exec_tool");
    assert_eq!(
        decision,
        AuthDecision::AlwaysConfirm,
        "write operation on prod should always confirm"
    );
}

// ============================================================================
// Edge Cases and Additional Tags Tests
// ============================================================================

#[test]
fn test_non_policy_tags_ignored() {
    let engine = make_engine();
    let tags = make_tags(&[
        "env:dev",
        "risk:low",
        "category:database",
        "owner:team-a",
        "project:myapp",
        "region:us-west-2",
    ]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AutoApprove,
        "non-policy tags should be ignored"
    );
}

#[test]
fn test_malformed_tags_ignored() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "risk:low", "invalid-tag", "another:tag:format"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AutoApprove,
        "malformed tags should be ignored"
    );
}

#[test]
fn test_case_sensitive_tags() {
    let engine = make_engine();
    let tags = make_tags(&["ENV:DEV", "RISK:LOW"]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    // These should not match (case-sensitive), so default to SessionApprove
    assert_eq!(
        decision,
        AuthDecision::SessionApprove,
        "uppercase tags should not match and should use defaults"
    );
}

#[test]
fn test_empty_string_tag_ignored() {
    let engine = make_engine();
    let tags = make_tags(&["env:dev", "risk:low", ""]);
    let decision = engine.decide(&tags, OperationType::Read, "any_tool");
    assert_eq!(
        decision,
        AuthDecision::AutoApprove,
        "empty string tags should be ignored"
    );
}

// ============================================================================
// All Environment Tag Variations
// ============================================================================

#[test]
fn test_all_env_with_risk_low() {
    let engine = make_engine();

    // env:dev + risk:low → AutoApprove
    let tags = make_tags(&["env:dev", "risk:low"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::AutoApprove
    );

    // env:test + risk:low → AutoApprove
    let tags = make_tags(&["env:test", "risk:low"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::AutoApprove
    );

    // env:staging + risk:low → SessionApprove
    let tags = make_tags(&["env:staging", "risk:low"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::SessionApprove
    );

    // env:prod + risk:low → AlwaysConfirm
    let tags = make_tags(&["env:prod", "risk:low"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::AlwaysConfirm
    );
}

#[test]
fn test_all_env_with_risk_medium() {
    let engine = make_engine();

    // env:dev + risk:medium → SessionApprove
    let tags = make_tags(&["env:dev", "risk:medium"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::SessionApprove
    );

    // env:test + risk:medium → SessionApprove
    let tags = make_tags(&["env:test", "risk:medium"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::SessionApprove
    );

    // env:staging + risk:medium → AlwaysConfirm
    let tags = make_tags(&["env:staging", "risk:medium"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::AlwaysConfirm
    );

    // env:prod + risk:medium → AlwaysConfirm
    let tags = make_tags(&["env:prod", "risk:medium"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::AlwaysConfirm
    );
}

#[test]
fn test_all_env_with_risk_high() {
    let engine = make_engine();

    // env:dev + risk:high → Deny
    let tags = make_tags(&["env:dev", "risk:high"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::Deny
    );

    // env:test + risk:high → SessionApprove
    let tags = make_tags(&["env:test", "risk:high"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::SessionApprove
    );

    // env:staging + risk:high → AlwaysConfirm
    let tags = make_tags(&["env:staging", "risk:high"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::AlwaysConfirm
    );

    // env:prod + risk:high → AlwaysConfirm
    let tags = make_tags(&["env:prod", "risk:high"]);
    assert_eq!(
        engine.decide(&tags, OperationType::Read, "tool"),
        AuthDecision::AlwaysConfirm
    );
}

// ============================================================================
// Tool Parameter Tests
// ============================================================================

#[test]
fn test_different_tool_names_same_decision() {
    let engine = make_engine();
    let tags = make_tags(&["env:prod", "risk:low"]);

    let tools = vec!["ssh", "api", "git", "exec", "list"];
    for tool in tools {
        let decision = engine.decide(&tags, OperationType::Read, tool);
        assert_eq!(
            decision,
            AuthDecision::AlwaysConfirm,
            "tool name should not affect policy decision"
        );
    }
}

// ============================================================================
// Policy Engine Reusability Tests
// ============================================================================

#[test]
fn test_engine_reusable_across_decisions() {
    let engine = make_engine();

    // First decision
    let tags1 = make_tags(&["env:dev", "risk:low"]);
    let decision1 = engine.decide(&tags1, OperationType::Read, "tool1");
    assert_eq!(decision1, AuthDecision::AutoApprove);

    // Second decision with different tags
    let tags2 = make_tags(&["env:prod", "risk:high"]);
    let decision2 = engine.decide(&tags2, OperationType::Read, "tool2");
    assert_eq!(decision2, AuthDecision::AlwaysConfirm);

    // Third decision back to low risk
    let tags3 = make_tags(&["env:test", "risk:low"]);
    let decision3 = engine.decide(&tags3, OperationType::Read, "tool3");
    assert_eq!(decision3, AuthDecision::AutoApprove);
}

// ============================================================================
// Default Trait Implementation Tests
// ============================================================================

#[test]
fn test_policy_engine_default() {
    let engine = PolicyEngine::default();
    let tags = make_tags(&["env:dev", "risk:low"]);
    let decision = engine.decide(&tags, OperationType::Read, "tool");
    assert_eq!(decision, AuthDecision::AutoApprove);
}
