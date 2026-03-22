//! Tree 组件
//!
//! 用于显示树形结构的组件，支持 Vim 风格导航和展开/折叠功能。

use crate::tui::error::TuiResult;
use crate::tui::models::tree::TreeNode;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// 树形组件
///
/// 支持功能：
/// - Vim 风格导航 (j/k 上下移动, h/l 展开/折叠)
/// - 光标位置管理
/// - 树节点展开/折叠
/// - 深度可视化缩进
pub struct TreeComponent<T: Clone> {
    /// 组件 ID
    id: ComponentId,
    /// 根节点
    root: TreeNode<T>,
    /// 当前选中的节点路径（从根节点到当前节点的路径）
    selected_path: Vec<String>,
    /// 可视化节点列表（已展开的节点的扁平化表示）
    visible_nodes: Vec<TreeNode<T>>,
    /// 当前选中的可见节点索引
    selected_index: usize,
    /// 滚动偏移量（用于滚动查看超出可视区域的内容）
    scroll_offset: usize,
}

impl<T: Clone> TreeComponent<T> {
    /// 创建新的树形组件
    pub fn new(root: TreeNode<T>) -> Self {
        let mut component = Self {
            id: ComponentId::new(0),
            root,
            selected_path: vec![],
            visible_nodes: vec![],
            selected_index: 0,
            scroll_offset: 0,
        };

        // 初始化可视化节点列表
        component.update_visible_nodes();
        component
    }

    /// 设置组件 ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// 获取根节点的引用
    pub fn root(&self) -> &TreeNode<T> {
        &self.root
    }

    /// 更新可视化节点列表（根据展开状态过滤节点）
    fn update_visible_nodes(&mut self) {
        let mut nodes = Vec::new();
        self.collect_visible_nodes_recursive(&self.root, &mut nodes, 0);

        self.visible_nodes = nodes;

        // 更新选中索引以匹配选中路径
        self.update_selected_index_from_path();
    }

    /// 递归收集可见节点
    fn collect_visible_nodes_recursive(
        &self,
        node: &TreeNode<T>,
        nodes: &mut Vec<TreeNode<T>>,
        depth: u8,
    ) {
        // 添加当前节点到列表
        let mut visible_node = node.clone();
        visible_node.level = depth;
        nodes.push(visible_node);

        // 如果节点已展开，则递归添加子节点
        if node.expanded {
            for child in &node.children {
                self.collect_visible_nodes_recursive(child, nodes, depth + 1);
            }
        }
    }

    /// 更新选中索引以匹配当前路径
    fn update_selected_index_from_path(&mut self) {
        if self.selected_path.is_empty() {
            self.selected_index = 0;
            return;
        }

        // 在可见节点中查找与路径匹配的节点
        for (index, node) in self.visible_nodes.iter().enumerate() {
            if node.id == *self.selected_path.last().unwrap_or(&String::new()) {
                self.selected_index = index;
                break;
            }
        }
    }

    /// 获取当前选中的节点（如果存在）
    fn selected_node(&self) -> Option<&TreeNode<T>> {
        if self.selected_index < self.visible_nodes.len() {
            Some(&self.visible_nodes[self.selected_index])
        } else {
            None
        }
    }

    /// 获取当前选中节点的可变引用（如果存在）
    fn selected_node_mut(&mut self) -> Option<&mut TreeNode<T>> {
        if self.selected_index < self.visible_nodes.len() {
            Some(&mut self.visible_nodes[self.selected_index])
        } else {
            None
        }
    }

    /// 移动选择到上一个节点
    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            // 如果已经在顶部，循环到最后一个节点
            if !self.visible_nodes.is_empty() {
                self.selected_index = self.visible_nodes.len() - 1;
            }
        }

        // 更新选中路径
        if let Some(node) = self.selected_node() {
            self.selected_path = self.get_path_to_node(node.id.clone());
        }
    }

    /// 移动选择到下一个节点
    fn move_down(&mut self) {
        if !self.visible_nodes.is_empty() {
            if self.selected_index < self.visible_nodes.len() - 1 {
                self.selected_index += 1;
            } else {
                // 如果已经在底部，循环到第一个节点
                self.selected_index = 0;
            }

            // 更新选中路径
            if let Some(node) = self.selected_node() {
                self.selected_path = self.get_path_to_node(node.id.clone());
            }
        }
    }

    /// 获取到达特定节点的路径
    fn get_path_to_node(&self, target_id: String) -> Vec<String> {
        let mut path = Vec::new();
        self.find_path_recursive(&self.root, &target_id, &mut path);
        path
    }

    /// 递归查找路径
    fn find_path_recursive(
        &self,
        node: &TreeNode<T>,
        target_id: &str,
        path: &mut Vec<String>,
    ) -> bool {
        path.push(node.id.clone());

        if node.id == target_id {
            return true;
        }

        // 如果目标节点在当前节点的子树中，继续递归
        for child in &node.children {
            if (self.node_has_descendant(child, target_id) || child.id == target_id)
                && self.find_path_recursive(child, target_id, path)
            {
                return true;
            }
        }

        path.pop(); // 回溯
        false
    }

    /// 检查节点是否包含指定的后代节点
    fn node_has_descendant(&self, node: &TreeNode<T>, target_id: &str) -> bool {
        if node.id == target_id {
            return true;
        }

        for child in &node.children {
            if self.node_has_descendant(child, target_id) {
                return true;
            }
        }

        false
    }

    /// 切换当前选中节点的展开状态
    fn toggle_expansion(&mut self) {
        if let Some(node) = self.selected_node_mut() {
            if node.is_folder() {
                node.toggle_expanded();

                // 更新原始树的展开状态
                self.update_root_expansion_state();

                // 重新构建可见节点列表
                self.update_visible_nodes();
            }
        }
    }

    /// 更新根节点的展开状态以匹配可见节点
    fn update_root_expansion_state(&mut self) {
        // 遍历可见节点并更新根节点中对应节点的展开状态
        for visible_node in &self.visible_nodes {
            if let Some(root_node) = self.root.find_mut(&visible_node.id) {
                root_node.expanded = visible_node.expanded;
            }
        }
    }

    /// 展开当前选中的节点
    fn expand_current(&mut self) {
        if let Some(node) = self.selected_node_mut() {
            if node.is_folder() {
                node.set_expanded(true);

                // 更新原始树的展开状态
                self.update_root_expansion_state();

                // 重新构建可见节点列表
                self.update_visible_nodes();
            }
        }
    }

    /// 折叠当前选中的节点
    fn collapse_current(&mut self) {
        if let Some(node) = self.selected_node_mut() {
            if node.is_folder() {
                node.set_expanded(false);

                // 更新原始树的展开状态
                self.update_root_expansion_state();

                // 重新构建可见节点列表
                self.update_visible_nodes();
            }
        }
    }

    /// 移动到顶部节点
    fn move_to_top(&mut self) {
        self.selected_index = 0;
        if let Some(node) = self.selected_node() {
            self.selected_path = self.get_path_to_node(node.id.clone());
        }
    }

    /// 移动到底部节点
    fn move_to_bottom(&mut self) {
        if !self.visible_nodes.is_empty() {
            self.selected_index = self.visible_nodes.len() - 1;
            if let Some(node) = self.selected_node() {
                self.selected_path = self.get_path_to_node(node.id.clone());
            }
        }
    }
}

impl<T: Clone> Render for TreeComponent<T> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        // 创建边框块
        let block = Block::default().borders(Borders::ALL).title("Tree View");

        // 渲染边框
        block.render(area, buf);

        // 计算内容区域（减去边框）
        let content_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width - 2,
            height: area.height - 2,
        };

        if content_area.height == 0 {
            return;
        }

        // 渲染节点列表
        let display_start = self.scroll_offset;
        let display_end = std::cmp::min(
            self.visible_nodes.len(),
            display_start + content_area.height as usize,
        );

        for (i, node) in self.visible_nodes[display_start..display_end]
            .iter()
            .enumerate()
        {
            let y = content_area.y + i as u16;
            if y >= content_area.bottom() {
                break;
            }

            // 计算相对于当前显示起始位置的实际索引
            let actual_index = display_start + i;

            // 确定样式（选中状态 vs 普通状态）
            let is_selected = actual_index == self.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            // 准备显示的文本
            let indent = "  ".repeat(node.level as usize); // 缩进表示层级
            let icon = if node.is_folder() {
                if node.expanded {
                    "-" // Use simple character for expanded folders
                } else {
                    "+" // Use simple character for collapsed folders
                }
            } else {
                "•" // Use bullet for leaf nodes
            };

            let node_text = format!("{}{} {}", indent, icon, node.name());

            // 创建文本段落
            let line = Line::from(vec![Span::styled(node_text, style)]);
            let paragraph = Paragraph::new(line);

            // 渲染行
            paragraph.render(
                Rect {
                    x: content_area.x,
                    y,
                    width: content_area.width,
                    height: 1,
                },
                buf,
            );
        }
    }
}

impl<T: Clone> Interactive for TreeComponent<T> {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        // 只处理按键按下事件
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            // Vim 风格导航：j - 向下移动
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                HandleResult::Consumed
            }
            // Vim 风格导航：k - 向上移动
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                HandleResult::Consumed
            }
            // Vim 风格：l - 展开节点或进入节点
            KeyCode::Char('l') | KeyCode::Right => {
                self.expand_current();
                HandleResult::Consumed
            }
            // Vim 风格：h - 折叠节点
            KeyCode::Char('h') | KeyCode::Left => {
                self.collapse_current();
                HandleResult::Consumed
            }
            // Vim 风格：g - 移动到顶部
            KeyCode::Char('g') => {
                self.move_to_top();
                HandleResult::Consumed
            }
            // Vim 风格：G - 移动到底部
            KeyCode::Char('G') => {
                self.move_to_bottom();
                HandleResult::Consumed
            }
            // 空格键也可以用于展开/折叠
            KeyCode::Char(' ') => {
                self.toggle_expansion();
                HandleResult::Consumed
            }
            // Enter 键展开/折叠
            KeyCode::Enter => {
                self.toggle_expansion();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl<T: Clone> Component for TreeComponent<T> {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        // 焦点获得时不需要特殊处理
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        // 焦点失去时不需要特殊处理
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::models::tree::TreeNodeItem;

    fn create_test_node(id: &str, name: &str, level: u8, parent_id: Option<&str>) -> TreeNode<i32> {
        TreeNode {
            id: id.to_string(),
            level,
            item: TreeNodeItem::Folder {
                name: name.to_string(),
                child_count: 0,
            },
            children: Vec::new(),
            expanded: false,
            parent_id: parent_id.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_tree_component_creation() {
        let root = create_test_node("root", "Root", 0, None);
        let component = TreeComponent::new(root);

        assert_eq!(component.id().value(), 0);
        assert!(component.can_focus());
    }

    #[test]
    fn test_vim_navigation_j_k() {
        // 创建一个多层树结构
        let mut root = create_test_node("root", "Root", 0, None);
        let mut child1 = create_test_node("child1", "Child 1", 1, Some("root"));
        child1.expanded = true;
        let grandchild = create_test_node("grandchild", "Grandchild", 2, Some("child1"));
        child1.children.push(grandchild);
        root.children.push(child1);
        root.expanded = true;

        let mut component = TreeComponent::new(root);
        component.update_visible_nodes();

        // 初始选中第一项
        assert_eq!(component.selected_index, 0);

        // 模拟 'j' 键 - 移动到下一项
        component.handle_key(KeyEvent::new(
            KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(component.selected_index, 1);

        // 再次模拟 'j' 键 - 移动到下一项
        component.handle_key(KeyEvent::new(
            KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(component.selected_index, 2);

        // 模拟 'k' 键 - 移动到上一项
        component.handle_key(KeyEvent::new(
            KeyCode::Char('k'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(component.selected_index, 1);
    }

    #[test]
    fn test_vim_expansion_h_l() {
        let mut root = create_test_node("root", "Root", 0, None);
        let child = create_test_node("child", "Child", 1, Some("root"));
        root.children.push(child);

        let mut component = TreeComponent::new(root);

        // 初始状态下没有展开
        assert_eq!(component.root().children.len(), 1);

        // 选中根节点然后尝试展开 ('l')
        component.handle_key(KeyEvent::new(
            KeyCode::Char('l'),
            crossterm::event::KeyModifiers::empty(),
        ));

        // 检查是否正确展开了节点
        if let Some(root_node) = component.root().find("root") {
            assert!(root_node.expanded);
        }
    }

    #[test]
    fn test_expansion_toggle() {
        let mut root = create_test_node("root", "Root", 0, None);
        let child = create_test_node("child", "Child", 1, Some("root"));
        root.children.push(child);

        let mut component = TreeComponent::new(root);

        // 记录初始状态
        let initial_expanded = if let Some(node) = component.root().find("root") {
            node.expanded
        } else {
            false
        };

        // 切换展开状态
        component.toggle_expansion();

        // 检查状态是否改变
        let after_toggle = if let Some(node) = component.root().find("root") {
            node.expanded
        } else {
            false
        };

        assert_ne!(initial_expanded, after_toggle);
    }

    #[test]
    fn test_top_bottom_navigation() {
        let mut root = create_test_node("root", "Root", 0, None);
        let child1 = create_test_node("child1", "Child 1", 1, Some("root"));
        let child2 = create_test_node("child2", "Child 2", 1, Some("root"));
        let child3 = create_test_node("child3", "Child 3", 1, Some("root"));
        root.children.push(child1);
        root.children.push(child2);
        root.children.push(child3);
        root.expanded = true; // 展开以显示所有子节点

        let mut component = TreeComponent::new(root);
        component.update_visible_nodes();

        // 初始选中第一项
        assert_eq!(component.selected_index, 0);

        // 移动到底部 ('G')
        component.move_to_bottom();
        assert_eq!(component.selected_index, 3); // 根节点 + 3 个子节点 = 4，索引为0-3

        // 移动到顶部 ('g')
        component.move_to_top();
        assert_eq!(component.selected_index, 0);
    }
}
