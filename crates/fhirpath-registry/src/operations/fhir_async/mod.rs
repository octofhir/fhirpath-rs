//! FHIR operations - async implementations
//!
//! These are FHIR operations that genuinely require async execution due to
//! ModelProvider calls, network access, or external system dependencies.

pub mod children;
pub mod conforms_to;
pub mod extension;
pub mod resolve;

pub use children::ChildrenFunction;
pub use conforms_to::ConformsToFunction;
pub use extension::ExtensionFunction;
pub use resolve::ResolveFunction;
