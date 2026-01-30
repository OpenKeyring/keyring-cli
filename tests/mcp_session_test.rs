//! Session Cache Tests
//!
//! Comprehensive tests for the SessionCache including TTL logic, eviction,
//! and cleanup functionality.

use keyring_cli::mcp::auth::session::SessionCache;
use std::thread;
use std::time::Duration;

#[test]
fn test_default_creation() {
    let cache = SessionCache::default();

    // Should have default values
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn test_custom_creation() {
    let cache = SessionCache::new(50, 1800);

    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn test_authorize_success() {
    let mut cache = SessionCache::new(10, 60);
    let result = cache.authorize("test-credential");

    assert!(result.is_ok(), "Authorization should succeed");
    assert_eq!(cache.len(), 1, "Cache should have one entry");
}

#[test]
fn test_authorize_empty_name() {
    let mut cache = SessionCache::new(10, 60);
    let result = cache.authorize("");

    assert!(result.is_err(), "Empty credential name should fail");
}

#[test]
fn test_authorize_whitespace_name() {
    let mut cache = SessionCache::new(10, 60);

    // Whitespace-only should be treated as non-empty
    // (it's up to the caller to validate credential names)
    let result = cache.authorize("   ");
    assert!(result.is_ok());
}

#[test]
fn test_is_authorized_after_authorize() {
    let mut cache = SessionCache::new(10, 60);
    cache.authorize("my-credential").unwrap();

    assert!(
        cache.is_authorized("my-credential"),
        "Should be authorized immediately after authorize()"
    );
}

#[test]
fn test_is_authorized_not_found() {
    let cache = SessionCache::new(10, 60);

    assert!(
        !cache.is_authorized("non-existent"),
        "Non-existent credential should not be authorized"
    );
}

#[test]
fn test_one_hour_ttl() {
    let mut cache = SessionCache::new(10, 3600); // 1 hour TTL
    cache.authorize("test-credential").unwrap();

    // Should be authorized immediately
    assert!(cache.is_authorized("test-credential"));

    // Check time remaining
    let remaining = cache.time_remaining("test-credential");
    assert!(remaining.is_some());
    assert!(remaining.unwrap() <= 3600);
    assert!(remaining.unwrap() > 3590); // Should have most of the time
}

#[test]
fn test_ttl_expiration_short() {
    let mut cache = SessionCache::new(10, 1); // 1 second TTL
    cache.authorize("test-credential").unwrap();

    // Should be authorized immediately
    assert!(
        cache.is_authorized("test-credential"),
        "Should be authorized immediately"
    );

    // Wait for TTL to expire
    thread::sleep(Duration::from_secs(2));

    // Should no longer be authorized
    assert!(
        !cache.is_authorized("test-credential"),
        "Should not be authorized after TTL expires"
    );
}

#[test]
fn test_ttl_expiration_medium() {
    let mut cache = SessionCache::new(10, 2); // 2 second TTL
    cache.authorize("test-credential").unwrap();

    // Should be authorized at 1 second
    thread::sleep(Duration::from_secs(1));
    assert!(cache.is_authorized("test-credential"));

    // Should not be authorized at 3 seconds
    thread::sleep(Duration::from_secs(2));
    assert!(!cache.is_authorized("test-credential"));
}

#[test]
fn test_cleanup_expired() {
    let mut cache = SessionCache::new(10, 1); // 1 second TTL
    cache.authorize("expiring-credential-1").unwrap();
    cache.authorize("expiring-credential-2").unwrap();
    cache.authorize("expiring-credential-3").unwrap();

    assert_eq!(cache.len(), 3, "Should have 3 entries");

    // Wait for expiration
    thread::sleep(Duration::from_secs(2));

    // Cleanup should remove expired entries
    cache.cleanup_expired();

    assert_eq!(cache.len(), 0, "All entries should be cleaned up");
    assert!(cache.is_empty(), "Cache should be empty");
}

#[test]
fn test_cleanup_expired_partial() {
    let mut cache = SessionCache::new(10, 1); // 1 second TTL

    // Add first batch
    cache.authorize("expiring-1").unwrap();
    cache.authorize("expiring-2").unwrap();

    // Wait for them to expire
    thread::sleep(Duration::from_secs(2));

    // Add new entry
    cache.authorize("fresh-credential").unwrap();

    assert_eq!(cache.len(), 3, "Should have 3 entries");

    // Cleanup should remove only expired entries
    cache.cleanup_expired();

    assert_eq!(cache.len(), 1, "Only fresh entry should remain");
    assert!(cache.is_authorized("fresh-credential"));
}

#[test]
fn test_cleanup_expired_none_expired() {
    let mut cache = SessionCache::new(10, 60); // 60 second TTL
    cache.authorize("credential-1").unwrap();
    cache.authorize("credential-2").unwrap();

    assert_eq!(cache.len(), 2);

    // Cleanup when nothing is expired
    cache.cleanup_expired();

    assert_eq!(cache.len(), 2, "No entries should be removed");
}

#[test]
fn test_max_entries_eviction_lru() {
    let mut cache = SessionCache::new(2, 60); // Max 2 entries

    cache.authorize("credential-1").unwrap();
    thread::sleep(Duration::from_millis(10));
    cache.authorize("credential-2").unwrap();
    thread::sleep(Duration::from_millis(10));
    cache.authorize("credential-3").unwrap(); // Should evict credential-1

    assert_eq!(cache.len(), 2, "Should have max 2 entries");
    assert!(
        !cache.is_authorized("credential-1"),
        "Oldest entry should be evicted"
    );
    assert!(
        cache.is_authorized("credential-2"),
        "Second entry should still be present"
    );
    assert!(
        cache.is_authorized("credential-3"),
        "Newest entry should be present"
    );
}

#[test]
fn test_max_entries_eviction_fifo_order() {
    let mut cache = SessionCache::new(3, 60); // Max 3 entries

    cache.authorize("cred-1").unwrap();
    thread::sleep(Duration::from_millis(10));
    cache.authorize("cred-2").unwrap();
    thread::sleep(Duration::from_millis(10));
    cache.authorize("cred-3").unwrap();
    thread::sleep(Duration::from_millis(10));
    cache.authorize("cred-4").unwrap(); // Evicts cred-1
    thread::sleep(Duration::from_millis(10));
    cache.authorize("cred-5").unwrap(); // Evicts cred-2

    assert_eq!(cache.len(), 3);
    assert!(!cache.is_authorized("cred-1"));
    assert!(!cache.is_authorized("cred-2"));
    assert!(cache.is_authorized("cred-3"));
    assert!(cache.is_authorized("cred-4"));
    assert!(cache.is_authorized("cred-5"));
}

#[test]
fn test_max_entries_exact() {
    let mut cache = SessionCache::new(2, 60);

    cache.authorize("credential-1").unwrap();
    cache.authorize("credential-2").unwrap();

    assert_eq!(cache.len(), 2, "Should have exactly 2 entries");
    assert!(cache.is_authorized("credential-1"));
    assert!(cache.is_authorized("credential-2"));
}

#[test]
fn test_clear() {
    let mut cache = SessionCache::new(10, 60);
    cache.authorize("credential-1").unwrap();
    cache.authorize("credential-2").unwrap();
    cache.authorize("credential-3").unwrap();

    assert_eq!(cache.len(), 3);

    cache.clear();

    assert_eq!(cache.len(), 0, "Cache should be empty after clear");
    assert!(cache.is_empty(), "is_empty should return true");
    assert!(!cache.is_authorized("credential-1"));
    assert!(!cache.is_authorized("credential-2"));
    assert!(!cache.is_authorized("credential-3"));
}

#[test]
fn test_time_remaining() {
    let mut cache = SessionCache::new(10, 60);
    cache.authorize("test-credential").unwrap();

    let remaining = cache.time_remaining("test-credential");

    assert!(remaining.is_some(), "Should return Some for existing credential");
    assert!(remaining.unwrap() <= 60, "Should not exceed TTL");
    assert!(remaining.unwrap() > 50, "Should have most time remaining");
}

#[test]
fn test_time_remaining_decreases() {
    let mut cache = SessionCache::new(10, 60);
    cache.authorize("test-credential").unwrap();

    let remaining1 = cache.time_remaining("test-credential").unwrap();

    thread::sleep(Duration::from_secs(1));

    let remaining2 = cache.time_remaining("test-credential").unwrap();

    assert!(
        remaining2 < remaining1,
        "Time remaining should decrease"
    );
}

#[test]
fn test_time_remaining_not_found() {
    let cache = SessionCache::new(10, 60);

    let remaining = cache.time_remaining("non-existent");

    assert!(remaining.is_none(), "Should return None for non-existent credential");
}

#[test]
fn test_time_remaining_expired() {
    let mut cache = SessionCache::new(10, 1); // 1 second TTL
    cache.authorize("test-credential").unwrap();

    // Wait for expiration
    thread::sleep(Duration::from_secs(2));

    // time_remaining might still return Some (with 0), but is_authorized should be false
    let remaining = cache.time_remaining("test-credential");

    // After expiration, time_remaining returns 0 due to saturating_sub
    assert_eq!(remaining, Some(0));
    assert!(!cache.is_authorized("test-credential"));
}

#[test]
fn test_multiple_credentials() {
    let mut cache = SessionCache::new(10, 60);
    cache.authorize("cred-1").unwrap();
    cache.authorize("cred-2").unwrap();
    cache.authorize("cred-3").unwrap();
    cache.authorize("cred-4").unwrap();
    cache.authorize("cred-5").unwrap();

    assert!(cache.is_authorized("cred-1"));
    assert!(cache.is_authorized("cred-2"));
    assert!(cache.is_authorized("cred-3"));
    assert!(cache.is_authorized("cred-4"));
    assert!(cache.is_authorized("cred-5"));
    assert_eq!(cache.len(), 5);
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
    assert!(remaining > 59, "Should have nearly full TTL after reauthorize");
}

#[test]
fn test_reauthorize_multiple_times() {
    let mut cache = SessionCache::new(10, 60);

    // Authorize same credential multiple times
    cache.authorize("test-credential").unwrap();
    thread::sleep(Duration::from_millis(50));
    cache.authorize("test-credential").unwrap();
    thread::sleep(Duration::from_millis(50));
    cache.authorize("test-credential").unwrap();

    // Should still have only one entry
    assert_eq!(cache.len(), 1);

    // But should have fresh timestamp
    let remaining = cache.time_remaining("test-credential").unwrap();
    assert!(remaining > 59);
}

#[test]
fn test_different_credentials_independent() {
    let mut cache = SessionCache::new(10, 2); // 2 second TTL

    cache.authorize("credential-1").unwrap();
    thread::sleep(Duration::from_secs(1));
    cache.authorize("credential-2").unwrap();

    // Both should be authorized at 1 second
    assert!(cache.is_authorized("credential-1"));
    assert!(cache.is_authorized("credential-2"));

    thread::sleep(Duration::from_secs(2));

    // At 3 seconds, both should be expired
    assert!(!cache.is_authorized("credential-1"));
    assert!(!cache.is_authorized("credential-2"));
}

#[test]
fn test_case_sensitive_credential_names() {
    let mut cache = SessionCache::new(10, 60);
    cache.authorize("MyCredential").unwrap();

    assert!(cache.is_authorized("MyCredential"));
    assert!(!cache.is_authorized("mycredential"));
    assert!(!cache.is_authorized("MYCREDENTIAL"));
}

#[test]
fn test_special_characters_in_credential_names() {
    let mut cache = SessionCache::new(10, 60);

    // Test various special characters
    let names = vec![
        "my-credential-1",
        "my_credential_2",
        "my.credential.3",
        "my/credential/4",
        "my@credential#5",
        "credential:with:colons",
        "credential with spaces",
    ];

    for name in &names {
        cache.authorize(name).unwrap();
        assert!(cache.is_authorized(name), "{} should be authorized", name);
    }

    assert_eq!(cache.len(), names.len());
}

#[test]
fn test_unicode_credential_names() {
    let mut cache = SessionCache::new(10, 60);

    // Test Unicode characters
    let names = vec!["credential-测试", "credential-🔑", "credential-привет"];

    for name in &names {
        cache.authorize(name).unwrap();
        assert!(cache.is_authorized(name));
    }

    assert_eq!(cache.len(), 3);
}

#[test]
fn test_single_entry_cache() {
    let mut cache = SessionCache::new(1, 60);

    cache.authorize("credential-1").unwrap();
    assert_eq!(cache.len(), 1);

    cache.authorize("credential-2").unwrap();
    assert_eq!(cache.len(), 1);

    // Only the last credential should be present
    assert!(!cache.is_authorized("credential-1"));
    assert!(cache.is_authorized("credential-2"));
}

#[test]
fn test_large_cache_performance() {
    let mut cache = SessionCache::new(1000, 60);

    // Add 100 entries
    for i in 0..100 {
        cache.authorize(&format!("credential-{}", i)).unwrap();
    }

    assert_eq!(cache.len(), 100);

    // Verify all are authorized
    for i in 0..100 {
        assert!(cache.is_authorized(&format!("credential-{}", i)));
    }
}

#[test]
fn test_cleanup_on_full_cache() {
    let mut cache = SessionCache::new(5, 1); // Small cache, 1 second TTL

    // Fill the cache
    for i in 0..5 {
        cache.authorize(&format!("credential-{}", i)).unwrap();
    }

    assert_eq!(cache.len(), 5);

    // Wait for expiration
    thread::sleep(Duration::from_secs(2));

    // All should be expired
    for i in 0..5 {
        assert!(!cache.is_authorized(&format!("credential-{}", i)));
    }

    // Cleanup should remove all
    cache.cleanup_expired();
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_no_cleanup_before_ttl() {
    let mut cache = SessionCache::new(10, 60);

    cache.authorize("credential-1").unwrap();
    cache.authorize("credential-2").unwrap();

    // Cleanup immediately after adding (before TTL)
    cache.cleanup_expired();

    // Entries should still be present
    assert_eq!(cache.len(), 2);
    assert!(cache.is_authorized("credential-1"));
    assert!(cache.is_authorized("credential-2"));
}

#[test]
fn test_is_authorized_case_exact_match() {
    let mut cache = SessionCache::new(10, 60);
    cache.authorize("ExactCase").unwrap();

    // Only exact match should work
    assert!(cache.is_authorized("ExactCase"));
    assert!(!cache.is_authorized("exactcase"));
    assert!(!cache.is_authorized("EXACTCASE"));
    assert!(!cache.is_authorized("exactCase"));
}
