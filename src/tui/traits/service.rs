//! 服务提供者 trait 定义
//!
//! 定义服务提供者、ID 生成器、构建上下文和可构建组件等接口。

use crate::tui::error::TuiResult;
use crate::tui::traits::clipboard::ClipboardService;
use crate::tui::traits::ComponentId;
use std::any::Any;
use std::sync::Arc;

// ============================================================================
// 服务提供者 Trait
// ============================================================================

/// 服务提供者 trait
///
/// 组件通过此接口获取所需的服务，实现依赖注入。
pub trait ServiceProvider: Send + Sync {
    /// 获取数据库服务
    fn database(&self) -> Option<Arc<dyn DatabaseService>>;

    /// 获取剪贴板服务
    fn clipboard(&self) -> Option<Arc<dyn ClipboardService>>;

    /// 获取加密服务
    fn crypto(&self) -> Option<Arc<dyn CryptoService>>;

    /// 获取密码服务
    fn password(&self) -> Option<Arc<dyn PasswordService>>;
}

// ============================================================================
// 安全清除 Trait
// ============================================================================

/// 安全清除 trait
pub trait SecureClear {
    /// 清除敏感数据
    fn clear_sensitive_data(&mut self);
}

// ============================================================================
// 服务 Trait 定义
// ============================================================================

/// 数据库服务 trait
#[async_trait::async_trait]
pub trait DatabaseService: Send + Sync + SecureClear {
    // === CRUD ===
    /// 根据 ID 获取密码记录
    async fn get_password(&self, id: &str) -> TuiResult<()>;
    /// 保存密码记录
    async fn save_password(&self, record: &()) -> TuiResult<()>;
    /// 删除密码记录（移入回收站或永久删除）
    async fn delete_password(&self, id: &str, to_trash: bool) -> TuiResult<()>;

    // === Query ===
    /// 带过滤和排序的查询
    async fn query(&self, request: ()) -> TuiResult<()>;
    /// 获取各过滤条件的计数
    async fn get_filter_counts(&self) -> TuiResult<std::collections::HashMap<String, usize>>;

    // === Groups ===
    /// 获取分组树
    async fn get_group_tree(&self) -> TuiResult<()>;
    /// 获取单个分组
    async fn get_group(&self, id: &str) -> TuiResult<()>;
    /// 保存分组
    async fn save_group(&self, group: &()) -> TuiResult<()>;
    /// 删除分组
    async fn delete_group(&self, id: &str) -> TuiResult<()>;

    // === Trash ===
    /// 获取回收站中的项目
    async fn get_trash_items(&self) -> TuiResult<Vec<()>>;
    /// 从回收站恢复
    async fn restore_password(&self, id: &str) -> TuiResult<()>;
    /// 永久删除
    async fn permanently_delete(&self, id: &str) -> TuiResult<()>;
    /// 清空回收站
    async fn empty_trash(&self) -> TuiResult<usize>;
}

/// 密码类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PasswordType {
    /// 随机密码
    #[default]
    Random,
    /// 易记密码
    Memorable,
    /// PIN 码
    Pin,
}

impl PasswordType {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Random => "Random Password",
            Self::Memorable => "Memorable (Word-based)",
            Self::Pin => "PIN Code",
        }
    }
}

/// 密码策略
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    /// 密码类型
    pub password_type: PasswordType,
    /// 密码长度
    pub length: u8,
    /// 最少数字数
    pub min_digits: u8,
    /// 最少特殊字符数
    pub min_special: u8,
    /// 最少小写字母数
    pub min_lowercase: u8,
    /// 最少大写字母数
    pub min_uppercase: u8,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            password_type: PasswordType::Random,
            length: 16,
            min_digits: 2,
            min_special: 1,
            min_lowercase: 1,
            min_uppercase: 1,
        }
    }
}

/// 密码强度（简化版本，用于 CryptoService）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// 弱
    Weak,
    /// 一般
    Fair,
    /// 良好
    Good,
    /// 强
    Strong,
}

/// 加密服务 trait
pub trait CryptoService: Send + Sync {
    /// 加密数据
    fn encrypt(&self, data: &[u8]) -> TuiResult<Vec<u8>>;
    /// 解密数据
    fn decrypt(&self, data: &[u8]) -> TuiResult<Vec<u8>>;
    /// 根据策略生成密码
    fn generate_password(&self, policy: &PasswordPolicy) -> TuiResult<String>;
    /// 检查密码强度
    fn check_password_strength(&self, password: &str) -> PasswordStrength;
}

/// 密码服务 trait
pub trait PasswordService: Send + Sync {}

// ============================================================================
// ID 生成器
// ============================================================================

/// ID 生成器 trait
pub trait IdGenerator: Send + Sync {
    /// 生成新 ID
    fn generate(&self) -> ComponentId;

    /// 生成带前缀的 ID
    fn generate_with_prefix(&self, prefix: &str) -> ComponentId;
}

/// 默认 ID 生成器
#[derive(Debug, Default)]
pub struct DefaultIdGenerator {
    counter: std::sync::atomic::AtomicU64,
}

impl DefaultIdGenerator {
    /// 创建新的 ID 生成器
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

impl IdGenerator for DefaultIdGenerator {
    fn generate(&self) -> ComponentId {
        let id = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst) as usize;
        ComponentId::new(id)
    }

    fn generate_with_prefix(&self, _prefix: &str) -> ComponentId {
        // 注意：当前 ComponentId 是 usize 类型，不支持字符串前缀
        // 返回普通 ID，实际使用中可以考虑修改 ComponentId 结构
        self.generate()
    }
}

// ============================================================================
// 组件配置 Trait
// ============================================================================

/// 组件配置 trait
///
/// 允许组件使用任意配置类型。
pub trait ComponentConfig: Send + Sync {
    /// 转换为 Any 类型
    fn as_any(&self) -> &dyn Any;
}

// 为所有 Send + Sync + 'static 类型实现 ComponentConfig
impl<T: Send + Sync + 'static> ComponentConfig for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// 构建上下文
// ============================================================================

/// 构建上下文
///
/// 提供组件创建所需的最小信息。
pub struct BuildContext<'a> {
    /// 服务提供者
    services: Option<&'a dyn ServiceProvider>,
    /// ID 生成器
    id_generator: Option<&'a dyn IdGenerator>,
    /// 父组件 ID
    parent_id: Option<ComponentId>,
    /// 组件配置
    config: Option<Arc<dyn ComponentConfig>>,
}

impl<'a> BuildContext<'a> {
    /// 创建新的构建上下文
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: None,
            id_generator: None,
            parent_id: None,
            config: None,
        }
    }

    /// 设置服务提供者
    #[must_use]
    pub const fn with_services(mut self, services: &'a dyn ServiceProvider) -> Self {
        self.services = Some(services);
        self
    }

    /// 设置 ID 生成器
    #[must_use]
    pub const fn with_id_generator(mut self, id_generator: &'a dyn IdGenerator) -> Self {
        self.id_generator = Some(id_generator);
        self
    }

    /// 设置父组件 ID
    #[must_use]
    pub const fn with_parent(mut self, parent: ComponentId) -> Self {
        self.parent_id = Some(parent);
        self
    }

    /// 设置组件配置
    #[must_use]
    pub fn with_config(mut self, config: Arc<dyn ComponentConfig>) -> Self {
        self.config = Some(config);
        self
    }

    /// 生成组件 ID
    #[must_use]
    pub fn generate_id(&self) -> ComponentId {
        self.id_generator
            .map(|gen| gen.generate())
            .unwrap_or_else(|| ComponentId::new(0))
    }

    /// 获取服务提供者
    #[must_use]
    pub const fn services(&self) -> Option<&'a dyn ServiceProvider> {
        self.services
    }

    /// 获取父组件 ID
    #[must_use]
    pub const fn parent_id(&self) -> Option<&ComponentId> {
        self.parent_id.as_ref()
    }

    /// 获取配置
    pub fn get_config<T: ComponentConfig + 'static>(&self) -> Option<&T> {
        self.config.as_ref()?.as_any().downcast_ref()
    }
}

impl<'a> Default for BuildContext<'a> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 可构建 Trait
// ============================================================================

/// 可构建的组件 trait
///
/// 允许组件从构建上下文创建。
pub trait Buildable: Sized {
    /// 从构建上下文创建
    fn build(context: &BuildContext) -> TuiResult<Self>;
}

// 空实现宏，用于简单的类型
macro_rules! impl_buildable_default {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Buildable for $ty {
                fn build(_context: &BuildContext) -> TuiResult<Self> {
                    Ok(Self::default())
                }
            }
        )*
    };
}

// 为常见类型实现默认的 Buildable
impl_buildable_default! {
    String,
    Vec<u8>,
    Vec<String>,
}

// ============================================================================
// 服务容器
// ============================================================================

/// 服务容器
///
/// 具体的服务容器实现，用于管理所有服务。
#[derive(Default)]
pub struct ServiceContainer {
    /// 数据库服务
    database: Option<Arc<dyn DatabaseService>>,
    /// 剪贴板服务
    clipboard: Option<Arc<dyn ClipboardService>>,
    /// 加密服务
    crypto: Option<Arc<dyn CryptoService>>,
    /// 密码服务
    password: Option<Arc<dyn PasswordService>>,
}

impl ServiceContainer {
    /// 创建新的服务容器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            database: None,
            clipboard: None,
            crypto: None,
            password: None,
        }
    }

    /// 设置数据库服务
    pub fn with_database(mut self, service: Arc<dyn DatabaseService>) -> Self {
        self.database = Some(service);
        self
    }

    /// 设置剪贴板服务
    pub fn with_clipboard(mut self, service: Arc<dyn ClipboardService>) -> Self {
        self.clipboard = Some(service);
        self
    }

    /// 设置加密服务
    pub fn with_crypto(mut self, service: Arc<dyn CryptoService>) -> Self {
        self.crypto = Some(service);
        self
    }

    /// 设置密码服务
    pub fn with_password(mut self, service: Arc<dyn PasswordService>) -> Self {
        self.password = Some(service);
        self
    }
}

impl ServiceProvider for ServiceContainer {
    fn database(&self) -> Option<Arc<dyn DatabaseService>> {
        self.database.clone()
    }

    fn clipboard(&self) -> Option<Arc<dyn ClipboardService>> {
        self.clipboard.clone()
    }

    fn crypto(&self) -> Option<Arc<dyn CryptoService>> {
        self.crypto.clone()
    }

    fn password(&self) -> Option<Arc<dyn PasswordService>> {
        self.password.clone()
    }
}
