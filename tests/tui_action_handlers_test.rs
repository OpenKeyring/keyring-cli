//! TUI Action Handler Tests
//!
//! The old action handlers (handle_key_event, output_lines) have been removed
//! as part of the TUI MVP refactoring. Actions are now routed through the
//! per-screen event loop in terminal.rs and handled in events.rs.
//! These tests will be rewritten to test the new action routing.
