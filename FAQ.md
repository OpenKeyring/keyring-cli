# Frequently Asked Questions (FAQ)

## General Questions

### What is OpenKeyring?

OpenKeyring is a privacy-first, local-first password manager with cross-platform synchronization. It stores your passwords securely in an encrypted local database and optionally syncs them across devices using cloud storage.

### Is OpenKeyring free?

Yes, the CLI version (`ok`) is completely free and open-source (MIT licensed). Future GUI apps for iOS/macOS will be paid one-time purchases.

### What platforms are supported?

- **CLI**: macOS, Linux, Windows
- **Future GUI apps**: iOS, macOS, Windows, Android, HarmonyOS

### How is this different from Bitwarden/1Password?

| Feature | OpenKeyring | Bitwarden | 1Password |
|---------|-------------|-----------|-----------|
| Zero-Knowledge | ✅ | ✅ | ✅ |
| Local-First | ✅ | ❌ (cloud required) | ❌ (cloud required) |
| Open Source | ✅ (CLI) | ✅ | ❌ |
| Self-Host Sync | ✅ | ✅ | ❌ |
| Pricing | Free CLI | Paid tiers | Subscription |

## Security

### How secure is OpenKeyring?

OpenKeyring uses industry-standard cryptographic primitives:
- **Key Derivation**: Argon2id (winner of Password Hashing Competition 2015)
- **Encryption**: AES-256-GCM (authenticated encryption)
- **Random Numbers**: Operating system's CSPRNG

### Where is my master password stored?

Your master password is **never stored** anywhere. It's used to derive your Master Key via Argon2id, and then immediately discarded from memory. The derived key decrypts your Data Encryption Key (DEK).

### What happens if I forget my master password?

You have **one recovery option**: your 24-word BIP39 recovery key that was shown during initialization.

- If you have the recovery key, you can restore access
- If you lose both your master password AND recovery key, your data is **permanently inaccessible** (by design - this is zero-knowledge architecture)

### Can OpenKeyring developers access my passwords?

**No.** This is zero-knowledge architecture:
- All encryption happens locally on your device
- Cloud storage only receives encrypted blobs
- The master password never leaves your device

### Has OpenKeyring been audited?

The CLI is open-source and available for community review. We plan to:
- ✅ Enable GitHub Advanced Security (Dependabot, CodeQL, Secret Scanning)
- 📅 Submit to OSS-Fuzz for continuous fuzzing (v0.2)
- 📅 Academic collaboration for formal review (v0.3)
- 📅 Third-party audit for v1.0 release

See [`docs/安全规划.md`](../docs/安全规划.md) for details.

### What if my device is stolen?

Your data remains encrypted in the local database. Without your master password or recovery key, the thief cannot access your passwords.

**Recommended action**: Use `ok devices` to revoke the stolen device if you have cloud sync enabled.

## Sync & Backup

### How does cloud sync work?

OpenKeyring uses **file-based sync** (not database sync):
1. Each record is stored as an individual encrypted JSON file: `{id}.json`
2. Files are uploaded to your cloud storage (iCloud, Dropbox, etc.)
3. Other devices download and merge changes

### Which cloud providers are supported?

- iCloud Drive (default on macOS/iOS)
- Dropbox
- Google Drive
- OneDrive
- WebDAV (self-hosted)
- SFTP (self-hosted)

### Is sync encrypted?

**Yes, end-to-end encrypted.** Cloud providers only store encrypted JSON files. They cannot read your data.

### What happens during sync conflicts?

OpenKeyring resolves conflicts using:
1. **Timestamp comparison**: Newer version wins
2. **Version numbers**: Higher version wins
3. **Device priority**: User-initiated > background sync
4. **Manual resolution**: Prompted if automatic resolution fails

### How do I backup my data?

**Automatic backups** (cloud sync enabled):
```bash
ok sync --full
```

**Manual backup**:
```bash
# Export database
cp ~/.local/share/open-keyring/passwords.db ~/backup/

# Export keystore (critical!)
cp ~/.config/open-keyring/keystore.json ~/backup/
```

### How do I restore from backup?

1. Stop OpenKeyring
2. Restore database and keystore:
   ```bash
   cp ~/backup/passwords.db ~/.local/share/open-keyring/
   cp ~/backup/keystore.json ~/.config/open-keyring/
   ```
3. Restart OpenKeyring

## Usage

### How do I change my master password?

```bash
ok config set master_password.change
```

You'll be prompted for your current password and new password.

**Warning**: This re-encrypts your entire database. For large databases (>1000 records), this may take several minutes.

### Can I use OpenKeyring offline?

**Yes, fully functional offline.** OpenKeyring is local-first:
- All operations work without internet
- Cloud sync only requires internet when explicitly syncing

### How do I migrate from another password manager?

OpenKeyring supports importing from:
- Bitwarden (encrypted JSON export)
- 1Password (unencrypted export)
- KeePass (KDBX databases)
- LastPass (CSV export)

```bash
ok import --format bitwarden --file export.json
```

### How do I export my data?

```bash
# Encrypted backup
ok export --file backup.json --encrypt

# Unencrypted CSV (not recommended)
ok export --file passwords.csv --format csv
```

### Can I use OpenKeyring with tools like `pass`?

Not directly, but you can export to encrypted JSON and write a custom script. Future versions may include a `pass` compatibility mode.

## Clipboard

### How does the secure clipboard work?

When you run `ok show <name> --copy`:
1. Password is copied to clipboard
2. Timer starts (default 30 seconds)
3. Clipboard is automatically cleared after timeout
4. Optional notification when cleared

### Is the clipboard secure?

**Reasonably secure, but not perfect.** Clipboard data can be accessed by:
- Other apps on your system
- Malware

**Mitigations**:
- Auto-clear after 30 seconds
- Smart content change detection
- Clipboard is cleared on exit

For maximum security, use `ok show <name>` and type manually.

## Troubleshooting

### "Master password verification failed"

- Check caps lock
- Verify keyboard layout
- If forgotten, use recovery key: `ok recovery --restore`

### "Database locked"

- Another OpenKeyring instance is running
- Wait 60 seconds for automatic unlock
- Or force unlock: `ok db --unlock` (risky if other instance is active)

### "Keystore not found"

- Restore from backup if available
- If no backup, you must reinitialize (data loss)

### "Sync failed: authentication error"

- Verify cloud provider credentials
- Re-authenticate: `ok sync --reauth`
- Check sync configuration: `ok config list | grep sync`

### Why is key derivation slow?

Argon2id is intentionally slow (300-500ms on typical hardware) to resist brute-force attacks.

You can adjust parameters in `~/.config/open-keyring/config.yaml`:
```yaml
crypto:
  argon2id_params:
    time: 2        # Reduce from 3 to 2 (less secure, faster)
    memory: 33554432  # Reduce from 64MB to 32MB
```

**Warning**: Reducing parameters reduces security.

## Development

### How do I build from source?

```bash
git clone https://github.com/open-keyring/keyring-cli.git
cd keyring-cli
cargo build --release
cargo install --path .
```

### How do I contribute?

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines.

### Is there an API/MCP server?

Yes! OpenKeyring includes an MCP (Model Context Protocol) server for AI assistants:

```bash
ok mcp start
ok mcp status
ok mcp logs
```

See [README.md - MCP Section](README.md#mcp-service-model-context-protocol) for details.

## Legal & Licensing

### What license is OpenKeyring under?

MIT License. See [`LICENSE`](LICENSE) or [`COPYING`](COPYING) for details.

### Can I use OpenKeyring in my company?

Yes, MIT license allows commercial use. However:
- No warranty or liability
- Self-hosted deployment recommended for sensitive data
- Consider enterprise support for compliance requirements

### Is OpenKeyring GDPR compliant?

As local-first software, OpenKeyring minimizes data processing:
- All data stored locally
- No telemetry by default
- You control your data

For corporate deployment, consult your legal team.

## Still Have Questions?

- 📧 Email: support@open-keyring.com
- 💬 Discord: [Join our community](https://discord.gg/openkeyring)
- 🐛 Issues: [GitHub Issues](https://github.com/open-keyring/keyring-cli/issues)
- 📖 Documentation: [CLI Documentation](docs/cli.md)
