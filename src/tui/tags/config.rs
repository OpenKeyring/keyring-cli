use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagConfig {
    pub env: Option<EnvTag>,
    pub risk: Option<RiskTag>,
    pub custom: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvTag {
    Dev,
    Test,
    Staging,
    Prod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTag {
    Low,
    Medium,
    High,
}

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("Contradiction in field '{field}': {message}")]
    Contradiction { field: String, message: String },

    #[error("Invalid tag format '{tag}': expected '{expected}'")]
    InvalidFormat { tag: String, expected: String },
}

impl EnvTag {
    pub fn to_string(&self) -> String {
        match self {
            Self::Dev => "env:dev",
            Self::Test => "env:test",
            Self::Staging => "env:staging",
            Self::Prod => "env:prod",
        }
        .to_string()
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dev => "dev (开发环境)",
            Self::Test => "test (测试环境)",
            Self::Staging => "staging (预发布环境)",
            Self::Prod => "prod (生产环境)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Dev => "本地开发环境，会话级授权",
            Self::Test => "测试环境，会话级授权",
            Self::Staging => "预发布环境，会话级授权",
            Self::Prod => "生产环境，每次需要确认 ⚠️",
        }
    }
}

impl RiskTag {
    pub fn to_string(&self) -> String {
        match self {
            Self::Low => "risk:low",
            Self::Medium => "risk:medium",
            Self::High => "risk:high",
        }
        .to_string()
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "low (低风险)",
            Self::Medium => "medium (中风险)",
            Self::High => "high (高风险)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Low => "只读操作，会话级授权",
            Self::Medium => "读写操作，需确认",
            Self::High => "危险操作，每次确认 ⚠️",
        }
    }
}

pub fn validate_tag_config(config: &TagConfig) -> Result<(), TagError> {
    // Check for contradictory combinations
    if matches!(config.env, Some(EnvTag::Prod)) && matches!(config.risk, Some(RiskTag::Low)) {
        return Err(TagError::Contradiction {
            field: "env:prod + risk:low".to_string(),
            message: "生产环境不应标记为低风险".to_string(),
        });
    }

    if matches!(config.env, Some(EnvTag::Dev)) && matches!(config.risk, Some(RiskTag::High)) {
        return Err(TagError::Contradiction {
            field: "env:dev + risk:high".to_string(),
            message: "开发环境不应标记为高风险".to_string(),
        });
    }

    // Validate custom tag format
    for tag in &config.custom {
        if !tag.contains(':') {
            return Err(TagError::InvalidFormat {
                tag: tag.clone(),
                expected: "key:value".to_string(),
            });
        }
    }

    Ok(())
}
