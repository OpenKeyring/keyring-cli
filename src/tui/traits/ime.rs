//! 输入法/组合键处理 trait 定义
//!
//! 定义 TUI 输入法状态感知和组合输入处理的接口。

use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

// ============================================================================
// 输入法状态
// ============================================================================

/// 输入法状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeState {
    /// 无输入法活动
    None,
    /// 正在组合输入（拼音输入中）
    Composing { preedit: String },
    /// 组合完成（待提交）
    Committed { text: String },
}

impl Default for ImeState {
    fn default() -> Self {
        Self::None
    }
}

impl ImeState {
    /// 是否正在组合输入
    #[must_use]
    pub fn is_composing(&self) -> bool {
        matches!(self, Self::Composing { .. })
    }

    /// 是否应该忽略快捷键
    #[must_use]
    pub fn should_ignore_shortcuts(&self) -> bool {
        self.is_composing()
    }

    /// 获取组合文本
    #[must_use]
    pub fn preedit(&self) -> &str {
        match self {
            Self::Composing { preedit } => preedit,
            _ => "",
        }
    }
}

// ============================================================================
// 输入模式
// ============================================================================

/// 输入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImeMode {
    /// 直接输入
    #[default]
    Direct,
    /// 组合输入
    Composing,
}

// ============================================================================
// 组合状态
// ============================================================================

/// 组合状态
#[derive(Debug, Clone, Default)]
pub struct CompositionState {
    /// 组合文本
    pub text: String,
    /// 光标位置
    pub cursor: usize,
    /// 候选词索引
    pub candidate_index: usize,
}

impl CompositionState {
    /// 创建新的组合状态
    #[must_use]
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            candidate_index: 0,
        }
    }

    /// 是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// 添加字符
    pub fn push(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += 1;
    }

    /// 删除前一个字符
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }
}

// ============================================================================
// 输入法服务 Trait
// ============================================================================

/// 输入法服务 trait
pub trait ImeService: Send + Sync {
    /// 获取当前输入模式
    fn mode(&self) -> ImeMode;

    /// 设置输入模式
    fn set_mode(&mut self, mode: ImeMode);

    /// 获取组合状态
    fn composition(&self) -> &CompositionState;

    /// 开始组合输入
    fn start_composition(&mut self);

    /// 结束组合输入
    fn end_composition(&mut self) -> Option<String>;

    /// 取消组合输入
    fn cancel_composition(&mut self);

    /// 处理按键事件
    fn handle_key(&mut self, key: KeyEvent) -> ImeHandleResult;

    /// 是否正在组合输入
    fn is_composing(&self) -> bool;
}

/// 输入法处理结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeHandleResult {
    /// 事件被处理
    Handled,
    /// 事件未被处理
    NotHandled,
    /// 需要提交文本
    Commit(String),
    /// 组合状态已更新
    CompositionUpdated,
}

// ============================================================================
// 输入法感知组件 Trait
// ============================================================================

/// 输入法感知组件 trait
pub trait ImeAware {
    /// 处理输入法状态变化
    fn handle_ime(&mut self, state: &ImeState);

    /// 获取当前输入法状态
    fn ime_state(&self) -> &ImeState;

    /// 设置输入法状态
    fn set_ime_state(&mut self, state: ImeState);
}

// ============================================================================
// 输入法事件检测器
// ============================================================================

/// 输入法事件检测器
pub struct ImeDetector {
    state: ImeState,
    /// 上一次按键（用于检测组合）
    last_key: Option<KeyEvent>,
}

impl ImeDetector {
    /// 创建新的检测器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: ImeState::None,
            last_key: None,
        }
    }

    /// 检测输入法状态
    ///
    /// Crossterm 在终端中的 IME 支持有限，
    /// 主要通过以下方式判断：
    /// 1. KeyEventKind::Repeat 可能表示组合输入
    /// 2. KeyEventState 包含 IME 相关标志（如果终端支持）
    pub fn detect(&mut self, key: KeyEvent) -> &ImeState {
        // 检查是否是普通 ASCII 字符
        if let crossterm::event::KeyCode::Char(c) = key.code {
            if c.is_ascii() {
                // ASCII 字符，清除组合状态
                self.state = ImeState::None;
                return &self.state;
            }
        }

        // 非 ASCII 字符可能是 IME 输入
        match key.kind {
            KeyEventKind::Press => {
                // 检查是否有 IME 标志（终端支持时）
                if key.state.contains(KeyEventState::empty()) {
                    // 终端 IME 通常直接发送最终结果
                }
            }
            KeyEventKind::Repeat => {
                // 可能是组合输入中
            }
            KeyEventKind::Release => {}
        }

        self.last_key = Some(key);
        &self.state
    }

    /// 获取当前状态
    #[must_use]
    pub const fn state(&self) -> &ImeState {
        &self.state
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.state = ImeState::None;
        self.last_key = None;
    }
}

impl Default for ImeDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 输入法感知的事件分发（扩展 Trait）
// ============================================================================

/// 输入法感知的事件处理器扩展
pub trait ImeAwareDispatcher: Send + Sync {
    /// 分发事件（带 IME 检测）
    fn dispatch_with_ime(
        &mut self,
        event: crossterm::event::Event,
        ime_state: &ImeState,
    ) -> ImeHandleResult;

    /// 仅分发到焦点组件
    fn dispatch_to_focused(&mut self, event: crossterm::event::Event) -> ImeHandleResult;
}
