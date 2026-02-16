//! 焦点管理 trait 定义
//!
//! 占位符模块，完整实现将在 Task A.4 中完成。

use crate::tui::traits::ComponentId;

/// 焦点管理器 trait
pub trait FocusManager: Send + Sync {
    /// 设置焦点
    fn set_focus(&mut self, id: ComponentId) -> bool;

    /// 获取当前焦点
    fn current_focus(&self) -> Option<ComponentId>;
}

/// 焦点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusState {
    /// 无焦点
    #[default]
    None,
    /// 有焦点
    Focused,
    /// 焦点被锁定
    Locked,
}

/// 焦点样式
#[derive(Debug, Clone, Default)]
pub struct FocusStyle {
    pub _border_color: Option<String>,
}
