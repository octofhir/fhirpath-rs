// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Type function operation implementation
//! Returns type information for any FHIRPath value

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{
        MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity, TypeConstraint,
    },
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue, JsonValue};
use sonic_rs::JsonValueTrait;

/// Type function operation - returns type information for values
pub struct TypeFunction {
    metadata: OperationMetadata,
}

impl Default for TypeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeFunction {
    pub fn new() -> Self {
        Self {
            metadata: Self::create_metadata(),
        }
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("type", OperationType::Function)
            .description("Returns type information for the input value")
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    /// Enhanced type determination that considers FHIR context
    async fn get_type_object_from_value(
        value: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Boolean(_) => {
                // Check if this value comes from FHIR context (i.e., from navigation like Patient.active)
                if Self::is_fhir_context(context) {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("FHIR"),
                        name: std::sync::Arc::from("boolean"),
                    })
                } else {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("System"),
                        name: std::sync::Arc::from("Boolean"),
                    })
                }
            }
            FhirPathValue::Integer(_) => {
                if Self::is_fhir_context(context) {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("FHIR"),
                        name: std::sync::Arc::from("integer"),
                    })
                } else {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("System"),
                        name: std::sync::Arc::from("Integer"),
                    })
                }
            }
            FhirPathValue::String(_) => {
                if Self::is_fhir_context(context) {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("FHIR"),
                        name: std::sync::Arc::from("string"),
                    })
                } else {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("System"),
                        name: std::sync::Arc::from("String"),
                    })
                }
            }
            FhirPathValue::Decimal(_) => {
                if Self::is_fhir_context(context) {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("FHIR"),
                        name: std::sync::Arc::from("decimal"),
                    })
                } else {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("System"),
                        name: std::sync::Arc::from("Decimal"),
                    })
                }
            }
            FhirPathValue::JsonValue(json_val) => {
                // For JSON values, try to determine the FHIR type from context
                if let Some(resource_type) = Self::extract_resource_type(json_val) {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("FHIR"),
                        name: std::sync::Arc::from(resource_type),
                    })
                } else {
                    Ok(FhirPathValue::TypeInfoObject {
                        namespace: std::sync::Arc::from("FHIR"),
                        name: std::sync::Arc::from("Element"),
                    })
                }
            }
            _ => Ok(FhirPathValue::TypeInfoObject {
                namespace: std::sync::Arc::from("FHIR"),
                name: std::sync::Arc::from("Element"),
            }),
        }
    }

    /// Check if the current evaluation context suggests we're working with FHIR data
    fn is_fhir_context(context: &EvaluationContext) -> bool {
        // Check if the root context contains FHIR resource data
        match &context.root {
            FhirPathValue::JsonValue(json_val) => {
                // Check if this looks like a FHIR resource
                json_val.as_inner().get("resourceType").is_some()
            }
            _ => false,
        }
    }

    /// Extract resource type from JSON value if it's a FHIR resource
    fn extract_resource_type(json_val: &JsonValue) -> Option<String> {
        if let Some(resource_type) = json_val.as_inner().get("resourceType") {
            if let Some(type_str) = resource_type.as_str() {
                return Some(type_str.to_string());
            }
        }
        None
    }
}

#[async_trait]
impl FhirPathOperation for TypeFunction {
    /// Get the operation identifier
    fn identifier(&self) -> &str {
        "type"
    }

    /// Get the operation type
    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    /// Get operation metadata
    fn metadata(&self) -> &OperationMetadata {
        &self.metadata
    }

    /// Evaluate the type function
    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Type function takes no arguments - it returns type info for the input value
        let input = &context.input;

        match input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                }

                // For collections, return the type of the first element
                let first_item = collection.first().ok_or_else(|| {
                    FhirPathError::evaluation_error("Empty collection in type function")
                })?;

                let type_object = Self::get_type_object_from_value(first_item, context).await?;
                Ok(FhirPathValue::Collection(Collection::from_vec(vec![
                    type_object,
                ])))
            }
            _ => {
                let type_object = Self::get_type_object_from_value(input, context).await?;
                Ok(FhirPathValue::Collection(Collection::from_vec(vec![
                    type_object,
                ])))
            }
        }
    }

    /// Try to evaluate synchronously (not supported for type function)
    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        None
    }

    /// Check if sync evaluation is supported
    fn supports_sync(&self) -> bool {
        false
    }

    /// Validate arguments (type function takes no arguments)
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::invalid_argument_count("type", 0, args.len()));
        }
        Ok(())
    }

    /// Get operation as Any trait object
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
