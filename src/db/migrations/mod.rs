//! Database migrations

mod v002_add_groups;

pub use v002_add_groups::V002AddGroups;

use crate::db::Migration;

/// Get all migrations in order
pub fn all_migrations() -> Vec<Box<dyn Migration>> {
    vec![
        Box::new(V002AddGroups),
    ]
}
