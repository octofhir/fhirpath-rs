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
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use sonic_rs::JsonValueTrait;

/// Type function operation - returns type information for values
pub struct TypeFunction;

impl Default for TypeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("type", OperationType::Function)
            .description("Returns type information for the input value")
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    /// Create a type object with namespace and name properties
    fn create_type_object(namespace: &str, name: &str) -> FhirPathValue {
        FhirPathValue::TypeInfoObject {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    /// Get type information for a FhirPathValue
    fn get_type_info(value: &FhirPathValue) -> (String, String) {
        match value {
            // System types
            FhirPathValue::Boolean(_) => ("System".to_string(), "Boolean".to_string()),
            FhirPathValue::Integer(_) => ("System".to_string(), "Integer".to_string()),
            FhirPathValue::Decimal(_) => ("System".to_string(), "Decimal".to_string()),
            FhirPathValue::String(_) => ("System".to_string(), "String".to_string()),
            FhirPathValue::Date(_) => ("System".to_string(), "Date".to_string()),
            FhirPathValue::DateTime(_) => ("System".to_string(), "DateTime".to_string()),
            FhirPathValue::Time(_) => ("System".to_string(), "Time".to_string()),
            FhirPathValue::Quantity(_) => ("System".to_string(), "Quantity".to_string()),

            // FHIR types - check different value variants
            FhirPathValue::Resource(resource) => {
                if let Some(resource_type) = resource.resource_type() {
                    ("FHIR".to_string(), resource_type.to_string())
                } else {
                    ("FHIR".to_string(), "Resource".to_string())
                }
            }
            FhirPathValue::JsonValue(json) => {
                if let Some(resource_type) =
                    json.as_inner().get("resourceType").and_then(|v| v.as_str())
                {
                    ("FHIR".to_string(), resource_type.to_string())
                } else {
                    ("System".to_string(), "Object".to_string())
                }
            }

            // Collections
            FhirPathValue::Collection(_) => ("System".to_string(), "Collection".to_string()),

            // Other types
            _ => ("System".to_string(), "Any".to_string()),
        }
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
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(TypeFunction::create_metadata);
        &METADATA
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

                let (namespace, name) = Self::get_type_info(first_item);
                Ok(FhirPathValue::Collection(Collection::from_vec(vec![
                    Self::create_type_object(&namespace, &name),
                ])))
            }
            _ => {
                let (namespace, name) = Self::get_type_info(input);
                Ok(FhirPathValue::Collection(Collection::from_vec(vec![
                    Self::create_type_object(&namespace, &name),
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
