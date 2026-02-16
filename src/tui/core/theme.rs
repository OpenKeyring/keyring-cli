//! 主题实现
//!
//! 实现主题管理器和重新导出主题结构体。

use crate::tui::traits::{Theme, ThemeName, ThemeManager};

// 重新导出 traits 模块中的主题实现
pub use crate::tui::traits::{DarkTheme, LightTheme};

/// 默认主题管理器
///
/// 管理主题切换和提供当前主题访问。
pub struct DefaultThemeManager {
    /// 当前主题
    current: Box<dyn Theme>,
}

impl std::fmt::Debug for DefaultThemeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultThemeManager")
            .field("current", &self.current.name())
            .finish()
    }
}

impl Default for DefaultThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultThemeManager {
    /// 创建新的主题管理器（默认深色主题）
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: Box::new(DarkTheme),
        }
    }

    /// 创建指定主题的管理器
    #[must_use]
    pub fn with_theme(name: ThemeName) -> Self {
        let theme: Box<dyn Theme> = match name {
            ThemeName::Dark => Box::new(DarkTheme),
            ThemeName::Light => Box::new(LightTheme),
        };
        Self { current: theme }
    }
}

impl ThemeManager for DefaultThemeManager {
    fn current(&self) -> &dyn Theme {
        self.current.as_ref()
    }

    fn set_theme(&mut self, name: ThemeName) {
        self.current = match name {
            ThemeName::Dark => Box::new(DarkTheme),
            ThemeName::Light => Box::new(LightTheme),
        };
    }
}

/// 主题切换器
///
/// 提供主题切换的辅助方法。
pub struct ThemeSwitcher {
    manager: DefaultThemeManager,
}

impl Default for ThemeSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeSwitcher {
    /// 创建新的主题切换器
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: DefaultThemeManager::new(),
        }
    }

    /// 获取当前主题
    #[must_use]
    pub fn current(&self) -> &dyn Theme {
        self.manager.current()
    }

    /// 切换到下一个主题
    pub fn toggle(&mut self) {
        self.manager.toggle_theme();
    }

    /// 设置主题
    pub fn set(&mut self, name: ThemeName) {
        self.manager.set_theme(name);
    }

    /// 获取管理器
    #[must_use]
    pub fn manager(&self) -> &DefaultThemeManager {
        &self.manager
    }

    /// 获取可变管理器
    #[must_use]
    pub fn manager_mut(&mut self) -> &mut DefaultThemeManager {
        &mut self.manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_manager() {
        let mut manager = DefaultThemeManager::new();
        assert_eq!(manager.current_name(), ThemeName::Dark);

        manager.set_theme(ThemeName::Light);
        assert_eq!(manager.current_name(), ThemeName::Light);

        manager.toggle_theme();
        assert_eq!(manager.current_name(), ThemeName::Dark);
    }

    #[test]
    fn test_theme_switcher() {
        let mut switcher = ThemeSwitcher::new();
        assert_eq!(switcher.current().name(), ThemeName::Dark);

        switcher.toggle();
        assert_eq!(switcher.current().name(), ThemeName::Light);

        switcher.set(ThemeName::Dark);
        assert_eq!(switcher.current().name(), ThemeName::Dark);
    }

    #[test]
    fn test_theme_consistency() {
        let manager = DefaultThemeManager::new();

        // 验证深色主题的颜色
        let bg = manager.current().background();
        let text = manager.current().text_primary();

        // 深色主题应该使用终端默认背景
        assert_eq!(format!("{:?}", bg), "Reset");

        // 切换到浅色主题
        let mut manager = DefaultThemeManager::new();
        manager.set_theme(ThemeName::Light);

        // 浅色主题应该有明亮的背景
        let bg = manager.current().background();
        assert!(format!("{:?}", bg).contains("Rgb"));
    }
}
