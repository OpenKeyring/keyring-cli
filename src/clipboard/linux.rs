use crate::clipboard::manager::ClipboardManager;
use crate::error::KeyringError;
use std::process::Command;
use std::time::Duration;

pub struct LinuxClipboard;

impl ClipboardManager for LinuxClipboard {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError> {
        // Try xclip first, then xsel as fallback
        let mut child = Command::new("xclip")
            .args(&["-selection", "clipboard", "-i"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;
        }

        child
            .wait()
            .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;

        Ok(())
    }

    fn get_content(&mut self) -> Result<String, KeyringError> {
        let output = Command::new("xclip")
            .args(&["-selection", "clipboard", "-o"])
            .output()
            .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(KeyringError::CommandFailed("xclip failed".to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn clear(&mut self) -> Result<(), KeyringError> {
        // Clear by writing empty string
        self.set_content("")
    }

    fn is_supported(&self) -> bool {
        // Check if xclip is available
        Command::new("which")
            .arg("xclip")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(45) // Linux often has longer clipboard timeouts
    }
}

// Alternative implementation using xsel
pub struct LinuxXselClipboard;

impl ClipboardManager for LinuxXselClipboard {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError> {
        let mut child = Command::new("xsel")
            .args(&["--clipboard", "--input"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;
        }

        child
            .wait()
            .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;

        Ok(())
    }

    fn get_content(&mut self) -> Result<String, KeyringError> {
        let output = Command::new("xsel")
            .args(&["--clipboard", "--output"])
            .output()
            .map_err(|e| KeyringError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(KeyringError::CommandFailed("xsel failed".to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn clear(&mut self) -> Result<(), KeyringError> {
        self.set_content("")
    }

    fn is_supported(&self) -> bool {
        Command::new("which")
            .arg("xsel")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(45)
    }
}
