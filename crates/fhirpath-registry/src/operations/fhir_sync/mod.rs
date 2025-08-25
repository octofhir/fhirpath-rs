//! FHIR operations - sync implementations
//! 
//! These are FHIR operations that work on data structure traversal and don't require
//! ModelProvider calls or network access.

pub mod children;
pub mod descendants;

pub use children::ChildrenFunction;
pub use descendants::DescendantsFunction;