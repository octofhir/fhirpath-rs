//! CDA-specific functions for FHIRPath expressions

mod has_template_id;

pub use has_template_id::HasTemplateIdOfFunction;

use crate::function::FunctionRegistry;

/// Register all CDA-specific functions
pub fn register_cda_functions(registry: &mut FunctionRegistry) {
    registry.register(HasTemplateIdOfFunction);
}