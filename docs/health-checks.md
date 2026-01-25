# Password Health Checks

OpenKeyring includes comprehensive password health checks to help you maintain strong security practices.

## Features

### 1. Weak Password Detection

Analyzes password strength using:
- Length analysis (12+ characters recommended)
- Character variety (lowercase, uppercase, digits, symbols)
- Pattern detection (sequential/repeated characters)
- Common pattern detection ("password", "qwerty", etc.)

### 2. Duplicate Password Detection

Finds passwords used across multiple accounts. Using the same password everywhere increases your risk - if one account is compromised, all accounts are at risk.

### 3. Compromised Password Detection

Checks passwords against the [Have I Been Pwned (HIBP)](https://haveibeenpwned.com) database using the k-anonymity model. Your full password is NEVER sent to the server - only the first 5 characters of the SHA-1 hash.

## Usage

```bash
# Run all health checks
ok health --all

# Run specific checks
ok health --weak          # Check for weak passwords
ok health --duplicate     # Check for duplicate passwords
ok health --leaks         # Check for compromised passwords

# Combine checks
ok health --weak --duplicate
```

## Example Output

```
🩺 Running password health check...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Checking 15 records...

Health Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total records checked: 15
Weak passwords:       2
Duplicate passwords:  3
Compromised:          1

Critical Issues:
  🔴 compromised-account - Password found in data breach

High Issues:
  🟠 weak-gmail, weak-facebook - Weak password (strength score: 25/100)

Medium Issues:
  🟡 shared-1, shared-2, shared-3 - Password used by 3 accounts

Recommendations:
  • Change compromised passwords immediately!
  • Update weak passwords to improve security
    Use: ok generate random -n <name> -l 16
  • Use unique passwords for each account
    Use: ok generate random -n <name> -l 20
```

## Scoring

Password strength is scored from 0-100:

| Score | Rating |
|-------|--------|
| 0-39  | Very Weak |
| 40-59 | Weak |
| 60-79 | Medium |
| 80-100 | Strong |

**Scoring factors:**
- Length: Up to 40 points (12+ chars recommended)
- Character variety: Up to 30 points (all 4 types recommended)
- Penalties for common patterns, sequential chars, repeated chars
- Bonuses for length > 16 chars and high unique character ratio

## Privacy Note

The compromised password check uses the HIBP k-anonymity model:
1. Your password is hashed using SHA-1
2. Only the first 5 characters of the hash are sent to HIBP
3. The remaining characters are compared locally
4. Your full password is NEVER transmitted or stored

## Architecture

The health check system is modular with pluggable checkers:

```
src/health/
├── mod.rs       # Main module exports
├── checker.rs   # HealthChecker orchestrator
├── strength.rs  # Password strength algorithm
├── hibp.rs      # HIBP API integration
└── report.rs    # HealthIssue, HealthReport types
```

**Key types:**
- `HealthChecker`: Main orchestrator with configurable checks
- `HealthIssue`: Individual issue found during checks
- `HealthReport`: Aggregated report with counts and recommendations
- `HealthIssueType`: WeakPassword, DuplicatePassword, CompromisedPassword
- `Severity`: Low, Medium, High, Critical

## API Usage

```rust
use keyring_cli::health::{HealthChecker, HealthReport};
use keyring_cli::crypto::CryptoManager;

// Initialize crypto
let mut crypto = CryptoManager::new();
crypto.initialize("master-password")?;

// Create checker with specific checks enabled
let checker = HealthChecker::new(crypto)
    .with_weak(true)
    .with_duplicates(true)
    .with_leaks(true);

// Run checks
let issues = checker.check_all(&records).await;

// Generate report
let report = HealthReport::from_issues(records.len(), issues);

if !report.is_healthy() {
    println!("Found {} issues", report.issues.len());
}
```

## Testing

Run health check tests:

```bash
# Unit tests
cargo test --lib health

# Integration tests
cargo test --test health_integration

# All health tests
cargo test health
```
