//! TUI 剪贴板服务适配器

pub struct TuiClipboardService;

impl TuiClipboardService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TuiClipboardService {
    fn default() -> Self {
        Self::new()
    }
}
