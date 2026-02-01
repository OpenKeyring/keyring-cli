# Cross-Compilation Guide

This document explains how to use `cross` for cross-platform compilation of keyring-cli.

## Overview

keyring-cli uses **pure Rust dependencies** to enable seamless cross-compilation without C library requirements. This approach eliminates the need for platform-specific C toolchains and simplifies the build process.

### Pure Rust Architecture

The project has been migrated from mixed C/Rust dependencies to pure Rust:

| Old Dependency (C) | New Dependency (Pure Rust) | Purpose |
|-------------------|---------------------------|---------|
| OpenSSL (via reqwest `native-tls-vendored`) | `rustls-tls` + `rustls-tls-native-roots` | TLS/HTTPS |
| libgit2 (via git2 crate) | `gix` (gitoxide) | Git operations |
| libssh2 (via openssh crate) | System SSH calls (`std::process::Command`) | SSH execution |

**Benefits**:
- No C compilation required during cross-compilation
- Faster build times
- Simpler CI/CD pipelines
- Better cross-platform support

## Prerequisites

1. **Docker**: Docker Desktop or OrbStack required
   - macOS: OrbStack recommended (faster) or Docker Desktop
   - Verify: `docker ps`

2. **cross tool**:
   ```bash
   cargo install cross --git https://github.com/cross-rs/cross
   ```
   - Verify installation: `cross --version`

## Quick Start

### Using Makefile (Recommended)

```bash
# Build Linux x86_64
make cross-linux

# Build Linux ARM64
make cross-linux-arm

# Build Windows x86_64 (requires Windows host or GitHub Actions)
make cross-windows

# Build all target platforms
make cross-all

# Run cross-compilation tests
make cross-test
```

### Using cross Directly

```bash
# Build specific targets
cross build --target x86_64-unknown-linux-gnu --release
cross build --target aarch64-unknown-linux-gnu --release
cross build --target x86_64-pc-windows-msvc --release
```

### Using Build Scripts

```bash
# Debug build
./scripts/cross-build.sh debug

# Release build (default)
./scripts/cross-build.sh release
```

Output location: `dist/debug/` or `dist/release/`

## Supported Targets

| Target Triple | Platform | Output Filename | Status |
|--------------|----------|----------------|--------|
| `x86_64-unknown-linux-gnu` | Linux x86_64 | `ok` | ✅ Supported |
| `aarch64-unknown-linux-gnu` | Linux ARM64 | `ok` | ✅ Supported |
| `x86_64-pc-windows-msvc` | Windows x86_64 | `ok.exe` | ✅ Supported* |

**Windows Note**: Windows cross-compilation from macOS has known limitations with the `cross` tool. Recommended approaches:
1. Use GitHub Actions with Windows runners (preferred for production)
2. Build natively on Windows
3. The code is pure Rust and WILL compile on Windows - it's a tooling limitation, not a code limitation

### Build Commands by Target

**Linux x86_64**:
```bash
cross build --target x86_64-unknown-linux-gnu --release
# Output: target/x86_64-unknown-linux-gnu/release/ok
```

**Linux ARM64**:
```bash
cross build --target aarch64-unknown-linux-gnu --release
# Output: target/aarch64-unknown-linux-gnu/release/ok
```

**Windows x86_64**:
```bash
# Option 1: Using cross (may have issues from macOS)
cross build --target x86_64-pc-windows-msvc --release

# Option 2: Native build on Windows
cargo build --target x86_64-pc-windows-msvc --release

# Option 3: GitHub Actions (recommended for production)
# Push to trigger CI/CD pipeline
```

## Architecture Details

### Dependency Migration

The project migrated from C-dependent libraries to pure Rust equivalents:

**Phase 1: reqwest → rustls**
- Before: `reqwest = { features = ["native-tls-vendored"] }` (requires OpenSSL)
- After: `reqwest = { features = ["rustls-tls", "rustls-tls-native-roots"] }`
- Result: No OpenSSL dependency, pure Rust TLS

**Phase 2: openssh → System Calls**
- Before: `openssh` crate (requires libssh2)
- After: `std::process::Command` invoking system `ssh` binary
- Result: Leverages user's SSH configuration, no C dependency

**Phase 3: git2 → gix**
- Before: `git2` crate (requires libgit2)
- After: `gix` (gitoxide) pure Rust Git implementation
- Result: Pure Rust Git operations, full API compatibility

### Verification

To verify pure Rust dependencies:

```bash
# Check for OpenSSL (should return nothing)
cargo tree | grep -i openssl

# Check for git2 (should return nothing)
cargo tree | grep git2

# Check our code doesn't use openssh
grep -r "use openssh" src/
```

## Troubleshooting

### Docker Issues

```bash
# macOS: Ensure OrbStack is running
orb

# Verify Docker is available
docker ps
```

### Image Pull Failures

First run automatically pulls Docker images (~500MB-1GB), which takes time.

Manual pre-pull if needed:
```bash
docker pull ghcr.io/cross/x86_64-unknown-linux-gnu:main
docker pull ghcr.io/cross/aarch64-unknown-linux-gnu:main
docker pull ghcr.io/cross/x86_64-pc-windows-msvc:main
```

## Verifying Builds

After building, verify binaries on target platforms:

```bash
# Check binary type
file target/x86_64-unknown-linux-gnu/release/ok
# Expected: ELF 64-bit LSB pie executable, x86-64

file target/aarch64-unknown-linux-gnu/release/ok
# Expected: ELF 64-bit LSB pie executable, ARM aarch64

file target/x86_64-pc-windows-msvc/release/ok.exe
# Expected: PE32+ executable (console) x86-64, for MS Windows

# Test in Docker (Linux)
docker run --rm -v "$(pwd)/target/x86_64-unknown-linux-gnu/release:/mnt" ubuntu:latest /mnt/ok --version
```

## CI/CD Integration

- **Local Development**: Use `cross` for cross-platform compilation verification
- **Production Builds**: GitHub Actions uses native builds on each platform (faster and more reliable)

Both approaches work independently. Use `cross` for quick local testing.

## Migration Notes

For developers upgrading from the old C-dependent version:

**What Changed**:
1. `reqwest` now uses `rustls-tls` instead of `native-tls-vendored`
2. Git operations use `gix` instead of `git2`
3. SSH executor uses system calls instead of `openssh` crate

**API Compatibility**:
- All public APIs remain unchanged
- No code changes required in consuming applications
- Behavior is identical from user perspective

**Build System**:
- Same Cargo commands work
- Cross-compilation now works without C toolchains
- Windows builds improved (pure Rust)
