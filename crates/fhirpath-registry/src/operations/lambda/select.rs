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

//! Select function implementation - transforms collection elements

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Select function - transforms each element in a collection
#[derive(Debug, Clone)]
pub struct SelectFunction;

impl SelectFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("select", OperationType::Function)
            .description("Transforms each element in a collection using the provided expression")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("Patient.name.select(given)")
            .example("Bundle.entry.select(resource)")
            .example("telecom.select(value)")
            .build()
    }

    fn apply_transform(item: &FhirPathValue, transform: &FhirPathValue) -> Result<FhirPathValue> {
        match transform {
            // Mock transformation: if transform is a string, extract that field from object
            FhirPathValue::String(field_name) => {
                match item {
                    FhirPathValue::JsonValue(obj) => {
                        if let Some(value) = obj.get(field_name.as_ref()) {
                            Ok(FhirPathValue::from(value.clone()))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    },
                    _ => Ok(FhirPathValue::Empty),
                }
            },
            // If transform is a function name as string, apply simple transforms
            FhirPathValue::String(func_name) if func_name.as_ref() == "upper" => {
                match item {
                    FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_uppercase().into())),
                    _ => Ok(item.clone()),
                }
            },
            FhirPathValue::String(func_name) if func_name.as_ref() == "lower" => {
                match item {
                    FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_lowercase().into())),
                    _ => Ok(item.clone()),
                }
            },
            // Direct value transformation
            _ => Ok(transform.clone()),
        }
    }
}

#[async_trait]
impl FhirPathOperation for SelectFunction {
    fn identifier(&self) -> &str {
        "select"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SelectFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 1, 
                actual: args.len() 
            });
        }

        let transform = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut transformed_items = Vec::new();

                for item in items.iter() {
                    // Create new context for each item
                    let _item_context = EvaluationContext::new(item.clone(), context.registry.clone(), context.model_provider.clone());
                    
                    // Apply transformation
                    let transformed = Self::apply_transform(item, transform)?;
                    
                    // Add non-empty results to collection
                    match transformed {
                        FhirPathValue::Empty => {}, // Skip empty results
                        FhirPathValue::Collection(inner_items) => {
                            transformed_items.extend(inner_items.iter().cloned());
                        },
                        single_result => transformed_items.push(single_result),
                    }
                }

                if transformed_items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if transformed_items.len() == 1 {
                    Ok(transformed_items.into_iter().next().unwrap())
                } else {
                    Ok(FhirPathValue::collection(transformed_items))
                }
            },
            single_item => {
                // Apply select to single item
                Self::apply_transform(single_item, transform)
            }
        }
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 1, 
                actual: args.len() 
            }));
        }

        let transform = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut transformed_items = Vec::new();

                for item in items.iter() {
                    match Self::apply_transform(item, transform) {
                        Ok(transformed) => {
                            match transformed {
                                FhirPathValue::Empty => {},
                                FhirPathValue::Collection(inner_items) => {
                                    transformed_items.extend(inner_items.iter().cloned());
                                },
                                single_result => transformed_items.push(single_result),
                            }
                        },
                        Err(e) => return Some(Err(e)),
                    }
                }

                if transformed_items.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if transformed_items.len() == 1 {
                    Some(Ok(transformed_items.into_iter().next().unwrap()))
                } else {
                    Some(Ok(FhirPathValue::collection(transformed_items)))
                }
            },
            single_item => {
                Some(Self::apply_transform(single_item, transform))
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_select_function_field_extraction() {
        let func = SelectFunction::new();

        // Test selecting field from objects
        let objects = vec![
            FhirPathValue::JsonValue(ArcJsonValue::new(json!({"name": "John", "age": 30}))),
            FhirPathValue::JsonValue(ArcJsonValue::new(json!({"name": "Jane", "age": 25}))),
            FhirPathValue::JsonValue(ArcJsonValue::new(json!({"age": 40}))), // missing name
        ];
        let collection = FhirPathValue::collection(objects);
        let ctx = EvaluationContext::new(collection, context.registry.clone(), context.model_provider.clone());

        let args = vec![FhirPathValue::String("name".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2); // Only John and Jane have names
                assert_eq!(items.get(0).unwrap(), FhirPathValue::String("John".into()));
                assert_eq!(items[1], FhirPathValue::String("Jane".into()));
            },
            _ => panic!("Expected collection"),
        }
    }

    #[tokio::test]
    async fn test_select_function_string_transform() {
        let func = SelectFunction::new();

        let strings = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into()),
        ];
        let collection = FhirPathValue::collection(strings);
        let ctx = EvaluationContext::new(collection, context.registry.clone(), context.model_provider.clone());

        // Test uppercase transform
        let args = vec![FhirPathValue::String("upper".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items.get(0).unwrap(), FhirPathValue::String("HELLO".into()));
                assert_eq!(items[1], FhirPathValue::String("WORLD".into()));
            },
            _ => panic!("Expected collection"),
        }
    }

    #[tokio::test]
    async fn test_select_function_single_item() {
        let func = SelectFunction::new();

        let obj = FhirPathValue::JsonValue(ArcJsonValue::new(json!({"name": "Test", "value": 42})));
        let ctx = EvaluationContext::new(obj, context.registry.clone(), context.model_provider.clone());

        let args = vec![FhirPathValue::String("value".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[tokio::test]
    async fn test_select_function_empty_result() {
        let func = SelectFunction::new();

        let objects = vec![
            FhirPathValue::JsonValue(ArcJsonValue::new(json!({"age": 30}))),
            FhirPathValue::JsonValue(ArcJsonValue::new(json!({"age": 25}))),
        ];
        let collection = FhirPathValue::collection(objects);
        let ctx = EvaluationContext::new(collection, context.registry.clone(), context.model_provider.clone());

        // Select field that doesn't exist
        let args = vec![FhirPathValue::String("nonexistent".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_select_function_sync() {
        let func = SelectFunction::new();
        
        let obj = FhirPathValue::JsonValue(ArcJsonValue::new(json!({"test": "value"})));
        let ctx = EvaluationContext::new(obj, context.registry.clone(), context.model_provider.clone());
        
        let args = vec![FhirPathValue::String("test".into())];
        let result = func.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::String("value".into()));
    }

    #[tokio::test]
    async fn test_select_function_invalid_args() {
        let func = SelectFunction::new();
        let ctx = EvaluationContext::new(FhirPathValue::Empty, context.registry.clone(), context.model_provider.clone());

        // No arguments
        let result = func.evaluate(&[], &ctx).await;
        assert!(result.is_err());

        // Too many arguments
        let args = vec![FhirPathValue::String("field".into()), FhirPathValue::String("other".into())];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }
}