//! Default function implementations for FHIRPath

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::{FhirPathValue, Result};
use crate::register_function;

impl FunctionRegistry {
    pub fn register_default_functions(&self) -> Result<()> {
        // Register basic utility functions
        self.register_empty_function()?;
        // exists() is now handled by lambda functions module

        // Register collection functions
        self.register_collection_functions()?;

        // Register math functions
        self.register_math_functions()?;

        // Register string functions
        self.register_string_functions()?;

        // Register type functions
        self.register_type_functions()?;

        // Register conversion functions
        self.register_conversion_functions()?;

        // Register datetime functions
        self.register_datetime_functions()?;

        // Register FHIR-specific functions
        self.register_fhir_functions()?;
        self.register_fhir_extension_functions()?;
        
        // Register provider-dependent functions
        self.register_provider_dependent_functions()?;

        // Register terminology functions
        self.register_terminology_functions()?;

        // NOTE: %terminologies built-in functions are handled by modifying existing functions
        // self.register_terminologies_builtin_functions()?;

        // Register logic functions
        self.register_logic_functions()?;

        // Register numeric functions
        self.register_numeric_functions()?;

        // Register lambda functions (where, select, aggregate, etc.)
        self.register_lambda_functions()?;

        Ok(())
    }

    fn register_empty_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "empty",
            category: FunctionCategory::Utility,
            description: "Returns true if the input collection is empty",
            parameters: [],
            return_type: "boolean",
            examples: ["Patient.name.empty()", "Bundle.entry.empty()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let is_empty = context.input.is_empty();
                Ok(FhirPathValue::boolean(is_empty))
            }
        )
    }

    // exists() function moved to lambda_functions.rs to handle both parameterless and lambda versions

    /// Register functions that demonstrate provider access
    fn register_provider_dependent_functions(&self) -> Result<()> {
        // resolve() function is registered in fhir.rs to avoid duplication

        // memberOf() function is registered in terminology.rs to avoid duplication

        Ok(())
    }
}

/// Helper function to extract reference string from FhirPathValue
fn extract_reference_string(value: &FhirPathValue) -> Option<String> {
    match value {
        FhirPathValue::String(s) => Some(s.clone()),
        FhirPathValue::JsonValue(json) => {
            // Try to extract reference from Reference resource
            json.get("reference")
                .and_then(|r| r.as_str())
                .map(|s| s.to_string())
        }
        _ => None,
    }
}
