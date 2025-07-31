//! FHIR-specific type system functions

pub mod comparable;
pub mod extension;
pub mod is;
pub mod resolve;

pub use comparable::ComparableFunction;
pub use extension::ExtensionFunction;
pub use is::IsFunction;
pub use resolve::ResolveFunction;
