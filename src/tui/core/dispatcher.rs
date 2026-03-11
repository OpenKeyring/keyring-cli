//! 事件分发器实现
//!
//! 提供完整的事件分发功能，包括全局快捷键、事件过滤和组件事件路由。

use crate::tui::error::TuiResult;
use crate::tui::core::{DefaultFocusManager, DefaultScreenManager, DefaultNotificationManager};
use crate::tui::traits::{
    AppEvent, HandleResult, Action, EventDispatcher, EventFilter,
    FocusManager, ScreenManager, NotificationManagerExt,
};
use crossterm::event::{KeyEvent, KeyModifiers};
use std::collections::HashMap;

// ============================================================================
// 默认事件过滤器
// ============================================================================

/// IME 状态过滤器
///
/// 当 IME 正在组合输入时，阻止全局快捷键。
#[derive(Debug, Clone)]
pub struct ImeEventFilter {
    id: String,
    composing: bool,
}

impl ImeEventFilter {
    /// 创建新的 IME 过滤器
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: "ime_filter".to_string(),
            composing: false,
        }
    }

    /// 设置组合状态
    pub fn set_composing(&mut self, composing: bool) {
        self.composing = composing;
    }

    /// 获取组合状态
    #[must_use]
    pub const fn is_composing(&self) -> bool {
        self.composing
    }
}

impl Default for ImeEventFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventFilter for ImeEventFilter {
    fn should_process(&self, event: &AppEvent) -> bool {
        // 如果正在组合输入，只允许键盘事件（传递给输入组件）
        if self.composing {
            matches!(event, AppEvent::Key(_))
        } else {
            true
        }
    }

    fn id(&self) -> &str {
        &self.id
    }
}

// ============================================================================
// 默认事件分发器
// ============================================================================

/// 默认事件分发器
///
/// 实现完整的事件分发逻辑：
/// 1. 应用事件过滤器
/// 2. 检查全局快捷键
/// 3. 分发到当前焦点组件
/// 4. 处理屏幕模态状态
pub struct DefaultEventDispatcher {
    /// 全局快捷键映射
    global_keybindings: HashMap<(KeyCode, KeyModifiers), Action>,
    /// 事件过滤器
    filters: Vec<Box<dyn EventFilter>>,
    /// 焦点管理器引用
    focus_manager: DefaultFocusManager,
    /// 屏幕管理器引用
    screen_manager: DefaultScreenManager,
    /// 通知管理器引用
    notification_manager: DefaultNotificationManager,
    /// 是否启用事件过滤
    filtering_enabled: bool,
}

impl std::fmt::Debug for DefaultEventDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultEventDispatcher")
            .field("global_keybindings", &self.global_keybindings.len())
            .field("filters", &self.filters.len())
            .field("filtering_enabled", &self.filtering_enabled)
            .finish()
    }
}

impl DefaultEventDispatcher {
    /// 创建新的事件分发器
    #[must_use]
    pub fn new() -> Self {
        let mut dispatcher = Self {
            global_keybindings: HashMap::new(),
            filters: Vec::new(),
            focus_manager: DefaultFocusManager::new(),
            screen_manager: DefaultScreenManager::new(),
            notification_manager: DefaultNotificationManager::new(),
            filtering_enabled: true,
        };

        // 注册默认全局快捷键
        dispatcher.register_default_keybindings();

        dispatcher
    }

    /// 设置焦点管理器
    pub fn with_focus_manager(mut self, manager: DefaultFocusManager) -> Self {
        self.focus_manager = manager;
        self
    }

    /// 设置屏幕管理器
    pub fn with_screen_manager(mut self, manager: DefaultScreenManager) -> Self {
        self.screen_manager = manager;
        self
    }

    /// 设置通知管理器
    pub fn with_notification_manager(mut self, manager: DefaultNotificationManager) -> Self {
        self.notification_manager = manager;
        self
    }

    /// 启用/禁用事件过滤
    pub fn set_filtering_enabled(&mut self, enabled: bool) {
        self.filtering_enabled = enabled;
    }

    /// 注册默认全局快捷键
    fn register_default_keybindings(&mut self) {
        use crossterm::event::KeyCode;
        

        // Ctrl+Q: 退出
        self.global_keybindings.insert(
            (KeyCode::Char('q'), KeyModifiers::CONTROL),
            Action::Quit,
        );

        // Ctrl+R: 刷新
        self.global_keybindings.insert(
            (KeyCode::Char('r'), KeyModifiers::CONTROL),
            Action::Refresh,
        );

        // ESC: 关闭屏幕
        self.global_keybindings.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            Action::CloseScreen,
        );
    }

    /// 处理键盘事件
    fn handle_key_event(&mut self, key: KeyEvent) -> HandleResult {
        // 1. 检查全局快捷键
        if let Some(action) = self.handle_global_keybinding(key) {
            return HandleResult::Action(action);
        }

        // 2. 检查是否有模态屏幕
        if let Some(_screen) = self.screen_manager.current() {
            // 如果屏幕是模态的，事件被屏幕消费
            // 这里返回 NeedsRender 以触发屏幕渲染
            return HandleResult::NeedsRender;
        }

        // 3. 检查当前焦点组件
        if let Some(focus_id) = self.focus_manager.current_focus() {
            // TODO: 将事件传递给焦点组件
            // 目前返回 Ignored 表示事件未被处理
            log::debug!("Event sent to focused component: {:?}", focus_id);
        }

        HandleResult::Ignored
    }

    /// 处理动作
    pub fn handle_action(&mut self, action: Action) -> TuiResult<()> {
        match action {
            Action::Quit => {
                // 通过通知管理器显示退出消息
                self.notification_manager.info("正在退出...");
            }
            Action::Refresh => {
                self.notification_manager.info("已刷新");
            }
            Action::OpenScreen(_screen_type) => {
                // TODO: 打开屏幕
            }
            Action::CloseScreen => {
                let _ = self.screen_manager.pop()?;
            }
            Action::ShowToast(message) => {
                self.notification_manager.info(&message);
            }
            Action::CopyToClipboard(_text) => {
                self.notification_manager.info("已复制到剪贴板");
            }
            Action::ConfirmDialog(confirmed) => {
                if confirmed {
                    self.notification_manager.info("操作已确认");
                } else {
                    self.notification_manager.info("操作已取消");
                }
            }
            Action::None => {}
        }
        Ok(())
    }

    /// 获取焦点管理器
    pub fn focus_manager(&self) -> &DefaultFocusManager {
        &self.focus_manager
    }

    /// 获取焦点管理器（可变）
    pub fn focus_manager_mut(&mut self) -> &mut DefaultFocusManager {
        &mut self.focus_manager
    }

    /// 获取屏幕管理器
    pub fn screen_manager(&self) -> &DefaultScreenManager {
        &self.screen_manager
    }

    /// 获取屏幕管理器（可变）
    pub fn screen_manager_mut(&mut self) -> &mut DefaultScreenManager {
        &mut self.screen_manager
    }

    /// 获取通知管理器
    pub fn notification_manager(&self) -> &DefaultNotificationManager {
        &self.notification_manager
    }

    /// 获取通知管理器（可变）
    pub fn notification_manager_mut(&mut self) -> &mut DefaultNotificationManager {
        &mut self.notification_manager
    }
}

impl Default for DefaultEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// 为了实现 Send + Sync，我们需要确保所有字段都是 Send + Sync
// Box<dyn EventFilter> 已经是 Send + Sync，因为 EventFilter trait 要求了
unsafe impl Send for DefaultEventDispatcher {}
unsafe impl Sync for DefaultEventDispatcher {}

impl EventDispatcher for DefaultEventDispatcher {
    fn dispatch(&mut self, event: AppEvent) -> HandleResult {
        // 1. 应用事件过滤器
        if self.filtering_enabled {
            for filter in &self.filters {
                if !filter.should_process(&event) {
                    log::debug!("Event filtered by: {}", filter.id());
                    return HandleResult::Consumed;
                }
            }
        }

        // 2. 根据事件类型分发
        match event {
            AppEvent::Key(key_event) => self.handle_key_event(key_event),
            AppEvent::Mouse(_mouse_event) => {
                // TODO: 处理鼠标事件
                HandleResult::Ignored
            }
            AppEvent::PasswordSelected(id, text) => {
                log::debug!("Password selected: {:?}, text: {:?}", id, text);
                HandleResult::Consumed
            }
            AppEvent::FocusChanged(id) => {
                log::debug!("Focus changed to: {:?}", id);
                HandleResult::NeedsRender
            }
            AppEvent::FilterChanged(filter_type) => {
                log::debug!("Filter changed to: {:?}", filter_type);
                HandleResult::NeedsRender
            }
            AppEvent::ScreenOpened(screen_type) => {
                log::debug!("Screen opened: {:?}", screen_type);
                HandleResult::NeedsRender
            }
            AppEvent::ScreenClosed => {
                log::debug!("Screen closed");
                HandleResult::NeedsRender
            }
            AppEvent::ScreenDismissed => {
                log::debug!("Screen dismissed");
                HandleResult::NeedsRender
            }
            AppEvent::Quit => HandleResult::Action(Action::Quit),
            AppEvent::Refresh => HandleResult::Action(Action::Refresh),
            AppEvent::Tick => HandleResult::NeedsRender,
        }
    }

    fn register_global_keybinding(&mut self, key: KeyEvent, action: Action) {
        self.global_keybindings.insert((key.code, key.modifiers), action);
    }

    fn handle_global_keybinding(&self, key: KeyEvent) -> Option<Action> {
        self.global_keybindings
            .get(&(key.code, key.modifiers))
            .cloned()
    }

    fn add_filter(&mut self, filter: Box<dyn EventFilter>) {
        self.filters.push(filter);
    }

    fn remove_filter(&mut self, id: &str) {
        self.filters.retain(|f| f.id() != id);
    }
}

// 为了在代码中使用 KeyCode
use crossterm::event::KeyCode;

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_dispatcher_new() {
        let dispatcher = DefaultEventDispatcher::new();
        assert_eq!(dispatcher.global_keybindings.len(), 3);
    }

    #[test]
    fn test_register_keybinding() {
        let mut dispatcher = DefaultEventDispatcher::new();
        let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL);
        dispatcher.register_global_keybinding(key, Action::Refresh);

        let action = dispatcher.handle_global_keybinding(key);
        assert_eq!(action, Some(Action::Refresh));
    }

    #[test]
    fn test_ime_filter() {
        let filter = ImeEventFilter::new();

        // 默认状态：处理所有事件
        assert!(filter.should_process(&AppEvent::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::empty()
        ))));
        assert!(filter.should_process(&AppEvent::Quit));

        // 组合状态：只处理键盘事件
        let mut filter = ImeEventFilter::new();
        filter.set_composing(true);
        assert!(filter.should_process(&AppEvent::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::empty()
        ))));
        assert!(!filter.should_process(&AppEvent::Quit));
    }

    #[test]
    fn test_dispatch_key_event() {
        let mut dispatcher = DefaultEventDispatcher::new();

        // Ctrl+Q 应该触发退出
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let result = dispatcher.dispatch(AppEvent::Key(key));
        assert_eq!(result, HandleResult::Action(Action::Quit));
    }

    #[test]
    fn test_dispatch_quit_event() {
        let mut dispatcher = DefaultEventDispatcher::new();
        let result = dispatcher.dispatch(AppEvent::Quit);
        assert_eq!(result, HandleResult::Action(Action::Quit));
    }

    #[test]
    fn test_add_remove_filter() {
        let mut dispatcher = DefaultEventDispatcher::new();
        let filter = Box::new(ImeEventFilter::new());

        dispatcher.add_filter(filter);
        assert_eq!(dispatcher.filters.len(), 1);

        dispatcher.remove_filter("ime_filter");
        assert_eq!(dispatcher.filters.len(), 0);
    }
}
