//! Cloud Storage Abstraction
//!
//! This module provides a unified interface for various cloud storage providers
//! using OpenDAL as the underlying abstraction layer.

pub mod config;
pub mod provider;

pub use config::{CloudConfig, CloudProvider};
pub use provider::create_operator;
