# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial CLI framework with password generation commands
- Random, memorable, and PIN password generation
- AES-256-GCM encryption for stored passwords
- Argon2id key derivation with dynamic parameter adjustment
- SQLite database with WAL mode for local storage
- BIP39 mnemonic support for wallet management
- Password health checking (HIBP integration, strength analysis)
- Clipboard management with auto-clear functionality
- Cross-platform support (macOS, Linux, Windows)

### Changed
- Migrated from stub implementations to actual command logic

### Fixed
- Resolved CLI parameter conflict (-c option between copy and config)

## [0.1.0] - 2026-01-XX

### Added
- First public release
- Core password management functionality
- Local-first architecture with zero-knowledge encryption
- Open source under MIT license
