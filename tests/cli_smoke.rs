//! CLI smoke tests - end-to-end workflow verification
//!
//! Tests the basic implemented workflow: init -> gen -> list -> show

#![cfg(feature = "test-env")]

use keyring_cli::onboarding;
use serial_test::serial;
use std::env;
use std::process::Command;
use tempfile::TempDir;

#[serial]
#[test]
fn cli_smoke_flow() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    // Set up test environment using the library helper
    onboarding::setup_test_system(&config_dir, &data_dir, "test-master-password")
        .expect("Failed to set up test system");

    env::set_var("OK_CONFIG_DIR", &config_dir);
    env::set_var("OK_DATA_DIR", &data_dir);
    env::set_var("OK_MASTER_PASSWORD", "test-master-password");

    let ok_bin = env!("CARGO_BIN_EXE_ok");

    // Step 1: Generate a password
    let generate_output = Command::new(ok_bin)
        .args(["new", "--name", "github", "--length", "16"])
        .output()
        .expect("failed to run ok new");

    assert!(
        generate_output.status.success(),
        "new command should succeed. stderr: {}",
        String::from_utf8_lossy(&generate_output.stderr)
    );

    // Step 2: List records
    let list_output = Command::new(ok_bin)
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

    // Step 3: Show record (check name field)
    let show_output = Command::new(ok_bin)
        .args(["show", "github", "--field", "name"])
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
}
