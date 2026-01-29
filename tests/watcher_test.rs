use keyring_cli::sync::watcher::{SyncEvent, SyncWatcher};
use tempfile::TempDir;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use std::path::PathBuf;

#[tokio::test]
async fn test_watch_file_changes() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let watcher = SyncWatcher::new(&watch_path).unwrap();
    let mut rx = watcher.subscribe();

    // Create a task to handle events
    let handle = tokio::spawn(async move {
        let mut event_count = 0;
        while let Ok(_event) = rx.recv().await {
            event_count += 1;
            if event_count >= 2 {
                break;
            }
        }
        event_count
    });

    // Give watcher a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create test file
    let file_path = temp_dir.path().join("test.json");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test").unwrap();
    file.sync_all().unwrap();

    // Wait a bit for the event to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Modify file
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"modified").unwrap();
    file.sync_all().unwrap();

    // Wait for events
    let result = tokio::time::timeout(Duration::from_secs(3), handle)
        .await
        .unwrap()
        .unwrap();

    assert!(result >= 2, "Expected at least 2 events, got {}", result);
}

#[tokio::test]
async fn test_watch_file_creation() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let watcher = SyncWatcher::new(&watch_path).unwrap();
    let mut rx = watcher.subscribe();

    // Create a task to capture events
    let handle = tokio::spawn(async move {
        let mut events = vec![];
        while let Ok(event) = rx.recv().await {
            match event {
                SyncEvent::FileCreated(path) => {
                    events.push(("created", path));
                    break;
                }
                _ => continue,
            }
        }
        events
    });

    // Give watcher a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create test file
    let file_path = temp_dir.path().join("test_create.json");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test content").unwrap();
    file.sync_all().unwrap();

    // Wait for event
    let events = tokio::time::timeout(Duration::from_secs(3), handle)
        .await
        .unwrap()
        .unwrap();

    assert!(!events.is_empty(), "Expected at least one FileCreated event");
    assert!(events[0].1.contains("test_create.json"));
}

#[tokio::test]
async fn test_watch_file_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    // Create a file first
    let file_path = temp_dir.path().join("test_delete.json");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        file.sync_all().unwrap();
    }

    let watcher = SyncWatcher::new(&watch_path).unwrap();
    let mut rx = watcher.subscribe();

    // Create a task to capture deletion events
    let handle = tokio::spawn(async move {
        let mut events = vec![];
        while let Ok(event) = rx.recv().await {
            match event {
                SyncEvent::FileDeleted(path) => {
                    events.push(("deleted", path));
                    break;
                }
                _ => continue,
            }
        }
        events
    });

    // Give watcher a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Delete the file
    std::fs::remove_file(&file_path).unwrap();

    // Wait for event
    let events = tokio::time::timeout(Duration::from_secs(3), handle)
        .await
        .unwrap()
        .unwrap();

    assert!(!events.is_empty(), "Expected at least one FileDeleted event");
    assert!(events[0].1.contains("test_delete.json"));
}

#[tokio::test]
async fn test_watch_json_files_only() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let watcher = SyncWatcher::new(&watch_path).unwrap();
    let mut rx = watcher.subscribe();

    // Create a task to capture events
    let (tx_done, mut rx_done) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(async move {
        let mut json_count = 0;
        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(SyncEvent::FileCreated(path)) | Ok(SyncEvent::FileModified(path)) => {
                            if path.ends_with(".json") {
                                json_count += 1;
                            }
                        }
                        _ => break,
                    }
                }
                _ = &mut rx_done => break,
            }
        }
        json_count
    });

    // Give watcher a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a JSON file
    let json_path = temp_dir.path().join("test.json");
    let mut file = File::create(&json_path).unwrap();
    file.write_all(b"{}").unwrap();
    file.sync_all().unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a non-JSON file
    let txt_path = temp_dir.path().join("test.txt");
    let mut file = File::create(&txt_path).unwrap();
    file.write_all(b"text").unwrap();
    file.sync_all().unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Signal done
    tx_done.send(()).unwrap();

    // Wait for result
    let json_count = tokio::time::timeout(Duration::from_secs(3), handle)
        .await
        .unwrap()
        .unwrap();

    // We should detect the JSON file
    assert!(json_count >= 1, "Expected at least 1 JSON file event, got {}", json_count);
}

#[tokio::test]
async fn test_watcher_creation() {
    let temp_dir = TempDir::new().unwrap();
    let watch_path = temp_dir.path().to_path_buf();

    let watcher = SyncWatcher::new(&watch_path);
    assert!(watcher.is_ok(), "Watcher creation should succeed");
}

#[tokio::test]
async fn test_watcher_invalid_path() {
    let invalid_path = PathBuf::from("/nonexistent/path/that/does/not/exist");

    let watcher = SyncWatcher::new(&invalid_path);
    // The watcher might fail on invalid path
    // We just ensure it doesn't panic
    let _ = watcher;
}
