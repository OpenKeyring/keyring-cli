//! CLI smoke tests - end-to-end workflow verification
//!
//! Tests the complete workflow: init -> gen -> list -> show -> update -> search -> delete

use std::env;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn cli_smoke_flow() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    env::set_var("OK_CONFIG_DIR", &config_dir);
    env::set_var("OK_DATA_DIR", &data_dir);
    env::set_var("OK_MASTER_PASSWORD", "test-master-password");

    let ok_bin = env!("CARGO_BIN_EXE_ok");

    // Step 1: Initialize (onboarding should happen automatically on first use)
    // This is implicit when we run the first command

    // Step 2: Generate a password
    let generate_output = Command::new(&ok_bin)
        .args([
            "generate",
            "--name",
            "github",
            "--length",
            "16",
        ])
        .output()
        .expect("failed to run ok generate");

    assert!(
        generate_output.status.success(),
        "generate command should succeed. stderr: {}",
        String::from_utf8_lossy(&generate_output.stderr)
    );

    // Step 3: List records
    let list_output = Command::new(&ok_bin)
        .args(["list"])
        .output()
        .expect("failed to run ok list");

    assert!(
        list_output.status.success(),
        "list command should succeed. stderr: {}",
        String::from_utf8_lossy(&list_output.stderr)
    );

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_stdout.contains("github"),
        "list output should contain 'github'. Output: {}",
        list_stdout
    );

    // Step 4: Show record
    let show_output = Command::new(&ok_bin)
        .args(["show", "github"])
        .output()
        .expect("failed to run ok show");

    assert!(
        show_output.status.success(),
        "show command should succeed. stderr: {}",
        String::from_utf8_lossy(&show_output.stderr)
    );

    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(
        show_stdout.contains("github"),
        "show output should contain 'github'. Output: {}",
        show_stdout
    );

    // Step 5: Update record
    let update_output = Command::new(&ok_bin)
        .args([
            "update",
            "github",
            "--username",
            "test@example.com",
        ])
        .output()
        .expect("failed to run ok update");

    assert!(
        update_output.status.success(),
        "update command should succeed. stderr: {}",
        String::from_utf8_lossy(&update_output.stderr)
    );

    // Verify update worked
    let show_after_update = Command::new(&ok_bin)
        .args(["show", "github"])
        .output()
        .expect("failed to run ok show after update");

    assert!(show_after_update.status.success());
    let show_after_update_stdout = String::from_utf8_lossy(&show_after_update.stdout);
    assert!(
        show_after_update_stdout.contains("test@example.com"),
        "show output after update should contain updated username. Output: {}",
        show_after_update_stdout
    );

    // Step 6: Search records
    let search_output = Command::new(&ok_bin)
        .args(["search", "github"])
        .output()
        .expect("failed to run ok search");

    assert!(
        search_output.status.success(),
        "search command should succeed. stderr: {}",
        String::from_utf8_lossy(&search_output.stderr)
    );

    let search_stdout = String::from_utf8_lossy(&search_output.stdout);
    assert!(
        search_stdout.contains("github"),
        "search output should contain 'github'. Output: {}",
        search_stdout
    );

    // Step 7: Delete record
    let delete_output = Command::new(&ok_bin)
        .args(["delete", "github", "--confirm"])
        .output()
        .expect("failed to run ok delete");

    assert!(
        delete_output.status.success(),
        "delete command should succeed. stderr: {}",
        String::from_utf8_lossy(&delete_output.stderr)
    );

    // Verify deletion worked
    let list_after_delete = Command::new(&ok_bin)
        .args(["list"])
        .output()
        .expect("failed to run ok list after delete");

    assert!(list_after_delete.status.success());
    let list_after_delete_stdout = String::from_utf8_lossy(&list_after_delete.stdout);
    assert!(
        !list_after_delete_stdout.contains("github"),
        "list output after delete should not contain 'github'. Output: {}",
        list_after_delete_stdout
    );
}
