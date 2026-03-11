//! 事件类型定义
//!
//! 定义 TUI 框架中使用的所有事件类型，包括应用事件、处理结果、操作等。

use crate::tui::traits::ComponentId;
use crate::tui::components::ConfirmAction;
use crossterm::event::{KeyEvent, MouseEvent};

// ============================================================================
// 应用事件
// ============================================================================

/// 应用事件
///
/// TUI 框架中的所有事件类型，包括输入事件、组件间通信事件、屏幕事件等。
#[derive(Debug, Clone)]
pub enum AppEvent {
    // ========== 输入事件 ==========
    /// 键盘事件
    Key(KeyEvent),
    /// 鼠标事件
    Mouse(MouseEvent),

    // ========== 组件间通信事件 ==========
    /// 密码被选择
    PasswordSelected(ComponentId, Option<String>),
    /// 焦点变化
    FocusChanged(ComponentId),
    /// 过滤器变化
    FilterChanged(FilterType),

    // ========== 屏幕事件 ==========
    /// 屏幕已打开
    ScreenOpened(ScreenType),
    /// 屏幕已关闭
    ScreenClosed,
    /// 屏幕被驳回
    ScreenDismissed,

    // ========== 系统事件 ==========
    /// 退出应用
    Quit,
    /// 刷新显示
    Refresh,
    /// 定时器事件（周期性触发）
    Tick,
}

// ============================================================================
// 过滤器类型
// ============================================================================

/// 过滤器类型
///
/// 用于密码列表的过滤。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterType {
    /// 无过滤
    None,
    /// 全部
    All,
    /// 收藏
    Favorites,
    /// 最近使用
    Recent,
    /// 按组过滤
    Group(String),
    /// 按标签过滤
    Tag(String),
    /// 搜索查询
    Search(String),
}

// ============================================================================
// 屏幕类型
// ============================================================================

/// 屏幕类型
///
/// 标识 TUI 应用中的不同屏幕。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenType {
    /// 向导（初次设置）
    Wizard,
    /// 新建密码
    NewPassword,
    /// 编辑密码
    EditPassword(String),
    /// 确认对话框
    ConfirmDialog(ConfirmAction),
    /// 回收站
    TrashBin,
    /// 设置
    Settings,
    /// 帮助
    Help,
    /// 主屏幕
    Main,
}

// ============================================================================
// 处理结果
// ============================================================================

/// 事件处理结果
///
/// 组件处理事件后返回的结果，指示事件是否被处理以及下一步操作。
#[derive(Debug, Clone, PartialEq)]
pub enum HandleResult {
    /// 事件已处理，停止传播
    Consumed,
    /// 事件未处理，继续传播
    Ignored,
    /// 需要重新渲染
    NeedsRender,
    /// 需要执行特定动作
    Action(Action),
}

impl Default for HandleResult {
    fn default() -> Self {
        Self::Ignored
    }
}

// ============================================================================
// 操作类型
// ============================================================================

/// 操作类型
///
/// 事件处理后可能需要执行的操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// 退出应用
    Quit,
    /// 打开指定屏幕
    OpenScreen(ScreenType),
    /// 关闭当前屏幕
    CloseScreen,
    /// 显示提示消息
    ShowToast(String),
    /// 复制到剪贴板
    CopyToClipboard(String),
    /// 刷新显示
    Refresh,
    /// 确认对话框结果 (true = confirmed, false = cancelled)
    ConfirmDialog(ConfirmAction),
    /// 无操作
    None,
}

impl Default for Action {
    fn default() -> Self {
        Self::None
    }
}

// ============================================================================
// 事件分发器 Trait
// ============================================================================


/// 事件分发器 trait
///
/// 负责将应用事件分发到正确的处理器，包括全局快捷键处理。
pub trait EventDispatcher: Send + Sync {
    /// 分发事件
    fn dispatch(&mut self, event: AppEvent) -> HandleResult;

    /// 注册全局快捷键
    fn register_global_keybinding(&mut self, key: KeyEvent, action: Action);

    /// 处理全局快捷键
    fn handle_global_keybinding(&self, key: KeyEvent) -> Option<Action>;

    /// 添加事件过滤器
    fn add_filter(&mut self, filter: Box<dyn EventFilter>);

    /// 移除事件过滤器
    fn remove_filter(&mut self, id: &str);
}

/// 事件过滤器 trait
///
/// 允许在事件分发前对事件进行过滤或转换。
pub trait EventFilter: Send + Sync {
    /// 检查事件是否应该被处理
    fn should_process(&self, event: &AppEvent) -> bool;

    /// 获取过滤器 ID
    fn id(&self) -> &str;
}

// ============================================================================
// 测试辅助函数
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_type_equality() {
        assert_eq!(FilterType::None, FilterType::None);
        assert_eq!(FilterType::All, FilterType::All);
        assert_ne!(FilterType::All, FilterType::None);
        assert_eq!(
            FilterType::Group("work".to_string()),
            FilterType::Group("work".to_string())
        );
    }

    #[test]
    fn test_screen_type_equality() {
        assert_eq!(ScreenType::Wizard, ScreenType::Wizard);
        assert_ne!(ScreenType::Wizard, ScreenType::Main);
        assert_eq!(
            ScreenType::EditPassword("test".to_string()),
            ScreenType::EditPassword("test".to_string())
        );
    }

    #[test]
    fn test_handle_result_default() {
        assert_eq!(HandleResult::default(), HandleResult::Ignored);
    }

    #[test]
    fn test_action_default() {
        assert_eq!(Action::default(), Action::None);
    }
}
