//! TUI New Command Handler
//!
//! Handles the /new command in TUI mode.

use crate::error::Result;

/// Handle the /new command
#[allow(dead_code)]
pub fn handle_new() -> Result<Vec<String>> {
    // TODO: Implement interactive new record wizard
    // For now, provide usage instructions
    Ok(vec![
        "✏️  Creating new record".to_string(),
        "".to_string(),
        "To create a new record, use the CLI command:".to_string(),
        "  ok generate --name <name> --length 16".to_string(),
        "".to_string(),
        "Or with memorable password:".to_string(),
        "  ok generate --name <name> --memorable --words 4".to_string(),
        "".to_string(),
        "(Interactive wizard coming soon to TUI)".to_string(),
    ])
}
