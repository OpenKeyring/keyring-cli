//! TUI 组件实现
//!
//! 提供可复用的 UI 组件，实现 Component trait。

mod status_bar;
mod text_input;

pub use status_bar::StatusBar;
pub use text_input::TextInput;
