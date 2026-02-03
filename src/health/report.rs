//! Health reporting structures

use serde::{Deserialize, Serialize};

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
        let weak_password_count = issues
            .iter()
            .filter(|i| i.issue_type == HealthIssueType::WeakPassword)
            .count();

        let duplicate_password_count = issues
            .iter()
            .filter(|i| i.issue_type == HealthIssueType::DuplicatePassword)
            .count();

        let compromised_password_count = issues
            .iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    // Severity enum tests
    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Low, Severity::Low);
        assert_eq!(Severity::Medium, Severity::Medium);
        assert_eq!(Severity::High, Severity::High);
        assert_eq!(Severity::Critical, Severity::Critical);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(Severity::Low, Severity::Medium);
        assert_ne!(Severity::Medium, Severity::High);
        assert_ne!(Severity::High, Severity::Critical);
        assert_ne!(Severity::Critical, Severity::Low);
    }

    #[test]
    fn test_severity_numeric_values() {
        assert_eq!(Severity::Low as i32, 1);
        assert_eq!(Severity::Medium as i32, 2);
        assert_eq!(Severity::High as i32, 3);
        assert_eq!(Severity::Critical as i32, 4);
    }

    #[test]
    fn test_severity_ord() {
        // Test ordering works correctly
        let mut severities = vec![
            Severity::High,
            Severity::Low,
            Severity::Critical,
            Severity::Medium,
        ];
        severities.sort();
        assert_eq!(
            severities,
            vec![
                Severity::Low,
                Severity::Medium,
                Severity::High,
                Severity::Critical
            ]
        );
    }

    // HealthIssueType enum tests
    #[test]
    fn test_health_issue_type_equality() {
        assert_eq!(HealthIssueType::WeakPassword, HealthIssueType::WeakPassword);
        assert_eq!(
            HealthIssueType::DuplicatePassword,
            HealthIssueType::DuplicatePassword
        );
        assert_eq!(
            HealthIssueType::CompromisedPassword,
            HealthIssueType::CompromisedPassword
        );
    }

    #[test]
    fn test_health_issue_type_inequality() {
        assert_ne!(
            HealthIssueType::WeakPassword,
            HealthIssueType::DuplicatePassword
        );
        assert_ne!(
            HealthIssueType::DuplicatePassword,
            HealthIssueType::CompromisedPassword
        );
        assert_ne!(
            HealthIssueType::CompromisedPassword,
            HealthIssueType::WeakPassword
        );
    }

    // HealthIssue struct tests
    #[test]
    fn test_health_issue_creation() {
        let issue = HealthIssue {
            issue_type: HealthIssueType::WeakPassword,
            record_names: vec!["record1".to_string(), "record2".to_string()],
            description: "Password is too weak".to_string(),
            severity: Severity::Medium,
        };

        assert_eq!(issue.issue_type, HealthIssueType::WeakPassword);
        assert_eq!(issue.record_names.len(), 2);
        assert_eq!(issue.severity, Severity::Medium);
    }

    #[test]
    fn test_health_issue_with_empty_record_names() {
        let issue = HealthIssue {
            issue_type: HealthIssueType::CompromisedPassword,
            record_names: vec![],
            description: "Password found in breach".to_string(),
            severity: Severity::Critical,
        };

        assert!(issue.record_names.is_empty());
        assert_eq!(issue.severity, Severity::Critical);
    }

    // HealthReport::from_issues tests
    #[test]
    fn test_health_report_from_issues_empty() {
        let report = HealthReport::from_issues(10, vec![]);

        assert_eq!(report.total_records, 10);
        assert_eq!(report.issues.len(), 0);
        assert_eq!(report.weak_password_count, 0);
        assert_eq!(report.duplicate_password_count, 0);
        assert_eq!(report.compromised_password_count, 0);
    }

    #[test]
    fn test_health_report_from_issues_weak_passwords() {
        let issues = vec![
            HealthIssue {
                issue_type: HealthIssueType::WeakPassword,
                record_names: vec!["record1".to_string()],
                description: "Weak".to_string(),
                severity: Severity::Low,
            },
            HealthIssue {
                issue_type: HealthIssueType::WeakPassword,
                record_names: vec!["record2".to_string()],
                description: "Weak".to_string(),
                severity: Severity::Low,
            },
        ];

        let report = HealthReport::from_issues(5, issues);

        assert_eq!(report.weak_password_count, 2);
        assert_eq!(report.duplicate_password_count, 0);
        assert_eq!(report.compromised_password_count, 0);
        assert_eq!(report.total_records, 5);
        assert_eq!(report.issues.len(), 2);
    }

    #[test]
    fn test_health_report_from_issues_duplicate_passwords() {
        let issues = vec![HealthIssue {
            issue_type: HealthIssueType::DuplicatePassword,
            record_names: vec!["record1".to_string(), "record2".to_string()],
            description: "Duplicate".to_string(),
            severity: Severity::Medium,
        }];

        let report = HealthReport::from_issues(5, issues.clone());

        assert_eq!(report.duplicate_password_count, 1);
        assert_eq!(report.weak_password_count, 0);
        assert_eq!(report.compromised_password_count, 0);
        assert_eq!(report.issues.len(), 1);
    }

    #[test]
    fn test_health_report_from_issues_compromised_passwords() {
        let issues = vec![
            HealthIssue {
                issue_type: HealthIssueType::CompromisedPassword,
                record_names: vec!["record1".to_string()],
                description: "Breach".to_string(),
                severity: Severity::Critical,
            },
            HealthIssue {
                issue_type: HealthIssueType::CompromisedPassword,
                record_names: vec!["record2".to_string()],
                description: "Breach".to_string(),
                severity: Severity::Critical,
            },
        ];

        let report = HealthReport::from_issues(10, issues);

        assert_eq!(report.compromised_password_count, 2);
        assert_eq!(report.weak_password_count, 0);
        assert_eq!(report.duplicate_password_count, 0);
    }

    #[test]
    fn test_health_report_from_issues_mixed_types() {
        let issues = vec![
            HealthIssue {
                issue_type: HealthIssueType::WeakPassword,
                record_names: vec!["weak1".to_string()],
                description: "Weak".to_string(),
                severity: Severity::Low,
            },
            HealthIssue {
                issue_type: HealthIssueType::DuplicatePassword,
                record_names: vec!["dup1".to_string(), "dup2".to_string()],
                description: "Dup".to_string(),
                severity: Severity::Medium,
            },
            HealthIssue {
                issue_type: HealthIssueType::CompromisedPassword,
                record_names: vec!["breach1".to_string()],
                description: "Breach".to_string(),
                severity: Severity::Critical,
            },
        ];

        let report = HealthReport::from_issues(20, issues);

        assert_eq!(report.weak_password_count, 1);
        assert_eq!(report.duplicate_password_count, 1);
        assert_eq!(report.compromised_password_count, 1);
        assert_eq!(report.total_records, 20);
        assert_eq!(report.issues.len(), 3);
    }

    #[test]
    fn test_health_report_from_issues_same_type_multiple() {
        let issues = vec![
            HealthIssue {
                issue_type: HealthIssueType::WeakPassword,
                record_names: vec!["w1".to_string()],
                description: "Weak".to_string(),
                severity: Severity::Low,
            },
            HealthIssue {
                issue_type: HealthIssueType::WeakPassword,
                record_names: vec!["w2".to_string()],
                description: "Weak".to_string(),
                severity: Severity::Low,
            },
            HealthIssue {
                issue_type: HealthIssueType::WeakPassword,
                record_names: vec!["w3".to_string()],
                description: "Weak".to_string(),
                severity: Severity::Low,
            },
        ];

        let report = HealthReport::from_issues(3, issues);

        assert_eq!(report.weak_password_count, 3);
        assert_eq!(report.issues.len(), 3);
    }

    // HealthReport::is_healthy tests
    #[test]
    fn test_health_report_is_healthy_empty_issues() {
        let report = HealthReport::from_issues(10, vec![]);

        assert!(report.is_healthy());
    }

    #[test]
    fn test_health_report_is_healthy_with_issues() {
        let issues = vec![HealthIssue {
            issue_type: HealthIssueType::WeakPassword,
            record_names: vec!["record1".to_string()],
            description: "Weak".to_string(),
            severity: Severity::Low,
        }];

        let report = HealthReport::from_issues(10, issues);

        assert!(!report.is_healthy());
    }

    #[test]
    fn test_health_report_is_healthy_all_types() {
        let issues = vec![
            HealthIssue {
                issue_type: HealthIssueType::WeakPassword,
                record_names: vec![],
                description: "Weak".to_string(),
                severity: Severity::Low,
            },
            HealthIssue {
                issue_type: HealthIssueType::DuplicatePassword,
                record_names: vec![],
                description: "Dup".to_string(),
                severity: Severity::Medium,
            },
            HealthIssue {
                issue_type: HealthIssueType::CompromisedPassword,
                record_names: vec![],
                description: "Breach".to_string(),
                severity: Severity::Critical,
            },
        ];

        let report = HealthReport::from_issues(10, issues);

        assert!(!report.is_healthy());
        assert_eq!(report.issues.len(), 3);
    }

    // HealthIssue serialization tests
    #[test]
    fn test_health_issue_serialization() {
        let issue = HealthIssue {
            issue_type: HealthIssueType::WeakPassword,
            record_names: vec!["test".to_string()],
            description: "Test issue".to_string(),
            severity: Severity::High,
        };

        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: HealthIssue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(issue.issue_type, deserialized.issue_type);
        assert_eq!(issue.record_names, deserialized.record_names);
        assert_eq!(issue.description, deserialized.description);
        assert_eq!(issue.severity, deserialized.severity);
    }

    #[test]
    fn test_health_report_serialization() {
        let report = HealthReport {
            total_records: 100,
            issues: vec![],
            weak_password_count: 0,
            duplicate_password_count: 0,
            compromised_password_count: 0,
        };

        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: HealthReport = serde_json::from_str(&serialized).unwrap();

        assert_eq!(report.total_records, deserialized.total_records);
        assert!(deserialized.issues.is_empty());
    }

    #[test]
    fn test_severity_serialization() {
        let severities = vec![
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ];

        for severity in severities {
            let serialized = serde_json::to_string(&severity).unwrap();
            let deserialized: Severity = serde_json::from_str(&serialized).unwrap();
            assert_eq!(severity, deserialized);
        }
    }

    #[test]
    fn test_health_issue_type_serialization() {
        let types = vec![
            HealthIssueType::WeakPassword,
            HealthIssueType::DuplicatePassword,
            HealthIssueType::CompromisedPassword,
        ];

        for issue_type in types {
            let serialized = serde_json::to_string(&issue_type).unwrap();
            let deserialized: HealthIssueType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(issue_type, deserialized);
        }
    }
}
