//! Provider Configuration Screen
//!
//! TUI screen for configuring cloud provider-specific settings.

use crate::cloud::CloudProvider;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

/// A single configuration field
#[derive(Debug, Clone)]
pub struct ConfigField {
    /// Field label (e.g., "WebDAV URL")
    pub label: String,
    /// Current field value
    pub value: String,
    /// Whether this is a password field (masked display)
    pub is_password: bool,
    /// Whether this field currently has focus
    pub is_focused: bool,
}

impl ConfigField {
    /// Creates a new configuration field
    pub fn new(label: &str, is_password: bool) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            is_password,
            is_focused: false,
        }
    }
}

/// Provider configuration data
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Cloud provider type
    pub provider: CloudProvider,
    /// Configuration values keyed by field name
    pub values: HashMap<String, String>,
}

impl ProviderConfig {
    /// Creates a new provider configuration
    pub fn new(provider: CloudProvider) -> Self {
        Self {
            provider,
            values: HashMap::new(),
        }
    }

    /// Sets a configuration value
    pub fn set(&mut self, key: &str, value: String) {
        self.values.insert(key.to_string(), value);
    }

    /// Gets a configuration value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
}

/// Provider configuration screen
#[derive(Debug, Clone)]
pub struct ProviderConfigScreen {
    /// Cloud provider being configured
    provider: CloudProvider,
    /// Configuration fields
    fields: Vec<ConfigField>,
    /// Currently focused field index
    focused_index: usize,
}

impl ProviderConfigScreen {
    /// Creates a new provider configuration screen
    pub fn new(provider: CloudProvider) -> Self {
        let fields = match provider {
            CloudProvider::ICloud => vec![ConfigField::new("iCloud 路径 (Path)", false)],
            CloudProvider::Dropbox => vec![ConfigField::new("Access Token", true)],
            CloudProvider::GDrive => vec![ConfigField::new("Access Token", true)],
            CloudProvider::OneDrive => vec![ConfigField::new("Access Token", true)],
            CloudProvider::WebDAV => vec![
                ConfigField::new("WebDAV URL", false),
                ConfigField::new("用户名", false),
                ConfigField::new("密码", true),
            ],
            CloudProvider::SFTP => vec![
                ConfigField::new("主机", false),
                ConfigField::new("端口", false),
                ConfigField::new("用户名", false),
                ConfigField::new("密码", true),
                ConfigField::new("根路径 (Root)", false),
            ],
            CloudProvider::AliyunDrive => {
                vec![ConfigField::new("Access Token / Refresh Token", true)]
            }
            CloudProvider::AliyunOSS => vec![
                ConfigField::new("Endpoint", false),
                ConfigField::new("Bucket", false),
                ConfigField::new("Access Key ID", false),
                ConfigField::new("Access Key Secret", true),
            ],
            CloudProvider::TencentCOS => vec![
                ConfigField::new("Secret ID", false),
                ConfigField::new("Secret Key", true),
                ConfigField::new("区域 (Region)", false),
                ConfigField::new("Bucket", false),
            ],
            CloudProvider::HuaweiOBS => vec![
                ConfigField::new("Endpoint", false),
                ConfigField::new("Bucket", false),
                ConfigField::new("Access Key ID", false),
                ConfigField::new("Secret Access Key", true),
            ],
            CloudProvider::UpYun => vec![
                ConfigField::new("Bucket", false),
                ConfigField::new("Operator", false),
                ConfigField::new("密码", true),
            ],
        };

        let focused_index = 0;

        Self {
            provider,
            fields,
            focused_index,
        }
    }

    /// Returns the list of configuration fields
    pub fn get_fields(&self) -> &[ConfigField] {
        &self.fields
    }

    /// Returns the currently focused field index
    pub fn get_focused_field_index(&self) -> usize {
        self.focused_index
    }

    /// Returns the value of a field by index
    pub fn get_field_value(&self, index: usize) -> Option<String> {
        self.fields.get(index).map(|f| f.value.clone())
    }

    /// Handles Tab key (move to next field)
    pub fn handle_tab(&mut self) {
        if !self.fields.is_empty() && self.focused_index < self.fields.len() - 1 {
            self.fields[self.focused_index].is_focused = false;
            self.focused_index += 1;
            self.fields[self.focused_index].is_focused = true;
        }
    }

    /// Handles Shift+Tab key (move to previous field)
    pub fn handle_shift_tab(&mut self) {
        if self.focused_index > 0 {
            self.fields[self.focused_index].is_focused = false;
            self.focused_index -= 1;
            self.fields[self.focused_index].is_focused = true;
        }
    }

    /// Handles character input (add to current field)
    pub fn handle_char(&mut self, c: char) {
        if let Some(field) = self.fields.get_mut(self.focused_index) {
            field.value.push(c);
        }
    }

    /// Handles backspace (remove last character from current field)
    pub fn handle_backspace(&mut self) {
        if let Some(field) = self.fields.get_mut(self.focused_index) {
            field.value.pop();
        }
    }

    /// Returns the current configuration
    pub fn get_config(&self) -> ProviderConfig {
        let mut config = ProviderConfig::new(self.provider);

        for (i, field) in self.fields.iter().enumerate() {
            config.set(&format!("field_{}", i), field.value.clone());
        }

        config
    }

    /// Converts the form fields to a CloudConfig
    pub fn to_cloud_config(&self) -> crate::cloud::CloudConfig {
        use crate::cloud::CloudConfig;
        use std::path::PathBuf;

        let mut config = CloudConfig {
            provider: self.provider,
            ..Default::default()
        };

        // Map fields by provider
        match self.provider {
            crate::cloud::CloudProvider::ICloud => {
                if let Some(field) = self.fields.first() {
                    config.icloud_path = Some(PathBuf::from(&field.value));
                }
            }
            crate::cloud::CloudProvider::Dropbox => {
                if let Some(field) = self.fields.first() {
                    config.dropbox_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::GDrive => {
                if let Some(field) = self.fields.first() {
                    config.gdrive_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::OneDrive => {
                if let Some(field) = self.fields.first() {
                    config.onedrive_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::WebDAV => {
                if self.fields.len() >= 3 {
                    config.webdav_endpoint = if self.fields[0].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[0].value.clone())
                    };
                    config.webdav_username = if self.fields[1].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[1].value.clone())
                    };
                    config.webdav_password = if self.fields[2].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[2].value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::SFTP => {
                if self.fields.len() >= 5 {
                    config.sftp_host = if self.fields[0].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[0].value.clone())
                    };
                    config.sftp_port = self.fields[1].value.parse().ok();
                    config.sftp_username = if self.fields[2].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[2].value.clone())
                    };
                    config.sftp_password = if self.fields[3].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[3].value.clone())
                    };
                    config.sftp_root = if self.fields[4].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[4].value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::AliyunDrive => {
                if let Some(field) = self.fields.first() {
                    config.aliyun_drive_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::AliyunOSS => {
                if self.fields.len() >= 4 {
                    config.aliyun_oss_endpoint = if self.fields[0].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[0].value.clone())
                    };
                    config.aliyun_oss_bucket = if self.fields[1].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[1].value.clone())
                    };
                    config.aliyun_oss_access_key = if self.fields[2].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[2].value.clone())
                    };
                    config.aliyun_oss_secret_key = if self.fields[3].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[3].value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::TencentCOS => {
                if self.fields.len() >= 4 {
                    config.tencent_cos_secret_id = if self.fields[0].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[0].value.clone())
                    };
                    config.tencent_cos_secret_key = if self.fields[1].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[1].value.clone())
                    };
                    config.tencent_cos_region = if self.fields[2].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[2].value.clone())
                    };
                    config.tencent_cos_bucket = if self.fields[3].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[3].value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::HuaweiOBS => {
                if self.fields.len() >= 4 {
                    config.huawei_obs_endpoint = if self.fields[0].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[0].value.clone())
                    };
                    config.huawei_obs_bucket = if self.fields[1].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[1].value.clone())
                    };
                    config.huawei_obs_access_key = if self.fields[2].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[2].value.clone())
                    };
                    config.huawei_obs_secret_key = if self.fields[3].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[3].value.clone())
                    };
                }
            }
            crate::cloud::CloudProvider::UpYun => {
                if self.fields.len() >= 3 {
                    config.upyun_bucket = if self.fields[0].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[0].value.clone())
                    };
                    config.upyun_operator = if self.fields[1].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[1].value.clone())
                    };
                    config.upyun_password = if self.fields[2].value.is_empty() {
                        None
                    } else {
                        Some(self.fields[2].value.clone())
                    };
                }
            }
        }

        config
    }

    /// Validate current form input
    pub fn validate(&self) -> Result<(), String> {
        // Check that non-password fields are not empty
        for field in self.fields.iter() {
            if !field.is_password && field.value.is_empty() {
                return Err(format!("{} cannot be empty", field.label));
            }
        }
        Ok(())
    }

    /// Test the current configuration
    pub async fn test_connection(&self) -> Result<String, String> {
        let config = self.to_cloud_config();

        crate::cloud::test_connection(&config)
            .await
            .map(|_| "Connection successful".to_string())
            .map_err(|e| format!("Connection failed: {}", e))
    }

    /// Renders the configuration screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Title
        let provider_name = match self.provider {
            CloudProvider::ICloud => "iCloud Drive",
            CloudProvider::Dropbox => "Dropbox",
            CloudProvider::GDrive => "Google Drive",
            CloudProvider::OneDrive => "OneDrive",
            CloudProvider::WebDAV => "WebDAV",
            CloudProvider::SFTP => "SFTP",
            CloudProvider::AliyunDrive => "阿里云盘",
            CloudProvider::AliyunOSS => "阿里云 OSS",
            CloudProvider::TencentCOS => "腾讯云 COS",
            CloudProvider::HuaweiOBS => "华为云 OBS",
            CloudProvider::UpYun => "又拍云",
        };

        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                format!("配置 {} / Configure {}", provider_name, provider_name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "输入配置信息，使用 Tab 切换字段",
                Style::default().fg(Color::Gray),
            )),
        ]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    ratatui::layout::Constraint::Length(4), // Title
                    ratatui::layout::Constraint::Min(0),    // Form fields
                    ratatui::layout::Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(title, chunks[0]);

        // Form fields
        let mut form_lines = vec![];

        for field in &self.fields {
            let display_value = if field.is_password && !field.value.is_empty() {
                "•".repeat(field.value.len())
            } else {
                field.value.clone()
            };

            let is_focused = field.is_focused;

            let line = if is_focused {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", field.label),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!(
                            "[{}]",
                            if display_value.is_empty() {
                                " "
                            } else {
                                &display_value
                            }
                        ),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", field.label),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        format!(
                            "[{}]",
                            if display_value.is_empty() {
                                " "
                            } else {
                                &display_value
                            }
                        ),
                        Style::default().fg(Color::White),
                    ),
                ])
            };

            form_lines.push(line);
            form_lines.push(Line::from("")); // Empty line between fields
        }

        let form = Paragraph::new(Text::from(form_lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("配置信息 / Configuration"),
        );

        frame.render_widget(form, chunks[1]);

        // Footer
        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::from("Enter: 测试连接  "),
            Span::from("Ctrl+S: 保存  "),
            Span::from("Tab: 切换字段  "),
            Span::from("Esc: 返回"),
        ])]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[2]);
    }
}
