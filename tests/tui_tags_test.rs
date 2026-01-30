use keyring_cli::tui::tags::config::{EnvTag, RiskTag, TagConfig, TagError, validate_tag_config};

#[test]
fn test_env_tag_to_string() {
    assert_eq!(EnvTag::Dev.to_string(), "env:dev");
    assert_eq!(EnvTag::Test.to_string(), "env:test");
    assert_eq!(EnvTag::Staging.to_string(), "env:staging");
    assert_eq!(EnvTag::Prod.to_string(), "env:prod");
}

#[test]
fn test_env_tag_display_name() {
    assert_eq!(EnvTag::Dev.display_name(), "dev (开发环境)");
    assert_eq!(EnvTag::Test.display_name(), "test (测试环境)");
    assert_eq!(EnvTag::Staging.display_name(), "staging (预发布环境)");
    assert_eq!(EnvTag::Prod.display_name(), "prod (生产环境)");
}

#[test]
fn test_env_tag_description() {
    assert_eq!(EnvTag::Dev.description(), "本地开发环境，会话级授权");
    assert_eq!(EnvTag::Test.description(), "测试环境，会话级授权");
    assert_eq!(EnvTag::Staging.description(), "预发布环境，会话级授权");
    assert_eq!(EnvTag::Prod.description(), "生产环境，每次需要确认 ⚠️");
}

#[test]
fn test_risk_tag_to_string() {
    assert_eq!(RiskTag::Low.to_string(), "risk:low");
    assert_eq!(RiskTag::Medium.to_string(), "risk:medium");
    assert_eq!(RiskTag::High.to_string(), "risk:high");
}

#[test]
fn test_risk_tag_display_name() {
    assert_eq!(RiskTag::Low.display_name(), "low (低风险)");
    assert_eq!(RiskTag::Medium.display_name(), "medium (中风险)");
    assert_eq!(RiskTag::High.display_name(), "high (高风险)");
}

#[test]
fn test_risk_tag_description() {
    assert_eq!(RiskTag::Low.description(), "只读操作，会话级授权");
    assert_eq!(RiskTag::Medium.description(), "读写操作，需确认");
    assert_eq!(RiskTag::High.description(), "危险操作，每次确认 ⚠️");
}

#[test]
fn test_validate_tag_config_valid() {
    let config = TagConfig {
        env: Some(EnvTag::Dev),
        risk: Some(RiskTag::Low),
        custom: vec!["team:backend".to_string()],
    };
    assert!(validate_tag_config(&config).is_ok());
}

#[test]
fn test_validate_tag_config_prod_with_low_risk() {
    let config = TagConfig {
        env: Some(EnvTag::Prod),
        risk: Some(RiskTag::Low),
        custom: vec![],
    };
    let result = validate_tag_config(&config);
    assert!(result.is_err());
    match result {
        Err(TagError::Contradiction { field, message }) => {
            assert_eq!(field, "env:prod + risk:low");
            assert_eq!(message, "生产环境不应标记为低风险");
        }
        _ => panic!("Expected Contradiction error"),
    }
}

#[test]
fn test_validate_tag_config_dev_with_high_risk() {
    let config = TagConfig {
        env: Some(EnvTag::Dev),
        risk: Some(RiskTag::High),
        custom: vec![],
    };
    let result = validate_tag_config(&config);
    assert!(result.is_err());
    match result {
        Err(TagError::Contradiction { field, message }) => {
            assert_eq!(field, "env:dev + risk:high");
            assert_eq!(message, "开发环境不应标记为高风险");
        }
        _ => panic!("Expected Contradiction error"),
    }
}

#[test]
fn test_validate_tag_config_invalid_custom_tag_format() {
    let config = TagConfig {
        env: None,
        risk: None,
        custom: vec!["invalid-tag".to_string()],
    };
    let result = validate_tag_config(&config);
    assert!(result.is_err());
    match result {
        Err(TagError::InvalidFormat { tag, expected }) => {
            assert_eq!(tag, "invalid-tag");
            assert_eq!(expected, "key:value");
        }
        _ => panic!("Expected InvalidFormat error"),
    }
}

#[test]
fn test_validate_tag_config_valid_custom_tags() {
    let config = TagConfig {
        env: None,
        risk: None,
        custom: vec!["team:backend".to_string(), "project:keyring".to_string()],
    };
    assert!(validate_tag_config(&config).is_ok());
}

#[test]
fn test_validate_tag_config_empty() {
    let config = TagConfig {
        env: None,
        risk: None,
        custom: vec![],
    };
    assert!(validate_tag_config(&config).is_ok());
}

#[test]
fn test_tag_config_serialization() {
    let config = TagConfig {
        env: Some(EnvTag::Prod),
        risk: Some(RiskTag::High),
        custom: vec!["service:api".to_string()],
    };

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: TagConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.env, config.env);
    assert_eq!(deserialized.risk, config.risk);
    assert_eq!(deserialized.custom, config.custom);
}
