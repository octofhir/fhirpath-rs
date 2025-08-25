//! FHIR operations - async implementations
//! 
//! These are FHIR operations that genuinely require async execution due to
//! ModelProvider calls, network access, or external system dependencies.

pub mod resolve;
pub mod conforms_to;
pub mod extension;

pub use resolve::ResolveFunction;
pub use conforms_to::ConformsToFunction;
pub use extension::ExtensionFunction;