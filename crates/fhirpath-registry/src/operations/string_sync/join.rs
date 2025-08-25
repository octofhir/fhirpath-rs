//! Simplified join function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified join function: joins a collection of strings into a single string using the specified separator
pub struct SimpleJoinFunction;

impl SimpleJoinFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleJoinFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleJoinFunction {
    fn name(&self) -> &'static str {
        "join"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "join",
            parameters: vec![ParameterType::String],
            return_type: ValueType::String,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "join".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get separator parameter
        let separator = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "join() separator argument must be a string".to_string()
                });
            }
        };

        // Get input collection - always convert input to collection for consistent handling
        let collection = match &context.input {
            FhirPathValue::Collection(items) => items.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::String("".into())),
            // Single item becomes a single-item collection
            single => vec![single.clone()].into(),
        };

        // Convert all items to strings and join
        let string_items: Result<Vec<String>> = collection
            .iter()
            .map(|item| match item {
                FhirPathValue::String(s) => Ok(s.as_ref().to_string()),
                FhirPathValue::Integer(i) => Ok(i.to_string()),
                FhirPathValue::Decimal(d) => Ok(d.to_string()),
                FhirPathValue::Boolean(b) => Ok(b.to_string()),
                FhirPathValue::DateTime(dt) => Ok(dt.to_string()),
                FhirPathValue::Date(d) => Ok(d.to_string()),
                FhirPathValue::Time(t) => Ok(t.to_string()),
                FhirPathValue::Empty => Ok("".to_string()),
                _ => Err(FhirPathError::TypeError {
                    message: format!("join() cannot convert {:?} to string", item)
                }),
            })
            .collect();

        let strings = string_items?;

        // If collection is empty, return empty string
        if strings.is_empty() {
            return Ok(FhirPathValue::String("".into()));
        }

        let result = strings.join(separator);
        Ok(FhirPathValue::String(result.into()))
    }
}