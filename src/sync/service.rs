//! Sync service for manual incremental synchronization

use crate::db::models::{StoredRecord, SyncStatus};
use crate::db::vault::Vault;
use crate::error::{KeyringError, Result};
use crate::sync::conflict::{ConflictResolution, ConflictResolver, DefaultConflictResolver};
use crate::sync::export::{JsonSyncExporter, SyncExporter, SyncRecord};
use crate::sync::import::{JsonSyncImporter, SyncImporter};
use std::fs;
use std::path::Path;

/// Sync service for managing incremental synchronization
pub struct SyncService {
    exporter: JsonSyncExporter,
    importer: JsonSyncImporter,
    conflict_resolver: DefaultConflictResolver,
}

impl Default for SyncService {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncService {
    pub fn new() -> Self {
        Self {
            exporter: JsonSyncExporter,
            importer: JsonSyncImporter,
            conflict_resolver: DefaultConflictResolver,
        }
    }

    /// Get pending records (records with Pending sync status)
    pub fn get_pending_records(&self, vault: &Vault) -> Result<Vec<StoredRecord>> {
        let all_records = vault.list_records()?;
        let mut pending = Vec::new();

        for record in all_records {
            let sync_state = vault.get_sync_state(&record.id.to_string())?;
            if let Some(state) = sync_state {
                if state.sync_status == SyncStatus::Pending {
                    pending.push(record);
                }
            } else {
                // No sync state means it's new and needs to be synced
                pending.push(record);
            }
        }

        Ok(pending)
    }

    /// Export pending records to sync directory
    pub fn export_pending_records(
        &self,
        vault: &Vault,
        sync_dir: &Path,
    ) -> Result<Vec<SyncRecord>> {
        let pending = self.get_pending_records(vault)?;
        let mut exported = Vec::new();

        // Ensure sync directory exists
        fs::create_dir_all(sync_dir).map_err(|e| {
            KeyringError::IoError(format!("Failed to create sync directory: {}", e))
        })?;

        for record in &pending {
            let sync_record = self.exporter.export_record(record)?;
            let file_path = sync_dir.join(format!("{}.json", record.id));
            self.exporter.write_to_file(&sync_record, &file_path)?;
            exported.push(sync_record);
        }

        Ok(exported)
    }

    /// Import records from sync directory with conflict detection
    pub fn import_from_directory(
        &self,
        vault: &mut Vault,
        sync_dir: &Path,
        conflict_resolution: ConflictResolution,
    ) -> Result<SyncStats> {
        if !sync_dir.exists() {
            return Ok(SyncStats {
                imported: 0,
                updated: 0,
                conflicts: 0,
            });
        }

        // Load all local records
        let local_records = vault.list_records()?;
        let local_sync_records: Vec<SyncRecord> = local_records
            .iter()
            .filter_map(|r| self.exporter.export_record(r).ok())
            .collect();

        // Load all remote records from directory
        let mut remote_records = Vec::new();
        for entry in fs::read_dir(sync_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(sync_record) = self.importer.import_from_file(&path) {
                    remote_records.push(sync_record);
                }
            }
        }

        // Detect conflicts
        let conflicts = self
            .conflict_resolver
            .detect_conflicts(&local_sync_records, &remote_records);

        // Resolve conflicts
        let resolved_conflicts = match conflict_resolution {
            ConflictResolution::Newer => self.conflict_resolver.auto_resolve_conflicts(&conflicts),
            _ => self
                .conflict_resolver
                .resolve_conflicts(&conflicts, conflict_resolution),
        };

        let mut stats = SyncStats {
            imported: 0,
            updated: 0,
            conflicts: conflicts.len(),
        };

        // Process resolved conflicts and new records
        let mut processed_ids = std::collections::HashSet::new();

        // Apply resolved conflicts (use higher version record)
        for conflict in &resolved_conflicts {
            if let Some(resolution) = &conflict.resolution {
                let record_to_use = match resolution {
                    ConflictResolution::Newer => {
                        if let (Some(local), Some(remote)) =
                            (&conflict.local_record, &conflict.remote_record)
                        {
                            // Use version-based comparison for conflict resolution
                            if local.version >= remote.version {
                                local.clone()
                            } else {
                                remote.clone()
                            }
                        } else if let Some(local) = &conflict.local_record {
                            local.clone()
                        } else if let Some(remote) = &conflict.remote_record {
                            remote.clone()
                        } else {
                            continue;
                        }
                    }
                    ConflictResolution::Local => {
                        if let Some(local) = &conflict.local_record {
                            local.clone()
                        } else {
                            continue;
                        }
                    }
                    ConflictResolution::Remote => {
                        if let Some(remote) = &conflict.remote_record {
                            remote.clone()
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

                let stored_record = self.importer.sync_record_to_db(record_to_use)?;
                vault.update_record(&stored_record)?;
                vault.set_sync_state(
                    &stored_record.id.to_string(),
                    Some(stored_record.updated_at.timestamp()),
                    SyncStatus::Synced,
                )?;
                processed_ids.insert(conflict.id.clone());
                stats.updated += 1;
            }
        }

        // Import new records (not in conflicts)
        for remote_record in remote_records {
            if processed_ids.contains(&remote_record.id) {
                continue;
            }

            // Check if record exists locally
            let exists_locally = local_sync_records.iter().any(|r| r.id == remote_record.id);

            let stored_record = self.importer.sync_record_to_db(remote_record.clone())?;

            if exists_locally {
                vault.update_record(&stored_record)?;
                stats.updated += 1;
            } else {
                vault.add_record(&stored_record)?;
                stats.imported += 1;
            }

            // Update sync state
            vault.set_sync_state(
                &stored_record.id.to_string(),
                Some(stored_record.updated_at.timestamp()),
                SyncStatus::Synced,
            )?;
        }

        // Mark pending records as synced after export
        let pending = self.get_pending_records(vault)?;
        for record in &pending {
            vault.set_sync_state(
                &record.id.to_string(),
                Some(record.updated_at.timestamp()),
                SyncStatus::Synced,
            )?;
        }

        Ok(stats)
    }

    /// Get sync status statistics
    pub fn get_sync_status(&self, vault: &Vault) -> Result<SyncStatusInfo> {
        let all_records = vault.list_records()?;
        let mut pending_count = 0;
        let mut conflict_count = 0;
        let mut synced_count = 0;

        for record in &all_records {
            let sync_state = vault.get_sync_state(&record.id.to_string())?;
            match sync_state {
                Some(state) => match state.sync_status {
                    SyncStatus::Pending => pending_count += 1,
                    SyncStatus::Conflict => conflict_count += 1,
                    SyncStatus::Synced => synced_count += 1,
                },
                None => pending_count += 1, // No sync state means pending
            }
        }

        Ok(SyncStatusInfo {
            total: all_records.len(),
            pending: pending_count,
            conflicts: conflict_count,
            synced: synced_count,
        })
    }
}

/// Statistics from sync operation
#[derive(Debug)]
pub struct SyncStats {
    pub imported: usize,
    pub updated: usize,
    pub conflicts: usize,
}

/// Sync status information
#[derive(Debug)]
pub struct SyncStatusInfo {
    pub total: usize,
    pub pending: usize,
    pub conflicts: usize,
    pub synced: usize,
}
