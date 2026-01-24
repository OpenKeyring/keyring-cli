# OpenKeyring CLI

A privacy-first, local-first password manager with cross-platform synchronization.

## Features

- 🔐 **Privacy-First**: All encryption happens locally, zero-knowledge architecture
- 🌍 **Cross-Platform**: macOS, Linux, Windows support
- 📡 **Local-First**: SQLite database stored locally, cloud sync is optional backup
- 🔑 **Strong Crypto**: Argon2id key derivation, AES-256-GCM encryption
- 📋 **Clipboard Integration**: Secure clipboard with auto-clear
- 🔄 **Cloud Sync**: iCloud Drive, Dropbox, Google Drive, OneDrive, WebDAV, SFTP
- 🤖 **AI Integration**: MCP (Model Context Protocol) support for AI assistants

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/open-keyring/keyring-cli.git
cd keyring-cli

# Build the project
cargo build --release

# Install the binary
cargo install --path .
```

### Initial Setup

**First-Time Initialization**

When you run your first command, OpenKeyring automatically initializes:

1. Creates the database at `~/.local/share/open-keyring/passwords.db`
2. Creates the keystore at `~/.config/open-keyring/keystore.json`
3. Prompts for a master password
4. Generates and displays a 24-word BIP39 recovery key

**⚠️ Important**: Save your recovery key securely! It's the only way to recover your data if you forget your master password.

```bash
# First command triggers initialization
ok generate --name "github" --length 16

# You'll see:
# 🔐 Enter master password: [your password]
# 🔑 Recovery Key (save securely): word1 word2 word3 ... word24
# ✅ Password generated successfully
```

**Recovery Key**

The recovery key is a 24-word BIP39 mnemonic phrase that serves as a backup to your master password.

- **When to use**: If you forget your master password or need to restore access on a new device
- **How to save**: Write it down on paper and store securely. Never store it digitally in plain text.
- **Security**: The recovery key is only shown once during initialization. If lost, data cannot be recovered.

**Basic Usage**

```bash
# Generate a password
ok generate --name "github" --length 16

# List all passwords
ok list

# Show a password
ok show "github" --copy

# Update a password
ok update "github" --password "new_password"

# Search passwords
ok search "github"

# Delete a password
ok delete "github" --confirm
```

## CLI Commands

### Password Management

```bash
# Generate passwords
ok generate --name "service" --length 16
ok generate --name "memorable" --memorable --words 4
ok generate --name "pin" --pin --length 6

# List records
ok list
ok list --type "password" --tags "work"
ok list --limit 10

# Show records
ok show "service"
ok show "service" --show-password
ok show "service" --copy

# Update records
ok update "service" --password "new_pass"
ok update "service" --username "user@domain.com" --url "https://example.com"

# Delete records
ok delete "service" --confirm
ok delete "service" --confirm --sync

# Search
ok search "github"
ok search "api" --type "api_credential"
```

### Security Features

```bash
# Password health check
ok health --weak --leaks --duplicate

# Mnemonic (crypto wallet)
ok mnemonic --generate 12 --name "wallet"
ok mnemonic --validate "word1 word2 word3..."

# Device management
ok devices
ok devices remove "device-id"
```

### Sync & Configuration

**Manual Sync**

Sync your passwords across devices using cloud storage:

```bash
# Preview changes (dry run)
ok sync --dry-run

# Full sync
ok sync --full

# Check sync status
ok sync --status

# Sync with specific provider
ok sync --provider "dropbox"
```

**Supported Sync Providers**

- iCloud Drive (default on macOS/iOS)
- Dropbox
- Google Drive
- OneDrive
- WebDAV (self-hosted)
- SFTP (self-hosted)

**Sync Configuration**

```bash
# Enable sync
ok config set sync.enabled true

# Set provider
ok config set sync.provider dropbox

# Set remote path
ok config set sync.remote_path "/OpenKeyring"

# Enable auto-sync
ok config set sync.auto_sync true
```

**Configuration**

```bash
# List all settings
ok config list

# Set configuration values
ok config set "database.path" "/custom/path"
ok config set "sync.enabled" "true"
```

## Architecture

### Core Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Single Device (e.g., MacBook)            │
│  ┌─────────────┐  ┌──────────────────┐  ┌─────────────┐    │
│  │    ok CLI   │  │  OpenKeyring App │  │  ok CLI     │    │
│  │  (Rust)     │  │     (Swift)      │  │  (Rust)     │    │
│  └──────┬──────┘  └────────┬─────────┘  └──────┬──────┘    │
│         └──────────────────┴───────────────────┘           │
│                           ↓                                 │
│              ┌─────────────────────────┐                   │
│              │  Crypto & Storage Core  │                   │
│              │  - Crypto (Argon2id,    │                   │
│              │    AES-256-GCM)         │                   │
│              │  - SQLite + WAL         │                   │
│              │  - Key Hierarchy        │                   │
│              └─────────────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

### Key Hierarchy

```
Master Password (user-provided)
    ↓ Argon2id/PBKDF2 derivation
Master Key (cross-device identical)
    ↓ decrypts wrapped keys
├── Data Encryption Key (DEK) - encrypts actual user data
├── Recovery Key (24-word BIP39) - emergency access
└── Device Key (per-device) - enables biometric unlock
```

### Data Storage

**Local Database** (`~/.local/share/open-keyring/passwords.db`):
- SQLite with WAL mode for concurrent access
- Each record encrypted individually with AES-256-GCM
- Schema: `records`, `tags`, `record_tags`, `metadata`, `sync_state`

**Cloud Sync**:
- Per-record JSON files (not full database sync)
- Format: `{id}.json` containing encrypted data + metadata
- Supported: iCloud Drive, Dropbox, Google Drive, OneDrive, WebDAV, SFTP

## Configuration

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

## Supported Data Types

| Type | Name | Required Fields |
|------|------|-----------------|
| `password` | Basic Password | name, password |
| `ssh_key` | SSH Key | name, host, username, private_key |
| `api_credential` | API Credential | name, api_key |
| `mnemonic` | Crypto Wallet Mnemonic | name, mnemonic (12/24 words) |
| `private_key` | Private Key | name, address, private_key |

All types support optional: `username`, `url`, `notes`, `tags`

## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Project Structure

```
keyring-cli/
├── src/
│   ├── crypto/      # Cryptographic primitives
│   ├── db/          # Database layer
│   ├── clipboard/   # Platform clipboard integration
│   ├── sync/        # Cloud synchronization
│   ├── mcp/         # Model Context Protocol
│   └── cli/         # CLI application
├── tests/           # Test files
└── Cargo.toml       # Dependencies and metadata
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run `cargo test` and `cargo clippy`
6. Submit a pull request

## Security Notes

- All encryption uses industry-standard algorithms (Argon2id, AES-256-GCM)
- Master password is never stored or transmitted
- Clipboard automatically clears after 30 seconds
- Each record is encrypted with a unique nonce
- Database is locked when not in use
- Regular security audits recommended

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Troubleshooting

### Common Issues

**"Master password verification failed"**
- Double-check your password (case-sensitive)
- If forgotten, use recovery key to restore access

**"Database locked"**
- Close other OpenKeyring instances
- Wait a few seconds and try again

**"Keystore not found"**
- If you have a backup, restore `~/.config/open-keyring/keystore.json`
- If no backup, you'll need to reinitialize (data loss)

**"Sync failed"**
- Check internet connection
- Verify sync provider credentials
- Check sync configuration: `ok sync --status`
- Try dry run first: `ok sync --dry-run`

### Recovery Procedures

**Forgot Master Password**
1. Use your recovery key to restore access
2. If recovery key is also lost, data cannot be recovered (by design)

**Lost Device**
1. Use `ok devices` to list trusted devices
2. Remove lost device: `ok devices --remove "device-id"`

For detailed troubleshooting, see [CLI Documentation](docs/cli.md).

## Support

- 📧 Email: support@open-keyring.com
- 💬 Discord: [Join our community](https://discord.gg/openkeyring)
- 🐛 Issues: [GitHub Issues](https://github.com/open-keyring/keyring-cli/issues)
- 📖 Documentation: [CLI Documentation](docs/cli.md)

---

Made with ❤️ by the OpenKeyring Team