//! Logical NOT operation - sync version

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Logical NOT operation
#[derive(Debug, Clone)]
pub struct NotOperation;

impl NotOperation {
    pub fn new() -> Self {
        Self
    }

    fn to_boolean(value: &FhirPathValue) -> Result<Option<bool>> {
        match value {
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Boolean(b) => Ok(Some(*b)),
            FhirPathValue::Integer(_) => Ok(Some(true)), // All integers are truthy in FHIRPath
            FhirPathValue::Decimal(d) => Ok(Some(!d.is_zero())), // 0.0 = false, non-zero = true
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(None)
                } else if c.len() == 1 {
                    Self::to_boolean(c.first().unwrap())
                } else {
                    Err(FhirPathError::TypeError {
                        message: "Cannot convert collection with multiple items to boolean"
                            .to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!("Cannot convert {} to boolean", value.type_name()),
            }),
        }
    }
}

impl SyncOperation for NotOperation {
    fn name(&self) -> &'static str {
        "not"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "not",
                parameters: vec![],
                return_type: ValueType::Boolean,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "not")?;

        let value = Self::to_boolean(&context.input)?;

        // Three-valued logic for NOT
        let result = match value {
            Some(true) => Some(false),
            Some(false) => Some(true),
            None => None, // NOT of empty is empty
        };

        match result {
            Some(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(b)])),
            None => Ok(FhirPathValue::Empty),
        }
    }
}

impl Default for NotOperation {
    fn default() -> Self {
        Self::new()
    }
}
