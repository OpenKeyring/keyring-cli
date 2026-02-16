//! TUI 组件 trait 测试
//!
//! 测试 Component、Container、Render、Interactive 等组件相关 trait。

use crate::tui::traits::{Component, Container, Render, Interactive, ComponentId};
use crate::tui::error::{TuiError, TuiResult};
use crate::tui::traits::{AppEvent, HandleResult, Action, ScreenType};
use crossterm::event::{KeyEvent, MouseEvent, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;

// ============================================================================
// 测试组件实现
// ============================================================================

/// 简单的测试组件
struct TestComponent {
    id: ComponentId,
    can_focus: bool,
    mounted: bool,
    focus_gained_count: usize,
    focus_lost_count: usize,
}

impl TestComponent {
    fn new(id: ComponentId, can_focus: bool) -> Self {
        Self {
            id,
            can_focus,
            mounted: false,
            focus_gained_count: 0,
            focus_lost_count: 0,
        }
    }
}

impl Render for TestComponent {
    fn render(&self, _area: Rect, _buf: &mut Buffer) {
        // 测试实现
    }
}

impl Interactive for TestComponent {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Char('q') => HandleResult::Action(Action::Quit),
            KeyCode::Char('r') => HandleResult::NeedsRender,
            KeyCode::Enter => HandleResult::Consumed,
            _ => HandleResult::Ignored,
        }
    }

    fn handle_mouse(&mut self, event: MouseEvent) -> HandleResult {
        if event.kind == MouseEventKind::Down(MouseButton::Left) {
            HandleResult::Consumed
        } else {
            HandleResult::Ignored
        }
    }
}

impl Component for TestComponent {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        self.can_focus
    }

    fn on_mount(&mut self) -> TuiResult<()> {
        self.mounted = true;
        Ok(())
    }

    fn on_unmount(&mut self) -> TuiResult<()> {
        self.mounted = false;
        Ok(())
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        self.focus_gained_count += 1;
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        self.focus_lost_count += 1;
        Ok(())
    }
}

// ============================================================================
// Render Trait 测试
// ============================================================================

#[test]
fn test_render_trait() {
    let comp = TestComponent::new(ComponentId::new(1), false);
    let area = Rect::new(0, 0, 10, 10);
    let mut buffer = Buffer::empty(area);

    // 渲染不应该 panic
    comp.render(area, &mut buffer);
}

// ============================================================================
// Interactive Trait 测试
// ============================================================================

#[test]
fn test_interactive_handle_key_quit() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    let result = comp.handle_key(key);
    assert!(matches!(result, HandleResult::Action(Action::Quit)));
}

#[test]
fn test_interactive_handle_key_needs_render() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    let result = comp.handle_key(key);
    assert_eq!(result, HandleResult::NeedsRender);
}

#[test]
fn test_interactive_handle_key_consumed() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let result = comp.handle_key(key);
    assert_eq!(result, HandleResult::Consumed);
}

#[test]
fn test_interactive_handle_key_ignored() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    let result = comp.handle_key(key);
    assert_eq!(result, HandleResult::Ignored);
}

#[test]
fn test_interactive_handle_mouse_click() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    let result = comp.handle_mouse(event);
    assert_eq!(result, HandleResult::Consumed);
}

#[test]
fn test_interactive_handle_mouse_ignored() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let event = MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: 5,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    let result = comp.handle_mouse(event);
    assert_eq!(result, HandleResult::Ignored);
}

// ============================================================================
// Component Trait 基础测试
// ============================================================================

#[test]
fn test_component_id() {
    let comp = TestComponent::new(ComponentId::new(42), true);
    assert_eq!(comp.id().value(), 42);
}

#[test]
fn test_component_can_focus_true() {
    let comp = TestComponent::new(ComponentId::new(1), true);
    assert!(comp.can_focus());
}

#[test]
fn test_component_can_focus_false() {
    let comp = TestComponent::new(ComponentId::new(1), false);
    assert!(!comp.can_focus());
}

#[test]
fn test_component_on_event_default() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let event = AppEvent::Refresh;
    let result = comp.on_event(&event);
    assert_eq!(result, HandleResult::Ignored);
}

// ============================================================================
// Component 生命周期钩子测试
// ============================================================================

#[test]
fn test_component_on_mount() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    assert!(!comp.mounted);

    comp.on_mount().unwrap();
    assert!(comp.mounted);
}

#[test]
fn test_component_on_unmount() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    comp.mounted = true;

    comp.on_unmount().unwrap();
    assert!(!comp.mounted);
}

#[test]
fn test_component_on_focus_gain() {
    let mut comp = TestComponent::new(ComponentId::new(1), true);
    assert_eq!(comp.focus_gained_count, 0);

    comp.on_focus_gain().unwrap();
    assert_eq!(comp.focus_gained_count, 1);

    comp.on_focus_gain().unwrap();
    assert_eq!(comp.focus_gained_count, 2);
}

#[test]
fn test_component_on_focus_loss() {
    let mut comp = TestComponent::new(ComponentId::new(1), true);
    assert_eq!(comp.focus_lost_count, 0);

    comp.on_focus_loss().unwrap();
    assert_eq!(comp.focus_lost_count, 1);
}

#[test]
fn test_component_before_render() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    assert!(comp.before_render().is_ok());
}

#[test]
fn test_component_after_render() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    assert!(comp.after_render().is_ok());
}

#[test]
fn test_component_lifecycle_flow() {
    let mut comp = TestComponent::new(ComponentId::new(1), true);

    // 模拟完整生命周期
    comp.on_mount().unwrap();
    assert!(comp.mounted);

    comp.on_focus_gain().unwrap();
    assert_eq!(comp.focus_gained_count, 1);

    comp.before_render().unwrap();
    // 渲染...
    comp.after_render().unwrap();

    comp.on_focus_loss().unwrap();
    assert_eq!(comp.focus_lost_count, 1);

    comp.on_unmount().unwrap();
    assert!(!comp.mounted);
}

// ============================================================================
// Component Trait 对象安全测试
// ============================================================================

#[test]
fn test_component_as_render() {
    let comp = TestComponent::new(ComponentId::new(1), false);
    let area = Rect::new(0, 0, 10, 10);
    let mut buffer = Buffer::empty(area);

    // 作为 Render trait 对象
    let renderable: &dyn Render = &comp;
    renderable.render(area, &mut buffer);
}

#[test]
fn test_component_as_interactive() {
    let mut comp = TestComponent::new(ComponentId::new(1), false);
    let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

    // 作为 Interactive trait 对象
    let interactive: &mut dyn Interactive = &mut comp;
    let result = interactive.handle_key(key);
    assert!(matches!(result, HandleResult::Action(Action::Quit)));
}

#[test]
fn test_component_as_component() {
    let comp = TestComponent::new(ComponentId::new(1), false);

    // 作为 Component trait 对象
    let component: &dyn Component = &comp;
    assert_eq!(component.id().value(), 1);
    assert!(!component.can_focus());
}

// ============================================================================
// 测试支持特定事件的组件
// ============================================================================

struct EventAwareComponent {
    id: ComponentId,
}

impl Render for EventAwareComponent {
    fn render(&self, _area: Rect, _buf: &mut Buffer) {}
}

impl Interactive for EventAwareComponent {
    fn handle_key(&mut self, _key: KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

impl Component for EventAwareComponent {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_event(&mut self, event: &AppEvent) -> HandleResult {
        match event {
            AppEvent::Quit => HandleResult::Action(Action::Quit),
            AppEvent::Refresh => HandleResult::NeedsRender,
            AppEvent::ScreenOpened(_) => HandleResult::NeedsRender,
            _ => HandleResult::Ignored,
        }
    }
}

#[test]
fn test_component_custom_on_event() {
    let mut comp = EventAwareComponent {
        id: ComponentId::new(1),
    };

    assert!(matches!(comp.on_event(&AppEvent::Quit), HandleResult::Action(Action::Quit)));
    assert_eq!(comp.on_event(&AppEvent::Refresh), HandleResult::NeedsRender);
    assert_eq!(comp.on_event(&AppEvent::Tick), HandleResult::Ignored);
}
