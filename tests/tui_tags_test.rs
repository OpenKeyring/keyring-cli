use keyring_cli::tui::tags::config::{EnvTag, RiskTag, TagConfig, TagError, validate_tag_config};
use keyring_cli::tui::tags::widget::{TagConfigWidget, TagFocus};

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

// Widget tests

#[test]
fn test_widget_creation() {
    let widget = TagConfigWidget::new("test-credential".to_string());

    assert_eq!(widget.credential_name, "test-credential");
    assert_eq!(widget.focus(), TagFocus::Env);
    assert!(!widget.can_save());
}

#[test]
fn test_widget_with_existing_config() {
    let config = TagConfig {
        env: Some(EnvTag::Dev),
        risk: Some(RiskTag::Low),
        custom: vec!["category:database".to_string()],
    };

    let widget = TagConfigWidget::with_config("prod-db".to_string(), config);

    assert_eq!(widget.config().env, Some(EnvTag::Dev));
    assert_eq!(widget.config().risk, Some(RiskTag::Low));
    assert_eq!(widget.config.custom.len(), 1);
    assert!(widget.can_save());
}

#[test]
fn test_widget_navigation_right() {
    let mut widget = TagConfigWidget::new("test".to_string());

    // Start at Env
    assert_eq!(widget.focus(), TagFocus::Env);

    // Move to Risk
    widget.on_key_right();
    assert_eq!(widget.focus(), TagFocus::Risk);

    // Move to Buttons
    widget.on_key_right();
    assert_eq!(widget.focus(), TagFocus::Buttons);

    // Should stay at Buttons
    widget.on_key_right();
    assert_eq!(widget.focus(), TagFocus::Buttons);
}

#[test]
fn test_widget_navigation_left() {
    let mut widget = TagConfigWidget::new("test".to_string());

    // Move to Buttons first
    widget.set_focus(TagFocus::Buttons);
    assert_eq!(widget.focus(), TagFocus::Buttons);

    // Move left to Risk
    widget.on_key_left();
    assert_eq!(widget.focus(), TagFocus::Risk);

    // Move left to Env
    widget.on_key_left();
    assert_eq!(widget.focus(), TagFocus::Env);

    // Should stay at Env
    widget.on_key_left();
    assert_eq!(widget.focus(), TagFocus::Env);
}

#[test]
fn test_widget_navigation_with_advanced() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.toggle_advanced(); // Enable advanced section

    assert_eq!(widget.focus(), TagFocus::Advanced);

    // Navigate right
    widget.on_key_right();
    assert_eq!(widget.focus(), TagFocus::Buttons);

    // Navigate left through all sections
    widget.on_key_left();
    assert_eq!(widget.focus(), TagFocus::Advanced);

    widget.on_key_left();
    assert_eq!(widget.focus(), TagFocus::Risk);

    widget.on_key_left();
    assert_eq!(widget.focus(), TagFocus::Env);
}

#[test]
fn test_env_tag_selection() {
    let mut widget = TagConfigWidget::new("test".to_string());

    // Select dev (index 0)
    widget.selected_env = Some(0);
    widget.update_config();

    assert_eq!(widget.config().env, Some(EnvTag::Dev));
    assert!(widget.can_save());
}

#[test]
fn test_env_tag_navigation() {
    let mut widget = TagConfigWidget::new("test".to_string());

    // Set initial selection
    widget.selected_env = Some(0);

    // Navigate down: 0 -> 1 -> 2 -> 3 -> 0 (wrap)
    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(1));

    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(2));

    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(3));

    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(0)); // Wrapped

    // Navigate up: 0 -> 3 -> 2 -> 1 -> 0
    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(3));

    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(2));

    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(1));

    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(0));
}

#[test]
fn test_risk_tag_selection() {
    let mut widget = TagConfigWidget::new("test".to_string());

    // Select low (index 0)
    widget.selected_risk = Some(0);
    widget.update_config();

    assert_eq!(widget.config().risk, Some(RiskTag::Low));
}

#[test]
fn test_risk_tag_navigation() {
    let mut widget = TagConfigWidget::new("test".to_string());

    // Set initial selection
    widget.selected_risk = Some(0);

    // Navigate down: 0 -> 1 -> 2 -> 0 (wrap)
    widget.on_key_down();
    assert_eq!(widget.selected_risk, Some(1));

    widget.on_key_down();
    assert_eq!(widget.selected_risk, Some(2));

    widget.on_key_down();
    assert_eq!(widget.selected_risk, Some(0)); // Wrapped

    // Navigate up: 0 -> 2 -> 1 -> 0
    widget.on_key_up();
    assert_eq!(widget.selected_risk, Some(2));

    widget.on_key_up();
    assert_eq!(widget.selected_risk, Some(1));

    widget.on_key_up();
    assert_eq!(widget.selected_risk, Some(0));
}

#[test]
fn test_advanced_toggle() {
    let mut widget = TagConfigWidget::new("test".to_string());

    assert!(!widget.show_advanced);

    widget.toggle_advanced();
    assert!(widget.show_advanced);
    assert_eq!(widget.focus(), TagFocus::Advanced);

    widget.toggle_advanced();
    assert!(!widget.show_advanced);
    assert_eq!(widget.focus(), TagFocus::Risk);
}

#[test]
fn test_custom_tag_addition() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.show_advanced = true;

    widget.add_custom_tag("category:database".to_string());
    assert_eq!(widget.config.custom.len(), 1);
    assert_eq!(widget.selected_custom, Some(0));

    widget.add_custom_tag("owner:team-a".to_string());
    assert_eq!(widget.config.custom.len(), 2);
    assert_eq!(widget.selected_custom, Some(1));

    // Try to add duplicate
    widget.add_custom_tag("category:database".to_string());
    assert_eq!(widget.config.custom.len(), 2); // No change
}

#[test]
fn test_custom_tag_removal() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.show_advanced = true;
    widget.config.custom = vec![
        "tag1".to_string(),
        "tag2".to_string(),
        "tag3".to_string(),
    ];
    widget.selected_custom = Some(1);

    // Remove middle tag
    widget.remove_selected_custom_tag();
    assert_eq!(widget.config.custom.len(), 2);
    assert_eq!(widget.config.custom, vec!["tag1".to_string(), "tag3".to_string()]);
    assert_eq!(widget.selected_custom, Some(1));

    // Remove last tag
    widget.remove_selected_custom_tag();
    assert_eq!(widget.config.custom.len(), 1);
    assert_eq!(widget.config.custom, vec!["tag1".to_string()]);
    assert_eq!(widget.selected_custom, Some(0));

    // Remove last remaining tag
    widget.remove_selected_custom_tag();
    assert_eq!(widget.config.custom.len(), 0);
    assert_eq!(widget.selected_custom, None);
}

#[test]
fn test_can_save_validation() {
    let mut widget = TagConfigWidget::new("test".to_string());

    // Cannot save without env tag
    assert!(!widget.can_save());

    // Set env tag
    widget.selected_env = Some(0);
    widget.update_config();
    assert!(widget.can_save());

    // Clear env tag
    widget.selected_env = None;
    widget.update_config();
    assert!(!widget.can_save());
}

#[test]
fn test_widget_into_config() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.selected_env = Some(2); // staging
    widget.selected_risk = Some(1); // medium
    widget.add_custom_tag("service:api".to_string());
    widget.update_config();

    let config = widget.into_config();

    assert_eq!(config.env, Some(EnvTag::Staging));
    assert_eq!(config.risk, Some(RiskTag::Medium));
    assert_eq!(config.custom.len(), 1);
    assert_eq!(config.custom[0], "service:api");
}

#[test]
fn test_widget_complete_workflow() {
    let mut widget = TagConfigWidget::new("production-db".to_string());

    // Simulate user selecting environment
    widget.selected_env = Some(3); // prod
    widget.update_config();
    assert_eq!(widget.config().env, Some(EnvTag::Prod));

    // Simulate user selecting risk
    widget.set_focus(TagFocus::Risk);
    widget.selected_risk = Some(0); // low
    widget.update_config();
    assert_eq!(widget.config().risk, Some(RiskTag::Low));

    // Enable advanced and add custom tag
    widget.toggle_advanced();
    widget.add_custom_tag("region:us-east".to_string());
    assert_eq!(widget.config.custom.len(), 1);

    // Verify can save
    assert!(widget.can_save());

    // Extract config
    let config = widget.into_config();
    assert_eq!(config.env, Some(EnvTag::Prod));
    assert_eq!(config.risk, Some(RiskTag::Low));
    assert_eq!(config.custom[0], "region:us-east");
}
