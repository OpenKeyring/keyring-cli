//! 剪贴板 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.5 中完成。

/// 剪贴板服务 trait
pub trait ClipboardService: Send + Sync {
    /// 复制到剪贴板
    fn copy(&mut self, content: ClipboardContent) -> std::io::Result<()>;

    /// 从剪贴板读取
    fn paste(&self) -> std::io::Result<Option<ClipboardContent>>;
}

/// 剪贴板内容
#[derive(Debug, Clone, Default)]
pub struct ClipboardContent {
    pub _text: Option<String>,
}
