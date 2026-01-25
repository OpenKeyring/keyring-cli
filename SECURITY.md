# Security Policy

## Supported Versions

Currently, only the latest version of OpenKeyring CLI is supported.
Security updates will be provided for the latest release.

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues.**

### How to Report

1. **Email**: Send an email to: security@open-keyring.org
2. **PGP Key**: Use the PGP key below to encrypt sensitive information

### PGP Public Key

```
-----BEGIN PGP PUBLIC KEY BLOCK-----
[TODO: Add actual PGP key before first release]
-----END PGP PUBLIC KEY BLOCK-----
```

### What to Include

- Description of the vulnerability
- Steps to reproduce (if applicable)
- Potential impact
- Suggested mitigation (if known)

### Response Timeline

- **24 hours**: Initial acknowledgment
- **7 days**: Initial assessment and severity rating
- **30 days**: Fix for critical vulnerabilities
- **90 days**: Public disclosure (after fix is released)

### Security Best Practices

1. **Master Password**: Use a strong, unique master password (12+ characters)
2. **Updates**: Keep the software updated to the latest version
3. **Backups**: Maintain encrypted backups of your vault
4. **Recovery Key**: Store your 24-word BIP39 recovery key securely

## Security Features

- **AES-256-GCM**: Industry-standard encryption for stored data
- **Argon2id**: Memory-hard key derivation (resistant to GPU/ASIC attacks)
- **Zero-Knowledge**: All data encrypted locally; cloud storage only sees encrypted blobs
- **Open Source**: Code available for security auditing
