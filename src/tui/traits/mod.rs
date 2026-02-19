//! TUI Trait 层定义
//!
//! 本模块定义了 TUI 框架的所有核心 trait，为组件、状态管理、主题等提供统一的抽象接口。

mod component;
mod layout;
mod state;
mod focus;
mod service;
mod theme;
mod notification;
mod validation;
mod password_strength;
mod clipboard;
mod ime;
mod screen;
mod event;
mod secure;
mod task;

// 重新导出所有公共 trait
pub use component::{Component, Container, Render, Interactive, Application};
pub use layout::{Layout, LayoutConstraints, LayoutResult};
pub use state::{
    StateManager, ReactiveState, StateValue, StateError, StateKey, StateChange, StateCallback,
    SubscriptionId, SubscriptionIdGenerator,
};
pub use focus::{FocusManager, FocusState, FocusStyle, FocusNavigation, Direction, FocusManagerExt};
pub use service::{
    ServiceProvider, IdGenerator, BuildContext, Buildable, ComponentConfig, ServiceContainer,
    DatabaseService, CryptoService, PasswordService, DefaultIdGenerator, SecureClear,
    PasswordPolicy, PasswordType,
};
// Re-export service::PasswordStrength with an alias to avoid conflict
pub use service::PasswordStrength as ServicePasswordStrength;
pub use theme::{Theme, ColorPalette, ThemeVariant, ThemeName, ThemeManager, DarkTheme, LightTheme};
pub use notification::{
    NotificationManager, NotificationManagerExt, Notification, NotificationLevel, NotificationId, NotificationPosition,
};
pub use validation::{FormValidator, ValidationResult, Validator, FieldValidation, ValidationTrigger};
// ValidationRule 是 Validator 的别名（兼容旧代码）
pub use Validator as ValidationRule;
pub use password_strength::{PasswordStrength, PasswordStrengthCalculator, StrengthLevel};
pub use clipboard::{
    ClipboardService, ClipboardContent, ClipboardState, ClipboardConfig, ClipboardContentType,
    ClipboardSensitivity, SecureClipboardContent,
};
pub use ime::{
    ImeService, ImeMode, CompositionState, ImeState, ImeHandleResult, ImeAware, ImeDetector,
    ImeAwareDispatcher,
};
pub use screen::{
    ScreenManager, Screen, ScreenStack, ScreenTransition, ScreenFactory, ScreenResult,
    WizardFlow, WizardStep, WizardBranch, WizardData,
};

// 重新导出事件类型
pub use event::{AppEvent, HandleResult, Action, ScreenType, FilterType, EventDispatcher, EventFilter};

// 重新导出安全类型
pub use secure::{Sensitivity, SecureString, PasswordField, HoldsSensitiveData};

// 重新导出任务管理类型
pub use task::{TaskManager, TaskId, TaskStatus, TaskResult, TaskCallback, TaskProgress};

// ============================================================================
// 基础类型定义
// ============================================================================

/// 组件唯一标识符
///
/// 每个组件都有一个唯一的 ID，用于：
/// - 焦点管理
/// - 事件路由
/// - 状态查找
/// - 调试追踪
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub usize);

impl ComponentId {
    /// 创建新的组件 ID
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// 获取 ID 的数值
    #[must_use]
    pub const fn value(&self) -> usize {
        self.0
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentId({})", self.0)
    }
}
