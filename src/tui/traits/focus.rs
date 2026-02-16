//! 焦点管理 trait 定义
//!
//! 定义焦点管理系统，包括焦点管理器、焦点状态和焦点样式。

use crate::tui::error::TuiResult;
use crate::tui::traits::ComponentId;

// ============================================================================
// 焦点管理器 Trait
// ============================================================================

/// 焦点管理器 trait
///
/// 管理组件之间的焦点导航和状态。
pub trait FocusManager: Send + Sync {
    /// 获取当前焦点组件
    fn current_focus(&self) -> Option<ComponentId>;

    /// 设置焦点到指定组件
    fn set_focus(&mut self, id: ComponentId) -> TuiResult<()>;

    /// 焦点移到下一个
    fn focus_next(&mut self) -> TuiResult<()>;

    /// 焦点移到上一个
    fn focus_prev(&mut self) -> TuiResult<()>;

    /// 清除焦点
    fn clear_focus(&mut self);
}

// ============================================================================
// 焦点状态
// ============================================================================

/// 焦点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusState {
    /// 不可聚焦
    NotFocusable,
    /// 可聚焦但未聚焦
    #[default]
    Focusable,
    /// 已聚焦
    Focused,
}

impl FocusState {
    /// 是否可以聚焦
    #[must_use]
    pub const fn is_focusable(&self) -> bool {
        !matches!(self, Self::NotFocusable)
    }

    /// 是否已聚焦
    #[must_use]
    pub const fn is_focused(&self) -> bool {
        matches!(self, Self::Focused)
    }
}

// ============================================================================
// 焦点样式
// ============================================================================

/// 焦点样式 - 用于渲染焦点指示
#[derive(Debug, Clone, Copy)]
pub struct FocusStyle {
    /// 焦点边框颜色
    pub border_color: ratatui::style::Color,
    /// 焦点背景色
    pub background: Option<ratatui::style::Color>,
    /// 是否显示光标
    pub show_cursor: bool,
}

impl Default for FocusStyle {
    fn default() -> Self {
        Self {
            border_color: ratatui::style::Color::Cyan,
            background: None,
            show_cursor: true,
        }
    }
}

impl FocusStyle {
    /// 创建新的焦点样式
    #[must_use]
    pub const fn new() -> Self {
        Self {
            border_color: ratatui::style::Color::Cyan,
            background: None,
            show_cursor: true,
        }
    }

    /// 设置边框颜色
    #[must_use]
    pub const fn with_border_color(mut self, color: ratatui::style::Color) -> Self {
        self.border_color = color;
        self
    }

    /// 设置背景色
    #[must_use]
    pub const fn with_background(mut self, color: ratatui::style::Color) -> Self {
        self.background = Some(color);
        self
    }

    /// 设置是否显示光标
    #[must_use]
    pub const fn with_cursor(mut self, show: bool) -> Self {
        self.show_cursor = show;
        self
    }
}

// ============================================================================
// 焦点导航策略
// ============================================================================

/// 焦点导航策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusNavigation {
    /// 线性导航（Tab/Shift+Tab）
    Linear,
    /// 空间导航（方向键）
    Spatial,
}

impl Default for FocusNavigation {
    fn default() -> Self {
        Self::Linear
    }
}

/// 方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// ============================================================================
// 焦点管理器扩展 Trait
// ============================================================================

/// 焦点管理器扩展 trait
///
/// 提供额外的焦点管理功能，如空间导航和动态组件注册。
pub trait FocusManagerExt: FocusManager {
    /// 设置导航策略
    fn set_navigation(&mut self, strategy: FocusNavigation);

    /// 获取导航策略
    fn navigation(&self) -> FocusNavigation;

    /// 处理方向键导航（仅 Spatial 策略有效）
    fn handle_direction(&mut self, direction: Direction) -> TuiResult<()>;

    /// 注册组件到焦点链
    fn register_focusable(&mut self, id: ComponentId);

    /// 注销组件
    fn unregister_focusable(&mut self, id: &ComponentId);

    /// 设置焦点顺序
    fn set_focus_order(&mut self, order: Vec<ComponentId>);

    /// 获取焦点顺序
    fn focus_order(&self) -> Vec<ComponentId>;
}
