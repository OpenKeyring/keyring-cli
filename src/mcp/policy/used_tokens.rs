// mcp/auth/used_tokens.rs
// Used token cache for replay attack prevention

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::error::Error;

/// Cache for tracking used one-time authentication tokens.
/// Prevents replay attacks by ensuring each token can only be used once.
pub struct UsedTokenCache {
    /// Set of token IDs that have been used
    used: HashSet<String>,
    /// Timestamps for when each token was used (for cleanup)
    /// Made pub for testing purposes
    pub timestamps: HashMap<String, Instant>,
}

impl UsedTokenCache {
    /// Create a new empty used token cache.
    pub fn new() -> Self {
        Self {
            used: HashSet::new(),
            timestamps: HashMap::new(),
        }
    }

    /// Mark a token as used.
    ///
    /// Returns an error if the token has already been used (replay attack detection).
    ///
    /// # Arguments
    /// * `token_id` - The unique identifier for the token (nonce or signature)
    ///
    /// # Returns
    /// * `Ok(())` - Token was successfully marked as used
    /// * `Err(Error::TokenAlreadyUsed)` - Token was previously used
    pub fn mark_used(&mut self, token_id: &str) -> Result<(), Error> {
        if self.used.contains(token_id) {
            return Err(Error::TokenAlreadyUsed(token_id.to_string()));
        }

        let now = Instant::now();
        self.used.insert(token_id.to_string());
        self.timestamps.insert(token_id.to_string(), now);
        Ok(())
    }

    /// Check if a token has been used.
    ///
    /// # Arguments
    /// * `token_id` - The token identifier to check
    ///
    /// # Returns
    /// * `true` - Token has been used
    /// * `false` - Token has not been used
    pub fn is_used(&self, token_id: &str) -> bool {
        self.used.contains(token_id)
    }

    /// Remove tokens older than 5 minutes (token expiry time).
    ///
    /// This prevents unbounded memory growth by removing expired entries.
    /// Tokens are valid for 5 minutes, so we can safely remove entries
    /// older than 300 seconds.
    pub fn cleanup_old_tokens(&mut self) {
        let now = Instant::now();
        let expiry_duration = std::time::Duration::from_secs(300); // 5 minutes

        // Find expired tokens
        let expired: Vec<String> = self
            .timestamps
            .iter()
            .filter(|(_, timestamp)| now.duration_since(**timestamp) > expiry_duration)
            .map(|(token_id, _)| token_id.clone())
            .collect();

        // Remove expired tokens
        for token_id in expired {
            self.used.remove(&token_id);
            self.timestamps.remove(&token_id);
        }
    }

    /// Get the number of tokens currently tracked in the cache.
    pub fn len(&self) -> usize {
        self.used.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.used.is_empty()
    }
}

impl Default for UsedTokenCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_new_token() {
        let mut cache = UsedTokenCache::new();
        let token_id = "test-token-123";

        assert!(!cache.is_used(token_id));
        assert!(cache.mark_used(token_id).is_ok());
        assert!(cache.is_used(token_id));
    }

    #[test]
    fn test_mark_used_token_fails() {
        let mut cache = UsedTokenCache::new();
        let token_id = "test-token-456";

        cache.mark_used(token_id).unwrap();
        let result = cache.mark_used(token_id);

        assert!(result.is_err());
        match result {
            Err(Error::TokenAlreadyUsed(id)) => assert_eq!(id, token_id),
            _ => panic!("Expected TokenAlreadyUsed error"),
        }
    }

    #[test]
    fn test_is_used_returns_correct_state() {
        let cache = UsedTokenCache::new();
        let token_id = "test-token-789";

        assert!(!cache.is_used(token_id));
    }

    #[test]
    fn test_cache_size_tracking() {
        let mut cache = UsedTokenCache::new();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        cache.mark_used("token1").unwrap();
        cache.mark_used("token2").unwrap();
        cache.mark_used("token3").unwrap();

        assert_eq!(cache.len(), 3);
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_cleanup_removes_expired_tokens() {
        let mut cache = UsedTokenCache::new();

        // Add a token
        cache.mark_used("old-token").unwrap();
        assert_eq!(cache.len(), 1);

        // Manually set timestamp to 6 minutes ago to simulate expiry
        let past = Instant::now() - std::time::Duration::from_secs(360);
        cache.timestamps.insert("old-token".to_string(), past);

        // Cleanup should remove the expired token
        cache.cleanup_old_tokens();
        assert_eq!(cache.len(), 0);
        assert!(!cache.is_used("old-token"));
    }

    #[test]
    fn test_cleanup_keeps_recent_tokens() {
        let mut cache = UsedTokenCache::new();

        // Add tokens
        cache.mark_used("recent-token1").unwrap();
        cache.mark_used("recent-token2").unwrap();

        assert_eq!(cache.len(), 2);

        // Cleanup should not remove recent tokens
        cache.cleanup_old_tokens();
        assert_eq!(cache.len(), 2);
        assert!(cache.is_used("recent-token1"));
        assert!(cache.is_used("recent-token2"));
    }

    #[test]
    fn test_multiple_tokens_independent() {
        let mut cache = UsedTokenCache::new();

        let token1 = "token-abc";
        let token2 = "token-def";
        let token3 = "token-ghi";

        // Mark tokens as used
        cache.mark_used(token1).unwrap();
        cache.mark_used(token2).unwrap();

        // Check each token independently
        assert!(cache.is_used(token1));
        assert!(cache.is_used(token2));
        assert!(!cache.is_used(token3));

        // Third token can still be used
        cache.mark_used(token3).unwrap();
        assert!(cache.is_used(token3));
    }
}
