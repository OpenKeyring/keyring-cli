//! Error types
//! This module will be implemented in Task 4

use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, anyhow::Error>;
