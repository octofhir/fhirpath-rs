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

//! Children function implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, PerformanceComplexity
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Children function - returns a collection with all immediate child nodes
#[derive(Debug, Clone)]
pub struct ChildrenFunction;

impl ChildrenFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("children", OperationType::Function)
            .description("Returns a collection with all immediate child nodes of all items in the input collection")
            .example("Patient.children()")
            .example("Bundle.entry.children()")
            .returns(TypeConstraint::Collection(Box::new(TypeConstraint::Any)))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn get_children_from_value(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_val) => {
                if let Some(obj) = json_val.as_object() {
                    // Get all property values as children
                    obj.values().map(|value| FhirPathValue::from(value.clone())).collect()
                } else {
                    Vec::new()
                }
            }
            FhirPathValue::Collection(items) => {
                // Get children of all items in collection
                let mut all_children = Vec::new();
                for item in items.iter() {
                    all_children.extend(self.get_children_from_value(item));
                }
                all_children
            }
            // Primitive values don't have children
            _ => Vec::new(),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ChildrenFunction {
    fn identifier(&self) -> &str {
        "children"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ChildrenFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            });
        }

        let input = &context.input;
        let children = self.get_children_from_value(input);

        Ok(FhirPathValue::Collection(Collection::from(children)))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // Validate no arguments
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            }));
        }

        let input = &context.input;
        let children = self.get_children_from_value(input);

        Some(Ok(FhirPathValue::Collection(Collection::from(children))))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_children_object() -> Result<()> {
        let function = ChildrenFunction::new();

        // Create a simple object with properties
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), FhirPathValue::String("John".into()));
        obj.insert("age".to_string(), FhirPathValue::Integer(30));
        obj.insert("active".to_string(), FhirPathValue::Boolean(true));

        let context = EvaluationContext::new(FhirPathValue::JsonValue(serde_json::Value::Object(obj.into())));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Collection(children) => {
                assert_eq!(children.len(), 3);

                // Should contain all the property values
                let values: Vec<_> = children.iter().cloned().collect();
                assert!(values.contains(&FhirPathValue::String("John".into())));
                assert!(values.contains(&FhirPathValue::Integer(30)));
                assert!(values.contains(&FhirPathValue::Boolean(true)));
            }
            _ => panic!("Expected Collection value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_children_array() -> Result<()> {
        let function = ChildrenFunction::new();

        // Create an array
        let array = vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::Integer(42),
        ];

        let context = EvaluationContext::new(FhirPathValue::Collection(array.clone()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Collection(children) => {
                assert_eq!(children.len(), 3);

                // Should contain all array elements
                let values: Vec<_> = children.iter().cloned().collect();
                assert_eq!(values, array);
            }
            _ => panic!("Expected Collection value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_children_nested_object() -> Result<()> {
        let function = ChildrenFunction::new();

        // Create a nested object structure
        let mut inner_obj = HashMap::new();
        inner_obj.insert("street".to_string(), FhirPathValue::String("123 Main St".into()));
        inner_obj.insert("city".to_string(), FhirPathValue::String("Anytown".into()));

        let mut outer_obj = HashMap::new();
        outer_obj.insert("name".to_string(), FhirPathValue::String("John".into()));
        outer_obj.insert("address".to_string(), FhirPathValue::JsonValue(serde_json::Value::Object(inner_obj.into())));

        let context = EvaluationContext::new(FhirPathValue::JsonValue(serde_json::Value::Object(outer_obj.into())));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Collection(children) => {
                assert_eq!(children.len(), 2);

                // Should contain name string and address object
                let values: Vec<_> = children.iter().cloned().collect();
                assert!(values.contains(&FhirPathValue::String("John".into())));

                // Check that the address object is included
                let has_address_obj = values.iter().any(|v| matches!(v, FhirPathValue::JsonValue(_)));
                assert!(has_address_obj);
            }
            _ => panic!("Expected Collection value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_children_collection() -> Result<()> {
        let function = ChildrenFunction::new();

        // Create a collection with multiple objects
        let mut obj1 = HashMap::new();
        obj1.insert("id".to_string(), FhirPathValue::String("1".into()));

        let mut obj2 = HashMap::new();
        obj2.insert("id".to_string(), FhirPathValue::String("2".into()));
        obj2.insert("name".to_string(), FhirPathValue::String("Test".into()));

        let collection = Collection::from(vec![
            FhirPathValue::JsonValue(serde_json::Value::Object(obj1.into())),
            FhirPathValue::JsonValue(serde_json::Value::Object(obj2.into())),
        ]);

        let context = EvaluationContext::new(FhirPathValue::Collection(collection));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Collection(children) => {
                assert_eq!(children.len(), 3); // 1 from obj1 + 2 from obj2

                // Should contain all property values from both objects
                let values: Vec<_> = children.iter().cloned().collect();
                assert!(values.contains(&FhirPathValue::String("1".into())));
                assert!(values.contains(&FhirPathValue::String("2".into())));
                assert!(values.contains(&FhirPathValue::String("Test".into())));
            }
            _ => panic!("Expected Collection value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_children_primitive() -> Result<()> {
        let function = ChildrenFunction::new();

        // Primitive values should have no children
        let context = EvaluationContext::new(FhirPathValue::String("test".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Collection(children) => {
                assert!(children.is_empty());
            }
            _ => panic!("Expected Collection value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_children_empty_object() -> Result<()> {
        let function = ChildrenFunction::new();

        let context = EvaluationContext::new(FhirPathValue::JsonValue(serde_json::Value::Object(HashMap::new().into())));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Collection(children) => {
                assert!(children.is_empty());
            }
            _ => panic!("Expected Collection value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_children_invalid_args() -> () {
        let function = ChildrenFunction::new();
        let context = EvaluationContext::new(FhirPathValue::String("test".into()));

        let result = function.evaluate(&[FhirPathValue::String("extra".into())], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::InvalidArgumentCount { expected, actual, .. }) = result {
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }
    }

    #[test]
    fn test_children_sync() -> Result<()> {
        let function = ChildrenFunction::new();

        let mut obj = HashMap::new();
        obj.insert("test".to_string(), FhirPathValue::String("value".into()));

        let context = EvaluationContext::new(FhirPathValue::JsonValue(serde_json::Value::Object(obj.into())));
        let result = function.try_evaluate_sync(&[], &context)
            .unwrap()?;

        match result {
            FhirPathValue::Collection(children) => {
                assert_eq!(children.len(), 1);
                assert_eq!(children.iter().next().unwrap(), &FhirPathValue::String("value".into()));
            }
            _ => panic!("Expected Collection value"),
        }

        Ok(())
    }
}
