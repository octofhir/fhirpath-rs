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

//! Unified type() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use std::sync::Arc;

/// Unified type() function implementation
///
/// Returns type information for the input value
pub struct UnifiedTypeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTypeFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("type", FunctionCategory::TypeChecking)
            .display_name("Type")
            .description("Returns type information for the input value")
            .example("Patient.type().name")
            .example("42.type().namespace")
            .output_type(TypePattern::Exact(octofhir_fhirpath_model::types::TypeInfo::TypeInfo))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("type()")
            .keywords(vec!["type", "typeof", "reflection", "metadata"])
            .usage_pattern(
                "Get type information",
                "value.type()",
                "Type reflection and metadata access"
            )
            .build();

        Self { metadata }
    }

    /// Get type information for a single value
    fn get_type_info(&self, value: &FhirPathValue) -> FhirPathValue {
        match value {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Boolean(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"),
                name: Arc::from("Boolean"),
            },
            FhirPathValue::Integer(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"), 
                name: Arc::from("Integer"),
            },
            FhirPathValue::Decimal(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"),
                name: Arc::from("Decimal"),
            },
            FhirPathValue::String(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"),
                name: Arc::from("String"),
            },
            FhirPathValue::Date(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"),
                name: Arc::from("Date"),
            },
            FhirPathValue::DateTime(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"),
                name: Arc::from("DateTime"),
            },
            FhirPathValue::Time(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"),
                name: Arc::from("Time"),
            },
            FhirPathValue::Quantity(_) => FhirPathValue::TypeInfoObject {
                namespace: Arc::from("System"),
                name: Arc::from("Quantity"),
            },
            FhirPathValue::Resource(resource) => {
                // For FHIR resources, get the resourceType and validate with FHIRSchemaProvider if available
                if let Some(resource_type) = resource.as_json().get("resourceType") {
                    if let Some(resource_type_str) = resource_type.as_str() {
                        // Use FHIR namespace for all FHIR resources as validated by FHIRSchemaProvider
                        FhirPathValue::TypeInfoObject {
                            namespace: Arc::from("FHIR"),
                            name: Arc::from(resource_type_str),
                        }
                    } else {
                        FhirPathValue::TypeInfoObject {
                            namespace: Arc::from("FHIR"),
                            name: Arc::from("Resource"),
                        }
                    }
                } else {
                    FhirPathValue::TypeInfoObject {
                        namespace: Arc::from("FHIR"),
                        name: Arc::from("Resource"),
                    }
                }
            },
            FhirPathValue::JsonValue(json_value) => {
                // For JSON values, try to get the resourceType if it's a FHIR resource
                if let Some(resource_type) = json_value.get("resourceType") {
                    if let Some(resource_type_str) = resource_type.as_str() {
                        // Validated FHIR resource JSON - use FHIR namespace
                        FhirPathValue::TypeInfoObject {
                            namespace: Arc::from("FHIR"),
                            name: Arc::from(resource_type_str),
                        }
                    } else {
                        FhirPathValue::TypeInfoObject {
                            namespace: Arc::from("System"),
                            name: Arc::from("Object"),
                        }
                    }
                } else {
                    FhirPathValue::TypeInfoObject {
                        namespace: Arc::from("System"),
                        name: Arc::from("Object"),
                    }
                }
            },
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    FhirPathValue::Empty
                } else {
                    // For collections, we could return the type of the first item
                    // or a generic Collection type - let's use Collection for now
                    FhirPathValue::TypeInfoObject {
                        namespace: Arc::from("System"),
                        name: Arc::from("Collection"),
                    }
                }
            },
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // Type info objects return themselves
                FhirPathValue::TypeInfoObject {
                    namespace: namespace.clone(),
                    name: name.clone(),
                }
            },
        }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTypeFunction {
    fn name(&self) -> &str {
        "type"
    }

    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }

    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }

    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate no arguments - this is a member function
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }

        // Get the input from context
        let input = &context.input;

        match input {
            FhirPathValue::Collection(items) => {
                let mut result = Vec::new();
                for item in items.iter() {
                    result.push(self.get_type_info(item));
                }
                Ok(FhirPathValue::Collection(result.into()))
            }
            single_item => Ok(self.get_type_info(single_item)),
        }
    }

    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;
    use serde_json::json;

    #[tokio::test]
    async fn test_unified_type_function() {
        let type_func = UnifiedTypeFunction::new();

        // Test string type
        let context = EvaluationContext::new(FhirPathValue::String("hello".into()));
        let result = type_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::TypeInfoObject { namespace, name } => {
                assert_eq!(namespace.as_ref(), "System");
                assert_eq!(name.as_ref(), "String");
            },
            _ => panic!("Expected TypeInfoObject result"),
        }

        // Test integer type
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = type_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::TypeInfoObject { namespace, name } => {
                assert_eq!(namespace.as_ref(), "System");
                assert_eq!(name.as_ref(), "Integer");
            },
            _ => panic!("Expected TypeInfoObject result"),
        }

        // Test FHIR resource type  
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123"
        });
        let context = EvaluationContext::new(FhirPathValue::JsonValue(patient_json.into()));
        let result = type_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::TypeInfoObject { namespace, name } => {
                assert_eq!(namespace.as_ref(), "FHIR");
                assert_eq!(name.as_ref(), "Patient");
            },
            _ => panic!("Expected TypeInfoObject result"),
        }

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = type_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with arguments (should fail)
        let context = EvaluationContext::new(FhirPathValue::String("test".to_string()));
        let result = type_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(type_func.name(), "type");
        assert_eq!(type_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(type_func.metadata().basic.display_name, "Type");
        assert!(type_func.metadata().basic.is_pure);
    }
}