//! 输入法 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.6 中完成。

/// 输入法服务 trait
pub trait ImeService: Send + Sync {
    /// 获取当前输入模式
    fn mode(&self) -> ImeMode;

    /// 获取组合状态
    fn composition(&self) -> &CompositionState;
}

/// 输入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImeMode {
    /// 直接输入
    #[default]
    Direct,
    /// 组合输入
    Composing,
}

/// 组合状态
#[derive(Debug, Clone, Default)]
pub struct CompositionState {
    pub _text: String,
    pub _cursor: usize,
}
