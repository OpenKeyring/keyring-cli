//! API Executor for MCP Tools
//!
//! This module provides HTTP request execution capabilities using reqwest,
//! with response size limiting for security and resource management.

use reqwest::Client;
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;

/// API response containing status, body, headers, and timing information
#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub duration_ms: u64,
}

/// Errors that can occur during API execution
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("Response too large: {size} bytes exceeds limit of {limit} bytes")]
    ResponseTooLarge { size: usize, limit: usize },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP error: {0}")]
    HttpError(u16),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// API executor for making HTTP requests with authentication and size limiting
pub struct ApiExecutor {
    client: Client,
    auth_type: String,
    auth_value: String,
    max_response_size: usize,
}

impl ApiExecutor {
    /// Default maximum response size (5MB)
    const DEFAULT_MAX_SIZE: usize = 5 * 1024 * 1024;

    /// Create a new API executor with default 5MB response size limit
    ///
    /// # Arguments
    /// * `auth_type` - Authentication type (e.g., "Bearer", "Basic", "ApiKey")
    /// * `auth_value` - Authentication value (e.g., token, credentials)
    ///
    /// # Example
    /// ```no_run
    /// use keyring_cli::mcp::executors::api::ApiExecutor;
    ///
    /// let executor = ApiExecutor::new("Bearer".to_string(), "my_token".to_string());
    /// ```
    pub fn new(auth_type: String, auth_value: String) -> Self {
        Self::new_with_limit(auth_type, auth_value, Self::DEFAULT_MAX_SIZE)
    }

    /// Create a new API executor with custom response size limit
    ///
    /// # Arguments
    /// * `auth_type` - Authentication type (e.g., "Bearer", "Basic", "ApiKey")
    /// * `auth_value` - Authentication value (e.g., token, credentials)
    /// * `max_response_size` - Maximum response size in bytes
    ///
    /// # Example
    /// ```no_run
    /// use keyring_cli::mcp::executors::api::ApiExecutor;
    ///
    /// // 1MB limit
    /// let executor = ApiExecutor::new_with_limit(
    ///     "Bearer".to_string(),
    ///     "my_token".to_string(),
    ///     1024 * 1024
    /// );
    /// ```
    pub fn new_with_limit(auth_type: String, auth_value: String, max_response_size: usize) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();

        Self {
            client,
            auth_type,
            auth_value,
            max_response_size,
        }
    }

    /// Get the authentication type
    pub fn get_auth_type(&self) -> &str {
        &self.auth_type
    }

    /// Get the authentication value
    pub fn get_auth_value(&self) -> &str {
        &self.auth_value
    }

    /// Get the maximum response size
    pub fn get_max_response_size(&self) -> usize {
        self.max_response_size
    }

    /// Perform a GET request
    ///
    /// # Arguments
    /// * `url` - The URL to request
    /// * `params` - Optional query parameters
    /// * `headers` - Optional additional headers
    ///
    /// # Example
    /// ```no_run
    /// # use keyring_cli::mcp::executors::api::ApiExecutor;
    /// # use std::collections::HashMap;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let executor = ApiExecutor::new("Bearer".to_string(), "token".to_string());
    ///
    /// let mut params = HashMap::new();
    /// params.insert("page".to_string(), "1".to_string());
    ///
    /// let response = executor.get("https://api.example.com/data", Some(&params), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(
        &self,
        url: &str,
        params: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<ApiResponse, ApiError> {
        let mut request = self.client.get(url);

        // Add query parameters
        if let Some(params) = params {
            request = request.query(params);
        }

        self.execute_request(request, headers, None).await
    }

    /// Perform a POST request
    ///
    /// # Arguments
    /// * `url` - The URL to request
    /// * `body` - Optional JSON body
    /// * `headers` - Optional additional headers
    ///
    /// # Example
    /// ```no_run
    /// # use keyring_cli::mcp::executors::api::ApiExecutor;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let executor = ApiExecutor::new("Bearer".to_string(), "token".to_string());
    ///
    /// let body = serde_json::json!({
    ///     "name": "test",
    ///     "value": 42
    /// });
    ///
    /// let response = executor.post(
    ///     "https://api.example.com/create",
    ///     Some(&body),
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post(
        &self,
        url: &str,
        body: Option<&serde_json::Value>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<ApiResponse, ApiError> {
        let mut request = self.client.post(url);

        if let Some(body) = body {
            request = request.json(body);
        }

        self.execute_request(request, headers, None).await
    }

    /// Perform a PUT request
    ///
    /// # Arguments
    /// * `url` - The URL to request
    /// * `body` - Optional JSON body
    /// * `headers` - Optional additional headers
    pub async fn put(
        &self,
        url: &str,
        body: Option<&serde_json::Value>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<ApiResponse, ApiError> {
        let mut request = self.client.put(url);

        if let Some(body) = body {
            request = request.json(body);
        }

        self.execute_request(request, headers, None).await
    }

    /// Perform a DELETE request
    ///
    /// # Arguments
    /// * `url` - The URL to request
    /// * `headers` - Optional additional headers
    pub async fn delete(
        &self,
        url: &str,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<ApiResponse, ApiError> {
        let request = self.client.delete(url);
        self.execute_request(request, headers, None).await
    }

    /// Perform a generic HTTP request
    ///
    /// # Arguments
    /// * `method` - HTTP method as a string (GET, POST, PUT, PATCH, DELETE, etc.)
    /// * `url` - The URL to request
    /// * `body` - Optional JSON body
    /// * `headers` - Optional additional headers
    ///
    /// # Example
    /// ```no_run
    /// # use keyring_cli::mcp::executors::api::ApiExecutor;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let executor = ApiExecutor::new("Bearer".to_string(), "token".to_string());
    ///
    /// let response = executor.request(
    ///     "PATCH",
    ///     "https://api.example.com/update/123",
    ///     None,
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<&serde_json::Value>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<ApiResponse, ApiError> {
        let method = method.to_uppercase();
        let mut request = self.client.request(method.parse().unwrap_or(reqwest::Method::GET), url);

        if let Some(body) = body {
            request = request.json(body);
        }

        self.execute_request(request, headers, None).await
    }

    /// Execute a prepared request with response size limiting
    ///
    /// # Flow
    /// 1. Start timer
    /// 2. Add authentication headers
    /// 3. Add custom headers
    /// 4. Send HTTP request
    /// 5. Wrap response in ResponseLimiter
    /// 6. Read chunks until done or size limit hit
    /// 7. Check if truncated
    /// 8. Convert body to string
    /// 9. Return ApiResponse with duration
    async fn execute_request(
        &self,
        mut request: reqwest::RequestBuilder,
        custom_headers: Option<&HashMap<String, String>>,
        _body: Option<&serde_json::Value>,
    ) -> Result<ApiResponse, ApiError> {
        let start_time = Instant::now();

        // Add authentication header
        match self.auth_type.as_str() {
            "Bearer" => {
                request = request.header("Authorization", format!("Bearer {}", self.auth_value));
            }
            "Basic" => {
                request = request.header("Authorization", format!("Basic {}", self.auth_value));
            }
            "ApiKey" | "API-Key" => {
                request = request.header("X-API-Key", &self.auth_value);
            }
            _ => {
                // Custom auth type - try to use as header name
                request = request.header(&self.auth_type, &self.auth_value);
            }
        }

        // Add custom headers
        if let Some(headers) = custom_headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        // Send request
        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::Timeout(e.to_string())
            } else if e.is_connect() {
                ApiError::RequestFailed(format!("Connection failed: {}", e))
            } else {
                ApiError::RequestFailed(e.to_string())
            }
        })?;

        let status = response.status().as_u16();

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(ApiError::HttpError(status));
        }

        // Collect headers
        let headers_map = response.headers().clone();
        let mut response_headers = HashMap::new();
        for (key, value) in headers_map.iter() {
            if let Ok(value_str) = value.to_str() {
                response_headers.insert(key.as_str().to_string(), value_str.to_string());
            }
        }

        // Use ResponseLimiter to read response with size limit
        let mut limiter = ResponseLimiter::new(response, self.max_response_size);

        let mut body_bytes = Vec::new();
        while let Some(chunk_result) = limiter.next_chunk().await {
            let chunk = chunk_result.map_err(|e: reqwest::Error| ApiError::RequestFailed(e.to_string()))?;
            body_bytes.extend_from_slice(&chunk);
        }

        // Check if response was truncated
        if limiter.was_truncated() {
            return Err(ApiError::ResponseTooLarge {
                size: limiter.bytes_read(),
                limit: self.max_response_size,
            });
        }

        // Convert to string
        let body = String::from_utf8(body_bytes).map_err(|e| {
            ApiError::SerializationError(format!("Invalid UTF-8 in response: {}", e))
        })?;

        let duration = start_time.elapsed();

        Ok(ApiResponse {
            status,
            body,
            headers: response_headers,
            duration_ms: duration.as_millis() as u64,
        })
    }
}

/// Response limiter that enforces size limits when reading HTTP response chunks
struct ResponseLimiter {
    response: reqwest::Response,
    remaining: usize,
    bytes_read: usize,
    truncated: bool,
}

impl ResponseLimiter {
    /// Create a new response limiter
    fn new(response: reqwest::Response, max_size: usize) -> Self {
        Self {
            response,
            remaining: max_size,
            bytes_read: 0,
            truncated: false,
        }
    }

    /// Get the next chunk of the response, respecting the size limit
    async fn next_chunk(&mut self) -> Option<Result<bytes::Bytes, reqwest::Error>> {
        if self.remaining == 0 || self.truncated {
            return None;
        }

        match self.response.chunk().await {
            Ok(Some(chunk)) => {
                let chunk_size = chunk.len();

                if chunk_size > self.remaining {
                    // Truncate this chunk
                    self.bytes_read += self.remaining;
                    self.truncated = true;
                    self.remaining = 0;
                    Some(Ok(chunk.slice(0..self.remaining)))
                } else {
                    self.bytes_read += chunk_size;
                    self.remaining -= chunk_size;
                    Some(Ok(chunk))
                }
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    /// Check if the response was truncated due to size limit
    fn was_truncated(&self) -> bool {
        self.truncated
    }

    /// Get the total number of bytes read
    fn bytes_read(&self) -> usize {
        self.bytes_read
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_executor_creation() {
        let executor = ApiExecutor::new("Bearer".to_string(), "test_token".to_string());
        assert_eq!(executor.get_auth_type(), "Bearer");
        assert_eq!(executor.get_auth_value(), "test_token");
        assert_eq!(executor.get_max_response_size(), 5 * 1024 * 1024);
    }

    #[test]
    fn test_api_executor_custom_limit() {
        let executor =
            ApiExecutor::new_with_limit("ApiKey".to_string(), "key123".to_string(), 1024);
        assert_eq!(executor.get_max_response_size(), 1024);
    }

    #[test]
    fn test_api_response_clone() {
        let response = ApiResponse {
            status: 200,
            body: "test".to_string(),
            headers: HashMap::new(),
            duration_ms: 100,
        };

        let cloned = response.clone();
        assert_eq!(response.status, cloned.status);
        assert_eq!(response.body, cloned.body);
    }
}
