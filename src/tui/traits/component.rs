//! 组件 trait 定义
//!
//! 占位符模块，完整实现将在 Task A.1 中完成。

use crate::tui::traits::ComponentId;

/// 基础组件 trait
pub trait Component: Send + Sync {
    /// 获取组件 ID
    fn id(&self) -> ComponentId;

    /// 渲染组件
    fn render(&self);
}

/// 容器组件 trait
pub trait Container: Component {
    /// 添加子组件
    fn add_child(&mut self, child: Box<dyn Component>);
}

/// 可渲染 trait
pub trait Render: Component {
    /// 渲染到终端
    fn render_to_terminal(&self);
}

/// 可交互 trait
pub trait Interactive: Component {
    /// 处理输入
    fn handle_input(&mut self, input: &[u8]) -> bool;
}

