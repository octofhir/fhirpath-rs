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

//! Unified is() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_model::types::TypeInfo;

/// Unified is() function implementation
/// 
/// Tests whether the input is of the given type.
/// Syntax: is(type)
pub struct UnifiedIsFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedIsFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "is",
            vec![ParameterInfo::required("type", TypeInfo::String)],
            TypeInfo::Boolean,
        );
        
        let metadata = MetadataBuilder::new("is", FunctionCategory::TypeChecking)
            .display_name("Is Type")
            .description("Tests whether the input is of the given type")
            .example("Patient.name.is(HumanName)")
            .example("'hello'.is(string)")
            .example("42.is(integer)")
            .signature(signature)
            .execution_mode(ExecutionMode::Async) // Async due to FHIR type resolution
            .input_types(vec![TypePattern::Any])
            .output_type(TypePattern::Boolean)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Logarithmic) // Type lookup
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("is(${1:type})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["is", "type", "check", "instanceof"])
            .usage_pattern(
                "Type checking",
                "value.is(type)",
                "Checking if values match specific FHIR types"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedIsFunction {
    fn name(&self) -> &str {
        "is"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Async
    }
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - exactly 1 required (type name)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let type_name = match &args[0] {
            FhirPathValue::String(name) => name.to_string(),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.get(0) {
                    Some(FhirPathValue::String(name)) => name.to_string(),
                    Some(FhirPathValue::TypeInfoObject { name, .. }) => name.to_string(),
                    _ => return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Type argument must be a string or type identifier".to_string(),
                    }),
                }
            }
            FhirPathValue::TypeInfoObject { name, .. } => name.to_string(),
            FhirPathValue::Resource(resource) => {
                // Handle case where bare type identifier like "Patient" evaluates to the resource itself
                // Extract the resourceType as the type name
                if let Some(resource_type_value) = resource.as_json().get("resourceType") {
                    if let Some(resource_type_str) = resource_type_value.as_str() {
                        resource_type_str.to_string()
                    } else {
                        return Err(FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: "Type argument must be a string or type identifier".to_string(),
                        });
                    }
                } else {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Type argument must be a string or type identifier".to_string(),
                    });
                }
            }
            FhirPathValue::JsonValue(json_value) => {
                // Handle case where bare type identifier evaluates to a JSON value
                if let Some(resource_type_value) = json_value.get("resourceType") {
                    if let Some(resource_type_str) = resource_type_value.as_str() {
                        resource_type_str.to_string()
                    } else {
                        return Err(FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: "Type argument must be a string or type identifier".to_string(),
                        });
                    }
                } else {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Type argument must be a string or type identifier".to_string(),
                    });
                }
            }
            _ => {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: format!("Type argument must be a string or type identifier, got: {:?}", args[0]),
                });
            }
        };
        
        // Handle input based on its type
        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut result = Vec::new();
                for item in items.iter() {
                    let is_type = self.check_type(item, &type_name, context).await?;
                    result.push(FhirPathValue::Boolean(is_type));
                }
                Ok(FhirPathValue::collection(result))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)),
            single_item => {
                let is_type = self.check_type(single_item, &type_name, context).await?;
                Ok(FhirPathValue::Boolean(is_type))
            }
        }
    }
}

impl UnifiedIsFunction {
    /// Check if a value is of the specified type using FHIRSchemaProvider
    async fn check_type(
        &self,
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> FunctionResult<bool> {
        // Handle primitive types first (System namespace types)
        match type_name.to_lowercase().as_str() {
            "string" => return Ok(matches!(value, FhirPathValue::String(_))),
            "integer" => return Ok(matches!(value, FhirPathValue::Integer(_))),
            "decimal" => return Ok(matches!(value, FhirPathValue::Decimal(_))),
            "boolean" => return Ok(matches!(value, FhirPathValue::Boolean(_))),
            "date" => return Ok(matches!(value, FhirPathValue::Date(_))),
            "datetime" => return Ok(matches!(value, FhirPathValue::DateTime(_))),
            "time" => return Ok(matches!(value, FhirPathValue::Time(_))),
            "quantity" => return Ok(matches!(value, FhirPathValue::Quantity(_))),
            "empty" => return Ok(matches!(value, FhirPathValue::Empty)),
            _ => {} // Continue to FHIR type checking
        }

        // For complex FHIR types, MUST use FHIRSchemaProvider as required
        if let Some(provider) = context.model_provider.as_ref() {
            match value {
                FhirPathValue::Resource(resource) => {
                    // Get resource type from the resource
                    if let Some(resource_type_value) = resource.as_json().get("resourceType") {
                        if let Some(resource_type_str) = resource_type_value.as_str() {
                            // Use FHIRSchemaProvider to check if the resource type is valid and matches
                            let is_compatible = provider.is_type_compatible(resource_type_str, type_name).await;
                            Ok(is_compatible)
                        } else {
                            Ok(false)
                        }
                    } else {
                        Ok(false)
                    }
                }
                FhirPathValue::JsonValue(json_value) => {
                    // Get resource type from JSON
                    if let Some(resource_type_value) = json_value.get("resourceType") {
                        if let Some(resource_type_str) = resource_type_value.as_str() {
                            // Use FHIRSchemaProvider to check type compatibility
                            let is_compatible = provider.is_type_compatible(resource_type_str, type_name).await;
                            Ok(is_compatible)
                        } else {
                            Ok(false)
                        }
                    } else {
                        // For non-resource JSON values, use FHIRSchemaProvider to validate type
                        match provider.get_type_reflection(type_name).await {
                            Some(_type_info) => {
                                // Type exists in schema, but value doesn't match expected structure
                                Ok(false)
                            }
                            None => Ok(false), // Type not found in schema
                        }
                    }
                }
                _ => {
                    // For other value types, check if the specified type exists in FHIR schema
                    match provider.get_type_reflection(type_name).await {
                        Some(_) => Ok(false), // Type exists but value doesn't match
                        None => Ok(false), // Type doesn't exist
                    }
                }
            }
        } else {
            // If no FHIRSchemaProvider is available, return error as required by user specification
            Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "FHIRSchemaProvider is required for FHIR type checking operations".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use serde_json::json;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_is_string() {
        let func = UnifiedIsFunction::new();
        let context = create_test_context(FhirPathValue::String("hello".into()));
        
        let args = vec![FhirPathValue::String("string".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_is_integer() {
        let func = UnifiedIsFunction::new();
        let context = create_test_context(FhirPathValue::Integer(42));
        
        let args = vec![FhirPathValue::String("integer".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_is_wrong_type() {
        let func = UnifiedIsFunction::new();
        let context = create_test_context(FhirPathValue::String("hello".into()));
        
        let args = vec![FhirPathValue::String("integer".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
    
    #[tokio::test]
    async fn test_is_collection() {
        let func = UnifiedIsFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::Integer(42),
        ]);
        let context = create_test_context(collection);
        
        let args = vec![FhirPathValue::String("string".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Boolean(true)));  // String is string
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Boolean(false))); // Integer is not string
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_is_resource_type() {
        let func = UnifiedIsFunction::new();
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123"
        });
        let context = create_test_context(FhirPathValue::JsonValue(patient_json.into()));
        
        let args = vec![FhirPathValue::String("Patient".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedIsFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "is");
        assert_eq!(metadata.execution_mode, ExecutionMode::Async);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::TypeChecking);
    }
}