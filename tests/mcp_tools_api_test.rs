//! Tests for API tool input/output structs
//!
//! Tests serialization/deserialization of all 6 API tool definitions.

use serde_json::{from_value, json};

// ============================================================================
// Tool 1: api_get
// ============================================================================

#[test]
fn test_api_get_input() {
    let input = keyring_cli::mcp::tools::api::ApiGetInput {
        credential_name: "github-api".to_string(),
        url: "https://api.github.com/user".to_string(),
        params: None,
        headers: None,
        confirmation_id: None,
        user_decision: None,
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiGetInput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.credential_name, "github-api");
    assert_eq!(roundtrip.url, "https://api.github.com/user");
}

#[test]
fn test_api_get_with_params() {
    use std::collections::HashMap;

    let mut params = HashMap::new();
    params.insert("page".to_string(), "1".to_string());
    params.insert("per_page".to_string(), "10".to_string());

    let input = keyring_cli::mcp::tools::api::ApiGetInput {
        credential_name: "api".to_string(),
        url: "https://api.example.com/users".to_string(),
        params: Some(params.clone()),
        headers: None,
        confirmation_id: None,
        user_decision: None,
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiGetInput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.params, Some(params));
}

#[test]
fn test_api_get_output() {
    use std::collections::HashMap;

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    let output = keyring_cli::mcp::tools::api::ApiGetOutput {
        status: 200,
        body: "{\"data\": \"test\"}".to_string(),
        headers: headers.clone(),
        duration_ms: 150,
    };

    let json_val = json!(output);
    let roundtrip: keyring_cli::mcp::tools::api::ApiGetOutput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.status, 200);
    assert_eq!(roundtrip.body, "{\"data\": \"test\"}");
    assert_eq!(roundtrip.duration_ms, 150);
    assert_eq!(roundtrip.headers, headers);
}

// ============================================================================
// Tool 2: api_post
// ============================================================================

#[test]
fn test_api_post_with_body() {
    let body = json!({"data": "test", "value": 123});

    let input = keyring_cli::mcp::tools::api::ApiPostInput {
        credential_name: "api".to_string(),
        url: "https://example.com/api".to_string(),
        body: Some(body.clone()),
        headers: None,
        confirmation_id: None,
        user_decision: None,
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiPostInput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.body.unwrap(), body);
}

#[test]
fn test_api_post_output() {
    use std::collections::HashMap;

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    let output = keyring_cli::mcp::tools::api::ApiPostOutput {
        status: 201,
        body: "{\"id\": 123}".to_string(),
        headers: headers.clone(),
        duration_ms: 200,
    };

    let json_val = json!(output);
    let roundtrip: keyring_cli::mcp::tools::api::ApiPostOutput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.status, 201);
    assert_eq!(roundtrip.body, "{\"id\": 123}");
}

// ============================================================================
// Tool 3: api_put
// ============================================================================

#[test]
fn test_api_put_input() {
    let body = json!({"name": "updated"});

    let input = keyring_cli::mcp::tools::api::ApiPutInput {
        credential_name: "api".to_string(),
        url: "https://example.com/resource/123".to_string(),
        body: Some(body.clone()),
        headers: None,
        confirmation_id: Some("confirm-123".to_string()),
        user_decision: Some("approve".to_string()),
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiPutInput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.credential_name, "api");
    assert_eq!(roundtrip.url, "https://example.com/resource/123");
    assert_eq!(roundtrip.confirmation_id, Some("confirm-123".to_string()));
    assert_eq!(roundtrip.user_decision, Some("approve".to_string()));
}

#[test]
fn test_api_put_output() {
    use std::collections::HashMap;

    let output = keyring_cli::mcp::tools::api::ApiPutOutput {
        status: 200,
        body: "{\"success\": true}".to_string(),
        headers: HashMap::new(),
        duration_ms: 180,
    };

    let json_val = json!(output);
    let roundtrip: keyring_cli::mcp::tools::api::ApiPutOutput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.status, 200);
    assert_eq!(roundtrip.body, "{\"success\": true}");
}

// ============================================================================
// Tool 4: api_delete (ALWAYS requires confirmation)
// ============================================================================

#[test]
fn test_api_delete_input() {
    use std::collections::HashMap;

    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "value".to_string());

    let input = keyring_cli::mcp::tools::api::ApiDeleteInput {
        credential_name: "prod-api".to_string(),
        url: "https://example.com/resource/123".to_string(),
        headers: Some(headers.clone()),
        confirmation_id: None,
        user_decision: None,
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiDeleteInput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.credential_name, "prod-api");
    assert_eq!(roundtrip.url, "https://example.com/resource/123");
    assert_eq!(roundtrip.headers, Some(headers));
}

#[test]
fn test_api_delete_output() {
    let output = keyring_cli::mcp::tools::api::ApiDeleteOutput {
        status: 204,
        body: "".to_string(),
        duration_ms: 100,
    };

    let json_val = json!(output);
    let roundtrip: keyring_cli::mcp::tools::api::ApiDeleteOutput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.status, 204);
    assert_eq!(roundtrip.body, "");
}

// ============================================================================
// Tool 5: api_request (generic)
// ============================================================================

#[test]
fn test_api_request_input() {
    let body = json!({"query": "test"});

    let input = keyring_cli::mcp::tools::api::ApiRequestInput {
        credential_name: "api".to_string(),
        method: "PATCH".to_string(),
        url: "https://example.com/resource".to_string(),
        body: Some(body.clone()),
        headers: None,
        confirmation_id: None,
        user_decision: None,
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiRequestInput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.method, "PATCH");
    assert_eq!(roundtrip.body.unwrap(), body);
}

#[test]
fn test_api_request_output() {
    use std::collections::HashMap;

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    let output = keyring_cli::mcp::tools::api::ApiRequestOutput {
        status: 200,
        body: "{\"result\": \"ok\"}".to_string(),
        headers: headers.clone(),
        duration_ms: 250,
    };

    let json_val = json!(output);
    let roundtrip: keyring_cli::mcp::tools::api::ApiRequestOutput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.status, 200);
    assert_eq!(roundtrip.headers, headers);
}

// ============================================================================
// Tool 6: api_list_credentials (low risk, no confirmation)
// ============================================================================

#[test]
fn test_api_list_credentials_input() {
    let input = keyring_cli::mcp::tools::api::ApiListCredentialsInput {
        filter_tags: Some(vec!["env:prod".to_string(), "team:backend".to_string()]),
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiListCredentialsInput = from_value(json_val).unwrap();

    assert_eq!(
        roundtrip.filter_tags,
        Some(vec!["env:prod".to_string(), "team:backend".to_string()])
    );
}

#[test]
fn test_api_list_credentials_input_empty() {
    let input = keyring_cli::mcp::tools::api::ApiListCredentialsInput {
        filter_tags: None,
    };

    let json_val = json!(input);
    let roundtrip: keyring_cli::mcp::tools::api::ApiListCredentialsInput = from_value(json_val).unwrap();

    assert_eq!(roundtrip.filter_tags, None);
}

#[test]
fn test_api_credential_info() {
    let info = keyring_cli::mcp::tools::api::ApiCredentialInfo {
        name: "github-api".to_string(),
        endpoint: Some("https://api.github.com".to_string()),
        tags: vec!["env:prod".to_string(), "type:api".to_string()],
    };

    let json_val = json!(info);
    let roundtrip: keyring_cli::mcp::tools::api::ApiCredentialInfo = from_value(json_val).unwrap();

    assert_eq!(roundtrip.name, "github-api");
    assert_eq!(roundtrip.endpoint, Some("https://api.github.com".to_string()));
    assert_eq!(roundtrip.tags, vec!["env:prod".to_string(), "type:api".to_string()]);
}

#[test]
fn test_api_list_credentials_output() {
    let output = keyring_cli::mcp::tools::api::ApiListCredentialsOutput {
        credentials: vec![
            keyring_cli::mcp::tools::api::ApiCredentialInfo {
                name: "github-api".to_string(),
                endpoint: Some("https://api.github.com".to_string()),
                tags: vec!["env:prod".to_string()],
            },
            keyring_cli::mcp::tools::api::ApiCredentialInfo {
                name: "internal-api".to_string(),
                endpoint: None,
                tags: vec!["env:dev".to_string()],
            },
        ],
    };

    let json_val = json!(output);
    let roundtrip: keyring_cli::mcp::tools::api::ApiListCredentialsOutput =
        from_value(json_val).unwrap();

    assert_eq!(roundtrip.credentials.len(), 2);
    assert_eq!(roundtrip.credentials[0].name, "github-api");
    assert_eq!(roundtrip.credentials[1].name, "internal-api");
}
