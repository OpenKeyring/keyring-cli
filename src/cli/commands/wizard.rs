//! CLI Wizard Command
//!
//! Interactive command-line wizard for first-time setup of OpenKeyring.

use crate::cli::ConfigManager;
use crate::crypto::passkey::Passkey;
use crate::error::Result;
use crate::onboarding::{is_initialized, initialize_keystore};
use anyhow::anyhow;

/// Wizard command arguments
#[derive(Debug, clap::Parser)]
pub struct WizardArgs {}

/// Run the onboarding wizard
pub async fn run_wizard(_args: WizardArgs) -> Result<()> {
    let config = ConfigManager::new()?;
    let keystore_path = config.get_keystore_path();

    if is_initialized(&keystore_path) {
        println!("✓ Already initialized");
        println!("  Keystore: {}", keystore_path.display());
        return Ok(());
    }

    println!("═══════════════════════════════════════════════════");
    println!("         OpenKeyring 初始化向导");
    println!("═══════════════════════════════════════════════════");
    println!();

    // Step 1: Welcome
    let choice = prompt_choice(
        "选择设置方式:",
        &[
            ("1", "全新使用（生成新的 Passkey）"),
            ("2", "导入已有 Passkey"),
        ],
    )?;

    let _passkey_words = if choice == "1" {
        // Generate new Passkey
        generate_new_passkey()?
    } else {
        // Import existing Passkey
        import_passkey()?
    };

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("         设置主密码");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("💡 此密码仅用于加密 Passkey");
    println!("   与其他设备的密码可以不同");
    println!();

    // Step 3: Master password
    let password = prompt_password("请输入主密码: ")?;
    let confirm = prompt_password("请再次输入主密码: ")?;

    if password != confirm {
        return Err(anyhow!("密码不匹配").into());
    }

    if password.len() < 8 {
        return Err(anyhow!("主密码至少需要 8 个字符").into());
    }

    // Initialize
    let keystore = initialize_keystore(&keystore_path, &password)
        .map_err(|e| anyhow!("Failed to initialize keystore: {}", e))?;

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("✓ 初始化完成");
    println!("═══════════════════════════════════════════════════");
    println!("✓ Keystore: {}", keystore_path.display());
    println!("✓ 恢复密钥: {}", keystore.recovery_key.as_ref().unwrap_or(&"(未生成)".to_string()));
    println!();
    println!("您现在可以开始使用 OpenKeyring 了！");

    Ok(())
}

/// Generate a new Passkey
fn generate_new_passkey() -> Result<Vec<String>> {
    println!("正在生成新的 Passkey...");

    let passkey = Passkey::generate(24)?;
    let words = passkey.to_words();

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("⚠️  请务必保存以下 24 词，这是恢复数据的唯一方式！");
    println!("═══════════════════════════════════════════════════");
    println!();

    for (i, word) in words.iter().enumerate() {
        print!("{:3}. {:<12}", i + 1, word);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════");
    println!();

    let confirmed = prompt_yes_no("已保存此 Passkey？", true)?;

    if !confirmed {
        return Err(anyhow!("必须保存 Passkey 才能继续").into());
    }

    Ok(words)
}

/// Import an existing Passkey
fn import_passkey() -> Result<Vec<String>> {
    println!("请输入您的 24 词 Passkey（用空格分隔）:");
    println!("提示: 输入完成后按 Enter 验证");
    println!();

    let input = prompt_input("> ")?;
    let words: Vec<String> = input.split_whitespace().map(String::from).collect();

    if words.len() != 12 && words.len() != 24 {
        return Err(anyhow!("Passkey 必须是 12 或 24 词（当前：{} 词）", words.len()).into());
    }

    // Validate BIP39 checksum
    Passkey::from_words(&words)
        .map_err(|e| anyhow!("无效的 Passkey: {}", e))?;

    println!("✓ Passkey 验证成功");

    Ok(words)
}

/// Prompt for a choice
fn prompt_choice(prompt: &str, options: &[(&str, &str)]) -> Result<String> {
    println!("{}", prompt);
    for (key, desc) in options {
        println!("  [{}] {}", key, desc);
    }
    println!();

    loop {
        let input = prompt_input(&format!("请输入选择 [{}-{}]: ",
            options.first().map(|(k, _)| *k).unwrap_or("1"),
            options.last().map(|(k, _)| *k).unwrap_or("2")
        ))?;

        if options.iter().any(|(k, _)| *k == input) {
            return Ok(input);
        }

        println!("无效的选择，请重试");
    }
}

/// Prompt for yes/no confirmation
fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    let default_hint = if default { "[Y/n]" } else { "[y/N]" };

    loop {
        let input = prompt_input(&format!("{} {} ", prompt, default_hint))?
            .to_lowercase();

        match input.as_str() {
            "" => return Ok(default),
            "y" | "yes" | "是" => return Ok(true),
            "n" | "no" | "否" => return Ok(false),
            _ => println!("请输入 y/yes/是 或 n/no/否"),
        }
    }
}

/// Prompt for password (hidden input)
fn prompt_password(prompt: &str) -> Result<String> {
    use std::io::Write;

    print!("{}", prompt);
    std::io::stdout().flush()?;

    // Note: In a real terminal, you'd use rpassword or similar
    // For now, we'll use regular input but note that this should be improved
    prompt_input("")
}

/// Prompt for regular input
fn prompt_input(prompt: &str) -> Result<String> {
    use std::io::{self, Write};

    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input)?;

    // Handle EOF (stdin closed or no input available)
    if bytes_read == 0 {
        return Err(anyhow!("No input available (EOF)").into());
    }

    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_args_parse() {
        use clap::Parser;

        let args = WizardArgs::parse_from(&["wizard"]);
        // Just verify it parses
        drop(args);
    }
}
