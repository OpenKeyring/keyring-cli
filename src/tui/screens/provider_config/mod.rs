//! Provider Configuration Screen
//!
//! TUI screen for configuring cloud provider-specific settings.

mod handlers;
mod render;
mod types;

#[cfg(test)]
mod tests;

pub use types::{ConfigField, ProviderConfig};

use crate::cloud::{CloudConfig, CloudProvider};
use std::path::PathBuf;

/// Provider configuration screen
#[derive(Debug, Clone)]
pub struct ProviderConfigScreen {
    /// Cloud provider being configured
    pub(super) provider: CloudProvider,
    /// Configuration fields
    pub(super) fields: Vec<ConfigField>,
    /// Currently focused field index
    pub(super) focused_index: usize,
}

impl ProviderConfigScreen {
    /// Creates a new provider configuration screen
    pub fn new(provider: CloudProvider) -> Self {
        let fields = match provider {
            CloudProvider::ICloud => vec![ConfigField::new("iCloud Path", false)],
            CloudProvider::Dropbox => vec![ConfigField::new("Access Token", true)],
            CloudProvider::GDrive => vec![ConfigField::new("Access Token", true)],
            CloudProvider::OneDrive => vec![ConfigField::new("Access Token", true)],
            CloudProvider::WebDAV => vec![
                ConfigField::new("WebDAV URL", false),
                ConfigField::new("Username", false),
                ConfigField::new("Password", true),
            ],
            CloudProvider::SFTP => vec![
                ConfigField::new("Host", false),
                ConfigField::new("Port", false),
                ConfigField::new("Username", false),
                ConfigField::new("Password", true),
                ConfigField::new("Root Path", false),
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
                ConfigField::new("Region", false),
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
                ConfigField::new("Password", true),
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

    /// Returns the current configuration
    pub fn get_config(&self) -> ProviderConfig {
        let mut config = ProviderConfig::new(self.provider);

        for (i, field) in self.fields.iter().enumerate() {
            config.set(&format!("field_{}", i), field.value.clone());
        }

        config
    }

    /// Converts the form fields to a CloudConfig
    pub fn to_cloud_config(&self) -> CloudConfig {
        let mut config = CloudConfig {
            provider: self.provider,
            ..Default::default()
        };

        // Map fields by provider
        match self.provider {
            CloudProvider::ICloud => {
                if let Some(field) = self.fields.first() {
                    config.icloud_path = Some(PathBuf::from(&field.value));
                }
            }
            CloudProvider::Dropbox => {
                if let Some(field) = self.fields.first() {
                    config.dropbox_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            CloudProvider::GDrive => {
                if let Some(field) = self.fields.first() {
                    config.gdrive_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            CloudProvider::OneDrive => {
                if let Some(field) = self.fields.first() {
                    config.onedrive_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            CloudProvider::WebDAV => {
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
            CloudProvider::SFTP => {
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
            CloudProvider::AliyunDrive => {
                if let Some(field) = self.fields.first() {
                    config.aliyun_drive_token = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    };
                }
            }
            CloudProvider::AliyunOSS => {
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
            CloudProvider::TencentCOS => {
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
            CloudProvider::HuaweiOBS => {
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
            CloudProvider::UpYun => {
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
}
