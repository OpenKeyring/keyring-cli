# OpenKeyring CLI Documentation

Complete guide to using the OpenKeyring CLI (`ok`) command-line tool.

## Table of Contents

- [Initial Setup](#initial-setup)
- [Password Management](#password-management)
- [Security Features](#security-features)
- [Sync Operations](#sync-operations)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

## Initial Setup

### First-Time Initialization

When you run your first command, OpenKeyring will automatically initialize:

1. **Database Creation**: Creates the SQLite database at `~/.local/share/open-keyring/passwords.db`
2. **Keystore Initialization**: Creates the keystore file at `~/.config/open-keyring/keystore.json`
3. **Master Password**: You'll be prompted to enter a master password
4. **Recovery Key**: A 24-word BIP39 recovery key will be generated and displayed

**Important**: Save your recovery key securely! It's the only way to recover your data if you forget your master password.

```bash
# First command triggers initialization
$ ok generate --name "github" --length 16

🔐 Enter master password: [your password]
🔑 Recovery Key (save securely): word1 word2 word3 ... word24
✅ Password generated successfully
```

### Recovery Key

The recovery key is a 24-word BIP39 mnemonic phrase that serves as a backup to your master password.

**When to use recovery key:**
- You forgot your master password
- You need to restore access on a new device
- Emergency recovery scenarios

**How to save recovery key:**
- Write it down on paper and store securely
- Use a secure password manager (separate from OpenKeyring)
- Store in a secure location (safe, encrypted file, etc.)
- **Never store it digitally in plain text**

## Password Management

### Generate Passwords

Generate secure random passwords:

```bash
# Basic random password (16 characters)
ok generate --name "github" --length 16

# Custom length
ok generate --name "email" --length 32

# Memorable password (word-based)
ok generate --name "wifi" --memorable --words 4

# PIN code
ok generate --name "atm" --pin --length 6

# With additional metadata
ok generate --name "github" \
  --username "user@example.com" \
  --url "https://github.com" \
  --tags "work,dev" \
  --notes "GitHub account"
```

### List Records

View all your stored passwords:

```bash
# List all records
ok list

# Filter by type
ok list --type password

# Filter by tags
ok list --tags work,dev

# Limit results
ok list --limit 10
```

### Show Records

Display password details:

```bash
# Show record (password hidden)
ok show "github"

# Show password in plain text
ok show "github" --password

# Copy password to clipboard
ok show "github" --copy

# Show specific field
ok show "github" --field username
```

### Update Records

Modify existing passwords:

```bash
# Update password
ok update "github" --password "new_password"

# Update username
ok update "github" --username "newuser@example.com"

# Update URL
ok update "github" --url "https://github.com/new"

# Add tags
ok update "github" --add-tags "personal"

# Remove tags
ok update "github" --remove-tags "work"

# Sync after update
ok update "github" --password "new_pass" --sync
```

### Search Records

Search across all fields:

```bash
# Basic search
ok search "github"

# Search by type
ok search "api" --type api_credential

# Search is case-insensitive and matches partial strings
ok search "git"
```

### Delete Records

Remove passwords:

```bash
# Delete with confirmation
ok delete "github" --confirm

# Delete and sync
ok delete "github" --confirm --sync
```

## Security Features

### Password Health Check

Analyze password strength and security:

```bash
# Check for weak passwords
ok health --weak

# Check for leaked passwords (requires internet)
ok health --leaks

# Check for duplicate passwords
ok health --duplicate

# Full health check
ok health --weak --leaks --duplicate
```

### Mnemonic (Crypto Wallet)

Manage cryptocurrency wallet mnemonics:

```bash
# Generate 12-word mnemonic
ok mnemonic --generate 12 --name "wallet1"

# Generate 24-word mnemonic
ok mnemonic --generate 24 --name "wallet2"

# Validate mnemonic phrase
ok mnemonic --validate "word1 word2 word3 ... word12"
```

### Device Management

Manage trusted devices:

```bash
# List all devices
ok devices

# Remove a device
ok devices --remove "device-id"
```

## Sync Operations

### Manual Sync

Sync your passwords across devices:

```bash
# Dry run (preview changes)
ok sync --dry-run

# Full sync
ok sync --full

# Check sync status
ok sync --status

# Sync with specific provider
ok sync --provider dropbox
```

### Sync Providers

Supported sync providers:

- **iCloud Drive** (default on macOS/iOS)
- **Dropbox**
- **Google Drive**
- **OneDrive**
- **WebDAV** (self-hosted)
- **SFTP** (self-hosted)

### Sync Configuration

Configure sync settings:

```bash
# Enable sync
ok config set sync.enabled true

# Set provider
ok config set sync.provider dropbox

# Set remote path
ok config set sync.remote_path "/OpenKeyring"

# Enable auto-sync
ok config set sync.auto_sync true

# Set conflict resolution
ok config set sync.conflict_resolution newer
```

## Configuration

### View Configuration

```bash
# List all settings
ok config list

# View specific setting
ok config get sync.enabled
```

### Configuration File

Configuration is stored in YAML format at `~/.config/open-keyring/config.yaml`:

```yaml
database:
  path: "~/.local/share/open-keyring/passwords.db"
  encryption_enabled: true

crypto:
  key_derivation: "argon2id"
  argon2id_params:
    time: 3
    memory: 67108864  # 64MB
    parallelism: 2
  pbkdf2_iterations: 600000

sync:
  enabled: false
  provider: "icloud"
  remote_path: "/OpenKeyring"
  auto_sync: false
  conflict_resolution: "newer"

clipboard:
  timeout_seconds: 30
  clear_after_copy: true
  max_content_length: 1024
```

### Environment Variables

Override default paths:

```bash
# Custom config directory
export OK_CONFIG_DIR="/custom/config/path"

# Custom data directory
export OK_DATA_DIR="/custom/data/path"

# Master password (for automation, not recommended)
export OK_MASTER_PASSWORD="your-password"
```

## Troubleshooting

### Common Issues

#### "Master password verification failed"

**Cause**: Incorrect master password entered.

**Solution**:
- Double-check your password (case-sensitive)
- If forgotten, use recovery key to restore access

#### "Database locked"

**Cause**: Another process is using the database.

**Solution**:
- Close other OpenKeyring instances
- Wait a few seconds and try again
- Check for background processes: `ps aux | grep ok`

#### "Keystore not found"

**Cause**: Keystore file is missing or corrupted.

**Solution**:
- If you have a backup, restore `~/.config/open-keyring/keystore.json`
- If no backup, you'll need to reinitialize (data loss)

#### "Sync failed"

**Cause**: Network issue or provider configuration error.

**Solution**:
- Check internet connection
- Verify sync provider credentials
- Check sync configuration: `ok sync --status`
- Try dry run first: `ok sync --dry-run`

#### "Recovery key invalid"

**Cause**: Incorrect recovery key entered.

**Solution**:
- Verify all 24 words are correct
- Check for typos
- Ensure words are in correct order
- Use BIP39 wordlist to verify words

### Recovery Procedures

#### Forgot Master Password

1. Use your recovery key to restore access
2. If recovery key is also lost, data cannot be recovered (by design)

#### Corrupted Database

1. Stop all OpenKeyring processes
2. Restore from backup if available
3. If no backup, database may need to be recreated

#### Lost Device

1. Use `ok devices` to list trusted devices
2. Remove lost device: `ok devices --remove "device-id"`
3. This invalidates sessions from that device

### Getting Help

- **Documentation**: See `README.md` for architecture details
- **Issues**: Report bugs on [GitHub Issues](https://github.com/open-keyring/keyring-cli/issues)
- **Community**: Join [Discord](https://discord.gg/openkeyring)

## Best Practices

1. **Regular Backups**: Export your passwords regularly: `ok export --output backup.json`
2. **Strong Master Password**: Use a long, unique master password
3. **Save Recovery Key**: Store recovery key securely offline
4. **Enable Sync**: Use cloud sync for backup (optional but recommended)
5. **Health Checks**: Regularly run `ok health` to check password strength
6. **Device Management**: Remove unused devices regularly
7. **Update Regularly**: Keep CLI tool updated for security patches

## Security Notes

- All encryption happens locally before sync
- Master password is never stored or transmitted
- Recovery key is only shown once during initialization
- Clipboard automatically clears after 30 seconds
- Each record is encrypted with a unique nonce
- Database is locked when not in use

---

For more information, see the [main README](../README.md).
