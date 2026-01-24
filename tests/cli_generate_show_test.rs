use std::env;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn cli_generate_then_show_decrypts() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    env::set_var("OK_CONFIG_DIR", &config_dir);
    env::set_var("OK_DATA_DIR", &data_dir);
    env::set_var("OK_MASTER_PASSWORD", "test-master-password");

    let ok_bin = env!("CARGO_BIN_EXE_ok");

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

    assert!(generate_output.status.success());
    let generate_stdout = String::from_utf8_lossy(&generate_output.stdout);
    let password_line = generate_stdout
        .lines()
        .find(|line| line.trim_start().starts_with("Password:"))
        .expect("password line missing from generate output");
    let generated_password = password_line
        .splitn(2, ':')
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string();
    assert!(!generated_password.is_empty());

    let show_output = Command::new(&ok_bin)
        .args(["show", "github", "--password"])
        .output()
        .expect("failed to run ok show");

    assert!(show_output.status.success());
    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(
        show_stdout.contains(&generated_password),
        "show output should include decrypted password"
    );
}
