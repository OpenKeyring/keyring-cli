//! Health reporting structures

use serde::{Serialize, Deserialize};

/// Represents a health issue found during checks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthIssue {
    pub issue_type: HealthIssueType,
    pub record_names: Vec<String>,
    pub description: String,
    pub severity: Severity,
}

/// Type of health issue
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthIssueType {
    WeakPassword,
    DuplicatePassword,
    CompromisedPassword,
}

/// Severity level of the issue
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Aggregated health report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthReport {
    pub total_records: usize,
    pub issues: Vec<HealthIssue>,
    pub weak_password_count: usize,
    pub duplicate_password_count: usize,
    pub compromised_password_count: usize,
}

impl HealthReport {
    pub fn from_issues(total_records: usize, issues: Vec<HealthIssue>) -> Self {
        let weak_password_count = issues.iter()
            .filter(|i| i.issue_type == HealthIssueType::WeakPassword)
            .count();

        let duplicate_password_count = issues.iter()
            .filter(|i| i.issue_type == HealthIssueType::DuplicatePassword)
            .count();

        let compromised_password_count = issues.iter()
            .filter(|i| i.issue_type == HealthIssueType::CompromisedPassword)
            .count();

        Self {
            total_records,
            issues,
            weak_password_count,
            duplicate_password_count,
            compromised_password_count,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.issues.is_empty()
    }
}
