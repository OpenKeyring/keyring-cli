//! Tests for TagConfigWidget
//!
//! Unit tests for the tag configuration widget.

use super::{TagConfigWidget, TagFocus};
use crate::tui::tags::config::{EnvTag, RiskTag, TagConfig};

#[test]
fn test_widget_default() {
    let widget = TagConfigWidget::default();
    assert_eq!(widget.credential_name, "Unnamed Credential");
    assert!(widget.config.env.is_none());
    assert!(widget.config.risk.is_none());
    assert!(widget.config.custom.is_empty());
}

#[test]
fn test_widget_new() {
    let widget = TagConfigWidget::new("test-credential".to_string());
    assert_eq!(widget.credential_name, "test-credential");
    assert_eq!(widget.focus, TagFocus::Env);
    assert!(!widget.show_advanced);
}

#[test]
fn test_widget_with_config() {
    let config = TagConfig {
        env: Some(EnvTag::Test),
        risk: Some(RiskTag::Medium),
        custom: vec!["custom:tag".to_string()],
    };

    let widget = TagConfigWidget::with_config("test".to_string(), config);
    assert_eq!(widget.selected_env, Some(1));
    assert_eq!(widget.selected_risk, Some(1));
    assert_eq!(widget.config.custom.len(), 1);
}

#[test]
fn test_on_key_down_env() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.selected_env = Some(0);

    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(1));

    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(2));

    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(3));

    widget.on_key_down();
    assert_eq!(widget.selected_env, Some(0)); // Wrap around
}

#[test]
fn test_on_key_up_env() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.selected_env = Some(3);

    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(2));

    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(1));

    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(0));

    widget.on_key_up();
    assert_eq!(widget.selected_env, Some(3)); // Wrap around
}

#[test]
fn test_on_key_down_risk() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.focus = TagFocus::Risk;
    widget.selected_risk = Some(0);

    widget.on_key_down();
    assert_eq!(widget.selected_risk, Some(1));

    widget.on_key_down();
    assert_eq!(widget.selected_risk, Some(2));

    widget.on_key_down();
    assert_eq!(widget.selected_risk, Some(0)); // Wrap around
}

#[test]
fn test_toggle_advanced() {
    let mut widget = TagConfigWidget::new("test".to_string());
    assert!(!widget.show_advanced);

    widget.toggle_advanced();
    assert!(widget.show_advanced);
    assert_eq!(widget.focus, TagFocus::Advanced);

    widget.toggle_advanced();
    assert!(!widget.show_advanced);
    assert_eq!(widget.focus, TagFocus::Risk);
}

#[test]
fn test_add_custom_tag() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.show_advanced = true;

    widget.add_custom_tag("category:database".to_string());
    assert_eq!(widget.config.custom.len(), 1);
    assert_eq!(widget.selected_custom, Some(0));

    // Try adding duplicate
    widget.add_custom_tag("category:database".to_string());
    assert_eq!(widget.config.custom.len(), 1);

    // Add another
    widget.add_custom_tag("owner:team-a".to_string());
    assert_eq!(widget.config.custom.len(), 2);
}

#[test]
fn test_remove_custom_tag() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.show_advanced = true;
    widget.config.custom = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
    widget.selected_custom = Some(1);

    widget.remove_selected_custom_tag();
    assert_eq!(widget.config.custom.len(), 2);
    assert_eq!(
        widget.config.custom,
        vec!["tag1".to_string(), "tag3".to_string()]
    );
    assert_eq!(widget.selected_custom, Some(1)); // Still at index 1

    widget.remove_selected_custom_tag();
    assert_eq!(widget.config.custom.len(), 1);
    assert_eq!(widget.selected_custom, Some(0));
}

#[test]
fn test_on_key_left_right() {
    let mut widget = TagConfigWidget::new("test".to_string());
    assert_eq!(widget.focus, TagFocus::Env);

    widget.on_key_right();
    assert_eq!(widget.focus, TagFocus::Risk);

    widget.on_key_right();
    assert_eq!(widget.focus, TagFocus::Buttons);

    widget.on_key_left();
    assert_eq!(widget.focus, TagFocus::Risk);

    widget.on_key_left();
    assert_eq!(widget.focus, TagFocus::Env);
}

#[test]
fn test_update_config() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.selected_env = Some(2);
    widget.selected_risk = Some(1);
    widget.update_config();

    assert_eq!(widget.config.env, Some(EnvTag::Staging));
    assert_eq!(widget.config.risk, Some(RiskTag::Medium));
}

#[test]
fn test_can_save() {
    let mut widget = TagConfigWidget::new("test".to_string());
    assert!(!widget.can_save());

    widget.selected_env = Some(0);
    widget.update_config();
    assert!(widget.can_save());
}

#[test]
fn test_into_config() {
    let mut widget = TagConfigWidget::new("test".to_string());
    widget.selected_env = Some(1);
    widget.selected_risk = Some(2);
    widget.add_custom_tag("custom:tag".to_string());
    widget.update_config();

    let config = widget.into_config();
    assert_eq!(config.env, Some(EnvTag::Test));
    assert_eq!(config.risk, Some(RiskTag::High));
    assert_eq!(config.custom.len(), 1);
}
