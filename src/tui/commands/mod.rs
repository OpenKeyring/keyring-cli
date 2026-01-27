//! TUI Command Handlers
//!
//! Handlers for slash commands in TUI mode.

mod list;
mod show;
mod new;
mod update;
mod delete;
mod search;

// Re-export command handlers for external use
#[allow(unused_imports)]
pub use list::handle_list;
#[allow(unused_imports)]
pub use show::handle_show;
#[allow(unused_imports)]
pub use new::handle_new;
#[allow(unused_imports)]
pub use update::handle_update;
#[allow(unused_imports)]
pub use delete::handle_delete;
pub use search::handle_search;

/// Parse a command string into command name and arguments
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
