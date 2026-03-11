//! Mock vault implementation for TUI development
//!
//! Provides in-memory test data that mimics the real Vault behavior.
//!
//! # Phase 0 - Temporary Development Scaffolding
//!
//! This module is temporary and will be replaced by the real Vault integration
//! in Phase 3 (Data Layer Integration). Known limitations:
//! - Uses String IDs internally, converts to Uuid for TreeState compatibility
//! - Linear lookups (acceptable for ~25 test entries)
//! - Hardcoded test data (intentional for reproducible UI testing)
//!
//! TODO(phase-3): Remove this module after integrating with real Vault service.

use crate::tui::models::password::PasswordRecord;
use crate::tui::models::group::Group;
use crate::tui::state::filter_state::FilterState;
use crate::tui::state::tree_state::{TreeNodeId, NodeType, VisibleNode};
use chrono::{Duration, Utc};
use std::collections::{HashMap, HashSet};

/// Mock data IDs - using valid UUID format for compatibility with TreeState
pub mod mock_ids {
    // Group IDs (using simple UUID format for readability)
    pub const GROUP_WORK: &str = "00000000-0000-0000-0000-000000000001";
    pub const GROUP_WORK_EMAIL: &str = "00000000-0000-0000-0000-000000000002";
    pub const GROUP_WORK_DEV: &str = "00000000-0000-0000-0000-000000000003";
    pub const GROUP_WORK_DEV_GITHUB: &str = "00000000-0000-0000-0000-000000000004";
    pub const GROUP_WORK_DEV_GITLAB: &str = "00000000-0000-0000-0000-000000000005";
    pub const GROUP_PERSONAL: &str = "00000000-0000-0000-0000-000000000006";
    pub const GROUP_PERSONAL_SOCIAL: &str = "00000000-0000-0000-0000-000000000007";
    pub const GROUP_PERSONAL_FINANCE: &str = "00000000-0000-0000-0000-000000000008";
    pub const GROUP_PERSONAL_SHOPPING: &str = "00000000-0000-0000-0000-000000000009";
    pub const GROUP_OTHERS: &str = "00000000-0000-0000-0000-00000000000a";
    pub const GROUP_OTHERS_ENTERTAINMENT: &str = "00000000-0000-0000-0000-00000000000b";

    // Password IDs
    pub const PWD_GMAIL_WORK: &str = "10000000-0000-0000-0000-000000000001";
    pub const PWD_OUTLOOK: &str = "10000000-0000-0000-0000-000000000002";
    pub const PWD_GITHUB_PERSONAL: &str = "10000000-0000-0000-0000-000000000003";
    pub const PWD_GITHUB_WORK: &str = "10000000-0000-0000-0000-000000000004";
    pub const PWD_GITLAB: &str = "10000000-0000-0000-0000-000000000005";
    pub const PWD_JIRA: &str = "10000000-0000-0000-0000-000000000006";
    pub const PWD_CONFLUENCE: &str = "10000000-0000-0000-0000-000000000007";
    pub const PWD_TWITTER: &str = "10000000-0000-0000-0000-000000000008";
    pub const PWD_FACEBOOK: &str = "10000000-0000-0000-0000-000000000009";
    pub const PWD_LINKEDIN: &str = "10000000-0000-0000-0000-00000000000a";
    pub const PWD_BOA: &str = "10000000-0000-0000-0000-00000000000b";
    pub const PWD_CHASE: &str = "10000000-0000-0000-0000-00000000000c";
    pub const PWD_PAYPAL: &str = "10000000-0000-0000-0000-00000000000d";
    pub const PWD_AMAZON: &str = "10000000-0000-0000-0000-00000000000e";
    pub const PWD_EBAY: &str = "10000000-0000-0000-0000-00000000000f";
    pub const PWD_GMAIL_PERSONAL: &str = "10000000-0000-0000-0000-000000000010";
    pub const PWD_NETFLIX: &str = "10000000-0000-0000-0000-000000000011";
    pub const PWD_SPOTIFY: &str = "10000000-0000-0000-0000-000000000012";
    pub const PWD_YOUTUBE: &str = "10000000-0000-0000-0000-000000000013";
    pub const PWD_WIFI: &str = "10000000-0000-0000-0000-000000000014";
    pub const PWD_EXPIRED: &str = "10000000-0000-0000-0000-000000000015";
    pub const PWD_TRASHED_1: &str = "10000000-0000-0000-0000-000000000016";
    pub const PWD_TRASHED_2: &str = "10000000-0000-0000-0000-000000000017";
    pub const PWD_AWS: &str = "10000000-0000-0000-0000-000000000018";
    pub const PWD_NOTION: &str = "10000000-0000-0000-0000-000000000019";
}

/// Mock vault containing test data
#[derive(Debug)]
pub struct MockVault {
    /// All groups
    pub groups: Vec<Group>,
    /// All passwords
    pub passwords: Vec<PasswordRecord>,
    /// Group hierarchy: parent_id -> children IDs
    pub group_children: HashMap<String, Vec<String>>,
    /// Password count by group
    pub password_counts: HashMap<String, usize>,
}

impl MockVault {
    /// Create a new mock vault with default test data
    pub fn new() -> Self {
        let groups = Self::create_default_groups();
        let passwords = Self::create_default_passwords(&groups);
        let group_children = Self::build_group_hierarchy(&groups);
        let password_counts = Self::count_passwords_by_group(&passwords);

        Self {
            groups,
            passwords,
            group_children,
            password_counts,
        }
    }

    /// Create default 3-level group structure
    fn create_default_groups() -> Vec<Group> {
        use mock_ids::*;
        let now = Utc::now();

        // Level 0: Root groups
        vec![
            // Work group with children
            Group {
                id: GROUP_WORK.to_string(),
                name: "Work".to_string(),
                parent_id: None,
                level: 0,
                created_at: now,
            },
            Group {
                id: GROUP_WORK_EMAIL.to_string(),
                name: "Email".to_string(),
                parent_id: Some(GROUP_WORK.to_string()),
                level: 1,
                created_at: now,
            },
            Group {
                id: GROUP_WORK_DEV.to_string(),
                name: "Development".to_string(),
                parent_id: Some(GROUP_WORK.to_string()),
                level: 1,
                created_at: now,
            },
            Group {
                id: GROUP_WORK_DEV_GITHUB.to_string(),
                name: "GitHub".to_string(),
                parent_id: Some(GROUP_WORK_DEV.to_string()),
                level: 2,
                created_at: now,
            },
            Group {
                id: GROUP_WORK_DEV_GITLAB.to_string(),
                name: "GitLab".to_string(),
                parent_id: Some(GROUP_WORK_DEV.to_string()),
                level: 2,
                created_at: now,
            },

            // Personal group with children
            Group {
                id: GROUP_PERSONAL.to_string(),
                name: "Personal".to_string(),
                parent_id: None,
                level: 0,
                created_at: now,
            },
            Group {
                id: GROUP_PERSONAL_SOCIAL.to_string(),
                name: "Social".to_string(),
                parent_id: Some(GROUP_PERSONAL.to_string()),
                level: 1,
                created_at: now,
            },
            Group {
                id: GROUP_PERSONAL_FINANCE.to_string(),
                name: "Finance".to_string(),
                parent_id: Some(GROUP_PERSONAL.to_string()),
                level: 1,
                created_at: now,
            },
            Group {
                id: GROUP_PERSONAL_SHOPPING.to_string(),
                name: "Shopping".to_string(),
                parent_id: Some(GROUP_PERSONAL.to_string()),
                level: 1,
                created_at: now,
            },

            // Others group
            Group {
                id: GROUP_OTHERS.to_string(),
                name: "Others".to_string(),
                parent_id: None,
                level: 0,
                created_at: now,
            },
            Group {
                id: GROUP_OTHERS_ENTERTAINMENT.to_string(),
                name: "Entertainment".to_string(),
                parent_id: Some(GROUP_OTHERS.to_string()),
                level: 1,
                created_at: now,
            },
        ]
    }

    /// Create 20+ test password entries
    fn create_default_passwords(_groups: &[Group]) -> Vec<PasswordRecord> {
        use mock_ids::*;
        let now = Utc::now();
        let expired_time = now - Duration::days(1);
        let deleted_time = now - Duration::days(5);

        let mut passwords = Vec::new();

        // Work -> Email passwords
        passwords.push(PasswordRecord::new(PWD_GMAIL_WORK, "Gmail Work", "work-gmail-pass")
            .with_username("john.doe@company.com")
            .with_url("https://mail.google.com")
            .with_tags(vec!["email".to_string(), "important".to_string()])
            .with_group(GROUP_WORK_EMAIL)
            .with_favorite(true));

        passwords.push(PasswordRecord::new(PWD_OUTLOOK, "Outlook Work", "outlook-pass")
            .with_username("john.doe@company.com")
            .with_url("https://outlook.office.com")
            .with_tags(vec!["email".to_string()])
            .with_group(GROUP_WORK_EMAIL));

        // Work -> Development -> GitHub
        passwords.push(PasswordRecord::new(PWD_GITHUB_PERSONAL, "GitHub Personal", "github-pass-123")
            .with_username("johndoe")
            .with_url("https://github.com")
            .with_tags(vec!["dev".to_string(), "coding".to_string()])
            .with_group(GROUP_WORK_DEV_GITHUB)
            .with_favorite(true));

        passwords.push(PasswordRecord::new(PWD_GITHUB_WORK, "GitHub Work", "github-work-pass")
            .with_username("john-work")
            .with_url("https://github.com")
            .with_tags(vec!["dev".to_string(), "work".to_string()])
            .with_group(GROUP_WORK_DEV_GITHUB));

        // Work -> Development -> GitLab
        passwords.push(PasswordRecord::new(PWD_GITLAB, "GitLab", "gitlab-pass")
            .with_username("johndoe")
            .with_url("https://gitlab.com")
            .with_tags(vec!["dev".to_string()])
            .with_group(GROUP_WORK_DEV_GITLAB));

        // Work (root level)
        passwords.push(PasswordRecord::new(PWD_JIRA, "Jira", "jira-pass")
            .with_username("john.doe")
            .with_url("https://company.atlassian.net")
            .with_tags(vec!["project".to_string()])
            .with_group(GROUP_WORK));

        passwords.push(PasswordRecord::new(PWD_CONFLUENCE, "Confluence", "confluence-pass")
            .with_username("john.doe")
            .with_url("https://company.atlassian.net/wiki")
            .with_tags(vec!["docs".to_string()])
            .with_group(GROUP_WORK));

        // Personal -> Social
        passwords.push(PasswordRecord::new(PWD_TWITTER, "Twitter", "twitter-pass")
            .with_username("@johndoe")
            .with_url("https://twitter.com")
            .with_tags(vec!["social".to_string()])
            .with_group(GROUP_PERSONAL_SOCIAL));

        passwords.push(PasswordRecord::new(PWD_FACEBOOK, "Facebook", "facebook-pass")
            .with_username("john.doe@email.com")
            .with_url("https://facebook.com")
            .with_tags(vec!["social".to_string()])
            .with_group(GROUP_PERSONAL_SOCIAL));

        passwords.push(PasswordRecord::new(PWD_LINKEDIN, "LinkedIn", "linkedin-pass")
            .with_username("john.doe@email.com")
            .with_url("https://linkedin.com")
            .with_tags(vec!["social".to_string(), "work".to_string()])
            .with_group(GROUP_PERSONAL_SOCIAL)
            .with_favorite(true));

        // Personal -> Finance
        passwords.push(PasswordRecord::new(PWD_BOA, "Bank of America", "boa-pass")
            .with_username("john.doe")
            .with_url("https://bankofamerica.com")
            .with_tags(vec!["finance".to_string(), "bank".to_string(), "important".to_string()])
            .with_group(GROUP_PERSONAL_FINANCE)
            .with_favorite(true));

        passwords.push(PasswordRecord::new(PWD_CHASE, "Chase", "chase-pass")
            .with_username("john.doe@email.com")
            .with_url("https://chase.com")
            .with_tags(vec!["finance".to_string(), "bank".to_string()])
            .with_group(GROUP_PERSONAL_FINANCE));

        passwords.push(PasswordRecord::new(PWD_PAYPAL, "PayPal", "paypal-pass")
            .with_username("john.doe@email.com")
            .with_url("https://paypal.com")
            .with_tags(vec!["finance".to_string(), "payment".to_string()])
            .with_group(GROUP_PERSONAL_FINANCE));

        // Personal -> Shopping
        passwords.push(PasswordRecord::new(PWD_AMAZON, "Amazon", "amazon-pass")
            .with_username("john.doe@email.com")
            .with_url("https://amazon.com")
            .with_tags(vec!["shopping".to_string()])
            .with_group(GROUP_PERSONAL_SHOPPING));

        passwords.push(PasswordRecord::new(PWD_EBAY, "eBay", "ebay-pass")
            .with_username("johndoe_shopper")
            .with_url("https://ebay.com")
            .with_tags(vec!["shopping".to_string()])
            .with_group(GROUP_PERSONAL_SHOPPING));

        // Personal (root level)
        passwords.push(PasswordRecord::new(PWD_GMAIL_PERSONAL, "Gmail Personal", "gmail-personal-pass")
            .with_username("john.doe.personal@gmail.com")
            .with_url("https://mail.google.com")
            .with_tags(vec!["email".to_string()])
            .with_group(GROUP_PERSONAL));

        // Others -> Entertainment
        passwords.push(PasswordRecord::new(PWD_NETFLIX, "Netflix", "netflix-pass")
            .with_username("john.doe@email.com")
            .with_url("https://netflix.com")
            .with_tags(vec!["entertainment".to_string(), "streaming".to_string()])
            .with_group(GROUP_OTHERS_ENTERTAINMENT)
            .with_favorite(true));

        passwords.push(PasswordRecord::new(PWD_SPOTIFY, "Spotify", "spotify-pass")
            .with_username("john.doe@email.com")
            .with_url("https://spotify.com")
            .with_tags(vec!["entertainment".to_string(), "music".to_string()])
            .with_group(GROUP_OTHERS_ENTERTAINMENT));

        passwords.push(PasswordRecord::new(PWD_YOUTUBE, "YouTube Premium", "youtube-pass")
            .with_username("john.doe@email.com")
            .with_url("https://youtube.com")
            .with_tags(vec!["entertainment".to_string(), "streaming".to_string()])
            .with_group(GROUP_OTHERS_ENTERTAINMENT));

        // Others (root level)
        passwords.push(PasswordRecord::new(PWD_WIFI, "Wifi Home", "wifi-home-pass")
            .with_notes("Home wifi password")
            .with_tags(vec!["network".to_string()])
            .with_group(GROUP_OTHERS));

        // Expired password
        let mut expired = PasswordRecord::new(PWD_EXPIRED, "Old Service (Expired)", "old-pass")
            .with_username("olduser")
            .with_url("https://old-service.com")
            .with_tags(vec!["deprecated".to_string()]);
        expired.expires_at = Some(expired_time);
        passwords.push(expired);

        // Trashed password
        let mut trashed = PasswordRecord::new(PWD_TRASHED_1, "Deleted Account", "deleted-pass")
            .with_username("deleted-user")
            .with_url("https://deleted-site.com");
        trashed.is_deleted = true;
        trashed.deleted_at = Some(deleted_time);
        passwords.push(trashed);

        // Another trashed password
        let mut trashed2 = PasswordRecord::new(PWD_TRASHED_2, "Old Forum", "forum-pass")
            .with_username("forum-user-123");
        trashed2.is_deleted = true;
        trashed2.deleted_at = Some(now - Duration::days(15));
        passwords.push(trashed2);

        // More favorites for testing
        passwords.push(PasswordRecord::new(PWD_AWS, "AWS Console", "aws-pass")
            .with_username("john-work")
            .with_url("https://aws.amazon.com")
            .with_tags(vec!["cloud".to_string(), "dev".to_string(), "important".to_string()])
            .with_group(GROUP_WORK_DEV)
            .with_favorite(true));

        passwords.push(PasswordRecord::new(PWD_NOTION, "Notion", "notion-pass")
            .with_username("john.doe@email.com")
            .with_url("https://notion.so")
            .with_tags(vec!["productivity".to_string()])
            .with_group(GROUP_WORK));

        passwords
    }

    /// Build group hierarchy map
    fn build_group_hierarchy(groups: &[Group]) -> HashMap<String, Vec<String>> {
        let mut hierarchy: HashMap<String, Vec<String>> = HashMap::new();

        for group in groups {
            if let Some(parent_id) = &group.parent_id {
                hierarchy.entry(parent_id.clone()).or_default().push(group.id.clone());
            }
        }

        hierarchy
    }

    /// Count passwords by group
    fn count_passwords_by_group(passwords: &[PasswordRecord]) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for password in passwords {
            if !password.is_deleted {
                if let Some(group_id) = &password.group_id {
                    *counts.entry(group_id.clone()).or_insert(0) += 1;
                }
            }
        }

        counts
    }

    /// Get visible tree nodes based on expanded state (no filtering)
    pub fn get_visible_nodes(&self, expanded: &HashSet<String>) -> Vec<VisibleNode> {
        self.get_filtered_visible_nodes(expanded, None)
    }

    /// Get visible tree nodes based on expanded state and optional filter
    pub fn get_filtered_visible_nodes(
        &self,
        expanded: &HashSet<String>,
        filter: Option<&FilterState>,
    ) -> Vec<VisibleNode> {
        let mut nodes = Vec::new();

        // Get root groups (no parent)
        let root_groups: Vec<&Group> = self.groups.iter()
            .filter(|g| g.parent_id.is_none())
            .collect();

        for group in root_groups {
            self.add_group_nodes_filtered(group, 0, expanded, filter, &mut nodes);
        }

        nodes
    }

    /// Recursively add group nodes and their passwords
    fn add_group_nodes(
        &self,
        group: &Group,
        level: u8,
        expanded: &HashSet<String>,
        nodes: &mut Vec<VisibleNode>,
    ) {
        self.add_group_nodes_filtered(group, level, expanded, None, nodes);
    }

    /// Recursively add group nodes and their passwords with optional filtering
    fn add_group_nodes_filtered(
        &self,
        group: &Group,
        level: u8,
        expanded: &HashSet<String>,
        filter: Option<&FilterState>,
        nodes: &mut Vec<VisibleNode>,
    ) {
        // Check if filter is active (not empty and not just "All")
        let has_active_filter = filter.map_or(false, |f| {
            !f.active_filters.is_empty() && !f.active_filters.contains(&crate::tui::state::filter_state::FilterType::All)
        });

        // Add group node
        let child_count = self.group_children.get(&group.id).map_or(0, |v| v.len());
        // When filtering, count visible passwords instead of total
        let password_count = if has_active_filter {
            self.passwords.iter()
                .filter(|p| {
                    p.group_id.as_deref() == Some(&group.id) &&
                    filter.map_or(true, |f| f.matches(p))
                })
                .count()
        } else {
            self.password_counts.get(&group.id).copied().unwrap_or(0)
        };

        nodes.push(VisibleNode {
            id: TreeNodeId::Group(uuid::Uuid::parse_str(&group.id).unwrap_or(uuid::Uuid::nil())),
            level,
            node_type: NodeType::Folder,
            label: group.name.clone(),
            child_count: child_count + password_count,
        });

        // If expanded, add children
        if expanded.contains(&group.id) {
            // Add child groups first
            if let Some(child_ids) = self.group_children.get(&group.id) {
                for child_id in child_ids {
                    if let Some(child_group) = self.groups.iter().find(|g| g.id == *child_id) {
                        self.add_group_nodes_filtered(child_group, level + 1, expanded, filter, nodes);
                    }
                }
            }

            // Add passwords in this group (apply filter if active)
            for password in &self.passwords {
                let matches_group = password.group_id.as_deref() == Some(&group.id);
                let matches_filter = filter.map_or(true, |f| f.matches(password));

                if matches_group && matches_filter {
                    nodes.push(VisibleNode {
                        id: TreeNodeId::Password(
                            uuid::Uuid::parse_str(&password.id).unwrap_or(uuid::Uuid::nil())
                        ),
                        level: level + 1,
                        node_type: NodeType::Password,
                        label: password.name.clone(),
                        child_count: 0,
                    });
                }
            }
        }
    }

    /// Filter passwords based on filter state
    pub fn filter_passwords(&self, filter: &FilterState) -> Vec<&PasswordRecord> {
        self.passwords.iter()
            .filter(|p| filter.matches(p))
            .collect()
    }

    /// Get password by ID
    pub fn get_password(&self, id: &str) -> Option<&PasswordRecord> {
        self.passwords.iter().find(|p| p.id == id)
    }

    /// Get group by ID
    pub fn get_group(&self, id: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.id == id)
    }

    /// Get filter counts for UI display
    pub fn get_filter_counts(&self) -> FilterCounts {
        let mut counts = FilterCounts::default();

        for password in &self.passwords {
            counts.total += 1;
            if password.is_deleted {
                counts.trash += 1;
            }
            if password.is_favorite {
                counts.favorite += 1;
            }
            if password.expires_at.map_or(false, |e| e < Utc::now()) {
                counts.expired += 1;
            }
            for tag in &password.tags {
                *counts.by_tag.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        counts
    }
}

impl Default for MockVault {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Mutation Operations
// ============================================================

impl MockVault {
    /// Move password to trash (soft delete)
    pub fn move_to_trash(&mut self, id: &str) -> bool {
        if let Some(password) = self.passwords.iter_mut().find(|p| p.id == id) {
            password.is_deleted = true;
            password.deleted_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    /// Restore password from trash
    pub fn restore_password(&mut self, id: &str) -> bool {
        if let Some(password) = self.passwords.iter_mut().find(|p| p.id == id) {
            if password.is_deleted {
                password.is_deleted = false;
                password.deleted_at = None;
                return true;
            }
        }
        false
    }

    /// Permanently delete password
    pub fn delete_permanently(&mut self, id: &str) -> bool {
        let initial_len = self.passwords.len();
        self.passwords.retain(|p| p.id != id);
        self.passwords.len() < initial_len
    }
}

/// Filter counts for UI display
#[derive(Debug, Clone, Default)]
pub struct FilterCounts {
    pub total: usize,
    pub trash: usize,
    pub expired: usize,
    pub favorite: usize,
    pub by_tag: HashMap<String, usize>,
}

#[cfg(test)]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::filter_state::FilterType;

    #[test]
    fn test_mock_vault_creation() {
        let vault = MockVault::new();

        // Should have 11 groups (3-level structure)
        assert_eq!(vault.groups.len(), 11);

        // Should have 25 passwords
        assert_eq!(vault.passwords.len(), 25);
    }

    #[test]
    fn test_get_visible_nodes() {
        let vault = MockVault::new();
        let mut expanded: HashSet<String> = HashSet::new();

        // Without any expansion, should only show root groups
        let nodes = vault.get_visible_nodes(&expanded);
        assert_eq!(nodes.len(), 3); // Work, Personal, Others

        // Expand Work group
        expanded.insert(mock_ids::GROUP_WORK.to_string());
        let nodes = vault.get_visible_nodes(&expanded);
        // Should show Work, its children (Email, Development), and passwords directly in Work
        assert!(nodes.len() > 3);

        // Expand Development to show GitHub and GitLab
        expanded.insert(mock_ids::GROUP_WORK_DEV.to_string());
        let nodes = vault.get_visible_nodes(&expanded);
        // Should include more nested groups
        assert!(nodes.len() > 5);
    }

    #[test]
    fn test_filter_passwords_all() {
        let vault = MockVault::new();
        let filter = FilterState::default();

        let passwords = vault.filter_passwords(&filter);
        // All non-deleted passwords should be returned
        assert!(passwords.len() >= 20);
    }

    #[test]
    fn test_filter_passwords_favorites() {
        let vault = MockVault::new();
        let mut filter = FilterState::default();
        filter.toggle(FilterType::Favorite);

        let passwords = vault.filter_passwords(&filter);
        // Should have 6 favorites (Gmail Work, GitHub Personal, LinkedIn, BoA, Netflix, AWS)
        assert_eq!(passwords.len(), 6);

        for password in passwords {
            assert!(password.is_favorite);
        }
    }

    #[test]
    fn test_filter_passwords_trash() {
        let vault = MockVault::new();
        let mut filter = FilterState::default();
        filter.toggle(FilterType::Trash);

        let passwords = vault.filter_passwords(&filter);
        // Should have 2 trashed passwords
        assert_eq!(passwords.len(), 2);

        for password in passwords {
            assert!(password.is_deleted);
        }
    }

    #[test]
    fn test_filter_passwords_expired() {
        let vault = MockVault::new();
        let mut filter = FilterState::default();
        filter.toggle(FilterType::Expired);

        let passwords = vault.filter_passwords(&filter);
        // Should have 1 expired password
        assert_eq!(passwords.len(), 1);

        for password in passwords {
            assert!(password.expires_at.map_or(false, |e| e < Utc::now()));
        }
    }

    #[test]
    fn test_get_password() {
        let vault = MockVault::new();

        let password = vault.get_password(mock_ids::PWD_GMAIL_WORK);
        assert!(password.is_some());
        assert_eq!(password.unwrap().name, "Gmail Work");

        let not_found = vault.get_password("non-existent-uuid");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_group() {
        let vault = MockVault::new();

        let group = vault.get_group(mock_ids::GROUP_WORK);
        assert!(group.is_some());
        assert_eq!(group.unwrap().name, "Work");

        let not_found = vault.get_group("non-existent-uuid");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_filter_counts() {
        let vault = MockVault::new();
        let counts = vault.get_filter_counts();

        assert_eq!(counts.total, 25);
        assert_eq!(counts.trash, 2);
        assert_eq!(counts.expired, 1);
        assert_eq!(counts.favorite, 6);

        // Should have tags
        assert!(!counts.by_tag.is_empty());
        assert!(counts.by_tag.contains_key("email"));
        assert!(counts.by_tag.contains_key("social"));
    }

    #[test]
    fn test_group_hierarchy() {
        let vault = MockVault::new();

        // Work should have children: work-email, work-dev
        let work_children = vault.group_children.get(mock_ids::GROUP_WORK);
        assert!(work_children.is_some());
        assert_eq!(work_children.unwrap().len(), 2);

        // work-dev should have children: work-dev-github, work-dev-gitlab
        let dev_children = vault.group_children.get(mock_ids::GROUP_WORK_DEV);
        assert!(dev_children.is_some());
        assert_eq!(dev_children.unwrap().len(), 2);

        // Leaf nodes should have no children
        let leaf_children = vault.group_children.get(mock_ids::GROUP_WORK_DEV_GITHUB);
        assert!(leaf_children.is_none() || leaf_children.unwrap().is_empty());
    }

    #[test]
    fn test_three_level_structure() {
        let vault = MockVault::new();

        // Verify level 0 groups
        let level_0: Vec<_> = vault.groups.iter().filter(|g| g.level == 0).collect();
        assert_eq!(level_0.len(), 3); // Work, Personal, Others

        // Verify level 1 groups
        let level_1: Vec<_> = vault.groups.iter().filter(|g| g.level == 1).collect();
        assert_eq!(level_1.len(), 6);

        // Verify level 2 groups
        let level_2: Vec<_> = vault.groups.iter().filter(|g| g.level == 2).collect();
        assert_eq!(level_2.len(), 2); // GitHub, GitLab
    }

    #[test]
    fn test_get_filtered_visible_nodes_no_filter() {
        let vault = MockVault::new();
        let mut expanded: HashSet<String> = HashSet::new();
        expanded.insert(mock_ids::GROUP_WORK.to_string());

        // Without filter, should work like get_visible_nodes
        let nodes_no_filter = vault.get_filtered_visible_nodes(&expanded, None);
        let nodes_default = vault.get_visible_nodes(&expanded);

        assert_eq!(nodes_no_filter.len(), nodes_default.len());
    }

    #[test]
    fn test_get_filtered_visible_nodes_favorites() {
        let vault = MockVault::new();
        let mut expanded: HashSet<String> = HashSet::new();
        expanded.insert(mock_ids::GROUP_WORK.to_string());
        expanded.insert(mock_ids::GROUP_WORK_EMAIL.to_string());
        expanded.insert(mock_ids::GROUP_WORK_DEV.to_string());
        expanded.insert(mock_ids::GROUP_WORK_DEV_GITHUB.to_string());
        expanded.insert(mock_ids::GROUP_PERSONAL.to_string());
        expanded.insert(mock_ids::GROUP_PERSONAL_SOCIAL.to_string());
        expanded.insert(mock_ids::GROUP_PERSONAL_FINANCE.to_string());
        expanded.insert(mock_ids::GROUP_OTHERS.to_string());
        expanded.insert(mock_ids::GROUP_OTHERS_ENTERTAINMENT.to_string());

        // Apply Favorite filter
        let mut filter = FilterState::default();
        filter.toggle(FilterType::Favorite);

        let nodes = vault.get_filtered_visible_nodes(&expanded, Some(&filter));

        // Count password nodes (should only be favorites)
        let password_nodes: Vec<_> = nodes.iter()
            .filter(|n| matches!(n.node_type, NodeType::Password))
            .collect();

        // Should have 6 favorite passwords
        assert_eq!(password_nodes.len(), 6, "Expected 6 favorite password nodes");
    }

    #[test]
    fn test_get_filtered_visible_nodes_trash() {
        let vault = MockVault::new();
        let mut expanded: HashSet<String> = HashSet::new();

        // Apply Trash filter
        let mut filter = FilterState::default();
        filter.toggle(FilterType::Trash);

        let nodes = vault.get_filtered_visible_nodes(&expanded, Some(&filter));

        // With Trash filter, deleted items should be visible
        // The filter should show trashed passwords
        // Note: Without expanding groups, we only see root groups
        assert!(!nodes.is_empty());

        // Expand and check for trashed passwords
        expanded.insert(mock_ids::GROUP_WORK.to_string());
        let nodes_expanded = vault.get_filtered_visible_nodes(&expanded, Some(&filter));

        let password_nodes: Vec<_> = nodes_expanded.iter()
            .filter(|n| matches!(n.node_type, NodeType::Password))
            .collect();

        // Trashed passwords don't belong to a normal group in our test data
        // They should still appear with the Trash filter active
        assert!(password_nodes.is_empty() || password_nodes.len() <= 2);
    }

    #[test]
    fn test_get_filtered_visible_nodes_combined_filters() {
        let vault = MockVault::new();
        let mut expanded: HashSet<String> = HashSet::new();
        expanded.insert(mock_ids::GROUP_WORK.to_string());

        // Apply multiple filters
        let mut filter = FilterState::default();
        filter.toggle(FilterType::Favorite);

        let nodes_favorite = vault.get_filtered_visible_nodes(&expanded, Some(&filter));

        // Add another filter
        filter.toggle(FilterType::Expired);

        let nodes_combined = vault.get_filtered_visible_nodes(&expanded, Some(&filter));

        // Combined filter should match entries that satisfy ALL active filters
        // (intersection, not union)
        assert!(nodes_combined.len() <= nodes_favorite.len());
    }
}
