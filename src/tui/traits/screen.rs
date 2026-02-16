//! 屏幕管理 trait 定义
//!
//! 定义 TUI 屏幕管理相关的接口，包括屏幕、屏幕管理器和屏幕工厂。

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::traits::{Component, BuildContext, Action, ScreenType};
use ratatui::layout::Rect;

// ============================================================================
// 屏幕转换
// ============================================================================

/// 屏幕转换方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenTransition {
    /// 推入新屏幕（保留当前屏幕）
    Push,
    /// 替换当前屏幕
    Replace,
    /// 弹出当前屏幕
    Pop,
    /// 清空所有屏幕后推入
    ClearAndPush,
}

impl Default for ScreenTransition {
    fn default() -> Self {
        Self::Push
    }
}

// ============================================================================
// 屏幕 Trait
// ============================================================================

/// 屏幕 trait - 覆盖层组件
///
/// 屏幕是特殊的组件，通常以模态方式显示在主界面之上。
pub trait Screen: Component + Send + Sync {
    /// 获取屏幕类型
    fn screen_type(&self) -> ScreenType;

    /// 是否阻止下层事件（模态）
    #[must_use]
    fn is_modal(&self) -> bool {
        true
    }

    /// 是否显示背景遮罩
    #[must_use]
    fn show_overlay(&self) -> bool {
        self.is_modal()
    }

    /// 获取屏幕尺寸（相对于终端）
    #[must_use]
    fn size(&self, terminal: Rect) -> Rect {
        let width = (terminal.width as f32 * 0.8) as u16;
        let height = (terminal.height as f32 * 0.8) as u16;
        let x = (terminal.width.saturating_sub(width)) / 2;
        let y = (terminal.height.saturating_sub(height)) / 2;
        Rect::new(x, y, width, height)
    }

    /// 关闭屏幕
    fn close(&mut self) -> TuiResult<()>;

    /// 获取屏幕返回值（如果有）
    #[must_use]
    fn result(&self) -> Option<ScreenResult> {
        None
    }
}

/// 屏幕结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenResult {
    /// 确认
    Confirmed,
    /// 取消
    Cancelled,
    /// 自定义动作
    Action(Action),
    /// 带数据的结果
    Data(String),
}

// ============================================================================
// 屏幕栈
// ============================================================================

/// 屏幕栈
///
/// 管理多个屏幕的堆栈结构。
#[derive(Default)]
pub struct ScreenStack {
    /// 屏幕列表
    screens: Vec<Box<dyn Screen>>,
    /// 最大深度
    max_depth: usize,
}

impl std::fmt::Debug for ScreenStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScreenStack")
            .field("screen_count", &self.screens.len())
            .field("max_depth", &self.max_depth)
            .finish()
    }
}

impl ScreenStack {
    /// 创建新的屏幕栈
    #[must_use]
    pub fn new() -> Self {
        Self {
            screens: Vec::new(),
            max_depth: 10,
        }
    }

    /// 设置最大深度
    #[must_use]
    pub const fn with_max_depth(mut self, max: usize) -> Self {
        self.max_depth = max;
        self
    }

    /// 推入屏幕
    pub fn push(&mut self, screen: Box<dyn Screen>) -> TuiResult<()> {
        if self.screens.len() >= self.max_depth {
            return Err(TuiError::invalid_state("屏幕栈已满"));
        }
        self.screens.push(screen);
        Ok(())
    }

    /// 弹出屏幕
    pub fn pop(&mut self) -> Option<Box<dyn Screen>> {
        self.screens.pop()
    }

    /// 获取当前屏幕
    #[must_use]
    pub fn current(&self) -> Option<&dyn Screen> {
        self.screens.last().map(|s| s.as_ref())
    }

    /// 获取当前屏幕（可变）
    pub fn current_mut(&mut self) -> Option<&mut (dyn Screen + '_)> {
        if self.screens.is_empty() {
            None
        } else {
            // SAFETY: 我们知道 Vec 非空，直接获取最后一个元素的索引
            let idx = self.screens.len() - 1;
            Some(self.screens[idx].as_mut())
        }
    }

    /// 检查是否有活动屏幕
    #[must_use]
    pub fn has_active(&self) -> bool {
        !self.screens.is_empty()
    }

    /// 获取栈深度
    #[must_use]
    pub fn depth(&self) -> usize {
        self.screens.len()
    }

    /// 清空所有屏幕
    pub fn clear(&mut self) {
        self.screens.clear();
    }

    /// 获取所有屏幕
    #[must_use]
    pub fn all(&self) -> &[Box<dyn Screen>] {
        &self.screens
    }
}

// ============================================================================
// 屏幕管理器 Trait
// ============================================================================

/// 屏幕管理器 trait
///
/// 管理应用中的屏幕导航和生命周期。
pub trait ScreenManager: Send + Sync {
    /// 推入新屏幕
    fn push(&mut self, screen: Box<dyn Screen>) -> TuiResult<()>;

    /// 弹出当前屏幕
    fn pop(&mut self) -> TuiResult<Option<Box<dyn Screen>>>;

    /// 替换当前屏幕
    fn replace(&mut self, screen: Box<dyn Screen>) -> TuiResult<()>;

    /// 获取当前屏幕
    fn current(&self) -> Option<&dyn Screen>;

    /// 获取当前屏幕（可变）
    fn current_mut(&mut self) -> Option<&mut (dyn Screen + '_)>;

    /// 检查是否有活动屏幕
    #[must_use]
    fn has_active_screen(&self) -> bool;

    /// 清空所有屏幕
    fn clear(&mut self) -> TuiResult<()>;

    /// 获取屏幕栈深度
    #[must_use]
    fn depth(&self) -> usize;

    /// 导航到指定屏幕类型
    fn navigate_to(&mut self, screen_type: ScreenType) -> TuiResult<()>;
}

// ============================================================================
// 屏幕工厂 Trait
// ============================================================================

/// 屏幕工厂 trait - 创建不同类型的屏幕
pub trait ScreenFactory: Send + Sync {
    /// 创建向导屏幕
    fn create_wizard(&self, context: &BuildContext) -> TuiResult<Box<dyn Screen>>;

    /// 创建新建密码屏幕
    fn create_new_password(&self, context: &BuildContext) -> TuiResult<Box<dyn Screen>>;

    /// 创建编辑密码屏幕
    fn create_edit_password(&self, context: &BuildContext, id: &str) -> TuiResult<Box<dyn Screen>>;

    /// 创建确认对话框
    fn create_confirm_dialog(&self, title: &str, message: &str) -> TuiResult<Box<dyn Screen>>;

    /// 创建回收箱屏幕
    fn create_trash_bin(&self, context: &BuildContext) -> TuiResult<Box<dyn Screen>>;

    /// 创建设置屏幕
    fn create_settings(&self, context: &BuildContext) -> TuiResult<Box<dyn Screen>> {
        self.create_confirm_dialog("设置", "设置功能")
    }

    /// 创建主屏幕
    fn create_main(&self, context: &BuildContext) -> TuiResult<Box<dyn Screen>> {
        self.create_confirm_dialog("主屏幕", "主屏幕功能")
    }

    /// 创建帮助屏幕
    fn create_help(&self) -> TuiResult<Box<dyn Screen>> {
        self.create_confirm_dialog("帮助", "帮助信息")
    }

    /// 根据屏幕类型创建屏幕
    fn create(&self, screen_type: &ScreenType, context: &BuildContext) -> TuiResult<Box<dyn Screen>> {
        match screen_type {
            ScreenType::Wizard => self.create_wizard(context),
            ScreenType::NewPassword => self.create_new_password(context),
            ScreenType::EditPassword(id) => self.create_edit_password(context, id),
            ScreenType::ConfirmDialog => self.create_confirm_dialog("确认", "确定要执行此操作吗？"),
            ScreenType::TrashBin => self.create_trash_bin(context),
            ScreenType::Help => self.create_help(),
            ScreenType::Settings => self.create_settings(context),
            ScreenType::Main => self.create_main(context),
        }
    }
}

// ============================================================================
// 向导屏幕 Trait (扩展)
// ============================================================================

/// 向导步骤
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Welcome,
    SetupEncryption,
    CreateMasterPassword,
    ImportOrFresh,
    ConfigurePreferences,
    Complete,
}

/// 向导分支
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardBranch {
    /// 全新安装
    Fresh,
    /// 导入现有数据
    Import,
}

/// 向导数据
#[derive(Debug, Clone)]
pub struct WizardData {
    /// 主密码
    pub master_password: Option<String>,
    /// 是否导入
    pub import_data: bool,
    /// 导入路径
    pub import_path: Option<String>,
    /// 偏好设置
    pub preferences: Option<String>,
}

impl Default for WizardData {
    fn default() -> Self {
        Self {
            master_password: None,
            import_data: false,
            import_path: None,
            preferences: None,
        }
    }
}

/// 向导屏幕 trait
pub trait WizardFlow: Screen {
    /// 获取当前步骤
    fn current_step(&self) -> WizardStep;

    /// 获取当前分支
    fn branch(&self) -> Option<WizardBranch>;

    /// 设置分支
    fn set_branch(&mut self, branch: WizardBranch) -> TuiResult<()>;

    /// 获取总步骤数
    #[must_use]
    fn total_steps(&self) -> usize {
        6
    }

    /// 当前步骤序号
    #[must_use]
    fn step_number(&self) -> usize;

    /// 是否可以前进
    #[must_use]
    fn can_go_next(&self) -> bool;

    /// 是否可以后退
    #[must_use]
    fn can_go_back(&self) -> bool;

    /// 前进到下一步
    fn go_next(&mut self) -> TuiResult<()>;

    /// 返回上一步
    fn go_back(&mut self) -> TuiResult<()>;

    /// 获取向导数据
    fn data(&self) -> &WizardData;
}
