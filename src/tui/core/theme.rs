//! 主题实现
//!
//! 占位符模块，完整实现将在 Task C.5 中完成。

use crate::tui::traits::{Theme, ColorPalette, ThemeVariant};
use std::sync::LazyLock;

/// 深色主题
#[derive(Debug, Clone, Copy, Default)]
pub struct DarkTheme;

impl Theme for DarkTheme {
    fn name(&self) -> &str {
        "dark"
    }

    fn palette(&self) -> &ColorPalette {
        static PALETTE: LazyLock<ColorPalette> = LazyLock::new(|| ColorPalette {
            _foreground: Some("#c0caf5".to_string()),
            _background: Some("#1a1b26".to_string()),
            _primary: Some("#7aa2f7".to_string()),
            _secondary: Some("#bb9af7".to_string()),
        });
        &PALETTE
    }
}

/// 浅色主题
#[derive(Debug, Clone, Copy, Default)]
pub struct LightTheme;

impl Theme for LightTheme {
    fn name(&self) -> &str {
        "light"
    }

    fn palette(&self) -> &ColorPalette {
        static PALETTE: LazyLock<ColorPalette> = LazyLock::new(|| ColorPalette {
            _foreground: Some("#3760bf".to_string()),
            _background: Some("#e1e2e7".to_string()),
            _primary: Some("#2e7de9".to_string()),
            _secondary: Some("#8c6c3e".to_string()),
        });
        &PALETTE
    }
}
