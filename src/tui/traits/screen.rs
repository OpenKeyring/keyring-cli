//! 屏幕管理 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.7 中完成。

use crate::tui::traits::ComponentId;

/// 屏幕管理器 trait
pub trait ScreenManager: Send + Sync {
    /// 推入屏幕
    fn push(&mut self, screen: Box<dyn Screen>) -> ComponentId;

    /// 弹出屏幕
    fn pop(&mut self) -> Option<Box<dyn Screen>>;

    /// 获取当前屏幕
    fn current(&self) -> Option<&dyn Screen>;
}

/// 屏幕 trait
pub trait Screen: Send + Sync {
    /// 渲染屏幕
    fn render(&self);

    /// 获取屏幕 ID
    fn id(&self) -> ComponentId;
}

/// 屏幕栈
#[derive(Debug, Default)]
pub struct ScreenStack {
    _private: (),
}

/// 屏幕转换
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenTransition {
    /// 推入新屏幕
    #[default]
    Push,
    /// 替换当前屏幕
    Replace,
    /// 弹出当前屏幕
    Pop,
}
