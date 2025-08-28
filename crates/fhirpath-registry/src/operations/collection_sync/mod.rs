//! Simplified collection operations module

// Core operations
pub mod count;
pub mod empty;
pub mod exists;
pub mod first;
pub mod last;
pub mod single;

// Navigation operations
pub mod skip;
pub mod tail;
pub mod take;

// Set operations
pub mod distinct;
pub mod exclude;
pub mod intersect;
pub mod union;

// Boolean operations
pub mod all_false;
pub mod all_true;
pub mod any_false;
pub mod any_true;

// Comparison operations
pub mod is_distinct;
pub mod subset_of;
pub mod superset_of;

// Combine operation
pub mod combine;

// Re-exports
pub use count::SimpleCountFunction;
pub use empty::SimpleEmptyFunction;
pub use exists::SimpleExistsFunction;
pub use first::SimpleFirstFunction;
pub use last::SimpleLastFunction;
pub use single::SimpleSingleFunction;

pub use skip::SimpleSkipFunction;
pub use tail::SimpleTailFunction;
pub use take::SimpleTakeFunction;

pub use distinct::SimpleDistinctFunction;
pub use exclude::SimpleExcludeFunction;
pub use intersect::SimpleIntersectFunction;
pub use union::SimpleUnionFunction;

pub use all_false::SimpleAllFalseFunction;
pub use all_true::SimpleAllTrueFunction;
pub use any_false::SimpleAnyFalseFunction;
pub use any_true::SimpleAnyTrueFunction;

pub use is_distinct::SimpleIsDistinctFunction;
pub use subset_of::SimpleSubsetOfFunction;
pub use superset_of::SimpleSupersetOfFunction;

pub use combine::SimpleCombineFunction;
