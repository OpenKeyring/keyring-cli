//! Tree Component 示例演示
//!
//! 演示如何使用 TreeComponent 实现 Vim 风格导航和树形结构展示

use keyring_cli::tui::components::TreeComponent;
use keyring_cli::tui::models::tree::{TreeNode, TreeNodeItem};
use keyring_cli::tui::traits::{HandleResult, Render, Interactive};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

/// 创建示例树结构
fn create_sample_tree() -> TreeNode<String> {
    let mut root = TreeNode {
        id: "root".to_string(),
        level: 0,
        item: TreeNodeItem::Folder {
            name: "Root Directory".to_string(),
            child_count: 3,
        },
        children: vec![],
        expanded: true, // 默认展开根节点
        parent_id: None,
    };

    // 添加子节点
    let mut folder1 = TreeNode {
        id: "folder1".to_string(),
        level: 1,
        item: TreeNodeItem::Folder {
            name: "Projects".to_string(),
            child_count: 2,
        },
        children: vec![],
        expanded: false,
        parent_id: Some("root".to_string()),
    };

    // 添加 Projects 下的子项
    let project_a = TreeNode {
        id: "project_a".to_string(),
        level: 2,
        item: TreeNodeItem::Data("Project A".to_string()),
        children: vec![],
        expanded: false,
        parent_id: Some("folder1".to_string()),
    };

    let project_b = TreeNode {
        id: "project_b".to_string(),
        level: 2,
        item: TreeNodeItem::Data("Project B".to_string()),
        children: vec![],
        expanded: false,
        parent_id: Some("folder1".to_string()),
    };

    folder1.children = vec![project_a, project_b];

    let mut folder2 = TreeNode {
        id: "folder2".to_string(),
        level: 1,
        item: TreeNodeItem::Folder {
            name: "Documents".to_string(),
            child_count: 2,
        },
        children: vec![],
        expanded: false,
        parent_id: Some("root".to_string()),
    };

    // 添加 Documents 下的子项
    let doc1 = TreeNode {
        id: "doc1".to_string(),
        level: 2,
        item: TreeNodeItem::Data("Document 1".to_string()),
        children: vec![],
        expanded: false,
        parent_id: Some("folder2".to_string()),
    };

    let doc2 = TreeNode {
        id: "doc2".to_string(),
        level: 2,
        item: TreeNodeItem::Data("Document 2".to_string()),
        children: vec![],
        expanded: false,
        parent_id: Some("folder2".to_string()),
    };

    folder2.children = vec![doc1, doc2];

    let settings_node = TreeNode {
        id: "settings".to_string(),
        level: 1,
        item: TreeNodeItem::Data("Settings".to_string()),
        children: vec![],
        expanded: false,
        parent_id: Some("root".to_string()),
    };

    root.children = vec![folder1, folder2, settings_node];

    root
}

/// 运行 TreeComponent 演示
fn run_demo() -> io::Result<()> {
    // 创建示例树
    let sample_tree = create_sample_tree();

    // 创建 TreeComponent 实例
    let mut tree_component = TreeComponent::new(sample_tree);

    // 启用原模式
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 主循环
    let mut running = true;
    while running {
        // 渲染界面
        terminal.draw(|f| {
            let size = f.size();

            // 将 TreeComponent 渲染到全屏区域
            tree_component.render(size, f.buffer_mut());
        })?;

        // 处理事件
        if event::poll(std::time::Duration::from_millis(16))? {
            // Filter to only handle Press events to avoid duplicate key handling on Windows
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                    // 退出程序
                    KeyCode::Char('q') => running = false,
                    KeyCode::Esc => running = false,

                    // 将键盘事件传递给 TreeComponent
                    _ => {
                        let result = tree_component.handle_key(key_event);

                        // 根据需要处理结果
                        match result {
                            HandleResult::Consumed => {
                                // 事件已被消费，无需额外处理
                            }
                            HandleResult::Ignored => {
                                // 事件未被处理，可以做其他事情
                            }
                            HandleResult::NeedsRender => {
                                // 组件请求重新渲染，我们会在下次循环中渲染
                            }
                            HandleResult::Action(_) => {
                                // 执行特定动作，这里不做处理
                            }
                        }
                    }
                }
            }
        }
    }

    // 恢复终端状态
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> io::Result<()> {
    // Install panic hook FIRST to ensure terminal recovery on panic
    keyring_cli::tui::panic_hook::install_panic_hook();

    println!("Tree Component 演示程序");
    println!("使用 Vim 风格导航:");
    println!("  - j/k 或 ↑/↓ : 上下移动");
    println!("  - l/→ : 展开节点");
    println!("  - h/← : 折叠节点");
    println!("  - Space/Enter : 切换展开状态");
    println!("  - g : 移动到顶部");
    println!("  - G : 移动到底部");
    println!("  - q/Esc : 退出");
    println!("\n按任意键开始...");

    // 等待按键
    event::read()?;

    run_demo()
}