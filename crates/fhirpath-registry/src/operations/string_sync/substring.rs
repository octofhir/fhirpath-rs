//! Simplified substring function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified substring function: extracts a substring from a string
pub struct SimpleSubstringFunction;

impl SimpleSubstringFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSubstringFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSubstringFunction {
    fn name(&self) -> &'static str {
        "substring"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "substring",
                parameters: vec![ParameterType::Integer, ParameterType::Integer],
                return_type: ValueType::String,
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
        // Validate arguments - requires 1 or 2 parameters (start, optional length)
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "substring".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get start index parameter
        let start = match &args[0] {
            FhirPathValue::Integer(n) => {
                if *n < 0 {
                    // Negative start index returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Empty);
                }
                *n as usize
            }
            _ => {
                // Return empty collection for invalid start parameter type per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };

        // Get optional length parameter
        let length = if args.len() == 2 {
            match &args[1] {
                FhirPathValue::Integer(n) => {
                    if *n < 0 {
                        // Negative length returns empty collection per FHIRPath spec
                        return Ok(FhirPathValue::Empty);
                    }
                    Some(*n as usize)
                }
                _ => {
                    // Return empty collection for invalid length parameter type per FHIRPath spec
                    return Ok(FhirPathValue::Empty);
                }
            }
        } else {
            None
        };

        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                if start >= chars.len() {
                    // Out-of-bounds start index returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Empty);
                }

                let end = if let Some(len) = length {
                    std::cmp::min(start + len, chars.len())
                } else {
                    chars.len()
                };

                let result: String = chars[start..end].iter().collect();
                Ok(FhirPathValue::String(result.into()))
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::String(s) => {
                            let chars: Vec<char> = s.chars().collect();
                            if start >= chars.len() {
                                // Out-of-bounds start index skips this item per FHIRPath spec
                                continue;
                            } else {
                                let end = if let Some(len) = length {
                                    std::cmp::min(start + len, chars.len())
                                } else {
                                    chars.len()
                                };

                                let result: String = chars[start..end].iter().collect();
                                results.push(FhirPathValue::String(result.into()));
                            }
                        }
                        _ => {
                            // For non-string values, skip them (per FHIRPath spec for string functions)
                            continue;
                        }
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                // For non-string values, return empty (per FHIRPath spec for string functions)
                Ok(FhirPathValue::Empty)
            }
        }
    }
}
