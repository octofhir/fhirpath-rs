//! ConformsTo function implementation - async version (simplified)

use crate::traits::{AsyncOperation, EvaluationContext};
use crate::signature::{FunctionSignature, ParameterType, ValueType};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ConformsTo function - validates if resource conforms to a StructureDefinition
#[derive(Debug, Clone)]
pub struct ConformsToFunction;

impl ConformsToFunction {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AsyncOperation for ConformsToFunction {
    fn name(&self) -> &'static str {
        "conformsTo"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature {
                name: "conformsTo",
                parameters: vec![ParameterType::String],
                return_type: ValueType::Boolean,
                variadic: false,
            }
        });
        &SIGNATURE
    }

    async fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "conformsTo".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let profile_url = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(coll) if coll.len() == 1 => {
                match coll.iter().next().unwrap() {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(FhirPathError::TypeError {
                            message: "conformsTo() profile URL argument must be a string"
                                .to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "conformsTo() profile URL argument must be a string".to_string(),
                });
            }
        };

        // Profile URL validation - if it doesn't look like a valid StructureDefinition URL, return empty
        // Valid profile URLs should contain "StructureDefinition" or be well-formed FHIR URLs
        if !profile_url.contains("StructureDefinition") && 
           !profile_url.starts_with("http://hl7.org/fhir/") &&
           !profile_url.starts_with("https://hl7.org/fhir/") {
            return Ok(FhirPathValue::Empty);
        }

        // Use ModelProvider to validate resource against profile
        let conforms = context
            .model_provider
            .validates_resource_against_profile(&context.input, profile_url)
            .await;

        match conforms {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(_) => {
                // If validation fails (e.g., invalid profile URL), return empty
                Ok(FhirPathValue::Empty)
            }
        }
    }
}

impl Default for ConformsToFunction {
    fn default() -> Self {
        Self::new()
    }
}