//! Simplified synchronous string operations
//!
//! This module contains all string operations migrated to the simplified sync system.
//! These operations perform string manipulation without async overhead.

use crate::registry::SyncRegistry;
use octofhir_fhirpath_core::Result;

pub mod length;
pub mod upper;
pub mod lower;
pub mod contains;
pub mod starts_with;
pub mod ends_with;
pub mod trim;
pub mod substring;
pub mod index_of;
pub mod last_index_of;
pub mod replace;
pub mod split;
pub mod join;
pub mod matches;
pub mod matches_full;
pub mod replace_matches;
pub mod to_chars;

pub use length::SimpleLengthFunction;
pub use upper::SimpleUpperFunction;
pub use lower::SimpleLowerFunction;
pub use contains::SimpleContainsFunction;
pub use starts_with::SimpleStartsWithFunction;
pub use ends_with::SimpleEndsWithFunction;
pub use trim::SimpleTrimFunction;
pub use substring::SimpleSubstringFunction;
pub use index_of::SimpleIndexOfFunction;
pub use last_index_of::SimpleLastIndexOfFunction;
pub use replace::SimpleReplaceFunction;
pub use split::SimpleSplitFunction;
pub use join::SimpleJoinFunction;
pub use matches::SimpleMatchesFunction;
pub use matches_full::SimpleMatchesFullFunction;
pub use replace_matches::SimpleReplaceMatchesFunction;
pub use to_chars::SimpleToCharsFunction;

/// Utility struct for registering all simplified string operations
pub struct SimpleStringOperations;

impl SimpleStringOperations {
    /// Register all simplified string operations in the sync registry
    pub async fn register_all(registry: &SyncRegistry) -> Result<()> {
        // Basic string operations
        registry.register(Box::new(SimpleLengthFunction::new())).await;
        registry.register(Box::new(SimpleUpperFunction::new())).await;
        registry.register(Box::new(SimpleLowerFunction::new())).await;
        registry.register(Box::new(SimpleTrimFunction::new())).await;
        
        // String search operations
        registry.register(Box::new(SimpleContainsFunction::new())).await;
        registry.register(Box::new(SimpleStartsWithFunction::new())).await;
        registry.register(Box::new(SimpleEndsWithFunction::new())).await;
        registry.register(Box::new(SimpleIndexOfFunction::new())).await;
        registry.register(Box::new(SimpleLastIndexOfFunction::new())).await;
        
        // String manipulation operations
        registry.register(Box::new(SimpleSubstringFunction::new())).await;
        registry.register(Box::new(SimpleReplaceFunction::new())).await;
        registry.register(Box::new(SimpleSplitFunction::new())).await;
        registry.register(Box::new(SimpleJoinFunction::new())).await;
        registry.register(Box::new(SimpleToCharsFunction::new())).await;
        
        // Regular expression operations (simplified)
        registry.register(Box::new(SimpleMatchesFunction::new())).await;
        registry.register(Box::new(SimpleMatchesFullFunction::new())).await;
        registry.register(Box::new(SimpleReplaceMatchesFunction::new())).await;
        
        Ok(())
    }
}