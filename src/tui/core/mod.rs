//! TUI 核心实现层
//!
//! 本模块提供了 TUI 框架所有 trait 的默认实现，包括：
//! - 焦点管理器
//! - 状态管理器
//! - 屏幕管理器
//! - 主题实现
//! - 事件调度器
//! - 应用程序框架

mod focus_manager;
mod state_manager;
mod screen_manager;
mod notification;
mod theme;
mod validation;
mod password_strength;
mod task_manager;
mod app;
mod dispatcher;

// 重新导出所有核心实现
pub use focus_manager::DefaultFocusManager;
pub use state_manager::DefaultStateManager;
pub use screen_manager::DefaultScreenManager;
pub use notification::DefaultNotificationManager;
// 主题实现从 traits 模块重新导出
pub use crate::tui::traits::{DarkTheme, LightTheme};
pub use validation::{DefaultFormValidator, SimpleValidationRule};
pub use password_strength::DefaultPasswordStrengthCalculator;
pub use task_manager::TokioTaskManager;
pub use app::{Application, TuiApp};
pub use dispatcher::EventDispatcher;
