use std::env;
use std::io::Write;
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
        .args(["generate", "--name", "github", "--length", "16"])
        .output()
        .expect("failed to run ok generate");

    // Print generate output for debugging
    let generate_stderr = String::from_utf8_lossy(&generate_output.stderr);
    let generate_stdout = String::from_utf8_lossy(&generate_output.stdout);
    eprintln!("Generate stderr: {}", generate_stderr);
    eprintln!("Generate stdout: {}", generate_stdout);
    eprintln!("Generate exit code: {:?}", generate_output.status.code());

    assert!(
        generate_output.status.success(),
        "Generate failed: stderr={}, stdout={}",
        generate_stderr,
        generate_stdout
    );

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

    // Run show command with stdin input for confirmation
    let show_process = Command::new(&ok_bin)
        .args(["show", "github", "--field", "password"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn ok show");

    // Write "y" to stdin for confirmation
    if let Some(mut stdin) = show_process.stdin.as_ref() {
        writeln!(stdin, "y").expect("failed to write to stdin");
    }

    let show_output = show_process
        .wait_with_output()
        .expect("failed to read show output");

    assert!(
        show_output.status.success(),
        "show command failed: {}",
        String::from_utf8_lossy(&show_output.stderr)
    );
    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(
        show_stdout.contains(&generated_password),
        "show output should include decrypted password. Got: {}",
        show_stdout
    );
}
