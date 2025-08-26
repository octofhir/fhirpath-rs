//! Simplified replaceMatches function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use regex::Regex;

/// Simplified replaceMatches function: replaces all matches of a regular expression with a substitution
pub struct SimpleReplaceMatchesFunction;

impl SimpleReplaceMatchesFunction {
    pub fn new() -> Self {
        Self
    }

    fn extract_string_from_value(&self, value: &FhirPathValue) -> Result<Option<String>> {
        match value {
            FhirPathValue::String(s) => Ok(Some(s.as_ref().to_string())),
            FhirPathValue::Integer(i) => Ok(Some(i.to_string())),
            FhirPathValue::Decimal(d) => Ok(Some(d.to_string())),
            FhirPathValue::Boolean(b) => Ok(Some(b.to_string())),
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(None)
                } else if items.len() == 1 {
                    self.extract_string_from_value(items.first().unwrap())
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn process_single_value(
        &self,
        value: &FhirPathValue,
        args: &[FhirPathValue],
    ) -> Result<FhirPathValue> {
        // Convert input to string (including numeric values)
        let input_str = match value {
            FhirPathValue::String(s) => s.as_ref().to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![]),
                ));
            }
            _ => {
                return Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![]),
                ));
            }
        };

        // Extract and convert pattern parameter to string (handle collections)
        let pattern = self.extract_string_from_value(&args[0])?;
        if pattern.is_none() {
            return Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from(vec![]),
            ));
        }
        let pattern = pattern.unwrap();

        // Extract and convert substitution parameter to string (handle collections)
        let substitution = self.extract_string_from_value(&args[1])?;
        if substitution.is_none() {
            return Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from(vec![]),
            ));
        }
        let substitution = substitution.unwrap();

        // Special case: empty pattern should return the original string unchanged for replaceMatches
        if pattern.is_empty() {
            return Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from(vec![FhirPathValue::String(
                    input_str.into(),
                )]),
            ));
        }

        // Compile regex
        let regex = Regex::new(&pattern).map_err(|e| FhirPathError::evaluation_error(format!("Invalid regex pattern '{pattern}': {e}")))?;

        // Perform regex replacement with capture group support
        let result = regex.replace_all(&input_str, &substitution);
        Ok(FhirPathValue::Collection(
            octofhir_fhirpath_model::Collection::from(vec![FhirPathValue::String(
                result.to_string().into(),
            )]),
        ))
    }
}

impl Default for SimpleReplaceMatchesFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleReplaceMatchesFunction {
    fn name(&self) -> &'static str {
        "replaceMatches"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "replaceMatches",
                parameters: vec![ParameterType::String, ParameterType::String],
                return_type: ValueType::String,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "replaceMatches".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        // Handle collection inputs
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![]),
                    ));
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![]),
                    ));
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                self.process_single_value(value, args)
            }
            _ => {
                // Process as single value
                self.process_single_value(input, args)
            }
        }
    }
}
