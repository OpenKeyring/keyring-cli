//! TUI Trait 层定义
//!
//! 本模块定义了 TUI 框架的所有核心 trait，为组件、状态管理、主题等提供统一的抽象接口。

mod clipboard;
mod component;
mod event;
mod focus;
mod ime;
mod layout;
mod notification;
mod password_strength;
mod screen;
mod secure;
mod service;
mod state;
mod task;
mod theme;
mod validation;
mod wizard_step;

// 重新导出所有公共 trait
pub use component::{Application, Component, Container, Interactive, Render};
pub use focus::{
    Direction, FocusManager, FocusManagerExt, FocusNavigation, FocusState, FocusStyle,
};
pub use layout::{Layout, LayoutConstraints, LayoutResult};
pub use service::{
    BuildContext, Buildable, ComponentConfig, CryptoService, DatabaseService, DefaultIdGenerator,
    IdGenerator, PasswordPolicy, PasswordService, PasswordType, SecureClear, ServiceContainer,
    ServiceProvider,
};
pub use state::{
    ReactiveState, StateCallback, StateChange, StateError, StateKey, StateManager, StateValue,
    SubscriptionId, SubscriptionIdGenerator,
};
// Re-export service::PasswordStrength with an alias to avoid conflict
pub use notification::{
    DefaultNotificationRenderer, LevelFilter, Notification, NotificationConfig, NotificationFilter,
    NotificationId, NotificationLevel, NotificationManager, NotificationManagerExt,
    NotificationPosition, NotificationQueue, NotificationRenderer,
};
pub use service::PasswordStrength as ServicePasswordStrength;
pub use theme::{
    ColorPalette, DarkTheme, LightTheme, Theme, ThemeManager, ThemeName, ThemeVariant,
};
pub use validation::{
    BuiltinValidator, FieldValidation, FormValidator, ValidationResult, ValidationTrigger,
    Validator,
};
// ValidationRule 是 Validator 的别名（兼容旧代码）
pub use clipboard::{
    ClipboardConfig, ClipboardContent, ClipboardContentType, ClipboardSensitivity,
    ClipboardService, ClipboardState, SecureClipboardContent,
};
pub use ime::{
    CompositionState, ImeAware, ImeAwareDispatcher, ImeDetector, ImeHandleResult, ImeMode,
    ImeService, ImeState,
};
pub use password_strength::{PasswordStrength, PasswordStrengthCalculator, StrengthLevel};
pub use screen::{
    Screen, ScreenFactory, ScreenManager, ScreenResult, ScreenStack, ScreenTransition,
    WizardBranch, WizardData, WizardFlow, WizardStep,
};
pub use Validator as ValidationRule;

// 重新导出事件类型
pub use event::{
    Action, AppEvent, EventDispatcher, EventFilter, FilterType, HandleResult, ScreenType,
};

// 重新导出安全类型
pub use secure::{HoldsSensitiveData, PasswordField, SecureString, Sensitivity};

// 重新导出任务管理类型
pub use task::{TaskCallback, TaskId, TaskManager, TaskProgress, TaskResult, TaskStatus};

// 重新导出向导步骤验证类型
pub use wizard_step::{WizardStepScreen, WizardStepValidator};

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
