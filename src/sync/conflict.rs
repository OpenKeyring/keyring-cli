use crate::error::KeyringError;
use crate::db::models::Record;
use crate::sync::export::SyncRecord;
use std::cmp::Ordering;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    Newer,
    Older,
    Local,
    Remote,
    Merge,
    Interactive,
}

#[derive(Debug)]
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
    fn detect_conflicts(&self, local_records: &[SyncRecord], remote_records: &[SyncRecord]) -> Vec<Conflict>;
    fn resolve_conflicts(&self, conflicts: &[Conflict], resolution: ConflictResolution) -> Vec<Conflict>;
    fn auto_resolve_conflicts(&self, conflicts: &[Conflict]) -> Vec<Conflict>;
}

pub struct DefaultConflictResolver;

impl ConflictResolver for DefaultConflictResolver {
    fn detect_conflicts(&self, local_records: &[SyncRecord], remote_records: &[SyncRecord]) -> Vec<Conflict> {
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
                        Some(remote.clone()),
                    ));
                }
            }
        }

        conflicts
    }

    fn resolve_conflicts(&self, conflicts: &[Conflict], resolution: ConflictResolution) -> Vec<Conflict> {
        conflicts.iter()
            .map(|c| {
                let mut resolved = c.clone();
                resolved.resolution = Some(resolution.clone());
                resolved
            })
            .collect()
    }

    fn auto_resolve_conflicts(&self, conflicts: &[Conflict]) -> Vec<Conflict> {
        self.resolve_conflicts(conflicts, ConflictResolution::Newer)
    }
}

impl DefaultConflictResolver {
    fn has_changes(&self, local: &SyncRecord, remote: &SyncRecord) -> bool {
        // Compare updated timestamps to determine if there are changes
        local.updated_at != remote.updated_at
    }

    fn get_newer_record(&self, local: &SyncRecord, remote: &SyncRecord) -> &SyncRecord {
        match local.updated_at.cmp(&remote.updated_at) {
            Ordering::Greater | Ordering::Equal => local,
            Ordering::Less => remote,
        }
    }
}