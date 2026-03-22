//! SecureBuffer Integration Tests
//!
//! Tests for cross-platform memory protection functionality.

use keyring_cli::mcp::secure_memory::SecureBuffer;

#[test]
fn test_secure_buffer_new_with_empty_data() {
    let empty = vec![];
    let buffer = SecureBuffer::new(empty).expect("Empty buffer should be created");
    assert_eq!(buffer.as_slice().len(), 0);
    assert!(buffer.is_empty());
}

#[test]
fn test_secure_buffer_new_with_non_empty_data() {
    let data = b"sensitive data".to_vec();
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");
    // Protection may fail on some platforms, but shouldn't cause an error
    assert_eq!(buffer.as_slice(), b"sensitive data");
}

#[test]
fn test_secure_buffer_into_inner() {
    let data = b"test data".to_vec();
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");
    let inner = buffer.into_inner();
    assert_eq!(inner, b"test data");
}

#[test]
fn test_secure_buffer_as_slice() {
    let data = b"hello world".to_vec();
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");
    assert_eq!(buffer.as_slice(), b"hello world");
}

#[test]
fn test_secure_buffer_clone() {
    let data = b"clone test".to_vec();
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");

    // Cloning should create a new buffer with the same data
    let cloned = buffer.clone();
    assert_eq!(buffer.as_slice(), cloned.as_slice());
}

#[test]
fn test_secure_buffer_with_large_data() {
    // Test with 1KB of data
    let data = vec![0x42u8; 1024];
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");
    assert_eq!(buffer.as_slice().len(), 1024);
    assert!(buffer.as_slice().iter().all(|&b| b == 0x42));
}

#[test]
fn test_secure_buffer_with_zero_bytes() {
    let data = vec![0u8; 100];
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");
    assert_eq!(buffer.as_slice().len(), 100);
    assert!(buffer.as_slice().iter().all(|&b| b == 0));
}

#[test]
fn test_secure_buffer_with_utf8_string() {
    let data = "Hello 世界 🌍".as_bytes().to_vec();
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");
    assert_eq!(buffer.as_slice(), "Hello 世界 🌍".as_bytes());
}

#[test]
fn test_secure_buffer_preserves_data_integrity() {
    let original = b"integrity test data 12345!@#$%".to_vec();
    let buffer = SecureBuffer::new(original.clone()).expect("Buffer should be created");
    assert_eq!(buffer.as_slice(), original);
}

#[test]
fn test_secure_buffer_multiple_clones() {
    let data = b"multi-clone test".to_vec();
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");

    let clone1 = buffer.clone();
    let clone2 = buffer.clone();
    let clone3 = clone1.clone();

    assert_eq!(buffer.as_slice(), clone1.as_slice());
    assert_eq!(clone1.as_slice(), clone2.as_slice());
    assert_eq!(clone2.as_slice(), clone3.as_slice());
}

#[test]
fn test_secure_buffer_into_inner_consumes_buffer() {
    let data = b"consume test".to_vec();
    let buffer = SecureBuffer::new(data).expect("Buffer should be created");

    // into_inner consumes the buffer and returns the data
    let inner = buffer.into_inner();
    assert_eq!(inner, b"consume test");
}

// Integration test for use in executors
#[test]
fn test_secure_buffer_executor_pattern() {
    // Simulate how an executor would use SecureBuffer
    let key_data = b"private-key-data-12345".to_vec();
    let secure_key = SecureBuffer::new(key_data).expect("Buffer should be created");

    // Executor can read the key when needed
    let key_for_use = secure_key.as_slice();
    assert_eq!(key_for_use, b"private-key-data-12345");

    // Simulate passing to external function (into_inner)
    let key_bytes = secure_key.into_inner();
    assert_eq!(
        String::from_utf8_lossy(&key_bytes),
        "private-key-data-12345"
    );
}
