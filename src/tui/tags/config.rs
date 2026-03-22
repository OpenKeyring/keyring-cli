use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for EnvTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dev => write!(f, "env:dev"),
            Self::Test => write!(f, "env:test"),
            Self::Staging => write!(f, "env:staging"),
            Self::Prod => write!(f, "env:prod"),
        }
    }
}

impl EnvTag {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dev => "dev (development)",
            Self::Test => "test (testing)",
            Self::Staging => "staging (pre-production)",
            Self::Prod => "prod (production)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Dev => "Local development environment, session-level authorization",
            Self::Test => "Test environment, session-level authorization",
            Self::Staging => "Staging environment, session-level authorization",
            Self::Prod => "Production environment, confirmation required each time ⚠️",
        }
    }
}

impl fmt::Display for RiskTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "risk:low"),
            Self::Medium => write!(f, "risk:medium"),
            Self::High => write!(f, "risk:high"),
        }
    }
}

impl RiskTag {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "low (low risk)",
            Self::Medium => "medium (medium risk)",
            Self::High => "high (high risk)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Low => "Read-only operations, session-level authorization",
            Self::Medium => "Read-write operations, confirmation required",
            Self::High => "Dangerous operations, confirmation each time ⚠️",
        }
    }
}

pub fn validate_tag_config(config: &TagConfig) -> Result<(), TagError> {
    // Check for contradictory combinations
    if matches!(config.env, Some(EnvTag::Prod)) && matches!(config.risk, Some(RiskTag::Low)) {
        return Err(TagError::Contradiction {
            field: "env:prod + risk:low".to_string(),
            message: "Production environment should not be marked as low risk".to_string(),
        });
    }

    if matches!(config.env, Some(EnvTag::Dev)) && matches!(config.risk, Some(RiskTag::High)) {
        return Err(TagError::Contradiction {
            field: "env:dev + risk:high".to_string(),
            message: "Development environment should not be marked as high risk".to_string(),
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
