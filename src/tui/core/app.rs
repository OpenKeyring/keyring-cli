//! TUI Application 实现
//!
//! 提供完整的应用程序框架，包括事件循环、渲染和组件管理。

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::core::{
    DefaultFocusManager, DefaultStateManager, DefaultScreenManager, DefaultNotificationManager,
    TokioTaskManager,
};
use crate::tui::traits::{
    Application, HandleResult, Action, ScreenType, BuildContext,
    DefaultIdGenerator, ServiceContainer, NotificationManagerExt, NotificationManager, ScreenManager,
    TaskManager,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::Paragraph,
    Frame,
    Terminal,
};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

// ============================================================================
// 应用程序配置
// ============================================================================

/// 应用程序配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 是否启用调试模式
    pub debug: bool,
    /// 是否启用鼠标支持
    pub enable_mouse: bool,
    /// 刷新率
    pub tick_rate: Duration,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            debug: false,
            enable_mouse: false,
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl AppConfig {
    /// 创建新的配置
    #[must_use]
    pub const fn new() -> Self {
        Self {
            debug: false,
            enable_mouse: false,
            tick_rate: Duration::from_millis(250),
        }
    }

    /// 设置调试模式
    #[must_use]
    pub const fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// 设置鼠标支持
    #[must_use]
    pub const fn with_mouse(mut self, enable: bool) -> Self {
        self.enable_mouse = enable;
        self
    }

    /// 设置刷新率
    #[must_use]
    pub const fn with_tick_rate(mut self, rate: Duration) -> Self {
        self.tick_rate = rate;
        self
    }
}

// ============================================================================
// 应用程序状态
// ============================================================================

/// 应用程序运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// 正在运行
    Running,
    /// 应该退出
    ShouldQuit,
    /// 正在退出
    Quitting,
}

impl AppState {
    /// 是否应该继续运行
    #[must_use]
    pub const fn should_continue(&self) -> bool {
        !matches!(self, Self::ShouldQuit | Self::Quitting)
    }

    /// 是否正在运行
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

// ============================================================================
// TUI 应用程序
// ============================================================================

/// TUI 应用程序
///
/// 主应用程序结构，负责：
/// - 管理应用程序生命周期
/// - 协调各个管理器
/// - 处理事件循环
pub struct TuiApp {
    /// 应用状态
    state: AppState,
    /// 配置
    config: AppConfig,
    /// 焦点管理器
    focus_manager: DefaultFocusManager,
    /// 状态管理器
    state_manager: DefaultStateManager,
    /// 屏幕管理器
    screen_manager: DefaultScreenManager,
    /// 通知管理器
    notification_manager: DefaultNotificationManager,
    /// 任务管理器
    task_manager: TokioTaskManager,
    /// 服务提供者
    services: ServiceContainer,
    /// ID 生成器
    id_generator: DefaultIdGenerator,
    /// 上次渲染时间
    last_render: Instant,
    /// 上次事件时间
    last_event: Instant,
}

impl TuiApp {
    /// 创建新的 TUI 应用
    pub fn new(config: AppConfig) -> TuiResult<Self> {
        // 创建任务管理器（会启动 tokio runtime）
        let task_manager = TokioTaskManager::new()
            .map_err(|e| TuiError::invalid_state(&format!("Failed to create task manager: {}", e)))?;

        Ok(Self {
            state: AppState::Running,
            config,
            focus_manager: DefaultFocusManager::new(),
            state_manager: DefaultStateManager::new(),
            screen_manager: DefaultScreenManager::new(),
            notification_manager: DefaultNotificationManager::new(),
            task_manager,
            services: ServiceContainer::new(),
            id_generator: DefaultIdGenerator::new(),
            last_render: Instant::now(),
            last_event: Instant::now(),
        })
    }

    /// 配置服务
    pub fn with_service(self, _service: impl Send + Sync + 'static) -> Self {
        // 这里可以添加各种服务
        self
    }

    /// 处理按键事件
    fn handle_key_event(&mut self, key: KeyEvent) -> TuiResult<HandleResult> {
        // 检查全局快捷键
        if let Some(result) = self.check_global_shortcut(key) {
            return Ok(result);
        }

        // 检查是否有活动屏幕
        if let Some(screen) = self.screen_manager.current_mut() {
            // 屏幕优先处理事件
            if screen.is_modal() {
                return Ok(HandleResult::Consumed);
            }
        }

        // 获取焦点组件并处理事件
        // TODO: 实现焦点组件的事件处理

        Ok(HandleResult::Ignored)
    }

    /// 检查全局快捷键
    fn check_global_shortcut(&mut self, key: KeyEvent) -> Option<HandleResult> {
        match key.code {
            KeyCode::Char('q') => Some(HandleResult::Action(Action::Quit)),
            KeyCode::Char('Q') => Some(HandleResult::Action(Action::Quit)),
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // 刷新
                Some(HandleResult::NeedsRender)
            }
            _ => None,
        }
    }

    /// 处理动作
    fn handle_action(&mut self, action: Action) -> TuiResult<()> {
        match action {
            Action::Quit => {
                self.state = AppState::ShouldQuit;
            }
            Action::Refresh => {
                // 强制重新渲染
                self.last_render = Instant::now() - Duration::from_secs(1);
            }
            Action::OpenScreen(screen_type) => {
                self.open_screen(screen_type)?;
            }
            Action::CloseScreen => {
                let _ = self.screen_manager.pop()?;
            }
            Action::ShowToast(message) => {
                self.notification_manager.info(&message);
            }
            _ => {}
        }
        Ok(())
    }

    /// 打开屏幕
    fn open_screen(&mut self, _screen_type: ScreenType) -> TuiResult<()> {
        // TODO: 使用 ScreenFactory 创建屏幕
        // 这里暂时使用占位符实现
        Ok(())
    }

    /// 单次事件迭代
    pub fn tick(&mut self) -> TuiResult<bool> {
        // 处理已完成的异步任务
        let results = self.task_manager.poll_completed();
        for (id, result) in results {
            match result {
                crate::tui::traits::TaskResult::Success(_data) => {
                    log::info!("Task {:?} completed successfully", id);
                }
                crate::tui::traits::TaskResult::Failed(error) => {
                    log::error!("Task {:?} failed: {:?}", id, error);
                }
            }
        }

        // 更新通知管理器（清除过期通知）
        self.notification_manager.tick();

        // 检查是否应该退出
        if !self.state.should_continue() {
            return Ok(false);
        }

        Ok(true)
    }

    /// 渲染界面
    pub fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> TuiResult<()> {
        let now = Instant::now();

        // 根据刷新率决定是否需要重新渲染
        let should_render = now.duration_since(self.last_render) > self.config.tick_rate;

        if !should_render {
            return Ok(());
        }

        terminal.draw(|frame| {
            // 渲染通知
            self.render_notifications(frame);

            // 渲染屏幕（如果有活动屏幕）
            if let Some(_screen) = self.screen_manager.current() {
                // TODO: 渲染屏幕
            }

            // 渲染主界面
            self.render_main_ui(frame);

            self.last_render = now;
        })?;

        Ok(())
    }

    /// 渲染通知
    fn render_notifications(&self, _frame: &mut Frame) {
        let notifications = self.notification_manager.active_notifications();
        if !notifications.is_empty() {
            // TODO: 实现通知渲染
        }
    }

    /// 渲染主界面
    fn render_main_ui(&self, frame: &mut Frame) {
        let size = frame.area();

        // 简单的占位符渲染
        let text = "OpenKeyring TUI\n\nPress 'q' to quit";

        let paragraph = Paragraph::new(text);
        let area = Rect::new(0, 0, size.width, size.height);
        frame.render_widget(paragraph, area);
    }

    /// 获取焦点管理器
    pub fn focus_manager(&self) -> &DefaultFocusManager {
        &self.focus_manager
    }

    /// 获取焦点管理器（可变）
    pub fn focus_manager_mut(&mut self) -> &mut DefaultFocusManager {
        &mut self.focus_manager
    }

    /// 获取状态管理器
    pub fn state_manager(&self) -> &DefaultStateManager {
        &self.state_manager
    }

    /// 获取状态管理器（可变）
    pub fn state_manager_mut(&mut self) -> &mut DefaultStateManager {
        &mut self.state_manager
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

    /// 获取任务管理器
    pub fn task_manager(&self) -> &TokioTaskManager {
        &self.task_manager
    }

    /// 获取任务管理器（可变）
    pub fn task_manager_mut(&mut self) -> &mut TokioTaskManager {
        &mut self.task_manager
    }

    /// 获取服务提供者
    pub fn services(&self) -> &ServiceContainer {
        &self.services
    }

    /// 获取服务提供者（可变）
    pub fn services_mut(&mut self) -> &mut ServiceContainer {
        &mut self.services
    }

    /// 获取应用状态
    #[must_use]
    pub const fn state(&self) -> AppState {
        self.state
    }

    /// 获取配置
    #[must_use]
    pub const fn config(&self) -> &AppConfig {
        &self.config
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new(AppConfig::default()).expect("Failed to create TuiApp")
    }
}

// ============================================================================
// Application trait 实现
// ============================================================================

impl Application for TuiApp {
    fn run(&mut self) -> io::Result<()> {
        // Install panic hook FIRST to ensure terminal recovery on panic
        crate::tui::panic_hook::install_panic_hook();

        // 设置终端
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).unwrap();

        // 主事件循环
        while self.state.should_continue() {
            // 读取事件
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        // 处理按键
                        let result = self.handle_key_event(key_event)
                            .map_err(|e| io::Error::other(e.to_string()))?;

                        // 处理动作
                        if let HandleResult::Action(action) = result {
                            self.handle_action(action)
                                .map_err(|e| io::Error::other(e.to_string()))?;
                        } else if let HandleResult::NeedsRender = result {
                            // 需要重新渲染
                            self.render(&mut terminal)
                                .map_err(|e| io::Error::other(e.to_string()))?;
                        }
                    }
                    _ => {}
                }
            }

            // 处理单次迭代（异步任务等）
            let should_continue = self.tick()
                .map_err(|e| io::Error::other(e.to_string()))?;
            if !should_continue {
                break;
            }

            // 渲染界面
            self.render(&mut terminal)
                .map_err(|e| io::Error::other(e.to_string()))?;
        }

        // 恢复终端状态
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;

        Ok(())
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 创建默认的构建上下文
pub fn create_build_context(app: &TuiApp) -> BuildContext<'_> {
    BuildContext::new()
        .with_id_generator(&app.id_generator)
        .with_services(&app.services)
}
