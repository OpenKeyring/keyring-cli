use crate::sync::export::SyncRecord;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    Newer,
    Older,
    Local,
    Remote,
    Merge,
    Interactive,
}

#[derive(Debug, Clone)]
pub struct Conflict {
    pub id: String,
    pub local_record: Option<SyncRecord>,
    pub remote_record: Option<SyncRecord>,
    pub resolution: Option<ConflictResolution>,
}

impl Conflict {
    pub fn new(id: String, local: Option<SyncRecord>, remote: Option<SyncRecord>) -> Self {
        Self {
            id,
            local_record: local,
            remote_record: remote,
            resolution: None,
        }
    }

    pub fn is_conflict(&self) -> bool {
        self.local_record.is_some() && self.remote_record.is_some()
    }
}

pub trait ConflictResolver {
    fn detect_conflicts(
        &self,
        local_records: &[SyncRecord],
        remote_records: &[SyncRecord],
    ) -> Vec<Conflict>;
    fn resolve_conflicts(
        &self,
        conflicts: &[Conflict],
        resolution: ConflictResolution,
    ) -> Vec<Conflict>;
    fn auto_resolve_conflicts(&self, conflicts: &[Conflict]) -> Vec<Conflict>;
}

pub struct DefaultConflictResolver;

impl ConflictResolver for DefaultConflictResolver {
    fn detect_conflicts(
        &self,
        local_records: &[SyncRecord],
        remote_records: &[SyncRecord],
    ) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        // Create a map for easier lookup
        let remote_map: std::collections::HashMap<String, &SyncRecord> =
            remote_records.iter().map(|r| (r.id.clone(), r)).collect();

        for local in local_records {
            if let Some(remote) = remote_map.get(&local.id) {
                if self.has_changes(local, remote) {
                    conflicts.push(Conflict::new(
                        local.id.clone(),
                        Some(local.clone()),
                        Some((*remote).clone()),
                    ));
                }
            }
        }

        conflicts
    }

    fn resolve_conflicts(
        &self,
        conflicts: &[Conflict],
        resolution: ConflictResolution,
    ) -> Vec<Conflict> {
        conflicts
            .iter()
            .cloned()
            .map(|mut c| {
                c.resolution = Some(resolution.clone());
                c
            })
            .collect()
    }

    fn auto_resolve_conflicts(&self, conflicts: &[Conflict]) -> Vec<Conflict> {
        self.resolve_conflicts(conflicts, ConflictResolution::Newer)
    }
}

impl DefaultConflictResolver {
    fn has_changes(&self, local: &SyncRecord, remote: &SyncRecord) -> bool {
        // Compare version numbers to determine if there are changes
        // If versions differ, there's a conflict
        local.version != remote.version
    }

    /// Get the record with the higher version number
    #[allow(dead_code)]
    fn get_newer_record<'a>(
        &self,
        local: &'a SyncRecord,
        remote: &'a SyncRecord,
    ) -> &'a SyncRecord {
        match local.version.cmp(&remote.version) {
            Ordering::Greater | Ordering::Equal => local,
            Ordering::Less => remote,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::RecordType;

    // Helper function to create test SyncRecord
    fn create_test_sync_record(id: &str, version: u64) -> SyncRecord {
        SyncRecord {
            id: id.to_string(),
            version,
            record_type: RecordType::Password,
            encrypted_data: "encrypted".to_string(),
            nonce: "nonce123".to_string(),
            metadata: crate::sync::export::RecordMetadata {
                name: "test".to_string(),
                tags: vec![],
                platform: "test".to_string(),
                device_id: "device1".to_string(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    // Conflict struct tests
    #[test]
    fn test_conflict_new() {
        let local = create_test_sync_record("record-1", 1);
        let remote = create_test_sync_record("record-1", 2);

        let conflict = Conflict::new("record-1".to_string(), Some(local), Some(remote));

        assert_eq!(conflict.id, "record-1");
        assert!(conflict.local_record.is_some());
        assert!(conflict.remote_record.is_some());
        assert!(conflict.resolution.is_none());
    }

    #[test]
    fn test_conflict_new_with_none_local() {
        let remote = create_test_sync_record("record-2", 1);

        let conflict = Conflict::new("record-2".to_string(), None, Some(remote));

        assert_eq!(conflict.id, "record-2");
        assert!(conflict.local_record.is_none());
        assert!(conflict.remote_record.is_some());
    }

    #[test]
    fn test_conflict_new_with_none_remote() {
        let local = create_test_sync_record("record-3", 1);

        let conflict = Conflict::new("record-3".to_string(), Some(local), None);

        assert_eq!(conflict.id, "record-3");
        assert!(conflict.local_record.is_some());
        assert!(conflict.remote_record.is_none());
    }

    #[test]
    fn test_conflict_is_conflict_true() {
        let local = create_test_sync_record("record-4", 1);
        let remote = create_test_sync_record("record-4", 2);

        let conflict = Conflict::new("record-4".to_string(), Some(local), Some(remote));

        assert!(conflict.is_conflict());
    }

    #[test]
    fn test_conflict_is_conflict_false_local_only() {
        let local = create_test_sync_record("record-5", 1);

        let conflict = Conflict::new("record-5".to_string(), Some(local), None);

        assert!(!conflict.is_conflict());
    }

    #[test]
    fn test_conflict_is_conflict_false_remote_only() {
        let remote = create_test_sync_record("record-6", 1);

        let conflict = Conflict::new("record-6".to_string(), None, Some(remote));

        assert!(!conflict.is_conflict());
    }

    #[test]
    fn test_conflict_is_conflict_false_both_none() {
        let conflict = Conflict::new("record-7".to_string(), None, None);

        assert!(!conflict.is_conflict());
    }

    // ConflictResolution enum tests
    #[test]
    fn test_conflict_resolution_equality() {
        assert_eq!(ConflictResolution::Newer, ConflictResolution::Newer);
        assert_eq!(ConflictResolution::Older, ConflictResolution::Older);
        assert_eq!(ConflictResolution::Local, ConflictResolution::Local);
        assert_eq!(ConflictResolution::Remote, ConflictResolution::Remote);
        assert_eq!(ConflictResolution::Merge, ConflictResolution::Merge);
        assert_eq!(
            ConflictResolution::Interactive,
            ConflictResolution::Interactive
        );
    }

    #[test]
    fn test_conflict_resolution_inequality() {
        assert_ne!(ConflictResolution::Newer, ConflictResolution::Older);
        assert_ne!(ConflictResolution::Local, ConflictResolution::Remote);
        assert_ne!(ConflictResolution::Merge, ConflictResolution::Interactive);
    }

    // DefaultConflictResolver::detect_conflicts tests
    #[test]
    fn test_detect_conflicts_with_different_versions() {
        let resolver = DefaultConflictResolver;

        let local = vec![create_test_sync_record("record-1", 2)];
        let remote = vec![create_test_sync_record("record-1", 1)];

        let conflicts = resolver.detect_conflicts(&local, &remote);

        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].is_conflict());
        assert_eq!(conflicts[0].id, "record-1");
    }

    #[test]
    fn test_detect_conflicts_with_same_versions() {
        let resolver = DefaultConflictResolver;

        let local = vec![create_test_sync_record("record-2", 1)];
        let remote = vec![create_test_sync_record("record-2", 1)];

        let conflicts = resolver.detect_conflicts(&local, &remote);

        // Same version means no conflict
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_local_only() {
        let resolver = DefaultConflictResolver;

        let local = vec![create_test_sync_record("record-3", 1)];
        let remote = vec![];

        let conflicts = resolver.detect_conflicts(&local, &remote);

        // No conflict if record only exists locally
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_remote_only() {
        let resolver = DefaultConflictResolver;

        let local = vec![];
        let remote = vec![create_test_sync_record("record-4", 1)];

        let conflicts = resolver.detect_conflicts(&local, &remote);

        // No conflict if record only exists remotely
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_multiple_records() {
        let resolver = DefaultConflictResolver;

        let local = vec![
            create_test_sync_record("record-5", 2), // Conflict
            create_test_sync_record("record-6", 1), // No conflict (same version)
            create_test_sync_record("record-7", 3), // Conflict
        ];
        let remote = vec![
            create_test_sync_record("record-5", 1), // Different version
            create_test_sync_record("record-6", 1), // Same version
            create_test_sync_record("record-7", 2), // Different version
        ];

        let conflicts = resolver.detect_conflicts(&local, &remote);

        assert_eq!(conflicts.len(), 2);
        assert!(conflicts.iter().any(|c| c.id == "record-5"));
        assert!(conflicts.iter().any(|c| c.id == "record-7"));
    }

    #[test]
    fn test_detect_conflicts_no_matching_ids() {
        let resolver = DefaultConflictResolver;

        let local = vec![create_test_sync_record("local-1", 1)];
        let remote = vec![create_test_sync_record("remote-1", 2)];

        let conflicts = resolver.detect_conflicts(&local, &remote);

        // No conflicts when IDs don't match
        assert_eq!(conflicts.len(), 0);
    }

    // DefaultConflictResolver::resolve_conflicts tests
    #[test]
    fn test_resolve_conflicts_with_resolution() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![Conflict::new(
            "record-1".to_string(),
            Some(create_test_sync_record("record-1", 2)),
            Some(create_test_sync_record("record-1", 1)),
        )];

        let resolved = resolver.resolve_conflicts(&conflicts, ConflictResolution::Newer);

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Newer));
    }

    #[test]
    fn test_resolve_conflicts_with_local_resolution() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![Conflict::new(
            "record-2".to_string(),
            Some(create_test_sync_record("record-2", 1)),
            Some(create_test_sync_record("record-2", 2)),
        )];

        let resolved = resolver.resolve_conflicts(&conflicts, ConflictResolution::Local);

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Local));
    }

    #[test]
    fn test_resolve_conflicts_with_remote_resolution() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![Conflict::new(
            "record-3".to_string(),
            Some(create_test_sync_record("record-3", 1)),
            Some(create_test_sync_record("record-3", 2)),
        )];

        let resolved = resolver.resolve_conflicts(&conflicts, ConflictResolution::Remote);

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Remote));
    }

    #[test]
    fn test_resolve_conflicts_with_merge_resolution() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![Conflict::new(
            "record-4".to_string(),
            Some(create_test_sync_record("record-4", 1)),
            Some(create_test_sync_record("record-4", 2)),
        )];

        let resolved = resolver.resolve_conflicts(&conflicts, ConflictResolution::Merge);

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Merge));
    }

    #[test]
    fn test_resolve_conflicts_with_interactive_resolution() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![Conflict::new(
            "record-5".to_string(),
            Some(create_test_sync_record("record-5", 1)),
            Some(create_test_sync_record("record-5", 2)),
        )];

        let resolved = resolver.resolve_conflicts(&conflicts, ConflictResolution::Interactive);

        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].resolution,
            Some(ConflictResolution::Interactive)
        );
    }

    #[test]
    fn test_resolve_conflicts_multiple() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![
            Conflict::new(
                "record-6".to_string(),
                Some(create_test_sync_record("record-6", 1)),
                Some(create_test_sync_record("record-6", 2)),
            ),
            Conflict::new(
                "record-7".to_string(),
                Some(create_test_sync_record("record-7", 3)),
                Some(create_test_sync_record("record-7", 1)),
            ),
        ];

        let resolved = resolver.resolve_conflicts(&conflicts, ConflictResolution::Older);

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Older));
        assert_eq!(resolved[1].resolution, Some(ConflictResolution::Older));
    }

    #[test]
    fn test_resolve_conflicts_empty_list() {
        let resolver = DefaultConflictResolver;

        let conflicts: Vec<Conflict> = vec![];
        let resolved = resolver.resolve_conflicts(&conflicts, ConflictResolution::Newer);

        assert_eq!(resolved.len(), 0);
    }

    // DefaultConflictResolver::auto_resolve_conflicts tests
    #[test]
    fn test_auto_resolve_conflicts_uses_newer_strategy() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![Conflict::new(
            "record-8".to_string(),
            Some(create_test_sync_record("record-8", 1)),
            Some(create_test_sync_record("record-8", 2)),
        )];

        let resolved = resolver.auto_resolve_conflicts(&conflicts);

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Newer));
    }

    #[test]
    fn test_auto_resolve_conflicts_multiple() {
        let resolver = DefaultConflictResolver;

        let conflicts = vec![
            Conflict::new(
                "record-9".to_string(),
                Some(create_test_sync_record("record-9", 1)),
                Some(create_test_sync_record("record-9", 2)),
            ),
            Conflict::new(
                "record-10".to_string(),
                Some(create_test_sync_record("record-10", 3)),
                Some(create_test_sync_record("record-10", 1)),
            ),
        ];

        let resolved = resolver.auto_resolve_conflicts(&conflicts);

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Newer));
        assert_eq!(resolved[1].resolution, Some(ConflictResolution::Newer));
    }

    // DefaultConflictResolver::get_newer_record tests
    #[test]
    fn test_get_newer_record_local_is_newer() {
        let resolver = DefaultConflictResolver;

        let local = create_test_sync_record("record-11", 3);
        let remote = create_test_sync_record("record-11", 1);

        let newer = resolver.get_newer_record(&local, &remote);

        assert_eq!(newer.id, local.id);
        assert_eq!(newer.version, 3);
    }

    #[test]
    fn test_get_newer_record_remote_is_newer() {
        let resolver = DefaultConflictResolver;

        let local = create_test_sync_record("record-12", 1);
        let remote = create_test_sync_record("record-12", 3);

        let newer = resolver.get_newer_record(&local, &remote);

        assert_eq!(newer.id, remote.id);
        assert_eq!(newer.version, 3);
    }

    #[test]
    fn test_get_newer_record_same_version_returns_local() {
        let resolver = DefaultConflictResolver;

        let local = create_test_sync_record("record-13", 2);
        let remote = create_test_sync_record("record-13", 2);

        let newer = resolver.get_newer_record(&local, &remote);

        // Equal versions return local
        assert_eq!(newer.id, local.id);
        assert_eq!(newer.version, 2);
    }

    // Integration tests
    #[test]
    fn test_full_conflict_detection_and_resolution_workflow() {
        let resolver = DefaultConflictResolver;

        // Step 1: Detect conflicts
        let local = vec![create_test_sync_record("record-14", 2)];
        let remote = vec![create_test_sync_record("record-14", 1)];

        let conflicts = resolver.detect_conflicts(&local, &remote);
        assert_eq!(conflicts.len(), 1);

        // Step 2: Auto-resolve
        let resolved = resolver.auto_resolve_conflicts(&conflicts);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Newer));

        // Step 3: Get the newer record
        if let (Some(local_rec), Some(remote_rec)) =
            (&resolved[0].local_record, &resolved[0].remote_record)
        {
            let newer = resolver.get_newer_record(local_rec, remote_rec);
            assert_eq!(newer.version, 2);
        }
    }

    #[test]
    fn test_conflict_clone_preserves_data() {
        let local = create_test_sync_record("record-15", 1);
        let remote = create_test_sync_record("record-15", 2);

        let conflict = Conflict::new(
            "record-15".to_string(),
            Some(local.clone()),
            Some(remote.clone()),
        );

        let cloned = conflict.clone();

        assert_eq!(cloned.id, conflict.id);
        assert_eq!(cloned.local_record.as_ref().unwrap().id, local.id);
        assert_eq!(cloned.remote_record.as_ref().unwrap().id, remote.id);
        assert_eq!(cloned.resolution, conflict.resolution);
    }
}
