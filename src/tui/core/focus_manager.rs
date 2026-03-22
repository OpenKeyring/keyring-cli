//! 默认焦点管理器实现
//!
//! 提供组件焦点导航、状态管理和顺序管理的功能。

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::traits::{ComponentId, FocusManager, FocusState};
use std::collections::VecDeque;

/// 默认焦点管理器
///
/// 维护可聚焦组件的有序列表，支持前后导航和循环。
#[derive(Debug)]
pub struct DefaultFocusManager {
    /// 当前聚焦的组件
    current: Option<ComponentId>,
    /// 当前状态
    state: FocusState,
    /// 可聚焦组件的导航顺序
    focus_order: VecDeque<ComponentId>,
    /// 组件到其在焦点顺序中位置的映射
    position_map: std::collections::HashMap<ComponentId, usize>,
}

impl Default for DefaultFocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultFocusManager {
    /// 创建新的焦点管理器
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: None,
            state: FocusState::Focusable,
            focus_order: VecDeque::new(),
            position_map: std::collections::HashMap::new(),
        }
    }

    /// 注册可聚焦组件
    ///
    /// 将组件添加到焦点导航顺序的末尾。
    pub fn register(&mut self, id: ComponentId) {
        if !self.position_map.contains_key(&id) {
            let pos = self.focus_order.len();
            self.focus_order.push_back(id);
            self.position_map.insert(id, pos);
        }
    }

    /// 批量注册可聚焦组件
    ///
    /// 按给定顺序设置焦点导航顺序。
    pub fn register_all(&mut self, ids: impl IntoIterator<Item = ComponentId>) {
        self.clear();
        for id in ids {
            self.register(id);
        }
    }

    /// 注销组件
    ///
    /// 从焦点导航顺序中移除组件。
    pub fn unregister(&mut self, id: &ComponentId) -> bool {
        if let Some(&pos) = self.position_map.get(id) {
            self.focus_order.remove(pos);
            // 更新位置映射
            for (i, &component_id) in self.focus_order.iter().enumerate() {
                if i >= pos {
                    self.position_map.insert(component_id, i);
                }
            }
            self.position_map.remove(id);

            // 如果当前焦点是被移除的组件，清除焦点
            if self.current == Some(*id) {
                self.clear_focus();
            }
            true
        } else {
            false
        }
    }

    /// 清除所有注册的组件
    pub fn clear(&mut self) {
        self.clear_focus();
        self.focus_order.clear();
        self.position_map.clear();
    }

    /// 获取所有已注册的组件
    #[must_use]
    pub fn registered_components(&self) -> Vec<ComponentId> {
        self.focus_order.iter().copied().collect()
    }

    /// 获取已注册组件数量
    #[must_use]
    pub fn len(&self) -> usize {
        self.focus_order.len()
    }

    /// 是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.focus_order.is_empty()
    }

    /// 检查组件是否已注册
    #[must_use]
    pub fn contains(&self, id: &ComponentId) -> bool {
        self.position_map.contains_key(id)
    }

    /// 设置焦点导航顺序
    pub fn set_focus_order(&mut self, order: Vec<ComponentId>) {
        self.focus_order = order.into_iter().collect();
        self.position_map = self
            .focus_order
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        // 如果当前焦点不在新的顺序中，清除它
        if let Some(current) = self.current {
            if !self.position_map.contains_key(&current) {
                self.current = None;
                self.state = FocusState::Focusable;
            }
        }
    }

    /// 查找下一个可聚焦的组件
    fn find_next(&self) -> Option<ComponentId> {
        if self.focus_order.is_empty() {
            return None;
        }

        match self.current {
            None => self.focus_order.front().copied(),
            Some(current) => {
                if let Some(&pos) = self.position_map.get(&current) {
                    let next_pos = (pos + 1) % self.focus_order.len();
                    self.focus_order.get(next_pos).copied()
                } else {
                    self.focus_order.front().copied()
                }
            }
        }
    }

    /// 查找上一个可聚焦的组件
    fn find_prev(&self) -> Option<ComponentId> {
        if self.focus_order.is_empty() {
            return None;
        }

        match self.current {
            None => self.focus_order.back().copied(),
            Some(current) => {
                if let Some(&pos) = self.position_map.get(&current) {
                    let prev_pos = if pos == 0 {
                        self.focus_order.len() - 1
                    } else {
                        pos - 1
                    };
                    self.focus_order.get(prev_pos).copied()
                } else {
                    self.focus_order.back().copied()
                }
            }
        }
    }
}

impl FocusManager for DefaultFocusManager {
    fn current_focus(&self) -> Option<ComponentId> {
        self.current
    }

    fn set_focus(&mut self, id: ComponentId) -> TuiResult<()> {
        // 检查组件是否已注册
        if !self.position_map.contains_key(&id) {
            return Err(TuiError::component_not_found(id));
        }

        self.current = Some(id);
        self.state = FocusState::Focused;
        Ok(())
    }

    fn focus_next(&mut self) -> TuiResult<()> {
        let next_id = self.find_next();
        if let Some(id) = next_id {
            self.current = Some(id);
            self.state = FocusState::Focused;
            Ok(())
        } else {
            // 没有可聚焦的组件
            Ok(())
        }
    }

    fn focus_prev(&mut self) -> TuiResult<()> {
        let prev_id = self.find_prev();
        if let Some(id) = prev_id {
            self.current = Some(id);
            self.state = FocusState::Focused;
            Ok(())
        } else {
            // 没有可聚焦的组件
            Ok(())
        }
    }

    fn clear_focus(&mut self) {
        self.current = None;
        self.state = FocusState::Focusable;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_new() {
        let manager = DefaultFocusManager::new();
        assert!(manager.is_empty());
        assert!(manager.current_focus().is_none());
    }

    #[test]
    fn test_register_single() {
        let mut manager = DefaultFocusManager::new();
        let id = ComponentId::new(1);
        manager.register(id);
        assert_eq!(manager.len(), 1);
        assert!(manager.contains(&id));
    }

    #[test]
    fn test_register_multiple() {
        let mut manager = DefaultFocusManager::new();
        let ids = vec![
            ComponentId::new(1),
            ComponentId::new(2),
            ComponentId::new(3),
        ];
        manager.register_all(ids.clone());
        assert_eq!(manager.len(), 3);
        assert_eq!(manager.registered_components(), ids);
    }

    #[test]
    fn test_set_focus() {
        let mut manager = DefaultFocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        manager.register(id1);
        manager.register(id2);

        assert!(manager.set_focus(id1).is_ok());
        assert_eq!(manager.current_focus(), Some(id1));

        // 设置到未注册的组件应该失败
        let unknown = ComponentId::new(999);
        assert!(manager.set_focus(unknown).is_err());
    }

    #[test]
    fn test_focus_next() {
        let mut manager = DefaultFocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);
        manager.register_all(vec![id1, id2, id3]);

        // 从无焦点开始，应该聚焦到第一个
        assert!(manager.focus_next().is_ok());
        assert_eq!(manager.current_focus(), Some(id1));

        // 移动到下一个
        assert!(manager.focus_next().is_ok());
        assert_eq!(manager.current_focus(), Some(id2));

        // 移动到下一个
        assert!(manager.focus_next().is_ok());
        assert_eq!(manager.current_focus(), Some(id3));

        // 循环回到第一个
        assert!(manager.focus_next().is_ok());
        assert_eq!(manager.current_focus(), Some(id1));
    }

    #[test]
    fn test_focus_prev() {
        let mut manager = DefaultFocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);
        manager.register_all(vec![id1, id2, id3]);

        // 从无焦点开始，应该聚焦到最后一个
        assert!(manager.focus_prev().is_ok());
        assert_eq!(manager.current_focus(), Some(id3));

        // 移动到上一个
        assert!(manager.focus_prev().is_ok());
        assert_eq!(manager.current_focus(), Some(id2));

        // 移动到上一个
        assert!(manager.focus_prev().is_ok());
        assert_eq!(manager.current_focus(), Some(id1));

        // 循环回到最后一个
        assert!(manager.focus_prev().is_ok());
        assert_eq!(manager.current_focus(), Some(id3));
    }

    #[test]
    fn test_clear_focus() {
        let mut manager = DefaultFocusManager::new();
        let id = ComponentId::new(1);
        manager.register(id);
        manager.set_focus(id).unwrap();

        manager.clear_focus();
        assert!(manager.current_focus().is_none());
    }

    #[test]
    fn test_unregister() {
        let mut manager = DefaultFocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);
        manager.register_all(vec![id1, id2, id3]);

        // 注销中间的组件
        assert!(manager.unregister(&id2));
        assert_eq!(manager.len(), 2);
        assert!(!manager.contains(&id2));

        // 验证顺序已更新
        assert_eq!(manager.registered_components(), vec![id1, id3]);
    }

    #[test]
    fn test_unregister_with_focus() {
        let mut manager = DefaultFocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        manager.register_all(vec![id1, id2]);

        // 设置焦点到 id1
        manager.set_focus(id1).unwrap();
        assert_eq!(manager.current_focus(), Some(id1));

        // 注销 id1，焦点应该被清除
        manager.unregister(&id1);
        assert!(manager.current_focus().is_none());
    }

    #[test]
    fn test_set_focus_order() {
        let mut manager = DefaultFocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);

        // 设置初始顺序
        manager.register_all(vec![id1, id2, id3]);
        manager.set_focus(id1).unwrap();

        // 更改顺序
        manager.set_focus_order(vec![id3, id2, id1]);
        assert_eq!(manager.registered_components(), vec![id3, id2, id1]);

        // 焦点应该保留（因为 id1 仍在顺序中）
        assert_eq!(manager.current_focus(), Some(id1));

        // next 应该按照新顺序工作
        manager.focus_next().unwrap();
        assert_eq!(manager.current_focus(), Some(id3)); // 循环到第一个
    }

    #[test]
    fn test_clear() {
        let mut manager = DefaultFocusManager::new();
        let id = ComponentId::new(1);
        manager.register(id);
        manager.set_focus(id).unwrap();

        manager.clear();
        assert!(manager.is_empty());
        assert!(manager.current_focus().is_none());
    }
}
