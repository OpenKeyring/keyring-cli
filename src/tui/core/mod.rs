//! TUI 核心实现层
//!
//! 本模块提供了 TUI 框架所有 trait 的默认实现，包括：
//! - 焦点管理器
//! - 状态管理器
//! - 屏幕管理器
//! - 主题实现
//! - 事件调度器

mod dispatcher;
mod focus_manager;
mod notification;
mod password_strength;
mod screen_manager;
mod state_manager;
mod task_manager;
mod theme;
mod validation;

// 重新导出所有核心实现
pub use focus_manager::DefaultFocusManager;
pub use notification::DefaultNotificationManager;
pub use screen_manager::DefaultScreenManager;
pub use state_manager::DefaultStateManager;
// 主题实现从 traits 模块重新导出
pub use crate::tui::traits::EventDispatcher;
pub use crate::tui::traits::{DarkTheme, LightTheme};
pub use dispatcher::{DefaultEventDispatcher, ImeEventFilter};
pub use password_strength::{
    estimate_crack_time, DefaultPasswordStrengthCalculator, PasswordStrengthDetails,
};
pub use task_manager::TokioTaskManager;
pub use validation::{DefaultFormValidator, SimpleValidationRule};
