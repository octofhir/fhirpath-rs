//! Simplified contains function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified contains function: returns true if the input string contains the given substring
pub struct SimpleContainsFunction;

impl SimpleContainsFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleContainsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleContainsFunction {
    fn name(&self) -> &'static str {
        "contains"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "contains",
                parameters: vec![ParameterType::Any], // Accept any type that can be converted to string
                return_type: ValueType::Boolean,
                variadic: false,
                category: FunctionCategory::Scalar,
                cardinality_requirement: CardinalityRequirement::RequiresScalar,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "contains".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get substring parameter - convert to string if possible
        let substring = match &args[0] {
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            FhirPathValue::Collection(col) if col.len() == 1 => {
                // Single-item collection - try to convert the item
                match col.first() {
                    Some(FhirPathValue::String(s)) => s.to_string(),
                    Some(FhirPathValue::Integer(i)) => i.to_string(),
                    Some(FhirPathValue::Decimal(d)) => d.to_string(),
                    Some(FhirPathValue::Boolean(b)) => {
                        if *b {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        }
                    }
                    Some(_) => {
                        return Err(FhirPathError::TypeError {
                            message: "contains() argument must be convertible to string"
                                .to_string(),
                        });
                    }
                    None => {
                        return Err(FhirPathError::TypeError {
                            message: "contains() argument must be convertible to string"
                                .to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "contains() argument must be convertible to string".to_string(),
                });
            }
        };

        match &context.input {
            FhirPathValue::String(s) => {
                let result = s.contains(&substring);
                Ok(FhirPathValue::Boolean(result))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "contains() can only be called on string values".to_string(),
            }),
        }
    }
}
