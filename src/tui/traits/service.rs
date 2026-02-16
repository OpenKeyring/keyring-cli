//! 服务提供者 trait 定义
//!
//! 占位符模块，完整实现将在 Task A.5 中完成。

/// 服务提供者 trait
pub trait ServiceProvider: Send + Sync {
    /// 获取服务
    fn get_service(&self, name: &str) -> Option<&dyn std::any::Any>;
}

/// ID 生成器
pub trait IdGenerator: Send + Sync {
    /// 生成新 ID
    fn generate(&mut self) -> usize;
}

/// 构建上下文
#[derive(Debug, Default)]
pub struct BuildContext {
    _private: (),
}

/// 可构建 trait
pub trait Buildable: Sized {
    /// 构建
    fn build(_ctx: &BuildContext) -> Self;
}
