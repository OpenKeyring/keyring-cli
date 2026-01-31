//! Session Cache for MCP Authorization
//!
//! This module provides an in-memory session cache with TTL (Time-To-Live)
//! for session-level authorization. Once a credential is authorized,
//! it can be reused for the duration of the TTL (default: 1 hour).
//!
//! # Example
//!
//! ```rust
//! use keyring_cli::mcp::policy::session::SessionCache;
//!
//! let mut cache = SessionCache::new(100, 3600); // max 100 entries, 1 hour TTL
//!
//! // Authorize a credential
//! cache.authorize("my-credential").unwrap();
//!
//! // Check if authorized (should be true)
//! assert!(cache.is_authorized("my-credential"));
//!
//! // After TTL expires, this will return false
//! ```

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::time::Instant;

/// Session cache for storing authorization state
///
/// Maintains a HashMap of authorized credentials with their authorization
/// timestamps. Entries expire after the configured TTL.
#[derive(Debug)]
pub struct SessionCache {
    /// Cache entries keyed by credential name
    entries: HashMap<String, CacheEntry>,

    /// Maximum number of entries before eviction
    max_entries: usize,

    /// Time-to-live for cache entries in seconds
    ttl_seconds: u64,
}

/// Individual cache entry
#[derive(Debug, Clone)]
#[allow(dead_code)]  // credential_name reserved for future debugging/auditing
struct CacheEntry {
    /// When this credential was authorized
    authorized_at: Instant,

    /// Name of the credential
    credential_name: String,
}

impl SessionCache {
    /// Create a new session cache
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of cached sessions before LRU eviction
    /// * `ttl_seconds` - Time-to-live for cached sessions in seconds
    ///
    /// # Returns
    /// A new SessionCache instance
    #[must_use]
    pub fn new(max_entries: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            ttl_seconds,
        }
    }

    /// Mark a credential as authorized for this session
    ///
    /// Stores the current timestamp for the credential. If the cache is at
    /// maximum capacity, the oldest entry will be evicted.
    ///
    /// # Arguments
    /// * `credential_name` - Name of the credential to authorize
    ///
    /// # Returns
    /// * `Ok(())` - Successfully authorized
    /// * `Err(Error)` - Authorization failed
    ///
    /// # Errors
    /// Returns an error if the credential name is empty
    pub fn authorize(&mut self, credential_name: &str) -> Result<()> {
        if credential_name.is_empty() {
            return Err(Error::InvalidInput {
                context: "Credential name cannot be empty".to_string(),
            });
        }

        // Evict oldest entry if at capacity
        if self.entries.len() >= self.max_entries {
            self.evict_oldest();
        }

        let entry = CacheEntry {
            authorized_at: Instant::now(),
            credential_name: credential_name.to_string(),
        };

        self.entries.insert(credential_name.to_string(), entry);

        Ok(())
    }

    /// Check if a credential is authorized (not expired)
    ///
    /// Returns true if:
    /// - The credential is in the cache
    /// - The authorization timestamp is within the TTL window
    ///
    /// # Arguments
    /// * `credential_name` - Name of the credential to check
    ///
    /// # Returns
    /// `true` if the credential is authorized and not expired, `false` otherwise
    #[must_use]
    pub fn is_authorized(&self, credential_name: &str) -> bool {
        if let Some(entry) = self.entries.get(credential_name) {
            let elapsed = entry.authorized_at.elapsed().as_secs();
            elapsed < self.ttl_seconds
        } else {
            false
        }
    }

    /// Remove expired entries from the cache
    ///
    /// Iterates through all entries and removes those that have exceeded
    /// the TTL period. This should be called periodically to maintain
    /// cache hygiene.
    pub fn cleanup_expired(&mut self) {
        let ttl = self.ttl_seconds;
        self.entries.retain(|_, entry| {
            let elapsed = entry.authorized_at.elapsed().as_secs();
            elapsed < ttl
        });
    }

    /// Get the current number of entries in the cache
    ///
    /// # Returns
    /// The number of cached sessions
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    ///
    /// # Returns
    /// `true` if no entries are cached, `false` otherwise
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Evict the oldest entry from the cache
    ///
    /// Uses LRU (Least Recently Used) policy based on authorization timestamp.
    /// This is automatically called when adding a new entry would exceed
    /// max_entries.
    fn evict_oldest(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        // Find the oldest entry
        let oldest_key = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.authorized_at)
            .map(|(key, _)| key.clone());

        if let Some(key) = oldest_key {
            self.entries.remove(&key);
        }
    }

    /// Get the time remaining for a credential's authorization
    ///
    /// # Arguments
    /// * `credential_name` - Name of the credential to check
    ///
    /// # Returns
    /// * `Some(seconds)` - Seconds remaining until expiration
    /// * `None` - Credential not found or already expired
    #[must_use]
    pub fn time_remaining(&self, credential_name: &str) -> Option<u64> {
        self.entries.get(credential_name).map(|entry| {
            let elapsed = entry.authorized_at.elapsed().as_secs();
            self.ttl_seconds.saturating_sub(elapsed)
        })
    }
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new(100, 3600) // 100 entries, 1 hour TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_default_creation() {
        let cache = SessionCache::default();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_authorize_success() {
        let mut cache = SessionCache::new(10, 60);
        let result = cache.authorize("test-credential");
        assert!(result.is_ok());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_authorize_empty_name() {
        let mut cache = SessionCache::new(10, 60);
        let result = cache.authorize("");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_authorized_after_authorize() {
        let mut cache = SessionCache::new(10, 60);
        cache.authorize("my-credential").unwrap();
        assert!(cache.is_authorized("my-credential"));
    }

    #[test]
    fn test_is_authorized_not_found() {
        let cache = SessionCache::new(10, 60);
        assert!(!cache.is_authorized("non-existent"));
    }

    #[test]
    fn test_ttl_expiration() {
        let mut cache = SessionCache::new(10, 1); // 1 second TTL
        cache.authorize("test-credential").unwrap();

        // Should be authorized immediately
        assert!(cache.is_authorized("test-credential"));

        // Wait for TTL to expire
        thread::sleep(Duration::from_secs(2));

        // Should no longer be authorized
        assert!(!cache.is_authorized("test-credential"));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut cache = SessionCache::new(10, 1); // 1 second TTL
        cache.authorize("expiring-credential").unwrap();
        cache.authorize("another-credential").unwrap();

        assert_eq!(cache.len(), 2);

        // Wait for expiration
        thread::sleep(Duration::from_secs(2));

        // Cleanup should remove expired entries
        cache.cleanup_expired();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_max_entries_eviction() {
        let mut cache = SessionCache::new(2, 60); // Max 2 entries

        cache.authorize("credential-1").unwrap();
        thread::sleep(Duration::from_millis(10));
        cache.authorize("credential-2").unwrap();
        thread::sleep(Duration::from_millis(10));
        cache.authorize("credential-3").unwrap(); // Should evict credential-1

        assert_eq!(cache.len(), 2);
        assert!(!cache.is_authorized("credential-1")); // Evicted
        assert!(cache.is_authorized("credential-2"));
        assert!(cache.is_authorized("credential-3"));
    }

    #[test]
    fn test_clear() {
        let mut cache = SessionCache::new(10, 60);
        cache.authorize("credential-1").unwrap();
        cache.authorize("credential-2").unwrap();

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_time_remaining() {
        let mut cache = SessionCache::new(10, 60);
        cache.authorize("test-credential").unwrap();

        let remaining = cache.time_remaining("test-credential");
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= 60);
        assert!(remaining.unwrap() > 50); // Should have most of the time left
    }

    #[test]
    fn test_time_remaining_not_found() {
        let cache = SessionCache::new(10, 60);
        assert!(cache.time_remaining("non-existent").is_none());
    }

    #[test]
    fn test_multiple_credentials() {
        let mut cache = SessionCache::new(10, 60);
        cache.authorize("cred-1").unwrap();
        cache.authorize("cred-2").unwrap();
        cache.authorize("cred-3").unwrap();

        assert!(cache.is_authorized("cred-1"));
        assert!(cache.is_authorized("cred-2"));
        assert!(cache.is_authorized("cred-3"));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_reauthorize_refreshes_timestamp() {
        let mut cache = SessionCache::new(10, 60);
        cache.authorize("test-credential").unwrap();

        thread::sleep(Duration::from_millis(100));

        // Re-authorize should refresh the timestamp
        cache.authorize("test-credential").unwrap();

        let remaining = cache.time_remaining("test-credential").unwrap();
        // Should have close to full TTL remaining
        assert!(remaining > 59);
    }
}
