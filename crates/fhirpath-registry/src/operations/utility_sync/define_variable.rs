//! DefineVariable function implementation - sync version

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// DefineVariable function - defines variables in evaluation context
#[derive(Debug, Clone)]
pub struct DefineVariableFunction;

impl DefineVariableFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for DefineVariableFunction {
    fn name(&self) -> &'static str {
        "defineVariable"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "defineVariable",
                parameters: vec![ParameterType::String, ParameterType::Any],
                return_type: ValueType::Any,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "defineVariable".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        let _var_name = args[0]
            .as_string()
            .ok_or_else(|| FhirPathError::TypeError {
                message: "Variable name must be a string".to_string(),
            })?;

        // Note: In a full implementation, we would need a mutable context
        // to store the variable. For now, we just return the input unchanged.
        // TODO: Implement proper variable storage in evaluation context

        // The defineVariable function should store the variable in the context
        // and return the input value unchanged per FHIRPath specification
        Ok(context.input.clone())
    }
}

impl Default for DefineVariableFunction {
    fn default() -> Self {
        Self::new()
    }
}
