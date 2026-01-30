//! API Tool Definitions for MCP
//!
//! This module defines input/output structures for 6 API MCP tools:
//! - api_get (by tag confirmation)
//! - api_post (by tag confirmation)
//! - api_put (by tag confirmation)
//! - api_delete (ALWAYS requires confirmation - high risk)
//! - api_request (generic, by tag confirmation)
//! - api_list_credentials (low risk - no confirmation)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Tool 1: api_get
// ============================================================================

/// Input for api_get tool
///
/// Makes an HTTP GET request to the specified URL.
/// Confirmation required based on credential tags.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiGetInput {
    /// Name of the stored API credential to use
    pub credential_name: String,

    /// URL to send GET request to
    pub url: String,

    /// Query parameters to append to URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,

    /// Custom HTTP headers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Confirmation token (if already confirmed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,

    /// User's decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output from api_get tool
///
/// Contains HTTP response status, body, headers, and timing.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiGetOutput {
    /// HTTP status code
    pub status: u16,

    /// Response body as string
    pub body: String,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Request duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 2: api_post
// ============================================================================

/// Input for api_post tool
///
/// Makes an HTTP POST request with JSON body to the specified URL.
/// Confirmation required based on credential tags.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiPostInput {
    /// Name of the stored API credential to use
    pub credential_name: String,

    /// URL to send POST request to
    pub url: String,

    /// JSON body to send in request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    /// Custom HTTP headers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Confirmation token (if already confirmed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,

    /// User's decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output from api_post tool
///
/// Contains HTTP response status, body, headers, and timing.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiPostOutput {
    /// HTTP status code
    pub status: u16,

    /// Response body as string
    pub body: String,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Request duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 3: api_put
// ============================================================================

/// Input for api_put tool
///
/// Makes an HTTP PUT request with JSON body to the specified URL.
/// Confirmation required based on credential tags.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiPutInput {
    /// Name of the stored API credential to use
    pub credential_name: String,

    /// URL to send PUT request to
    pub url: String,

    /// JSON body to send in request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    /// Custom HTTP headers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Confirmation token (if already confirmed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,

    /// User's decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output from api_put tool
///
/// Contains HTTP response status, body, headers, and timing.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiPutOutput {
    /// HTTP status code
    pub status: u16,

    /// Response body as string
    pub body: String,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Request duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 4: api_delete (ALWAYS requires confirmation)
// ============================================================================

/// Input for api_delete tool
///
/// Makes an HTTP DELETE request to the specified URL.
/// **WARNING: This operation ALWAYS requires confirmation** due to high risk.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiDeleteInput {
    /// Name of the stored API credential to use
    pub credential_name: String,

    /// URL to send DELETE request to
    pub url: String,

    /// Custom HTTP headers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Confirmation token (required for DELETE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,

    /// User's decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output from api_delete tool
///
/// Contains HTTP response status, body, and timing.
/// Note: DELETE responses typically don't include headers.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiDeleteOutput {
    /// HTTP status code
    pub status: u16,

    /// Response body as string (may be empty for 204 No Content)
    pub body: String,

    /// Request duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 5: api_request (generic)
// ============================================================================

/// Input for api_request tool
///
/// Makes a generic HTTP request with custom method, URL, body, and headers.
/// Confirmation required based on credential tags.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiRequestInput {
    /// Name of the stored API credential to use
    pub credential_name: String,

    /// HTTP method (GET, POST, PUT, DELETE, PATCH, etc.)
    pub method: String,

    /// URL to send request to
    pub url: String,

    /// JSON body to send in request (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    /// Custom HTTP headers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Confirmation token (if already confirmed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,

    /// User's decision (approve/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_decision: Option<String>,
}

/// Output from api_request tool
///
/// Contains HTTP response status, body, headers, and timing.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiRequestOutput {
    /// HTTP status code
    pub status: u16,

    /// Response body as string
    pub body: String,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Request duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Tool 6: api_list_credentials (low risk, no confirmation)
// ============================================================================

/// Input for api_list_credentials tool
///
/// Lists stored API credentials, optionally filtered by tags.
/// No confirmation required (low risk operation).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiListCredentialsInput {
    /// Optional tags to filter credentials by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_tags: Option<Vec<String>>,
}

/// Information about a single API credential
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiCredentialInfo {
    /// Name/identifier of the credential
    pub name: String,

    /// API endpoint URL (if applicable)
    pub endpoint: Option<String>,

    /// Tags associated with this credential
    pub tags: Vec<String>,
}

/// Output from api_list_credentials tool
///
/// Contains list of API credentials matching the filter.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiListCredentialsOutput {
    /// List of API credentials
    pub credentials: Vec<ApiCredentialInfo>,
}
