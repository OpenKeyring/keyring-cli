// tests/used_tokens_test.rs
// Integration tests for used token cache

use keyring_cli::mcp::UsedTokenCache;
use std::time::Duration;

#[test]
fn test_token_replay_prevention() {
    let mut cache = UsedTokenCache::new();
    let token_id = "replay-test-token";

    // First use should succeed
    assert!(cache.mark_used(token_id).is_ok());
    assert!(cache.is_used(token_id));

    // Second use should fail (replay attack)
    let result = cache.mark_used(token_id);
    assert!(result.is_err());

    // Verify the error message contains the token ID
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains(token_id));
    assert!(err_msg.contains("already used"));
}

#[test]
fn test_multiple_unique_tokens() {
    let mut cache = UsedTokenCache::new();

    // Use multiple different tokens
    let tokens = vec!["token-1", "token-2", "token-3"];
    for token in &tokens {
        assert!(cache.mark_used(token).is_ok());
        assert!(cache.is_used(token));
    }

    // Verify all tokens are tracked
    assert_eq!(cache.len(), 3);

    // Re-using any of them should fail
    for token in &tokens {
        assert!(cache.mark_used(token).is_err());
    }
}

#[test]
fn test_cleanup_old_tokens() {
    let mut cache = UsedTokenCache::new();

    // Add a token that we'll mark as old
    cache.mark_used("old-token").unwrap();

    // Manually expire the token by modifying its timestamp
    // (In real usage, this would happen naturally over time)
    let past = std::time::Instant::now() - Duration::from_secs(360); // 6 minutes ago
    cache.timestamps.insert("old-token".to_string(), past);

    // Add recent tokens
    cache.mark_used("recent-token-1").unwrap();
    cache.mark_used("recent-token-2").unwrap();

    assert_eq!(cache.len(), 3);

    // Cleanup should remove only the old token
    cache.cleanup_old_tokens();

    assert_eq!(cache.len(), 2);
    assert!(!cache.is_used("old-token"));
    assert!(cache.is_used("recent-token-1"));
    assert!(cache.is_used("recent-token-2"));
}

#[test]
fn test_cache_size_tracking() {
    let mut cache = UsedTokenCache::new();

    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());

    // Add tokens
    for i in 1..=5 {
        cache.mark_used(&format!("token-{}", i)).unwrap();
    }

    assert_eq!(cache.len(), 5);
    assert!(!cache.is_empty());
}

#[test]
fn test_concurrent_token_use() {
    let mut cache = UsedTokenCache::new();
    let token_id = "concurrent-token";

    // First thread marks token as used
    let result1 = cache.mark_used(token_id);
    assert!(result1.is_ok());

    // Simulate another thread trying to use the same token
    let result2 = cache.mark_used(token_id);
    assert!(result2.is_err());

    // Both should see the token as used
    assert!(cache.is_used(token_id));
}

#[test]
fn test_token_expiry_boundary() {
    let mut cache = UsedTokenCache::new();

    // Add tokens at different times
    cache.mark_used("token-4min").unwrap();
    cache.mark_used("token-5min").unwrap();
    cache.mark_used("token-6min").unwrap();

    // Manually set timestamps
    let now = std::time::Instant::now();
    cache
        .timestamps
        .insert("token-4min".to_string(), now - Duration::from_secs(240)); // 4 min
    cache
        .timestamps
        .insert("token-5min".to_string(), now - Duration::from_secs(300)); // 5 min
    cache
        .timestamps
        .insert("token-6min".to_string(), now - Duration::from_secs(360)); // 6 min

    // Before cleanup
    assert_eq!(cache.len(), 3);

    // Cleanup removes tokens older than 5 minutes
    cache.cleanup_old_tokens();

    // Only token-4min should remain
    assert_eq!(cache.len(), 1);
    assert!(cache.is_used("token-4min"));
    assert!(!cache.is_used("token-5min"));
    assert!(!cache.is_used("token-6min"));
}

#[test]
fn test_empty_cache_behavior() {
    let cache = UsedTokenCache::new();

    // Empty cache should report no tokens used
    assert!(!cache.is_used("any-token"));
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());

    // Cleanup on empty cache should be safe
    let mut cache = UsedTokenCache::new();
    cache.cleanup_old_tokens();
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_large_token_set() {
    let mut cache = UsedTokenCache::new();

    // Add a large number of tokens
    let num_tokens = 1000;
    for i in 0..num_tokens {
        let token_id = format!("bulk-token-{:04}", i);
        assert!(cache.mark_used(&token_id).is_ok());
    }

    assert_eq!(cache.len(), num_tokens);

    // Verify a random sample
    assert!(cache.is_used("bulk-token-0000"));
    assert!(cache.is_used("bulk-token-0500"));
    assert!(cache.is_used("bulk-token-0999"));

    // Try to reuse a token
    let result = cache.mark_used("bulk-token-0420");
    assert!(result.is_err());
}
