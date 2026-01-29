use anyhow::{Context, Result};
use notify::{EventKind, RecursiveMode, RecommendedWatcher, Watcher};
use std::path::Path;
use tokio::sync::broadcast;

/// Events emitted by the file system watcher
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Multiple files changed in the cloud directory
    CloudChanged(Vec<String>),
    /// A new file was created
    FileCreated(String),
    /// An existing file was modified
    FileModified(String),
    /// A file was deleted
    FileDeleted(String),
}

/// File system watcher for monitoring cloud storage directories
///
/// This watcher monitors a directory for changes and emits sync events
/// that can be used to trigger automatic synchronization in TUI mode.
pub struct SyncWatcher {
    _watcher: RecommendedWatcher,
    broadcast_tx: broadcast::Sender<SyncEvent>,
}

impl SyncWatcher {
    /// Creates a new file system watcher for the given path
    ///
    /// # Arguments
    /// * `watch_path` - The directory path to monitor
    ///
    /// # Returns
    /// A Result containing the SyncWatcher or an error
    pub fn new(watch_path: &Path) -> Result<Self> {
        let (broadcast_tx, _rx) = broadcast::channel(100);

        // Create a channel for the notify watcher
        let (tx, rx) = std::sync::mpsc::sync_channel(100);

        // Create the file system watcher
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }).context("Failed to create file system watcher")?;

        // Start watching the directory recursively
        watcher.watch(watch_path, RecursiveMode::Recursive)
            .context(format!("Failed to watch path: {}", watch_path.display()))?;

        // Spawn a task to bridge notify events to sync events
        let tx_clone = broadcast_tx.clone();
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                for path in event.paths {
                    let sync_event = match event.kind {
                        EventKind::Create(_) => {
                            SyncEvent::FileCreated(path.to_string_lossy().to_string())
                        }
                        EventKind::Modify(_) => {
                            SyncEvent::FileModified(path.to_string_lossy().to_string())
                        }
                        EventKind::Remove(_) => {
                            SyncEvent::FileDeleted(path.to_string_lossy().to_string())
                        }
                        _ => continue,
                    };

                    // Use try_send to avoid blocking on a full channel
                    let _ = tx_clone.send(sync_event);
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            broadcast_tx,
        })
    }

    /// Subscribes to sync events from this watcher
    ///
    /// # Returns
    /// A receiver that will emit sync events as they occur
    pub fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Creates a new subscription with a custom buffer size
    ///
    /// # Arguments
    /// * `buffer_size` - The size of the event buffer
    ///
    /// # Returns
    /// A receiver that will emit sync events as they occur
    pub fn subscribe_with_buffer(&self, buffer_size: usize) -> broadcast::Receiver<SyncEvent> {
        let (tx, rx) = broadcast::channel(buffer_size);

        // Forward events from the main channel to the new subscriber
        let mut main_rx = self.broadcast_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = main_rx.recv().await {
                tx.send(event).ok();
            }
        });

        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_event_creation() {
        let event = SyncEvent::FileCreated("/path/to/file.json".to_string());
        match event {
            SyncEvent::FileCreated(path) => {
                assert_eq!(path, "/path/to/file.json");
            }
            _ => panic!("Expected FileCreated event"),
        }
    }

    #[test]
    fn test_sync_event_clone() {
        let event = SyncEvent::FileModified("/path/to/file.json".to_string());
        let cloned = event.clone();
        match cloned {
            SyncEvent::FileModified(path) => {
                assert_eq!(path, "/path/to/file.json");
            }
            _ => panic!("Expected FileModified event"),
        }
    }
}
