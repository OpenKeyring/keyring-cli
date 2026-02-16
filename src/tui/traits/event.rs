//! 事件类型定义
//!
//! 占位符模块，完整实现将在 Task 0.3 中完成。

use crossterm::event::KeyEvent;

/// 应用事件
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// 键盘事件
    Key(KeyEvent),
    /// 定时器事件
    Tick,
    /// 退出事件
    Quit,
}

/// 处理结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HandleResult {
    /// 继续处理
    #[default]
    Continue,
    /// 停止处理
    Stop,
    /// 需要重新渲染
    NeedsRedraw,
}

/// 操作
#[derive(Debug, Clone, Default)]
pub enum Action {
    /// 无操作
    #[default]
    None,
    /// 退出
    Quit,
    /// 切换屏幕
    SwitchScreen(String),
}

/// 屏幕类型
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScreenType {
    /// 主屏幕
    #[default]
    Main,
    /// 设置屏幕
    Settings,
    /// 帮助屏幕
    Help,
}
