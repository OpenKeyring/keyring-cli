//! 布局系统 Trait 定义
//!
//! 定义组件布局计算相关的接口，包括布局约束、布局结果和布局算法。

use std::collections::HashMap;
use ratatui::layout::Rect;
use crate::tui::traits::ComponentId;

/// 布局约束
///
/// 定义组件在布局时的尺寸限制。
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutConstraints {
    /// 最小宽度
    pub min_width: u16,
    /// 最大宽度（None 表示无限制）
    pub max_width: Option<u16>,
    /// 最小高度
    pub min_height: u16,
    /// 最大高度（None 表示无限制）
    pub max_height: Option<u16>,
    /// 是否希望填充可用空间
    pub fill_available: bool,
    /// 权重（用于分配剩余空间）
    pub flex_weight: Option<u32>,
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self {
            min_width: 0,
            max_width: None,
            min_height: 0,
            max_height: None,
            fill_available: false,
            flex_weight: None,
        }
    }
}

impl LayoutConstraints {
    /// 创建固定尺寸约束
    #[must_use]
    pub const fn fixed(width: u16, height: u16) -> Self {
        Self {
            min_width: width,
            max_width: Some(width),
            min_height: height,
            max_height: Some(height),
            fill_available: false,
            flex_weight: None,
        }
    }

    /// 创建最小尺寸约束
    #[must_use]
    pub const fn min(width: u16, height: u16) -> Self {
        Self {
            min_width: width,
            max_width: None,
            min_height: height,
            max_height: None,
            fill_available: true,
            flex_weight: None,
        }
    }

    /// 创建弹性约束（指定权重）
    #[must_use]
    pub const fn flex(weight: u32) -> Self {
        Self {
            min_width: 0,
            max_width: None,
            min_height: 0,
            max_height: None,
            fill_available: true,
            flex_weight: Some(weight),
        }
    }

    /// 检查尺寸是否满足约束
    #[must_use]
    pub fn satisfies(&self, width: u16, height: u16) -> bool {
        let width_ok = width >= self.min_width
            && self.max_width.map_or(true, |max| width <= max);
        let height_ok = height >= self.min_height
            && self.max_height.map_or(true, |max| height <= max);
        width_ok && height_ok
    }

    /// 限制尺寸在约束范围内
    #[must_use]
    pub fn constrain(&self, mut width: u16, mut height: u16) -> (u16, u16) {
        width = width.max(self.min_width);
        if let Some(max) = self.max_width {
            width = width.min(max);
        }
        height = height.max(self.min_height);
        if let Some(max) = self.max_height {
            height = height.min(max);
        }
        (width, height)
    }
}

/// 布局结果
///
/// 包含所有子组件的布局位置和焦点顺序。
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// 组件 ID 到布局区域的映射
    pub areas: HashMap<ComponentId, Rect>,
    /// 焦点导航顺序
    pub focus_order: Vec<ComponentId>,
}

impl LayoutResult {
    /// 创建空的布局结果
    #[must_use]
    pub fn new() -> Self {
        Self {
            areas: HashMap::new(),
            focus_order: Vec::new(),
        }
    }

    /// 添加组件布局
    pub fn insert(&mut self, id: ComponentId, area: Rect) {
        self.areas.insert(id, area);
    }

    /// 获取组件布局区域
    #[must_use]
    pub fn get(&self, id: &ComponentId) -> Option<&Rect> {
        self.areas.get(id)
    }

    /// 设置焦点顺序
    pub fn set_focus_order(&mut self, order: Vec<ComponentId>) {
        self.focus_order = order;
    }

    /// 获取焦点顺序中的下一个组件
    #[must_use]
    pub fn next_focus(&self, current: &ComponentId) -> Option<&ComponentId> {
        let pos = self.focus_order.iter().position(|id| id == current)?;
        let next = (pos + 1) % self.focus_order.len();
        self.focus_order.get(next)
    }

    /// 获取焦点顺序中的上一个组件
    #[must_use]
    pub fn prev_focus(&self, current: &ComponentId) -> Option<&ComponentId> {
        let pos = self.focus_order.iter().position(|id| id == current)?;
        let prev = if pos == 0 {
            self.focus_order.len() - 1
        } else {
            pos - 1
        };
        self.focus_order.get(prev)
    }

    /// 检查布局是否包含指定组件
    #[must_use]
    pub fn contains(&self, id: &ComponentId) -> bool {
        self.areas.contains_key(id)
    }
}

impl Default for LayoutResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 布局方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    /// 水平布局（从左到右）
    Horizontal,
    /// 垂直布局（从上到下）
    Vertical,
}

/// 布局对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// 开始位置对齐
    Start,
    /// 居中对齐
    Center,
    /// 结束位置对齐
    End,
    /// 两端对齐（组件之间均匀分布）
    SpaceBetween,
    /// 环绕对齐（组件周围均匀分布）
    SpaceAround,
}

/// 布局 Trait
///
/// 定义组件的布局计算接口。
pub trait Layout: Send + Sync {
    /// 计算布局
    ///
    /// 根据给定的可用区域计算所有子组件的布局位置。
    fn calculate(&self, available: Rect) -> Result<LayoutResult, LayoutError>;

    /// 更新布局约束
    fn update_constraints(&mut self, constraints: LayoutConstraints);

    /// 获取当前约束
    fn get_constraints(&self) -> &LayoutConstraints;

    /// 获取首选尺寸
    ///
    /// 返回组件希望拥有的尺寸（不考虑可用空间限制）。
    #[must_use]
    fn preferred_size(&self) -> (u16, u16) {
        (0, 0)
    }

    /// 获取最小尺寸
    ///
    /// 返回组件能正常工作的最小尺寸。
    #[must_use]
    fn min_size(&self) -> (u16, u16) {
        (self.get_constraints().min_width, self.get_constraints().min_height)
    }
}

/// 布局错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// 可用空间不足以容纳最小尺寸
    InsufficientSpace { required: (u16, u16), available: (u16, u16) },
    /// 无效的布局约束
    InvalidConstraints(String),
    /// 组件不存在
    ComponentNotFound(ComponentId),
    /// 布局计算失败
    CalculationFailed(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientSpace { required, available } => write!(
                f,
                "空间不足: 需要 {}x{}, 可用 {}x{}",
                required.0, required.1, available.0, available.1
            ),
            Self::InvalidConstraints(msg) => write!(f, "无效的布局约束: {}", msg),
            Self::ComponentNotFound(id) => write!(f, "组件不存在: {}", id),
            Self::CalculationFailed(msg) => write!(f, "布局计算失败: {}", msg),
        }
    }
}

impl std::error::Error for LayoutError {}

/// 容器布局 Trait
///
/// 扩展 Layout，支持管理子组件的布局。
pub trait ContainerLayout: Layout {
    /// 添加子组件
    fn add_child(&mut self, id: ComponentId, constraints: LayoutConstraints);

    /// 移除子组件
    fn remove_child(&mut self, id: &ComponentId) -> Option<LayoutConstraints>;

    /// 更新子组件约束
    fn update_child_constraints(&mut self, id: ComponentId, constraints: LayoutConstraints);

    /// 获取子组件数量
    #[must_use]
    fn child_count(&self) -> usize;

    /// 获取子组件列表
    #[must_use]
    fn children(&self) -> Vec<ComponentId>;
}

/// 布局生成器
///
/// 用于构建复杂布局的辅助结构。
pub struct LayoutBuilder {
    constraints: LayoutConstraints,
    children: Vec<(ComponentId, LayoutConstraints)>,
    direction: LayoutDirection,
    alignment: Alignment,
    gap: u16,
}

impl LayoutBuilder {
    /// 创建新的布局生成器
    #[must_use]
    pub fn new() -> Self {
        Self {
            constraints: LayoutConstraints::default(),
            children: Vec::new(),
            direction: LayoutDirection::Vertical,
            alignment: Alignment::Start,
            gap: 0,
        }
    }

    /// 设置约束
    #[must_use]
    pub const fn constraints(mut self, constraints: LayoutConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// 添加子组件
    #[must_use]
    pub fn child(mut self, id: ComponentId, constraints: LayoutConstraints) -> Self {
        self.children.push((id, constraints));
        self
    }

    /// 设置布局方向
    #[must_use]
    pub const fn direction(mut self, direction: LayoutDirection) -> Self {
        self.direction = direction;
        self
    }

    /// 设置对齐方式
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// 设置组件间距
    #[must_use]
    pub const fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// 构建布局
    ///
    /// 根据配置计算布局。
    pub fn build(&self, available: Rect) -> LayoutResult {
        let mut result = LayoutResult::new();
        let mut focus_order = Vec::new();

        let count = self.children.len();
        if count == 0 {
            return result;
        }

        let total_gap = if count > 0 {
            self.gap * (count - 1) as u16
        } else {
            0
        };

        match self.direction {
            LayoutDirection::Horizontal => {
                let available_width = available.width.saturating_sub(total_gap);
                let mut x = available.x;

                for (id, constraints) in &self.children {
                    let width = if let Some(weight) = constraints.flex_weight {
                        // 按权重分配
                        let total_weight: u32 = self.children
                            .iter()
                            .filter_map(|(_, c)| c.flex_weight)
                            .sum();
                        let share = (available_width * weight as u16 / total_weight as u16).max(constraints.min_width);
                        constraints.max_width.map_or(share, |max| share.min(max))
                    } else if constraints.fill_available {
                        // 平均分配
                        let avg = available_width / count as u16;
                        avg.max(constraints.min_width)
                    } else {
                        // 使用首选尺寸或最小尺寸
                        constraints.max_width.unwrap_or(constraints.min_width)
                    };

                    let height = if constraints.fill_available {
                        available.height
                    } else {
                        constraints.max_height.unwrap_or(constraints.min_height.min(available.height))
                    };

                    let area = Rect {
                        x,
                        y: available.y,
                        width: width.min(available.width),
                        height,
                    };

                    result.insert(*id, area);
                    focus_order.push(*id);
                    x = x.saturating_add(width).saturating_add(self.gap);
                }
            }
            LayoutDirection::Vertical => {
                let available_height = available.height.saturating_sub(total_gap);
                let mut y = available.y;

                for (id, constraints) in &self.children {
                    let height = if let Some(weight) = constraints.flex_weight {
                        let total_weight: u32 = self.children
                            .iter()
                            .filter_map(|(_, c)| c.flex_weight)
                            .sum();
                        let share = (available_height * weight as u16 / total_weight as u16).max(constraints.min_height);
                        constraints.max_height.map_or(share, |max| share.min(max))
                    } else if constraints.fill_available {
                        let avg = available_height / count as u16;
                        avg.max(constraints.min_height)
                    } else {
                        constraints.max_height.unwrap_or(constraints.min_height)
                    };

                    let width = if constraints.fill_available {
                        available.width
                    } else {
                        constraints.max_width.unwrap_or(constraints.min_width.min(available.width))
                    };

                    let area = Rect {
                        x: available.x,
                        y,
                        width,
                        height: height.min(available.height),
                    };

                    result.insert(*id, area);
                    focus_order.push(*id);
                    y = y.saturating_add(height).saturating_add(self.gap);
                }
            }
        }

        result.set_focus_order(focus_order);
        result
    }
}

impl Default for LayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constraints_fixed() {
        let constraints = LayoutConstraints::fixed(10, 20);
        assert!(constraints.satisfies(10, 20));
        assert!(!constraints.satisfies(9, 20));
        assert!(!constraints.satisfies(10, 19));

        let (w, h) = constraints.constrain(100, 100);
        assert_eq!((w, h), (10, 20));
    }

    #[test]
    fn test_layout_constraints_min() {
        let constraints = LayoutConstraints::min(10, 20);
        assert!(constraints.satisfies(10, 20));
        assert!(constraints.satisfies(100, 100));
        assert!(!constraints.satisfies(9, 20));

        let (w, h) = constraints.constrain(5, 5);
        assert_eq!((w, h), (10, 20));
    }

    #[test]
    fn test_layout_result_navigation() {
        let mut result = LayoutResult::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);

        result.insert(id1, Rect::default());
        result.insert(id2, Rect::default());
        result.insert(id3, Rect::default());
        result.set_focus_order(vec![id1, id2, id3]);

        assert_eq!(result.next_focus(&id1), Some(&id2));
        assert_eq!(result.next_focus(&id3), Some(&id1)); // 循环
        assert_eq!(result.prev_focus(&id2), Some(&id1));
        assert_eq!(result.prev_focus(&id1), Some(&id3)); // 循环
    }

    #[test]
    fn test_layout_builder_horizontal() {
        let builder = LayoutBuilder::new()
            .direction(LayoutDirection::Horizontal)
            .child(ComponentId::new(1), LayoutConstraints::fixed(10, 20))
            .child(ComponentId::new(2), LayoutConstraints::fixed(15, 20))
            .gap(2);

        let result = builder.build(Rect { x: 0, y: 0, width: 50, height: 20 });

        assert_eq!(result.get(&ComponentId::new(1)), Some(&Rect { x: 0, y: 0, width: 10, height: 20 }));
        assert_eq!(result.get(&ComponentId::new(2)), Some(&Rect { x: 12, y: 0, width: 15, height: 20 }));
    }

    #[test]
    fn test_layout_builder_vertical() {
        let builder = LayoutBuilder::new()
            .direction(LayoutDirection::Vertical)
            .child(ComponentId::new(1), LayoutConstraints::fixed(20, 10))
            .child(ComponentId::new(2), LayoutConstraints::fixed(20, 15))
            .gap(2);

        let result = builder.build(Rect { x: 0, y: 0, width: 20, height: 50 });

        assert_eq!(result.get(&ComponentId::new(1)), Some(&Rect { x: 0, y: 0, width: 20, height: 10 }));
        assert_eq!(result.get(&ComponentId::new(2)), Some(&Rect { x: 0, y: 12, width: 20, height: 15 }));
    }
}
