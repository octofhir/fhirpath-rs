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

//! Unified ofType() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use octofhir_fhirpath_model::ModelProvider;
use octofhir_fhir_model::reflection::TypeReflectionInfo;
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_model::types::TypeInfo;

/// Unified ofType() function implementation
/// 
/// Filters collection to items of a specific type.
/// Syntax: ofType(type)
pub struct UnifiedOfTypeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedOfTypeFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "ofType",
            vec![ParameterInfo::required("type", TypeInfo::String)],
            TypeInfo::Collection(Box::new(TypeInfo::Any)),
        );
        
        let metadata = MetadataBuilder::new("ofType", FunctionCategory::Collections)
            .display_name("Of Type")
            .description("Returns items from collection that are of the specified type")
            .example("Bundle.entry.resource.ofType(Patient)")
            .example("value.ofType(string)")
            .signature(signature)
            .execution_mode(ExecutionMode::Async)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("ofType(${1:type})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["ofType", "type", "filter", "cast"])
            .usage_pattern(
                "Type-based filtering",
                "Bundle.entry.resource.ofType(Patient)",
                "Filtering collections by type"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedOfTypeFunction {
    fn name(&self) -> &str {
        "ofType"
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
        // Validate arguments - exactly 1 required (type identifier)
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
                        message: "ofType() requires a type name as string or type identifier".to_string(),
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
                            message: "ofType() requires a type name as string or type identifier".to_string(),
                        });
                    }
                } else {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "ofType() requires a type name as string or type identifier".to_string(),
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
                            message: "ofType() requires a type name as string or type identifier".to_string(),
                        });
                    }
                } else {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "ofType() requires a type name as string or type identifier".to_string(),
                    });
                }
            }
            _ => {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: format!("ofType() requires a type name as string or type identifier, got: {:?}", args[0]),
                });
            }
        };
        
        let mut filtered_items = Vec::new();
        
        match &context.input {
            FhirPathValue::Collection(items) => {
                // Filter items by type using ModelProvider for proper FHIR type checking
                for item in items.iter() {
                    if self.matches_type_with_provider(item, &type_name, context).await {
                        filtered_items.push(item.clone());
                    }
                }
            }
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            single_item => {
                // Process single item
                if self.matches_type_with_provider(single_item, &type_name, context).await {
                    filtered_items.push(single_item.clone());
                }
            }
        };
        
        if filtered_items.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(filtered_items))
        }
    }
}

impl UnifiedOfTypeFunction {
    /// Check if a value matches the specified type using ModelProvider for FHIR type checking
    async fn matches_type_with_provider(&self, value: &FhirPathValue, type_name: &str, context: &EvaluationContext) -> bool {
        // First handle primitive types that don't require ModelProvider
        match (value, type_name.to_lowercase().as_str()) {
            (FhirPathValue::String(_), "string") => return true,
            (FhirPathValue::Integer(_), "integer") => return true,
            (FhirPathValue::Decimal(_), "decimal") => return true,
            (FhirPathValue::Boolean(_), "boolean") => return true,
            (FhirPathValue::Date(_), "date") => return true,
            (FhirPathValue::DateTime(_), "datetime") => return true,
            (FhirPathValue::Time(_), "time") => return true,
            (FhirPathValue::Quantity(_), "quantity") => return true,
            (FhirPathValue::Collection(_), "collection") => return true,
            _ => {}
        }
        
        // For FHIR resources and complex types, use ModelProvider
        match value {
            FhirPathValue::Resource(resource) => {
                // Use ModelProvider to check if resource conforms to the specified type
                if let Some(model_provider) = context.model_provider.as_ref() {
                    // Get the resource type from the resource
                    if let Some(resource_type_value) = resource.as_json().get("resourceType") {
                        if let Some(actual_resource_type) = resource_type_value.as_str() {
                            // Check if the actual resource type matches or is a subtype of the requested type
                            return self.is_type_or_subtype(actual_resource_type, type_name, model_provider.as_ref()).await;
                        }
                    }
                }
                // Fallback to simple string comparison
                if let Some(resource_type_field) = resource.as_json().get("resourceType") {
                    if let Some(actual_type) = resource_type_field.as_str() {
                        return actual_type.to_lowercase() == type_name.to_lowercase();
                    }
                }
                false
            }
            FhirPathValue::JsonValue(_json_value) => {
                // For JSON values, we might need to check their type definition
                if let Some(_model_provider) = context.model_provider.as_ref() {
                    // This would require more sophisticated type checking
                    // For now, return false and could be enhanced later
                    false
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Check if a type is the same as or a subtype of the target type using ModelProvider
    async fn is_type_or_subtype(&self, actual_type: &str, target_type: &str, model_provider: &dyn ModelProvider) -> bool {
        // Direct match
        if actual_type.to_lowercase() == target_type.to_lowercase() {
            return true;
        }
        
        // Use ModelProvider to check inheritance/subtyping
        // This is a simplified implementation - in practice, we'd need to:
        // 1. Get the type definition for actual_type
        // 2. Check if it has base classes or implements interfaces that match target_type
        // 3. Walk the inheritance hierarchy
        
        // For now, implement basic type checking using get_type_reflection
        match model_provider.get_type_reflection(actual_type).await {
            Some(type_reflection) => {
                // Check if the type info indicates this type is compatible with target_type
                // This is a placeholder - real implementation would check base types, etc.
                match type_reflection {
                    TypeReflectionInfo::SimpleType { name, .. } => {
                        name.to_lowercase() == target_type.to_lowercase()
                    }
                    TypeReflectionInfo::ClassInfo { name, .. } => {
                        name.to_lowercase() == target_type.to_lowercase()
                    }
                    TypeReflectionInfo::ListType { .. } => {
                        // For lists, we might need more complex logic
                        false
                    }
                    TypeReflectionInfo::TupleType { .. } => {
                        // For tuples, we might need more complex logic
                        false
                    }
                }
            }
            None => false,
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
    async fn test_oftype_function() {
        let func = UnifiedOfTypeFunction::new();
        
        // Test filtering by string type
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::String("world".into()),
            FhirPathValue::Boolean(true),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("string".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert!(matches!(items.iter().nth(0), Some(FhirPathValue::String(_))));
            assert!(matches!(items.iter().nth(1), Some(FhirPathValue::String(_))));
        } else {
            panic!("Expected collection result");
        }
        
        // Test filtering by integer type
        let args = vec![FhirPathValue::String("integer".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert!(matches!(items.iter().nth(0), Some(FhirPathValue::Integer(42))));
        } else {
            panic!("Expected collection result");
        }
        
        // Test filtering by resource type
        let resource_json = json!({
            "resourceType": "Patient",
            "id": "123"
        });
        let resource = FhirPathValue::JsonValue(resource_json.into());
        let collection = FhirPathValue::collection(vec![
            resource,
            FhirPathValue::String("not-a-patient".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("Patient".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert!(matches!(items.iter().nth(0), Some(FhirPathValue::JsonValue(_))));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_oftype_empty_collection() {
        let func = UnifiedOfTypeFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        let args = vec![FhirPathValue::String("string".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedOfTypeFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "ofType");
        assert_eq!(metadata.execution_mode, ExecutionMode::Async);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}