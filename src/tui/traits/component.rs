//! 组件 trait 定义
//!
//! 定义 TUI 框架的核心组件接口，包括 Component、Container、Render、Interactive 等。

use crate::tui::error::TuiResult;
use crate::tui::traits::{AppEvent, ComponentId, HandleResult};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::io;

// ============================================================================
// Render Trait
// ============================================================================

/// 渲染 trait
///
/// 定义组件如何将自己渲染到终端缓冲区。
pub trait Render {
    /// 渲染组件到指定区域
    fn render(&self, area: Rect, buf: &mut Buffer);
}

// ============================================================================
// Interactive Trait
// ============================================================================

/// 交互 trait
///
/// 定义组件如何处理用户输入事件。
pub trait Interactive {
    /// 处理键盘事件
    fn handle_key(&mut self, _key: KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }

    /// 处理鼠标事件
    fn handle_mouse(&mut self, _event: MouseEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

// ============================================================================
// Component Trait
// ============================================================================

/// 组件基础 trait
///
/// 所有 UI 组件的公共接口，组合了 Render 和 Interactive 能力。
///
/// # 生命周期钩子
///
/// 组件提供多个生命周期钩子，默认实现为空，组件可以按需重写：
/// - `on_mount`: 组件添加到视图时调用
/// - `on_unmount`: 组件从视图移除时调用
/// - `on_focus_gain`: 组件获得焦点时调用
/// - `on_focus_loss`: 组件失去焦点时调用
/// - `before_render`: 渲染前调用
/// - `after_render`: 渲染后调用
pub trait Component: Render + Interactive {
    /// 获取组件 ID
    fn id(&self) -> ComponentId;

    /// 是否可以获取焦点
    fn can_focus(&self) -> bool {
        false
    }

    /// 处理应用事件
    fn on_event(&mut self, _event: &AppEvent) -> HandleResult {
        HandleResult::Ignored
    }

    // ========== 生命周期钩子（提供默认实现） ==========

    /// 组件挂载时调用
    fn on_mount(&mut self) -> TuiResult<()> {
        Ok(())
    }

    /// 组件卸载时调用
    fn on_unmount(&mut self) -> TuiResult<()> {
        Ok(())
    }

    /// 获得焦点时调用
    fn on_focus_gain(&mut self) -> TuiResult<()> {
        Ok(())
    }

    /// 失去焦点时调用
    fn on_focus_loss(&mut self) -> TuiResult<()> {
        Ok(())
    }

    /// 渲染前调用
    fn before_render(&mut self) -> TuiResult<()> {
        Ok(())
    }

    /// 渲染后调用
    fn after_render(&mut self) -> TuiResult<()> {
        Ok(())
    }
}

// ============================================================================
// Container Trait
// ============================================================================

/// 容器组件 trait
///
/// 管理子组件的组件，可以添加、移除子组件，并管理焦点。
pub trait Container: Component {
    /// 添加子组件
    fn add_child(&mut self, child: Box<dyn Component>) -> TuiResult<()>;

    /// 移除子组件
    fn remove_child(&mut self, id: &ComponentId) -> TuiResult<()>;

    /// 获取当前焦点的子组件
    fn focused_child(&self) -> Option<&ComponentId>;

    /// 设置焦点子组件
    fn set_focus(&mut self, id: ComponentId) -> TuiResult<()>;

    /// 获取子组件数量
    fn child_count(&self) -> usize;

    /// 传播事件到子组件
    fn propagate_event(&mut self, event: &AppEvent) -> HandleResult;

    /// 获取所有子组件 ID
    fn children(&self) -> Vec<ComponentId> {
        Vec::new()
    }

    /// 获取子组件（通过 ID）
    fn get_child(&self, id: &ComponentId) -> Option<&dyn Component>;

    /// 获取可变子组件（通过 ID）
    fn get_child_mut(&mut self, id: &ComponentId) -> Option<&mut dyn Component>;
}

// ============================================================================
// Application Trait
// ============================================================================

/// TUI 应用程序 trait
///
/// 定义应用程序的运行接口，负责启动和运行主事件循环。
pub trait Application {
    /// 运行应用程序
    ///
    /// 此方法应该：
    /// 1. 设置终端（raw mode、alternate screen）
    /// 2. 启动主事件循环
    /// 3. 处理用户输入
    /// 4. 渲染 UI
    /// 5. 在退出时清理终端状态
    fn run(&mut self) -> io::Result<()>;
}

// ============================================================================
// 测试辅助函数
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 简单的测试组件实现
    struct TestComponent {
        id: ComponentId,
        can_focus: bool,
    }

    impl Render for TestComponent {
        fn render(&self, _area: Rect, _buf: &mut Buffer) {
            // 测试实现
        }
    }

    impl Interactive for TestComponent {
        fn handle_key(&mut self, _key: KeyEvent) -> HandleResult {
            HandleResult::Ignored
        }
    }

    impl Component for TestComponent {
        fn id(&self) -> ComponentId {
            self.id
        }

        fn can_focus(&self) -> bool {
            self.can_focus
        }
    }

    #[test]
    fn test_component_id() {
        let comp = TestComponent {
            id: ComponentId::new(42),
            can_focus: true,
        };
        assert_eq!(comp.id().value(), 42);
    }

    #[test]
    fn test_component_can_focus() {
        let comp1 = TestComponent {
            id: ComponentId::new(1),
            can_focus: true,
        };
        assert!(comp1.can_focus());

        let comp2 = TestComponent {
            id: ComponentId::new(2),
            can_focus: false,
        };
        assert!(!comp2.can_focus());
    }

    #[test]
    fn test_component_default_hooks() {
        let mut comp = TestComponent {
            id: ComponentId::new(1),
            can_focus: false,
        };

        // 所有的生命周期钩子都应该成功（默认实现返回 Ok）
        assert!(comp.on_mount().is_ok());
        assert!(comp.on_unmount().is_ok());
        assert!(comp.on_focus_gain().is_ok());
        assert!(comp.on_focus_loss().is_ok());
        assert!(comp.before_render().is_ok());
        assert!(comp.after_render().is_ok());
    }
}
