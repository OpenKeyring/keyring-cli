//! Tokio 任务管理器实现
//!
//! 提供在同步 TUI 事件循环中管理异步任务的功能。

use crate::tui::error::{ErrorKind, TuiError, TuiResult};
use crate::tui::traits::{TaskCallback, TaskId, TaskManager, TaskResult, TaskStatus};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::mpsc::{self as tokio_mpsc, Sender};

/// Tokio 任务管理器
///
/// 使用 tokio runtime 在后台执行异步任务，通过通道传递结果。
///
/// 由于 trait 需要 `Send + Sync`，我们使用 `Arc<Mutex<>>` 包装非 Sync 的部分。
pub struct TokioTaskManager {
    /// 内部可变状态
    inner: Arc<StdMutex<TokioTaskManagerInner>>,
    /// Tokio runtime（Arc 允许共享）
    runtime: Arc<tokio::runtime::Runtime>,
    /// 已完成任务发送器
    completed_tx: Sender<(TaskId, TaskResult)>,
    /// 下一个任务 ID
    next_task_id: Arc<core::sync::atomic::AtomicU64>,
}

/// 内部可变状态
struct TokioTaskManagerInner {
    /// 任务状态映射
    tasks: HashMap<TaskId, TaskInfo>,
    /// 处理中的任务句柄（用于取消）
    handles: HashMap<TaskId, tokio::task::AbortHandle>,
    /// 已完成任务接收器
    completed_rx: tokio_mpsc::Receiver<(TaskId, TaskResult)>,
}

/// 任务信息
struct TaskInfo {
    /// 任务名称
    name: String,
    /// 任务状态
    status: TaskStatus,
    /// 完成回调
    callback: Option<TaskCallback>,
}

impl std::fmt::Debug for TaskInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskInfo")
            .field("name", &self.name)
            .field("status", &self.status)
            .field("has_callback", &self.callback.is_some())
            .finish()
    }
}

impl TokioTaskManager {
    /// 创建新的任务管理器
    pub fn new() -> TuiResult<Self> {
        let (tx, rx) = tokio_mpsc::channel(100);

        // 创建多线程 tokio runtime
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            TuiError::new(ErrorKind::IoError(format!(
                "Failed to create tokio runtime: {}",
                e
            )))
        })?;

        Ok(Self {
            inner: Arc::new(StdMutex::new(TokioTaskManagerInner {
                tasks: HashMap::new(),
                handles: HashMap::new(),
                completed_rx: rx,
            })),
            runtime: Arc::new(runtime),
            completed_tx: tx,
            next_task_id: Arc::new(core::sync::atomic::AtomicU64::new(1)),
        })
    }

    /// 获取下一个任务 ID
    fn next_id(&self) -> TaskId {
        TaskId(
            self.next_task_id
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst),
        )
    }
}

impl Default for TokioTaskManager {
    fn default() -> Self {
        Self::new().expect("Failed to create TokioTaskManager")
    }
}

// SAFETY: TokioTaskManager 使用 Arc<Mutex<>> 保护内部状态，
// runtime 和 sender 都支持跨线程共享，所以是 Send + Sync 的
unsafe impl Send for TokioTaskManager {}
unsafe impl Sync for TokioTaskManager {}

impl TaskManager for TokioTaskManager {
    fn submit<F>(&mut self, name: &str, task: F, callback: Option<TaskCallback>) -> TaskId
    where
        F: std::future::Future<Output = TuiResult<Box<dyn Any + Send>>> + Send + 'static,
    {
        let id = self.next_id();
        let task_name = name.to_string();

        // 创建任务信息
        let task_info = TaskInfo {
            name: task_name.clone(),
            status: TaskStatus::Running,
            callback,
        };

        // 在 tokio runtime 中执行任务
        let sender = self.completed_tx.clone();
        let task_id = id;

        let handle = self.runtime.spawn(async move {
            let result = match task.await {
                Ok(data) => TaskResult::Success(data),
                Err(e) => TaskResult::Failed(e.clone()),
            };

            // 发送结果到主线程
            let _ = sender.send((task_id, result)).await;
        });

        // 存储任务信息和句柄
        if let Ok(mut guard) = self.inner.lock() {
            guard.tasks.insert(id, task_info);
            guard.handles.insert(id, handle.abort_handle());
        }

        id
    }

    fn cancel(&mut self, id: TaskId) -> TuiResult<()> {
        if let Ok(mut guard) = self.inner.lock() {
            // 尝试中止任务
            if let Some(handle) = guard.handles.remove(&id) {
                handle.abort();
            }

            // 更新任务状态
            if let Some(info) = guard.tasks.get_mut(&id) {
                info.status = TaskStatus::Cancelled;
            }
        }
        Ok(())
    }

    fn status(&self, id: TaskId) -> Option<TaskStatus> {
        self.inner
            .lock()
            .ok()?
            .tasks
            .get(&id)
            .map(|info| info.status.clone())
    }

    fn poll_completed(&mut self) -> Vec<(TaskId, TaskResult)> {
        let mut results = Vec::new();

        // 非阻塞接收所有已完成的任务
        if let Ok(mut guard) = self.inner.lock() {
            while let Ok((id, result)) = guard.completed_rx.try_recv() {
                // 确定状态更新（需要克隆状态，不是结果）
                let status_update = match &result {
                    TaskResult::Success(_) => TaskStatus::Completed,
                    TaskResult::Failed(e) => TaskStatus::Failed(e.clone()),
                };

                // 检查是否有回调
                let has_callback = guard.tasks.get_mut(&id).and_then(|info| {
                    info.status = status_update;
                    info.callback.take()
                });

                // 如果有回调，执行回调但不返回结果
                if let Some(callback) = has_callback {
                    callback(result);
                } else {
                    // 没有回调，返回结果
                    results.push((id, result));
                }

                // 移除任务句柄
                guard.handles.remove(&id);
            }
        }

        results
    }

    fn active_count(&self) -> usize {
        self.inner
            .lock()
            .ok()
            .map(|guard| {
                guard
                    .tasks
                    .values()
                    .filter(|info| !info.status.is_finished())
                    .count()
            })
            .unwrap_or(0)
    }

    fn wait_all(&mut self) -> TuiResult<()> {
        // 注意：这是一个阻塞操作，应该谨慎使用
        // 在实际 TUI 应用中，应该使用轮询方式
        while self.active_count() > 0 {
            let _ = self.poll_completed();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }
}

// TaskResult 辅助方法
impl TaskResult {
    /// 检查是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // 创建一个简单的异步任务用于测试
    async fn simple_task(value: u32) -> TuiResult<Box<dyn Any + Send>> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(Box::new(value) as Box<dyn Any + Send>)
    }

    async fn failing_task() -> TuiResult<Box<dyn Any + Send>> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Err(TuiError::invalid_state("Task failed"))
    }

    #[test]
    fn test_task_manager_new() {
        let manager = TokioTaskManager::new();
        assert!(manager.is_ok());
        let manager = manager.unwrap();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_submit_task() {
        let mut manager = TokioTaskManager::new().unwrap();

        let id = manager.submit("test", simple_task(42), None);
        assert_eq!(id.value(), 1);

        let status = manager.status(id);
        assert!(status.is_some());
        assert!(status.unwrap().is_running());
    }

    #[test]
    fn test_poll_completed() {
        let mut manager = TokioTaskManager::new().unwrap();

        // 提交一个快速完成的任务
        let id = manager.submit("quick", simple_task(123), None);

        // 等待任务完成
        std::thread::sleep(Duration::from_millis(50));

        // 轮询已完成任务
        let results = manager.poll_completed();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);

        // 检查结果
        match &results[0].1 {
            TaskResult::Success(any) => {
                let value = any.downcast_ref::<u32>();
                assert_eq!(value, Some(&123));
            }
            TaskResult::Failed(_) => panic!("Task should succeed"),
        }
    }

    #[test]
    fn test_failed_task() {
        let mut manager = TokioTaskManager::new().unwrap();

        let id = manager.submit("failing", failing_task(), None);

        // 等待任务完成
        std::thread::sleep(Duration::from_millis(50));

        let results = manager.poll_completed();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);

        // 检查结果是失败
        assert!(results[0].1.is_failed());

        // 检查状态
        let status = manager.status(id);
        assert!(status.is_some());
        assert!(!status.unwrap().is_success());
    }

    #[test]
    fn test_callback() {
        let mut manager = TokioTaskManager::new().unwrap();

        // 使用 Arc<Mutex<bool>> 来跟踪回调是否被调用
        let callback_called = Arc::new(Mutex::new(false));
        let callback_called_clone = callback_called.clone();

        let callback = Box::new(move |result| match result {
            TaskResult::Success(_) => {
                *callback_called_clone.lock().unwrap() = true;
            }
            TaskResult::Failed(_) => {}
        });

        manager.submit("callback_test", simple_task(99), Some(callback));

        // 等待任务完成
        std::thread::sleep(Duration::from_millis(50));

        let results = manager.poll_completed();
        assert_eq!(results.len(), 0); // 有回调时不返回结果

        // 检查回调是否被调用
        std::thread::sleep(Duration::from_millis(10));
        assert!(*callback_called.lock().unwrap());
    }

    #[test]
    fn test_active_count() {
        let mut manager = TokioTaskManager::new().unwrap();

        assert_eq!(manager.active_count(), 0);

        // 提交多个任务
        let id1 = manager.submit("task1", simple_task(1), None);
        let id2 = manager.submit("task2", simple_task(2), None);
        let id3 = manager.submit("task3", simple_task(3), None);

        assert_eq!(manager.active_count(), 3);

        // 等待任务完成
        std::thread::sleep(Duration::from_millis(50));

        let _ = manager.poll_completed();

        // 完成后活跃任务应该为 0
        assert_eq!(manager.active_count(), 0);

        // 检查所有任务状态
        assert!(manager.status(id1).unwrap().is_finished());
        assert!(manager.status(id2).unwrap().is_finished());
        assert!(manager.status(id3).unwrap().is_finished());
    }

    #[test]
    fn test_cancel_task() {
        let mut manager = TokioTaskManager::new().unwrap();

        // 提交一个慢速任务
        let slow_task = async {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            Ok(Box::new(42) as Box<dyn Any + Send>)
        };

        let id = manager.submit("slow", slow_task, None);

        // 立即取消
        assert!(manager.cancel(id).is_ok());

        // 检查状态
        let status = manager.status(id);
        assert!(status.is_some());
        assert!(matches!(status.unwrap(), TaskStatus::Cancelled));
    }

    #[test]
    fn test_task_id() {
        let id = TaskId::new(100);
        assert_eq!(id.value(), 100);
        assert_eq!(format!("{}", id), "TaskId(100)");
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
    }
}
