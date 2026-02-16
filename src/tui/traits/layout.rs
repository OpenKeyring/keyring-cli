//! 布局 trait 定义
//!
//! 占位符模块，完整实现将在 Task A.2 中完成。

/// 布局 trait
pub trait Layout: Send + Sync {
    /// 计算布局
    fn calculate(&self) -> LayoutResult;
}

/// 布局约束
#[derive(Debug, Clone, Default)]
pub struct LayoutConstraints {
    pub _width: Option<u16>,
    pub _height: Option<u16>,
}

/// 布局结果
#[derive(Debug, Clone, Default)]
pub struct LayoutResult {
    pub _x: u16,
    pub _y: u16,
    pub _width: u16,
    pub _height: u16,
}
