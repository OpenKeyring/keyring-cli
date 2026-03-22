//! MCP Authentication and Authorization
//!
//! This module provides confirmation tokens and related authentication
//! utilities for the MCP (Model Context Protocol) server.

// Note: policy.rs file contains the main policy implementation
// The module inception warning is acceptable here as it provides
// a cleaner namespace (policy::PolicyEngine vs policy::policy::PolicyEngine)
#[allow(clippy::module_inception)]
pub mod policy;

pub mod session;
pub mod token;
pub mod used_tokens;

pub use policy::{AuthDecision, EnvTag, OperationType, PolicyEngine, RiskTag};
pub use session::SessionCache;
pub use token::ConfirmationToken;
pub use used_tokens::UsedTokenCache;
