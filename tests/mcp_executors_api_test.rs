//! Tests for API executor
//!
//! This module tests the API executor which handles HTTP requests with response size limiting.

use keyring_cli::mcp::executors::api::{ApiError, ApiResponse, ApiExecutor};
use std::collections::HashMap;

#[tokio::test]
async fn test_api_executor_new() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    assert_eq!(executor.get_auth_type(), "Bearer");
    assert_eq!(executor.get_auth_value(), "test_token");
    assert_eq!(executor.get_max_response_size(), 5 * 1024 * 1024); // 5MB default
}

#[tokio::test]
async fn test_api_executor_new_with_limit() {
    let executor =
        ApiExecutor::new_with_limit("ApiKey".to_string(), "key123".to_string(), 1024 * 1024);

    assert_eq!(executor.get_auth_type(), "ApiKey");
    assert_eq!(executor.get_auth_value(), "key123");
    assert_eq!(executor.get_max_response_size(), 1024 * 1024);
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_get_request() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    // Using a real public API for testing (httpbin)
    let url = "https://httpbin.org/get";
    let result = executor.get(url, None, None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    assert!(!response.body.is_empty());
    assert!(response.duration_ms > 0);
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_get_with_params() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/get";
    let mut params = HashMap::new();
    params.insert("foo".to_string(), "bar".to_string());
    params.insert("test".to_string(), "value".to_string());

    let result = executor.get(url, Some(&params), None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    // Response should contain the params we sent
    assert!(response.body.contains("foo"));
    assert!(response.body.contains("bar"));
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_post_request() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/post";
    let body = serde_json::json!({
        "message": "hello",
        "value": 42
    });

    let result = executor.post(url, Some(&body), None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.contains("hello"));
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_put_request() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/put";
    let body = serde_json::json!({
        "updated": true
    });

    let result = executor.put(url, Some(&body), None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.contains("updated"));
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_delete_request() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/delete";

    let result = executor.delete(url, None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_with_custom_headers() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/headers";
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    headers.insert("X-Another-Header".to_string(), "another-value".to_string());

    let result = executor.get(url, None, Some(&headers)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    // Should have our custom headers echoed back
    assert!(response.body.contains("X-Custom-Header"));
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_generic_request() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/patch";
    let body = serde_json::json!({
        "patched": true
    });

    let result = executor.request("PATCH", url, Some(&body), None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_response_headers() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/get";
    let result = executor.get(url, None, None).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    // Should have some headers
    assert!(!response.headers.is_empty());
    // Common headers
    assert!(
        response.headers.contains_key("content-type")
            || response.headers.contains_key("Content-Type")
    );
}

#[tokio::test]
async fn test_api_executor_error_handling() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    // Invalid URL
    let result = executor.get("invalid://url", None, None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::RequestFailed(_) => {}
        _ => panic!("Expected RequestFailed error"),
    }
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_size_limit() {
    // Create executor with very small limit
    let executor = ApiExecutor::new_with_limit("Bearer".to_string(), "test_token".to_string(), 100);

    // This should return more than 100 bytes
    let url = "https://httpbin.org/bytes/1000";
    let result = executor.get(url, None, None).await;

    // Should either fail or truncate
    match result {
        Ok(response) => {
            // If successful, body should be truncated
            assert!(response.body.len() <= 100);
        }
        Err(ApiError::ResponseTooLarge { .. }) => {
            // Expected error for large response
        }
        Err(_) => {
            panic!("Expected ResponseTooLarge or truncated response");
        }
    }
}

#[tokio::test]
async fn test_api_executor_connection_timeout() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    // Use a non-routable IP (should timeout)
    let result = executor.get("http://192.0.2.1:12345", None, None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_response_clone() {
    let response = ApiResponse {
        status: 200,
        body: "test body".to_string(),
        headers: HashMap::new(),
        duration_ms: 100,
    };

    let cloned = response.clone();
    assert_eq!(response.status, cloned.status);
    assert_eq!(response.body, cloned.body);
    assert_eq!(response.duration_ms, cloned.duration_ms);
}

#[tokio::test]
async fn test_api_error_display() {
    let err = ApiError::RequestFailed("Connection refused".to_string());
    assert!(format!("{}", err).contains("Connection refused"));

    let err = ApiError::ResponseTooLarge {
        size: 10_000_000,
        limit: 5_000_000,
    };
    let err_str = format!("{}", err);
    assert!(err_str.contains("10_000_000") || err_str.contains("10000000"));
    assert!(err_str.contains("5_000_000") || err_str.contains("5000000"));

    let err = ApiError::InvalidUrl("invalid url".to_string());
    assert!(format!("{}", err).contains("invalid url"));

    let err = ApiError::HttpError(404);
    assert!(format!("{}", err).contains("404"));
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_empty_body() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    // POST with no body
    let url = "https://httpbin.org/post";
    let result = executor.post(url, None, None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
}

#[tokio::test]
#[ignore = "Requires network access to httpbin.org"]
async fn test_api_executor_query_params_encoding() {
    let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());

    let url = "https://httpbin.org/get";
    let mut params = HashMap::new();
    params.insert("space key".to_string(), "value with spaces".to_string());
    params.insert("special".to_string(), "!@#$%".to_string());

    let result = executor.get(url, Some(&params), None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn test_api_executor_basic_auth() {
    let executor = ApiExecutor::new("Basic".to_string(), "credentials".to_string());

    assert_eq!(executor.get_auth_type(), "Basic");
    assert_eq!(executor.get_auth_value(), "credentials");
}

#[tokio::test]
async fn test_api_executor_apikey_auth() {
    let executor = ApiExecutor::new("ApiKey".to_string(), "my-secret-key".to_string());

    assert_eq!(executor.get_auth_type(), "ApiKey");
    assert_eq!(executor.get_auth_value(), "my-secret-key");
}

#[tokio::test]
async fn test_api_executor_custom_auth() {
    let executor = ApiExecutor::new("X-Custom-Auth".to_string(), "custom-token".to_string());

    assert_eq!(executor.get_auth_type(), "X-Custom-Auth");
    assert_eq!(executor.get_auth_value(), "custom-token");
}
