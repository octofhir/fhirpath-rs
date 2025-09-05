//! Default function implementations for FHIRPath

use super::{FunctionRegistry, FunctionCategory, FunctionContext};
use crate::core::{FhirPathValue, FhirPathError, Result};
use crate::{register_function};

impl FunctionRegistry {
    pub fn register_default_functions(&self) -> Result<()> {
        // Register basic utility functions
        self.register_empty_function()?;
        self.register_exists_function()?;
        
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

        // Register terminology functions
        self.register_terminology_functions()?;
        
        // Register logic functions
        self.register_logic_functions()?;
        
        // Register numeric functions
        self.register_numeric_functions()?;
        
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let is_empty = context.input.is_empty();
                Ok(vec![FhirPathValue::boolean(is_empty)])
            }
        )
    }

    fn register_exists_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "exists",
            category: FunctionCategory::Utility,
            description: "Returns true if the input collection is not empty",
            parameters: [],
            return_type: "boolean",
            examples: ["Patient.name.exists()", "Bundle.entry.exists()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let exists = !context.input.is_empty();
                Ok(vec![FhirPathValue::boolean(exists)])
            }
        )
    }
}
