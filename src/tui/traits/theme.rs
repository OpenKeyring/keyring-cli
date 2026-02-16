//! 主题 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.1 中完成。

/// 主题 trait
pub trait Theme: Send + Sync {
    /// 获取主题名称
    fn name(&self) -> &str;

    /// 获取配色方案
    fn palette(&self) -> &ColorPalette;
}

/// 颜色调色板
#[derive(Debug, Clone, Default)]
pub struct ColorPalette {
    pub _foreground: Option<String>,
    pub _background: Option<String>,
    pub _primary: Option<String>,
    pub _secondary: Option<String>,
}

/// 主题变体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    /// 浅色主题
    #[default]
    Light,
    /// 深色主题
    Dark,
}
