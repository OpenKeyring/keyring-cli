//! Sensitive data types with automatic memory zeroization
//!
//! This module provides wrapper types for sensitive data that automatically
//! zeroize memory when dropped, preventing sensitive data from remaining in memory.
//!
//! # Integration Status
//!
//! **M1 v0.1**: Type implemented and used in TUI password widget
//! **M1 v0.2**: Full integration planned (Vault, Record, crypto operations)
//!
//! See `docs/plans/2026-01-27-m1-security-and-tui-design.md` for details.

use zeroize::Zeroize;

/// Wrapper for sensitive data that auto-zeroizes on drop
///
/// # Type Parameters
/// * `T` - The inner type (must implement Zeroize)
///
/// # Security
/// - No Clone implementation (prevents accidental duplication)
/// - Custom Debug that redacts output
/// - Auto-zeroizes via Drop implementation
/// - Controlled read access via `.get()`
///
/// # Examples
/// ```rust
/// use keyring_cli::types::SensitiveString;
///
/// // Wrap a password
/// let password = SensitiveString::new("secret123".to_string());
///
/// // Access the value
/// assert_eq!(password.get(), &"secret123".to_string());
///
/// // When dropped, the memory is zeroized
/// drop(password);
/// ```
pub struct SensitiveString<T: Zeroize> {
    inner: T,
}

impl<T: Zeroize> SensitiveString<T> {
    /// Create a new SensitiveString wrapper
    ///
    /// # Arguments
    /// * `value` - The sensitive value to wrap
    pub fn new(value: T) -> Self
    where
        T: Zeroize,
    {
        Self { inner: value }
    }

    /// Get a reference to the inner value
    ///
    /// # Returns
    /// A reference to the wrapped value
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Consume the wrapper and return the inner value
    ///
    /// # Warning
    /// This transfers ownership of the sensitive data.
    /// The caller is responsible for ensuring the data is properly zeroized.
    pub fn into_inner(self) -> T {
        // Use ManuallyDrop to prevent Drop from running while extracting the value
        let this = std::mem::ManuallyDrop::new(self);
        // SAFETY: self is being consumed and won't be dropped
        unsafe { std::ptr::read(&this.inner as *const T) }
    }
}

impl<T: Zeroize> Drop for SensitiveString<T> {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

// Prevent cloning (security measure)
impl<T: Zeroize> Clone for SensitiveString<T> {
    fn clone(&self) -> Self {
        panic!("SensitiveString cannot be cloned - this prevents accidental duplication of sensitive data");
    }
}

// Custom Debug that redacts output
impl<T: Zeroize> std::fmt::Debug for SensitiveString<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SensitiveString")
            .field("inner", &"***REDACTED***")
            .finish()
    }
}

// Custom Display that redacts output
impl<T: Zeroize> std::fmt::Display for SensitiveString<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "***REDACTED***")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_string_creation() {
        let s = SensitiveString::new("test".to_string());
        assert_eq!(s.get(), &"test".to_string());
    }

    #[test]
    fn test_sensitive_string_into_inner() {
        let s = SensitiveString::new("test".to_string());
        let inner = s.into_inner();
        assert_eq!(inner, "test");
    }

    #[test]
    fn test_sensitive_string_debug_redacts() {
        let s = SensitiveString::new("secret".to_string());
        let debug_str = format!("{:?}", s);
        assert!(!debug_str.contains("secret"));
        assert!(debug_str.contains("REDACTED"));
    }

    #[test]
    fn test_sensitive_string_display_redacts() {
        let s = SensitiveString::new("secret".to_string());
        let display_str = format!("{}", s);
        assert_eq!(display_str, "***REDACTED***");
    }

    #[test]
    #[should_panic(expected = "cannot be cloned")]
    fn test_sensitive_string_no_clone() {
        let s = SensitiveString::new("test".to_string());
        let _ = s.clone();
    }
}
