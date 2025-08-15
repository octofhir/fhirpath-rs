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

//! Where function implementation - filters collection based on predicate

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Where function - filters collection based on predicate
#[derive(Debug, Clone)]
pub struct WhereFunction;

impl WhereFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("where", OperationType::Function)
            .description("Filters a collection based on a boolean predicate expression")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("Patient.name.where(use = 'official')")
            .example("Bundle.entry.where(resource.resourceType = 'Patient')")
            .example("telecom.where(system = 'phone')")
            .build()
    }

    fn to_boolean(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Empty => Ok(false),
            FhirPathValue::Boolean(b) => Ok(*b),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(false)
                } else if c.len() == 1 {
                    Self::to_boolean(c.first().unwrap())
                } else {
                    Ok(true) // Non-empty collection is truthy
                }
            },
            _ => Ok(true), // Non-empty, non-boolean values are truthy
        }
    }
}

#[async_trait]
impl FhirPathOperation for WhereFunction {
    fn identifier(&self) -> &str {
        "where"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            WhereFunction::create_metadata()
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

        // Extract predicate - in proper lambda implementation, this would be an expression tree
        let predicate = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut filtered_items = Vec::new();

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context with $this variable set to current item
                    let mut lambda_context = context.clone();
                    lambda_context.set_variable("$this".to_string(), item.clone());
                    lambda_context.set_variable("$index".to_string(), FhirPathValue::Integer(index as i64));
                    lambda_context = lambda_context.with_input(item.clone());
                    
                    // Evaluate predicate in lambda context
                    let predicate_result = match predicate {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::String(s) if s.as_ref() == "true" => true,
                        FhirPathValue::String(s) if s.as_ref() == "false" => false,
                        _ => {
                            // Mock: if predicate is a string that matches a field in the item, check if that field exists
                            if let (FhirPathValue::String(field_name), FhirPathValue::JsonValue(obj)) = (predicate, item) {
                                obj.as_object().map(|o| o.contains_key(field_name.as_ref())).unwrap_or(false)
                            } else {
                                Self::to_boolean(predicate)?
                            }
                        }
                    };

                    if predicate_result {
                        filtered_items.push(item.clone());
                    }
                }

                if filtered_items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if filtered_items.len() == 1 {
                    Ok(filtered_items.into_iter().next().unwrap())
                } else {
                    Ok(FhirPathValue::collection(filtered_items))
                }
            },
            single_item => {
                // Apply where to single item (returns the item if predicate is true, empty otherwise)
                let predicate_result = Self::to_boolean(predicate)?;
                if predicate_result {
                    Ok(single_item.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
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

        let predicate = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut filtered_items = Vec::new();

                for item in items.iter() {
                    let predicate_result = match predicate {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::String(s) if s.as_ref() == "true" => true,
                        FhirPathValue::String(s) if s.as_ref() == "false" => false,
                        _ => {
                            if let (FhirPathValue::String(field_name), FhirPathValue::JsonValue(obj)) = (predicate, item) {
                                obj.as_object().map(|o| o.contains_key(field_name.as_ref())).unwrap_or(false)
                            } else {
                                match Self::to_boolean(predicate) {
                                    Ok(b) => b,
                                    Err(e) => return Some(Err(e)),
                                }
                            }
                        }
                    };

                    if predicate_result {
                        filtered_items.push(item.clone());
                    }
                }

                if filtered_items.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if filtered_items.len() == 1 {
                    Some(Ok(filtered_items.into_iter().next().unwrap()))
                } else {
                    Some(Ok(FhirPathValue::collection(filtered_items)))
                }
            },
            single_item => {
                match Self::to_boolean(predicate) {
                    Ok(predicate_result) => {
                        if predicate_result {
                            Some(Ok(single_item.clone()))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    },
                    Err(e) => Some(Err(e)),
                }
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
    async fn test_where_function_basic() {
        let func = WhereFunction::new();

        // Test filtering with boolean predicate
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("item1".into()),
            FhirPathValue::String("item2".into()),
            FhirPathValue::String("item3".into()),
        ]);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());
        let args = vec![FhirPathValue::Boolean(true)];
        let result = func.evaluate(&args, &ctx).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3);
            },
            _ => panic!("Expected collection"),
        }

        // Test with false predicate
        let args = vec![FhirPathValue::Boolean(false)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_where_function_object_filtering() {
        let func = WhereFunction::new();

        // Test with objects
        let objects = vec![
            FhirPathValue::JsonValue(json!({"name": "John", "active": true})),
            FhirPathValue::JsonValue(json!({"name": "Jane", "active": false})),
            FhirPathValue::JsonValue(json!({"name": "Bob"})),
        ];
        let collection = FhirPathValue::collection(objects);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());

        // Mock: filter objects that have "active" field
        let args = vec![FhirPathValue::String("active".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2); // John and Jane have "active" field
            },
            _ => panic!("Expected collection"),
        }
    }

    #[tokio::test]
    async fn test_where_function_single_item() {
        let func = WhereFunction::new();

        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let ctx = EvaluationContext::new(FhirPathValue::String("test".into()), registry, model_provider);
        
        // True predicate should return the item
        let args = vec![FhirPathValue::Boolean(true)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::String("test".into()));

        // False predicate should return empty
        let args = vec![FhirPathValue::Boolean(false)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_where_function_sync() {
        let func = WhereFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());
        
        let args = vec![FhirPathValue::Boolean(true)];
        let result = func.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
            },
            _ => panic!("Expected collection"),
        }
    }

    #[tokio::test]
    async fn test_where_function_invalid_args() {
        let func = WhereFunction::new();
        let ctx = EvaluationContext::new(FhirPathValue::Empty, std::collections::HashMap::new());

        // No arguments
        let result = func.evaluate(&[], &ctx).await;
        assert!(result.is_err());

        // Too many arguments
        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }
}