//! Simplified synchronous string operations
//!
//! This module contains all string operations migrated to the simplified sync system.
//! These operations perform string manipulation without async overhead.

use crate::registry::SyncRegistry;
use octofhir_fhirpath_core::Result;

pub mod contains;
pub mod ends_with;
pub mod index_of;
pub mod join;
pub mod last_index_of;
pub mod length;
pub mod lower;
pub mod matches;
pub mod matches_full;
pub mod replace;
pub mod replace_matches;
pub mod split;
pub mod starts_with;
pub mod substring;
pub mod to_chars;
pub mod trim;
pub mod upper;

pub use contains::SimpleContainsFunction;
pub use ends_with::SimpleEndsWithFunction;
pub use index_of::SimpleIndexOfFunction;
pub use join::SimpleJoinFunction;
pub use last_index_of::SimpleLastIndexOfFunction;
pub use length::SimpleLengthFunction;
pub use lower::SimpleLowerFunction;
pub use matches::SimpleMatchesFunction;
pub use matches_full::SimpleMatchesFullFunction;
pub use replace::SimpleReplaceFunction;
pub use replace_matches::SimpleReplaceMatchesFunction;
pub use split::SimpleSplitFunction;
pub use starts_with::SimpleStartsWithFunction;
pub use substring::SimpleSubstringFunction;
pub use to_chars::SimpleToCharsFunction;
pub use trim::SimpleTrimFunction;
pub use upper::SimpleUpperFunction;

/// Utility struct for registering all simplified string operations
pub struct SimpleStringOperations;

impl SimpleStringOperations {
    /// Register all simplified string operations in the sync registry
    pub async fn register_all(registry: &SyncRegistry) -> Result<()> {
        // Basic string operations
        registry
            .register(Box::new(SimpleLengthFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleUpperFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleLowerFunction::new()))
            .await;
        registry.register(Box::new(SimpleTrimFunction::new())).await;

        // String search operations
        registry
            .register(Box::new(SimpleContainsFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleStartsWithFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleEndsWithFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleIndexOfFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleLastIndexOfFunction::new()))
            .await;

        // String manipulation operations
        registry
            .register(Box::new(SimpleSubstringFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleReplaceFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleSplitFunction::new()))
            .await;
        registry.register(Box::new(SimpleJoinFunction::new())).await;
        registry
            .register(Box::new(SimpleToCharsFunction::new()))
            .await;

        // Regular expression operations (simplified)
        registry
            .register(Box::new(SimpleMatchesFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleMatchesFullFunction::new()))
            .await;
        registry
            .register(Box::new(SimpleReplaceMatchesFunction::new()))
            .await;

        Ok(())
    }
}
