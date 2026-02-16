//! 异步任务管理 Trait 定义
//!
//! 定义 TUI 框架中异步操作的接口，用于协调同步事件循环与异步服务调用。

use std::any::Any;
use std::fmt;

use crate::tui::error::{TuiError, TuiResult};

// ============================================================================
// 异步任务 ID
// ============================================================================

/// 异步任务 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

impl TaskId {
    /// 创建新的任务 ID
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取 ID 值
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TaskId({})", self.0)
    }
}

// ============================================================================
// 异步任务状态
// ============================================================================

/// 异步任务状态
#[derive(Debug, Clone)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 已完成
    Completed,
    /// 执行失败
    Failed(TuiError),
    /// 已取消
    Cancelled,
}

impl TaskStatus {
    /// 检查任务是否已完成（成功或失败）
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed(_) | Self::Cancelled)
    }

    /// 检查任务是否正在运行
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// 获取任务是否成功
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

// ============================================================================
// 异步任务结果
// ============================================================================

/// 异步任务结果
pub enum TaskResult {
    /// 成功（结果可以是任意类型）
    Success(Box<dyn Any + Send>),
    /// 失败
    Failed(TuiError),
}

impl TaskResult {
    /// 获取成功结果
    #[must_use]
    pub fn success<T: 'static>(&self) -> Option<&T> {
        match self {
            Self::Success(any) => any.downcast_ref::<T>(),
            Self::Failed(_) => None,
        }
    }

    /// 检查是否成功
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }
}

// ============================================================================
// 任务完成回调
// ============================================================================

/// 任务完成回调
pub type TaskCallback = Box<dyn FnOnce(TaskResult) + Send>;

// ============================================================================
// 任务管理器 Trait
// ============================================================================

/// 异步任务管理器 trait
///
/// 用于在同步 TUI 事件循环中管理异步任务。
pub trait TaskManager: Send + Sync {
    /// 提交异步任务
    ///
    /// 将异步任务提交给 tokio runtime 执行。
    fn submit<F>(
        &mut self,
        name: &str,
        task: F,
        callback: Option<TaskCallback>,
    ) -> TaskId
    where
        F: std::future::Future<Output = TuiResult<Box<dyn Any + Send>>> + Send + 'static;

    /// 取消任务
    fn cancel(&mut self, id: TaskId) -> TuiResult<()>;

    /// 获取任务状态
    ///
    /// 返回任务当前状态，如果任务不存在则返回 None。
    fn status(&self, id: TaskId) -> Option<TaskStatus>;

    /// 轮询已完成的任务（非阻塞）
    ///
    /// 返回自上次调用以来所有完成的任务及其结果。
    fn poll_completed(&mut self) -> Vec<(TaskId, TaskResult)>;

    /// 获取活跃任务数量
    #[must_use]
    fn active_count(&self) -> usize;

    /// 等待所有任务完成（阻塞）
    ///
    /// 注意：在 TUI 事件循环中谨慎使用，可能阻塞 UI。
    fn wait_all(&mut self) -> TuiResult<()>;
}

// ============================================================================
// 任务进度（可选功能）
// ============================================================================

/// 任务进度信息
#[derive(Debug, Clone)]
pub struct TaskProgress {
    /// 进度百分比 (0-100)
    pub percent: u8,
    /// 当前状态描述
    pub message: String,
}

impl TaskProgress {
    /// 创建新的进度信息
    #[must_use]
    pub fn new(percent: u8, message: impl Into<String>) -> Self {
        Self {
            percent,
            message: message.into(),
        }
    }

    /// 创建初始进度
    #[must_use]
    pub fn initial() -> Self {
        Self {
            percent: 0,
            message: "Starting...".to_string(),
        }
    }

    /// 创建完成进度
    #[must_use]
    pub fn complete() -> Self {
        Self {
            percent: 100,
            message: "Complete".to_string(),
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id() {
        let id = TaskId::new(42);
        assert_eq!(id.value(), 42);
        assert_eq!(format!("{}", id), "TaskId(42)");
    }

    #[test]
    fn test_task_status() {
        let status = TaskStatus::Pending;
        assert!(!status.is_finished());
        assert!(!status.is_running());

        let status = TaskStatus::Running;
        assert!(status.is_running());
        assert!(!status.is_finished());

        let status = TaskStatus::Completed;
        assert!(status.is_finished());
        assert!(status.is_success());

        let status = TaskStatus::Failed(TuiError::invalid_state("test"));
        assert!(status.is_finished());
        assert!(!status.is_success());
    }

    #[test]
    fn test_task_progress() {
        let progress = TaskProgress::new(50, "Half done");
        assert_eq!(progress.percent, 50);
        assert_eq!(progress.message, "Half done");

        let progress = TaskProgress::initial();
        assert_eq!(progress.percent, 0);

        let progress = TaskProgress::complete();
        assert_eq!(progress.percent, 100);
    }
}
