//! TUI Command Handlers
//!
//! Handlers for slash commands in TUI mode.

pub mod config;
pub mod delete;
pub mod health;
pub mod list;
pub mod new;
pub mod search;
pub mod show;
pub mod update;

// Re-export command handlers for external use
// Note: Command handlers are exported but may not be used internally
// They are part of the public API for external consumers
#[allow(unused_imports)]
pub use config::handle_config;
#[allow(unused_imports)]
pub use delete::handle_delete;
#[allow(unused_imports)]
pub use health::handle_health;
#[allow(unused_imports)]
pub use list::handle_list;
#[allow(unused_imports)]
pub use new::handle_new;
#[allow(unused_imports)]
pub use search::handle_search;
#[allow(unused_imports)]
pub use show::handle_show;
#[allow(unused_imports)]
pub use update::handle_update;

/// Parse a command string into command name and arguments
#[allow(dead_code)]
pub fn parse_command(input: &str) -> Option<(&str, Vec<&str>)> {
    let input = input.trim();
    if !input.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let command = parts[0];
    let args = if parts.len() > 1 {
        parts[1].split_whitespace().collect()
    } else {
        Vec::new()
    };

    Some((command, args))
}
