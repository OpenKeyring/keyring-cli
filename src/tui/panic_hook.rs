//! Panic hook for TUI terminal recovery
//!
//! This module provides a panic hook that ensures the terminal is properly
//! restored when a panic occurs during TUI operation. Without this hook,
//! the terminal would remain in raw mode and alternate screen, leaving
//! the user's terminal in an unusable state.
//!
//! # Usage
//!
//! Call [`install_panic_hook()`] at the very beginning of any TUI entry point:
//!
//! ```rust,ignore
//! use keyring_cli::tui::panic_hook::install_panic_hook;
//!
//! fn main() -> Result<()> {
//!     install_panic_hook(); // Must be called before terminal initialization
//!     // ... rest of the TUI setup
//! }
//! ```

use std::io::{self, Write};
use std::panic;

/// Install a panic hook that restores the terminal state before panicking.
///
/// This function sets up a custom panic hook that:
/// 1. Attempts to disable raw mode
/// 2. Attempts to leave the alternate screen
/// 3. Shows the cursor
/// 4. Calls the original panic hook to display the panic message
///
/// # Note
///
/// This should be called at the very beginning of any TUI entry point,
/// before any terminal initialization occurs.
///
/// # Errors in Recovery
///
/// If terminal recovery fails (which is possible during a panic), the errors
/// are silently ignored. This is intentional - we're already in a panic
/// situation and should focus on showing the panic message rather than
/// handling secondary errors.
pub fn install_panic_hook() {
    let original_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // Silently attempt to restore terminal state
        // We ignore errors because we're already in a panic situation
        let _ = restore_terminal_silent();

        // Call the original panic hook to display the panic message
        original_hook(panic_info);
    }));
}

/// Attempt to restore terminal state without producing any output.
///
/// This is used in the panic hook where we want to silently restore
/// the terminal without adding more error messages to the panic output.
fn restore_terminal_silent() -> io::Result<()> {
    use crossterm::{
        cursor::Show,
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };

    // Disable raw mode first
    disable_raw_mode()?;

    // Leave alternate screen and show cursor
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, Show)?;

    // Ensure output is flushed
    stdout.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_panic_hook_does_not_panic() {
        // This test verifies that installing the panic hook doesn't panic
        install_panic_hook();
        // Install again to verify it's idempotent
        install_panic_hook();
    }
}
