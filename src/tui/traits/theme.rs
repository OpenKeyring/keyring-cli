//! 主题系统 Trait 定义
//!
//! 定义 TUI 主题和配色方案的接口。

use crate::tui::traits::password_strength::StrengthLevel;
use ratatui::style::{Color, Modifier, Style};

/// 主题名称
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    /// 深色主题
    Dark,
    /// 浅色主题
    Light,
}

impl Default for ThemeName {
    fn default() -> Self {
        Self::Dark // TUI 默认使用深色主题
    }
}

impl std::fmt::Display for ThemeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dark => write!(f, "Dark"),
            Self::Light => write!(f, "Light"),
        }
    }
}

/// 主题 Trait
///
/// 定义所有颜色和样式的接口。
pub trait Theme: Send + Sync {
    /// 主题名称
    fn name(&self) -> ThemeName;

    // === 背景色 ===

    /// 主背景色
    fn background(&self) -> Color;

    /// 面板背景色
    fn panel_background(&self) -> Color;

    /// 输入框背景色
    fn input_background(&self) -> Color;

    /// 模态背景色（用于遮罩层）
    fn modal_background(&self) -> Color {
        Color::Reset
    }

    // === 文字颜色 ===

    /// 主要文字颜色
    fn text_primary(&self) -> Color;

    /// 次要文字颜色
    fn text_secondary(&self) -> Color;

    /// 禁用文字颜色
    fn text_disabled(&self) -> Color;

    /// 占位符文字颜色
    fn text_placeholder(&self) -> Color {
        self.text_secondary()
    }

    // === 边框颜色 ===

    /// 默认边框颜色
    fn border(&self) -> Color;

    /// 焦点边框颜色
    fn border_focused(&self) -> Color;

    /// 面板边框颜色
    fn border_panel(&self) -> Color {
        self.border()
    }

    // === 状态颜色 ===

    /// 成功色
    fn success(&self) -> Color;

    /// 警告色
    fn warning(&self) -> Color;

    /// 错误色
    fn error(&self) -> Color;

    /// 信息色
    fn info(&self) -> Color;

    // === 强调色 ===

    /// 主强调色
    fn accent(&self) -> Color;

    /// 次要强调色
    fn accent_secondary(&self) -> Color;

    // === 特殊颜色 ===

    /// 选中行背景色
    fn selection_background(&self) -> Color;

    /// 选中行文字色
    fn selection_foreground(&self) -> Color {
        self.text_primary()
    }

    /// 高亮文字颜色
    fn highlight(&self) -> Color;

    /// 链接颜色
    fn link(&self) -> Color;

    /// 光标颜色
    fn cursor(&self) -> Color {
        self.text_primary()
    }

    // === 密码强度颜色 ===

    /// 密码强度 - 非常弱
    fn strength_very_weak(&self) -> Color;

    /// 密码强度 - 弱
    fn strength_weak(&self) -> Color;

    /// 密码强度 - 一般
    fn strength_fair(&self) -> Color;

    /// 密码强度 - 强
    fn strength_strong(&self) -> Color;

    /// 密码强度 - 非常强
    fn strength_very_strong(&self) -> Color;

    /// 获取密码强度对应颜色
    fn strength_color(&self, level: StrengthLevel) -> Color {
        match level {
            StrengthLevel::VeryWeak => self.strength_very_weak(),
            StrengthLevel::Weak => self.strength_weak(),
            StrengthLevel::Fair => self.strength_fair(),
            StrengthLevel::Strong => self.strength_strong(),
            StrengthLevel::VeryStrong => self.strength_very_strong(),
        }
    }

    // === 样式辅助方法 ===

    /// 获取默认样式
    fn default_style(&self) -> Style {
        Style::default()
            .fg(self.text_primary())
            .bg(self.background())
    }

    /// 获取面板样式
    fn panel_style(&self) -> Style {
        Style::default()
            .fg(self.text_primary())
            .bg(self.panel_background())
    }

    /// 获取输入框样式
    fn input_style(&self) -> Style {
        Style::default()
            .fg(self.text_primary())
            .bg(self.input_background())
    }

    /// 获取焦点样式
    fn focus_style(&self) -> Style {
        Style::default()
            .fg(self.text_primary())
            .bg(self.selection_background())
    }

    /// 获取禁用样式
    fn disabled_style(&self) -> Style {
        Style::default()
            .fg(self.text_disabled())
            .add_modifier(Modifier::DIM)
    }

    /// 获取成功样式
    fn success_style(&self) -> Style {
        Style::default().fg(self.success())
    }

    /// 获取警告样式
    fn warning_style(&self) -> Style {
        Style::default().fg(self.warning())
    }

    /// 获取错误样式
    fn error_style(&self) -> Style {
        Style::default().fg(self.error())
    }

    /// 获取信息样式
    fn info_style(&self) -> Style {
        Style::default().fg(self.info())
    }

    /// 获取强调样式
    fn accent_style(&self) -> Style {
        Style::default().fg(self.accent())
    }

    /// 获取边框样式
    fn border_style(&self) -> Style {
        Style::default().fg(self.border())
    }

    /// 获取焦点边框样式
    fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused())
    }
}

/// 深色主题
pub struct DarkTheme;

impl Theme for DarkTheme {
    fn name(&self) -> ThemeName {
        ThemeName::Dark
    }

    // 背景色
    fn background(&self) -> Color {
        Color::Reset
    }
    fn panel_background(&self) -> Color {
        Color::Rgb(30, 30, 40)
    }
    fn input_background(&self) -> Color {
        Color::Rgb(40, 40, 50)
    }
    fn modal_background(&self) -> Color {
        Color::Rgb(0, 0, 0)
    }

    // 文字颜色
    fn text_primary(&self) -> Color {
        Color::Rgb(230, 230, 230)
    }
    fn text_secondary(&self) -> Color {
        Color::Rgb(150, 150, 150)
    }
    fn text_disabled(&self) -> Color {
        Color::Rgb(100, 100, 100)
    }

    // 边框颜色
    fn border(&self) -> Color {
        Color::Rgb(60, 60, 70)
    }
    fn border_focused(&self) -> Color {
        Color::Cyan
    }

    // 状态颜色
    fn success(&self) -> Color {
        Color::Green
    }
    fn warning(&self) -> Color {
        Color::Yellow
    }
    fn error(&self) -> Color {
        Color::Red
    }
    fn info(&self) -> Color {
        Color::Blue
    }

    // 强调色
    fn accent(&self) -> Color {
        Color::Cyan
    }
    fn accent_secondary(&self) -> Color {
        Color::Magenta
    }

    // 特殊颜色
    fn selection_background(&self) -> Color {
        Color::Rgb(50, 50, 70)
    }
    fn highlight(&self) -> Color {
        Color::Yellow
    }
    fn link(&self) -> Color {
        Color::Cyan
    }

    // 密码强度颜色
    fn strength_very_weak(&self) -> Color {
        Color::Red
    }
    fn strength_weak(&self) -> Color {
        Color::Rgb(255, 128, 0)
    }
    fn strength_fair(&self) -> Color {
        Color::Yellow
    }
    fn strength_strong(&self) -> Color {
        Color::Green
    }
    fn strength_very_strong(&self) -> Color {
        Color::Rgb(0, 200, 100)
    }
}

/// 浅色主题
pub struct LightTheme;

impl Theme for LightTheme {
    fn name(&self) -> ThemeName {
        ThemeName::Light
    }

    // 背景色
    fn background(&self) -> Color {
        Color::Rgb(250, 250, 250)
    }
    fn panel_background(&self) -> Color {
        Color::Rgb(240, 240, 245)
    }
    fn input_background(&self) -> Color {
        Color::Rgb(255, 255, 255)
    }
    fn modal_background(&self) -> Color {
        Color::Rgb(200, 200, 200)
    }

    // 文字颜色
    fn text_primary(&self) -> Color {
        Color::Rgb(30, 30, 30)
    }
    fn text_secondary(&self) -> Color {
        Color::Rgb(100, 100, 100)
    }
    fn text_disabled(&self) -> Color {
        Color::Rgb(180, 180, 180)
    }

    // 边框颜色
    fn border(&self) -> Color {
        Color::Rgb(200, 200, 200)
    }
    fn border_focused(&self) -> Color {
        Color::Rgb(0, 150, 200)
    }

    // 状态颜色
    fn success(&self) -> Color {
        Color::Rgb(0, 150, 0)
    }
    fn warning(&self) -> Color {
        Color::Rgb(200, 150, 0)
    }
    fn error(&self) -> Color {
        Color::Rgb(200, 0, 0)
    }
    fn info(&self) -> Color {
        Color::Rgb(0, 100, 200)
    }

    // 强调色
    fn accent(&self) -> Color {
        Color::Rgb(0, 150, 200)
    }
    fn accent_secondary(&self) -> Color {
        Color::Rgb(150, 0, 150)
    }

    // 特殊颜色
    fn selection_background(&self) -> Color {
        Color::Rgb(220, 230, 250)
    }
    fn highlight(&self) -> Color {
        Color::Rgb(200, 150, 0)
    }
    fn link(&self) -> Color {
        Color::Rgb(0, 100, 200)
    }

    // 密码强度颜色
    fn strength_very_weak(&self) -> Color {
        Color::Rgb(200, 0, 0)
    }
    fn strength_weak(&self) -> Color {
        Color::Rgb(200, 100, 0)
    }
    fn strength_fair(&self) -> Color {
        Color::Rgb(180, 150, 0)
    }
    fn strength_strong(&self) -> Color {
        Color::Rgb(0, 150, 0)
    }
    fn strength_very_strong(&self) -> Color {
        Color::Rgb(0, 120, 60)
    }
}

/// 颜色调色板
///
/// 主题颜色的结构化表示。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPalette {
    // 背景
    pub background: Color,
    pub panel_background: Color,
    pub input_background: Color,

    // 文字
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    // 边框
    pub border: Color,
    pub border_focused: Color,

    // 状态
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // 强调
    pub accent: Color,
    pub accent_secondary: Color,

    // 特殊
    pub selection_background: Color,
    pub highlight: Color,
    pub link: Color,

    // 密码强度
    pub strength_very_weak: Color,
    pub strength_weak: Color,
    pub strength_fair: Color,
    pub strength_strong: Color,
    pub strength_very_strong: Color,
}

impl ColorPalette {
    /// 从主题创建调色板
    pub fn from_theme(theme: &dyn Theme) -> Self {
        Self {
            background: theme.background(),
            panel_background: theme.panel_background(),
            input_background: theme.input_background(),
            text_primary: theme.text_primary(),
            text_secondary: theme.text_secondary(),
            text_disabled: theme.text_disabled(),
            border: theme.border(),
            border_focused: theme.border_focused(),
            success: theme.success(),
            warning: theme.warning(),
            error: theme.error(),
            info: theme.info(),
            accent: theme.accent(),
            accent_secondary: theme.accent_secondary(),
            selection_background: theme.selection_background(),
            highlight: theme.highlight(),
            link: theme.link(),
            strength_very_weak: theme.strength_very_weak(),
            strength_weak: theme.strength_weak(),
            strength_fair: theme.strength_fair(),
            strength_strong: theme.strength_strong(),
            strength_very_strong: theme.strength_very_strong(),
        }
    }

    /// 创建深色调色板
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: Color::Reset,
            panel_background: Color::Rgb(30, 30, 40),
            input_background: Color::Rgb(40, 40, 50),
            text_primary: Color::Rgb(230, 230, 230),
            text_secondary: Color::Rgb(150, 150, 150),
            text_disabled: Color::Rgb(100, 100, 100),
            border: Color::Rgb(60, 60, 70),
            border_focused: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            accent: Color::Cyan,
            accent_secondary: Color::Magenta,
            selection_background: Color::Rgb(50, 50, 70),
            highlight: Color::Yellow,
            link: Color::Cyan,
            strength_very_weak: Color::Red,
            strength_weak: Color::Rgb(255, 128, 0),
            strength_fair: Color::Yellow,
            strength_strong: Color::Green,
            strength_very_strong: Color::Rgb(0, 200, 100),
        }
    }

    /// 创建浅色调色板
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: Color::Rgb(250, 250, 250),
            panel_background: Color::Rgb(240, 240, 245),
            input_background: Color::Rgb(255, 255, 255),
            text_primary: Color::Rgb(30, 30, 30),
            text_secondary: Color::Rgb(100, 100, 100),
            text_disabled: Color::Rgb(180, 180, 180),
            border: Color::Rgb(200, 200, 200),
            border_focused: Color::Rgb(0, 150, 200),
            success: Color::Rgb(0, 150, 0),
            warning: Color::Rgb(200, 150, 0),
            error: Color::Rgb(200, 0, 0),
            info: Color::Rgb(0, 100, 200),
            accent: Color::Rgb(0, 150, 200),
            accent_secondary: Color::Rgb(150, 0, 150),
            selection_background: Color::Rgb(220, 230, 250),
            highlight: Color::Rgb(200, 150, 0),
            link: Color::Rgb(0, 100, 200),
            strength_very_weak: Color::Rgb(200, 0, 0),
            strength_weak: Color::Rgb(200, 100, 0),
            strength_fair: Color::Rgb(180, 150, 0),
            strength_strong: Color::Rgb(0, 150, 0),
            strength_very_strong: Color::Rgb(0, 120, 60),
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

/// 主题变体（兼容旧代码）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    /// 浅色主题
    Light,
    /// 深色主题
    Dark,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Dark
    }
}

impl From<ThemeName> for ThemeVariant {
    fn from(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self::Dark,
            ThemeName::Light => Self::Light,
        }
    }
}

impl From<ThemeVariant> for ThemeName {
    fn from(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Dark => Self::Dark,
            ThemeVariant::Light => Self::Light,
        }
    }
}

/// 主题管理器 Trait
pub trait ThemeManager: Send + Sync {
    /// 获取当前主题
    fn current(&self) -> &dyn Theme;

    /// 获取当前主题名称
    fn current_name(&self) -> ThemeName {
        self.current().name()
    }

    /// 切换主题
    fn set_theme(&mut self, name: ThemeName);

    /// 可用的主题列表
    fn available_themes(&self) -> Vec<ThemeName> {
        vec![ThemeName::Dark, ThemeName::Light]
    }

    /// 切换到下一个主题
    fn toggle_theme(&mut self) {
        let next = match self.current_name() {
            ThemeName::Dark => ThemeName::Light,
            ThemeName::Light => ThemeName::Dark,
        };
        self.set_theme(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme() {
        let theme = DarkTheme;
        assert_eq!(theme.name(), ThemeName::Dark);
        assert_eq!(theme.background(), Color::Reset);
        assert_eq!(theme.text_primary(), Color::Rgb(230, 230, 230));
        assert_eq!(theme.success(), Color::Green);
        assert_eq!(theme.error(), Color::Red);
    }

    #[test]
    fn test_light_theme() {
        let theme = LightTheme;
        assert_eq!(theme.name(), ThemeName::Light);
        assert_eq!(theme.background(), Color::Rgb(250, 250, 250));
        assert_eq!(theme.text_primary(), Color::Rgb(30, 30, 30));
    }

    #[test]
    fn test_theme_styles() {
        let theme = DarkTheme;

        let style = theme.default_style();
        assert_eq!(style.fg, Some(Color::Rgb(230, 230, 230)));

        let error_style = theme.error_style();
        assert_eq!(error_style.fg, Some(Color::Red));

        let focus_style = theme.focus_style();
        assert_eq!(focus_style.bg, Some(Color::Rgb(50, 50, 70)));
    }

    #[test]
    fn test_strength_colors() {
        let theme = DarkTheme;

        use crate::tui::traits::StrengthLevel;
        assert_eq!(theme.strength_color(StrengthLevel::VeryWeak), Color::Red);
        assert_eq!(theme.strength_color(StrengthLevel::Strong), Color::Green);
    }

    #[test]
    fn test_color_palette() {
        let palette = ColorPalette::dark();
        assert_eq!(palette.background, Color::Reset);
        assert_eq!(palette.text_primary, Color::Rgb(230, 230, 230));

        let light = ColorPalette::light();
        assert_eq!(light.background, Color::Rgb(250, 250, 250));
    }
}
