//! Filter condition state management

use crate::tui::models::password::PasswordRecord;
use std::collections::HashSet;

/// Filter type enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FilterType {
    /// All passwords (default, no filtering)
    All,
    /// Trash bin
    Trash,
    /// Expired passwords
    Expired,
    /// Favorites
    Favorite,
    /// Tag-based filtering
    Tag(String),
}

/// Filter condition state
#[derive(Debug, Clone, Default)]
pub struct FilterState {
    /// Currently active filters
    pub active_filters: HashSet<FilterType>,
    /// Currently selected tags (for tag filtering)
    pub selected_tags: HashSet<String>,
    /// Search query (optional, for future extension)
    pub search_query: Option<String>,
}

impl FilterState {
    /// Create new filter state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle a filter on/off
    pub fn toggle(&mut self, filter: FilterType) {
        if self.active_filters.contains(&filter) {
            self.active_filters.remove(&filter);
        } else {
            self.active_filters.insert(filter);
        }
    }

    /// Check if an entry matches the current filters
    pub fn matches(&self, entry: &PasswordRecord) -> bool {
        // Empty filters or All filter match all
        if self.active_filters.is_empty() {
            return true;
        }

        // If All is active, match everything
        if self.active_filters.contains(&FilterType::All) {
            return true;
        }

        for filter in &self.active_filters {
            match filter {
                FilterType::All => {
                    // All matches everything
                    return true;
                }
                FilterType::Trash => {
                    if !entry.is_deleted {
                        return false;
                    }
                }
                FilterType::Expired => {
                    // Entry is expired if expires_at is set and in the past
                    if entry.expires_at.map_or(true, |e| e > chrono::Utc::now()) {
                        return false;
                    }
                }
                FilterType::Favorite => {
                    if !entry.is_favorite {
                        return false;
                    }
                }
                FilterType::Tag(tag) => {
                    if !entry.tags.contains(tag) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check if a specific filter is active
    pub fn is_active(&self, filter: &FilterType) -> bool {
        self.active_filters.contains(filter)
    }

    /// Check if any non-default filters are active
    pub fn has_active_filters(&self) -> bool {
        // Consider filters active if there are any filters set
        // that are not just "All" (which is the default)
        if self.active_filters.is_empty() {
            return false;
        }
        // If only "All" is active, that's not really a filter
        if self.active_filters.len() == 1 && self.active_filters.contains(&FilterType::All) {
            return false;
        }
        true
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.active_filters.clear();
        self.selected_tags.clear();
        self.search_query = None;
    }

    /// Check if an entry matches the current search query
    pub fn matches_search(&self, entry: &PasswordRecord) -> bool {
        match &self.search_query {
            Some(query) if !query.is_empty() => {
                let query_lower = query.to_lowercase();
                // Search in name, username, url, notes, tags
                entry.name.to_lowercase().contains(&query_lower)
                    || entry.username.as_ref().map_or(false, |u| {
                        u.to_lowercase().contains(&query_lower)
                    })
                    || entry.url.as_ref().map_or(false, |u| {
                        u.to_lowercase().contains(&query_lower)
                    })
                    || entry.notes.as_ref().map_or(false, |n| {
                        n.to_lowercase().contains(&query_lower)
                    })
                    || entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            }
            _ => true, // No search query = match all
        }
    }

    /// Set search query
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = Some(query);
    }

    /// Clear search query
    pub fn clear_search(&mut self) {
        self.search_query = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry() -> PasswordRecord {
        PasswordRecord::new("test-1", "Test Entry", "password123")
            .with_favorite(true)
    }

    #[test]
    fn test_filter_state_default() {
        let state = FilterState::default();
        assert!(state.active_filters.is_empty());
        assert!(state.selected_tags.is_empty());
        assert!(state.search_query.is_none());
    }

    #[test]
    fn test_toggle_filter() {
        let mut state = FilterState::default();

        state.toggle(FilterType::Favorite);
        assert!(state.active_filters.contains(&FilterType::Favorite));

        state.toggle(FilterType::Favorite);
        assert!(!state.active_filters.contains(&FilterType::Favorite));
    }

    #[test]
    fn test_matches_favorite() {
        let mut state = FilterState::default();
        let entry = create_test_entry();

        state.toggle(FilterType::Favorite);
        assert!(state.matches(&entry));
    }

    #[test]
    fn test_matches_empty_filters() {
        let state = FilterState::default();
        let entry = create_test_entry();

        // Empty filters should match all entries
        assert!(state.matches(&entry));
    }

    #[test]
    fn test_is_active() {
        let mut state = FilterState::default();
        assert!(!state.is_active(&FilterType::Favorite));

        state.toggle(FilterType::Favorite);
        assert!(state.is_active(&FilterType::Favorite));
    }

    #[test]
    fn test_clear() {
        let mut state = FilterState::default();
        state.toggle(FilterType::Favorite);
        state.toggle(FilterType::Trash);
        state.selected_tags.insert("work".to_string());
        state.search_query = Some("test".to_string());

        state.clear();

        assert!(state.active_filters.is_empty());
        assert!(state.selected_tags.is_empty());
        assert!(state.search_query.is_none());
    }

    #[test]
    fn test_matches_search() {
        let mut state = FilterState::default();
        let mut entry = PasswordRecord::new("test-1", "Gmail Account", "password123");
        entry.username = Some("user@gmail.com".to_string());
        entry.url = Some("https://gmail.com".to_string());
        entry.tags = vec!["email".to_string()];

        // No search query - matches all
        assert!(state.matches_search(&entry));

        // Search by name
        state.search_query = Some("gmail".to_string());
        assert!(state.matches_search(&entry));

        // Search by username
        state.search_query = Some("user@".to_string());
        assert!(state.matches_search(&entry));

        // Search by url
        state.search_query = Some("https".to_string());
        assert!(state.matches_search(&entry));

        // Search by tag
        state.search_query = Some("email".to_string());
        assert!(state.matches_search(&entry));

        // Non-matching search
        state.search_query = Some("xyz123notfound".to_string());
        assert!(!state.matches_search(&entry));
    }

    #[test]
    fn test_set_and_clear_search() {
        let mut state = FilterState::default();

        state.set_search_query("test".to_string());
        assert_eq!(state.search_query, Some("test".to_string()));

        state.clear_search();
        assert_eq!(state.search_query, None);
    }
}
