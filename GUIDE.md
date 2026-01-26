# OpenKeyring CLI User Guide

This guide covers common workflows and best practices for using OpenKeyring CLI (`ok`).

## Table of Contents

1. [Getting Started](#getting-started)
2. [Password Management](#password-management)
3. [Secure Clipboard](#secure-clipboard)
4. [Cloud Synchronization](#cloud-synchronization)
5. [Password Health](#password-health)
6. [Cryptocurrency Wallets](#cryptocurrency-wallets)
7. [SSH Keys](#ssh-keys)
8. [Configuration](#configuration)
9. [Backup & Recovery](#backup--recovery)
10. [Security Best Practices](#security-best-practices)

---

## Getting Started

### First-Time Setup

When you first run `ok`, it will automatically initialize:

```bash
ok generate --name "example" --length 16
```

You'll be prompted to:
1. **Create a master password** - Make it strong and memorable
2. **Save your recovery key** - 24 words that can restore access

**⚠️ Critical**: Write down your recovery key on paper and store it securely. This is your ONLY backup if you forget your master password.

### Your First Password

```bash
# Generate a random password
ok generate --name "github" --length 20

# Generate a memorable password
ok generate --name "wifi" --memorable --words 4
# Example: "correct-horse-battery-staple"

# Generate a PIN
ok generate --name "phone" --pin --length 6
```

### Finding Your Passwords

```bash
# List all passwords
ok list

# Search for a password
ok search "github"

# Show a password (masked)
ok show "github"

# Show password (plaintext, be careful!)
ok show "github" --show-password

# Copy to clipboard (recommended)
ok show "github" --copy
```

---

## Password Management

### Adding Passwords

```bash
# Generate and store a new password
ok generate --name "service" --length 16

# Add an existing password
ok add --name "bank" --password "MyP@ssw0rd" \
    --username "john@example.com" \
    --url "https://bank.com"
```

### Organizing with Tags

```bash
# Add tags when creating
ok generate --name "work-github" --length 16 --tags "work,git"

# Add tags later
ok update "github" --add-tags "social,dev"

# List by tags
ok list --tags "work"
ok list --tags "dev,social"
```

### Editing Records

```bash
# Change password
ok update "github" --password "new_password"

# Update metadata
ok update "github" \
    --username "newemail@example.com" \
    --url "https://github.com/newuser" \
    --notes "Personal account"

# Remove tags
ok update "github" --remove-tags "old-tag"
```

### Deleting Records

```bash
# Delete with confirmation
ok delete "github" --confirm

# Delete and sync to cloud
ok delete "github" --confirm --sync
```

---

## Secure Clipboard

### Basic Usage

```bash
# Copy password to clipboard (auto-clears after 30s)
ok show "github" --copy

# Copy with custom timeout
ok show "github" --copy --timeout 60
```

### How It Works

1. Password is copied to clipboard
2. Timer starts (default: 30 seconds)
3. Clipboard is automatically cleared:
   - **clear mode**: Clipboard set to empty string
   - **restore mode**: Original content restored (if detected)
4. Optional notification shown when cleared

### Configuration

```bash
# Set timeout (10-300 seconds)
ok config set clipboard.timeout 45

# Enable smart restore
ok config set clipboard.on_copy_action restore

# Disable notifications
ok config set clipboard.show_notification false
```

### Platform Support

| Platform | Clipboard Tool |
|----------|---------------|
| macOS | `pbcopy` / `pbpaste` |
| Linux (X11) | `xclip` / `xsel` |
| Linux (Wayland) | `wl-copy` / `wl-paste` |
| Windows | Win32 API |

---

## Cloud Synchronization

### Setting Up Sync

```bash
# Enable sync
ok config set sync.enabled true

# Choose provider
ok config set sync.provider dropbox

# Set remote path
ok config set sync.remote_path "/OpenKeyring"

# Enable auto-sync (after add/update/delete)
ok config set sync.auto_sync true
```

### Supported Providers

| Provider | Config Key | Notes |
|----------|-----------|-------|
| iCloud Drive | `icloud` | Default on macOS/iOS |
| Dropbox | `dropbox` | Requires OAuth setup |
| Google Drive | `google` | Requires OAuth setup |
| OneDrive | `onedrive` | Requires OAuth setup |
| WebDAV | `webdav` | Self-hosted |
| SFTP | `sftp` | Self-hosted |

### Manual Sync

```bash
# Preview changes (recommended first)
ok sync --dry-run

# Full sync
ok sync --full

# Sync specific provider
ok sync --provider dropbox

# Check sync status
ok sync --status
```

### Understanding Sync Status

```bash
ok sync --status
```

Output:
```
Sync Status: Last sync 5 minutes ago
Local: 42 records (3 modified since last sync)
Remote: 42 records
Conflicts: 0
```

### Conflict Resolution

When conflicts occur:
1. **Timestamp comparison**: Newer record wins
2. **Version number**: Higher version wins
3. **User-initiated priority**: Manual sync > background sync
4. **Interactive prompt**: If still unresolved

```bash
# Set conflict resolution strategy
ok config set sync.conflict_resolution newer  # or: newer, older, manual
```

### Sync Best Practices

1. **Always dry-run first**: `ok sync --dry-run`
2. **Enable auto-sync**: `ok config set sync.auto_sync true`
3. **Check status regularly**: `ok sync --status`
4. **Test after setup**: Use `--dry-run` before first full sync

---

## Password Health

### Checking Password Strength

```bash
# Check for weak passwords
ok health --weak

# Check for leaked passwords (requires internet)
ok health --leaks

# Check for duplicate passwords
ok health --duplicate

# Check everything
ok health --leaks --weak --duplicate
```

### Understanding the Report

```
Password Health Report
======================

Weak Passwords (3):
  - old-site: Uses only lowercase letters
  - wifi: Only 8 characters

Leaked Passwords (1):
  - linkedin: Found in 2021 LinkedIn breach

Duplicate Passwords (2):
  - "P@ssw0rd123" used in:
    - site1
    - site2
```

### Taking Action

```bash
# Regenerate weak passwords
ok update "old-site" --password $(ok gen-random --length 20)

# Update leaked passwords immediately
ok update "linkedin" --password $(ok gen-random --length 24)

# Fix duplicates
ok update "site1" --password $(ok gen-random --length 16)
ok update "site2" --password $(ok gen-random --length 16)
```

---

## Cryptocurrency Wallets

### Storing Mnemonic Phrases

```bash
# Generate and store a 12-word mnemonic
ok mnemonic generate --words 12 --name "hot-wallet"

# Generate and store a 24-word mnemonic
ok mnemonic generate --words 24 --name "cold-wallet"

# Store existing mnemonic
ok mnemonic add --name "ledger" \
    "abandon abandon abandon abandon abandon abandon \
     abandon abandon abandon abandon abandon about"
```

### Validating Mnemonics

```bash
# Validate BIP39 mnemonic
ok mnemonic validate \
    "abandon abandon abandon abandon abandon abandon \
     abandon abandon abandon abandon abandon about"
```

### Retrieving Mnemonics

```bash
# Show mnemonic (prompt required)
ok mnemonic show "cold-wallet"

# Copy to clipboard (auto-clears)
ok mnemonic show "hot-wallet" --copy
```

### Security Notes

⚠️ **Critical Security Practices**:

1. **Never store hot wallet mnemonics digitally** if possible
2. **Use hardware wallets** for significant holdings
3. **Write down cold wallet mnemonics on paper**
4. **Never share photos of your mnemonic phrase**
5. **Verify address before sending funds**

---

## SSH Keys

### Adding SSH Keys

```bash
# Add SSH key
ok ssh add --name "github-server" \
    --host "github.com" \
    --username "git" \
    --private-key "~/.ssh/id_rsa"

# Add with public key
ok ssh add --name "aws-server" \
    --host "aws.example.com" \
    --username "ubuntu" \
    --private-key "~/.ssh/aws.pem" \
    --public-key "~/.ssh/aws.pub"
```

### Using SSH Keys

```bash
# Copy private key to clipboard
ok ssh copy "github-server"

# Copy with connection command
ok ssh copy "github-server" --command

# Shows: ssh -i ~/.ssh/id_rsa git@github.com
```

### Listing SSH Keys

```bash
# List all SSH keys
ok ssh list

# Filter by host
ok ssh list --host "github.com"
```

---

## Configuration

### Configuration File

Location: `~/.config/open-keyring/config.yaml`

```yaml
# Database
database:
  path: "~/.local/share/open-keyring/passwords.db"
  auto_lock_timeout: 300  # 5 minutes

# Cryptography
crypto:
  key_derivation: "argon2id"
  argon2id_params:
    time: 3
    memory: 67108864  # 64MB
    parallelism: 2

# Sync
sync:
  enabled: true
  provider: "dropbox"
  remote_path: "/OpenKeyring"
  auto_sync: true
  conflict_resolution: "newer"

# Clipboard
clipboard:
  timeout: 30
  on_copy_action: "clear"  # or "restore"
  show_notification: true

# Security
security:
  require_master_password: true
  auto_lock_on_inactivity: true
  clipboard_clear_on_exit: true
```

### CLI Configuration

```bash
# View all configuration
ok config list

# Set specific values
ok config set database.auto_lock_timeout 600
ok config set clipboard.timeout 45
ok config set sync.auto_sync true
```

---

## Backup & Recovery

### Creating Backups

```bash
# Manual backup (encrypted)
ok backup --file "~/backup/openkeyring-$(date +%Y%m%d).json"

# Include database and keystore
ok backup --file "~/backup/full-backup.tar.gz" --full

# Cloud backup (via sync)
ok sync --full
```

### Restoring from Backup

```bash
# Restore from backup file
ok restore --file "~/backup/openkeyring-20240126.json"

# Restore full backup
ok restore --file "~/backup/full-backup.tar.gz" --full
```

### Using Recovery Key

If you forget your master password:

```bash
ok recovery --restore
```

You'll be prompted for your 24-word recovery key.

**⚠️ Important**: After using recovery key:
- Consider changing your master password
- Re-encrypt your database with new password

### Emergency: Reset Everything

If you lose both master password and recovery key, your data is permanently inaccessible. To start fresh:

```bash
ok reset --confirm-i-understand-data-loss
```

**Warning**: This permanently deletes all data.

---

## Security Best Practices

### Master Password

✅ **DO**:
- Use 12+ characters
- Mix uppercase, lowercase, numbers, symbols
- Use a passphrase (easier to remember)
- Never reuse across services

❌ **DON'T**:
- Use common words or patterns
- Share with anyone
- Store in plain text
- Reuse from other services

### Recovery Key

✅ **DO**:
- Write down on paper
- Store in secure location (safe, locked drawer)
- Make copies (2-3)
- Consider safety deposit box for critical data

❌ **DON'T**:
- Store digitally (notes app, screenshots)
- Share photos of it
- Keep it with your device
- Assume you'll remember it

### Daily Usage

✅ **DO**:
- Use clipboard feature (`--copy`)
- Enable auto-sync
- Run password health checks regularly
- Lock database when not in use

❌ **DON'T**:
- Use `--show-password` unnecessarily
- Leave passwords in clipboard
- Ignore sync conflicts
- Skip password health checks

### Environment Security

✅ **DO**:
- Keep system updated
- Use antivirus on Windows
- Enable disk encryption (FileVault, BitLocker)
- Use screen lock

❌ **DON'T**:
- Use on shared/public devices
- Leave device unlocked
- Ignore security updates
- Disable screen lock

### Cloud Sync

✅ **DO**:
- Enable 2FA on cloud provider
- Use `--dry-run` before full sync
- Check sync status regularly
- Understand conflict resolution

❌ **DON'T**:
- Sync to untrusted cloud
- Ignore sync errors
- Skip `--dry-run` testing
- Use weak cloud password

---

## Advanced Usage

### Custom Password Generation

```bash
# Generate without storing
ok gen-random --length 20 --symbols --numbers

# Generate memorable passphrase
ok gen-memorable --words 5 --separator "-"

# Generate PIN
ok gen-pin --length 6
```

### Batch Operations

```bash
# Import from CSV
ok import --format csv --file passwords.csv

# Export to encrypted JSON
ok export --file backup.json --encrypt

# Export to CSV (use with caution!)
ok export --file passwords.csv --format csv --unencrypt
```

### Scripting

```bash
# Get password for scripting (use with caution)
PASSWORD=$(ok show "service" --raw)

# Copy to specific clipboard (Linux)
ok show "service" --copy --clipboard-command "wl-copy"

# Sync on schedule (cron)
0 */6 * * * /usr/bin/ok sync --full
```

---

## Troubleshooting

### Common Issues

**Master password not accepted**
- Check caps lock and keyboard layout
- Try recovery key: `ok recovery --restore`

**Database locked**
- Wait 60 seconds for auto-unlock
- Force unlock: `ok db --unlock`

**Sync authentication failed**
- Re-authenticate: `ok sync --reauth`
- Check cloud provider credentials

**Clipboard not clearing**
- Check configuration: `ok config list | grep clipboard`
- Verify clipboard tool is installed

### Getting Help

```bash
# General help
ok --help

# Command-specific help
ok generate --help
ok sync --help

# Version info
ok --version
```

### Support Resources

- 📖 [Documentation](README.md)
- ❓ [FAQ](FAQ.md)
- 🐛 [GitHub Issues](https://github.com/open-keyring/keyring-cli/issues)
- 💬 [Discord Community](https://discord.gg/openkeyring)
- 📧 Email: support@open-keyring.com

---

## Summary

OpenKeyring provides a secure, local-first password management solution with flexible cloud sync. Key takeaways:

1. **Save your recovery key** - It's your only backup
2. **Use the clipboard** - Safer than `--show-password`
3. **Enable auto-sync** - Keep devices up to date
4. **Run health checks** - Ensure password security
5. **Follow best practices** - Protect your digital identity

For the latest updates and features, check the [GitHub repository](https://github.com/open-keyring/keyring-cli).
