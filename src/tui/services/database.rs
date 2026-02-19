//! TUI 数据库服务适配器

pub struct TuiDatabaseService;

impl TuiDatabaseService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TuiDatabaseService {
    fn default() -> Self {
        Self::new()
    }
}
