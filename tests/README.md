# Testing Guide

This directory contains integration tests for the OpenKeyring CLI application.

## Test Environment Feature

Most tests require the `test-env` feature to be enabled. This feature:
- Uses temporary directories for keyring data instead of the user's actual keyring
- Isolates test environments to prevent interference with your real data
- Enables proper test setup and teardown

## Running Tests

### Run all tests with test-env feature (recommended):
```bash
cargo test
```

This works because `.cargo/config.toml` configures `test` alias to automatically include `--features test-env`.

### Run specific test file:
```bash
cargo test cli_smoke
cargo test clipboard_test
```

### Run tests with output:
```bash
cargo test -- --nocapture
cargo test cli_smoke -- --nocapture
```

### Run tests in a single file:
```bash
cargo test --test cli_smoke
```

## Test Categories

### CLI Tests (`cli_*.rs`)
Functional tests for CLI commands and user workflows:
- `cli_smoke.rs` - Basic smoke tests
- `cli_search_test.rs` - Search functionality
- `cli_delete_test.rs` - Delete operations
- `cli_update_test.rs` - Update operations
- `cli_generate_show_test.rs` - Password generation and display
- `cli_config_test.rs` - Configuration management
- `cli_mnemonic_test.rs` - Mnemonic phrase handling
- `cli_keybindings_test.rs` - Keyboard shortcuts

### Cloud Tests (`cloud_*.rs`)
Cloud synchronization and storage tests:
- `cloud_provider_test.rs` - Provider selection and configuration
- `cloud_storage_test.rs` - Storage operations
- `cloud_metadata_test.rs` - Metadata handling
- `cloud_service_test.rs` - Service integration

### Integration Tests (`integration/`)
Multi-component integration tests.

### MCP Tests (`mcp/`)
Model Context Protocol server tests.

### Unit Tests (`*_test.rs`)
Component-specific unit tests:
- `audit_test.rs` - Security audit functionality
- `change_password_test.rs` - Password change workflows
- `clipboard_test.rs` - Clipboard operations
- `crypto_keystore_test.rs` - Cryptographic key storage
- `diagnostics_integration_test.rs` - System diagnostics
- `tui_*.rs` - Terminal UI component tests

## Test Organization

```
tests/
├── README.md                 # This file
├── CLAUDE.md                 # Testing guidelines for Claude Code
├── cloud/                    # Cloud storage integration tests
├── integration/              # Multi-component integration tests
├── mcp/                      # MCP server tests
├── cli_*.rs                  # CLI command tests
├── cloud_*.rs                # Cloud feature tests
└── *_test.rs                 # Unit tests for specific modules
```

## Writing New Tests

1. Place test files in the `tests/` directory
2. Use the `test-env` feature for setup:
   ```rust
   use keyring_cli::onboarding;

   #[tokio::test]
   async fn test_my_feature() {
       let temp_dir = tempfile::tempdir().unwrap();
       onboarding::test_helper::initialize_minimal_system(&temp_dir).await;
       // Your test code here
   }
   ```

3. Clean up resources in test teardown
4. Use descriptive test names following the pattern `test_<feature>_<scenario>`

## Debugging Failed Tests

To debug a failing test:
```bash
# Run with output
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run only ignored tests
cargo test -- --ignored
```

## Coverage

To generate test coverage reports:
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --features test-env
```
