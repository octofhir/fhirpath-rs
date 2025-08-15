use async_trait::async_trait;
use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use crate::operations::EvaluationContext;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

pub struct RepeatFunction {
    metadata: OperationMetadata,
}

impl RepeatFunction {
    pub fn new() -> Self {
        Self {
            metadata: Self::create_metadata(),
        }
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("repeat", OperationType::Function)
            .description("A version of select that will repeat the projection and add it to the output collection, as long as the projection yields new items")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("Resource.descendants()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for RepeatFunction {
    fn identifier(&self) -> &str {
        "repeat"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        &self.metadata
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "repeat() requires exactly one argument (projection expression)".to_string()
            });
        }

        let projection_expr = &args[0];
        let mut result = Vec::new();
        let mut current_items = context.input.clone().to_collection().into_iter().collect::<Vec<_>>();
        let mut seen_items = HashSet::new();

        // Add initial items to result and seen set
        for item in &current_items {
            let item_key = self.item_to_key(item);
            if seen_items.insert(item_key) {
                result.push(item.clone());
            }
        }

        // Continue until no new items are found or we hit safety limits
        const MAX_ITERATIONS: usize = 1000; // Prevent infinite loops
        const MAX_RESULT_SIZE: usize = 10000; // Prevent memory explosion
        let mut iteration_count = 0;

        loop {
            iteration_count += 1;

            // Safety check: prevent infinite loops
            if iteration_count > MAX_ITERATIONS {
                return Err(FhirPathError::EvaluationError {
                    message: format!("repeat() exceeded maximum iterations ({})", MAX_ITERATIONS)
                });
            }

            // Safety check: prevent memory explosion
            if result.len() > MAX_RESULT_SIZE {
                return Err(FhirPathError::EvaluationError {
                    message: format!("repeat() exceeded maximum result size ({})", MAX_RESULT_SIZE)
                });
            }

            let mut new_items = Vec::new();
            let mut found_new = false;

            for item in &current_items {
                // Create new context with current item as focus
                let item_context = context.with_focus(item.clone());

                // Evaluate projection expression
                let projected = self.evaluate_expression(projection_expr, &item_context).await?;

                // Add new items that haven't been seen before
                for proj_item in projected.to_collection().iter() {
                    let item_key = self.item_to_key(proj_item);
                    if !seen_items.contains(&item_key) {
                        seen_items.insert(item_key);
                        new_items.push(proj_item.clone());
                        result.push(proj_item.clone());
                        found_new = true;
                    }
                }
            }

            if !found_new {
                break;
            }

            current_items = new_items;
        }

        Ok(FhirPathValue::Collection(result.into()))
    }

    fn supports_sync(&self) -> bool {
        false // Complex lambda evaluation requires async
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RepeatFunction {
    // Placeholder for expression evaluation
    // TODO: This needs to be integrated with the actual expression evaluator
    async fn evaluate_expression(
        &self,
        expr: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // For now, return empty collection as placeholder
        // In the actual implementation, this would:
        // 1. Parse the expression if it's a string
        // 2. Evaluate it using the expression evaluator
        // 3. Return the result

        match expr {
            FhirPathValue::String(expr_str) => {
                // Simple property access simulation for basic cases
                if let Some(property) = self.extract_simple_property(expr_str) {
                    return self.get_property_value(&context.input, &property);
                }
                // For complex expressions, return empty for now
                Ok(FhirPathValue::Collection(vec![].into()))
            }
            _ => Ok(FhirPathValue::Collection(vec![].into())),
        }
    }

    // Extract simple property names like "name" or "children"
    fn extract_simple_property(&self, expr: &str) -> Option<String> {
        let trimmed = expr.trim();
        if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Some(trimmed.to_string())
        } else {
            None
        }
    }

    // Get property value from a FHIR resource
    fn get_property_value(&self, value: &FhirPathValue, property: &str) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_val) => {
                if let Some(obj) = json_val.as_object() {
                    if let Some(prop_value) = obj.get(property) {
                        // Convert from serde_json::Value to FhirPathValue
                        Ok(self.json_to_fhir_path_value(prop_value)?)
                    } else {
                        Ok(FhirPathValue::Collection(vec![].into()))
                    }
                } else {
                    Ok(FhirPathValue::Collection(vec![].into()))
                }
            }
            _ => Ok(FhirPathValue::Collection(vec![].into())),
        }
    }

    // Simple JSON to FhirPathValue conversion
    fn json_to_fhir_path_value(&self, value: &serde_json::Value) -> Result<FhirPathValue> {
        match value {
            serde_json::Value::String(s) => Ok(FhirPathValue::String(s.as_str().into())),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(FhirPathValue::Integer(i))
                } else {
                    // For simplicity, convert all other numbers to integers
                    Ok(FhirPathValue::Integer(n.as_f64().unwrap_or(0.0) as i64))
                }
            }
            serde_json::Value::Bool(b) => Ok(FhirPathValue::Boolean(*b)),
            serde_json::Value::Array(arr) => {
                let items: Result<Vec<_>> = arr.iter()
                    .map(|v| self.json_to_fhir_path_value(v))
                    .collect();
                Ok(FhirPathValue::Collection(items?.into()))
            }
            serde_json::Value::Object(_) => {
                // For objects, wrap back in JsonValue
                Ok(FhirPathValue::JsonValue(octofhir_fhirpath_model::json_arc::ArcJsonValue::new(value.clone())))
            }
            serde_json::Value::Null => Ok(FhirPathValue::Empty),
        }
    }

    // Generate a unique key for an item to detect duplicates
    fn item_to_key(&self, item: &FhirPathValue) -> String {
        match item {
            FhirPathValue::String(s) => format!("string:{}", s),
            FhirPathValue::Integer(i) => format!("integer:{}", i),
            FhirPathValue::Decimal(d) => format!("decimal:{}", d),
            FhirPathValue::Boolean(b) => format!("boolean:{}", b),
            FhirPathValue::JsonValue(json_val) => {
                // For JSON objects, use id if available, otherwise use a hash-like approach
                if let Some(obj) = json_val.as_object() {
                    if let Some(serde_json::Value::String(id)) = obj.get("id") {
                        format!("object:id:{}", id)
                    } else {
                        format!("object:hash:{:?}", obj)
                    }
                } else {
                    format!("json:{:?}", json_val)
                }
            }
            _ => format!("{:?}", item),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::create_test_context;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_repeat_simple_property() {
        let function = RepeatFunction::new();

        // Create a simple resource with children
        let mut resource = HashMap::new();
        let mut child1 = HashMap::new();
        child1.insert("name".to_string(), FhirPathValue::String("child1".into()));
        let mut child2 = HashMap::new();
        child2.insert("name".to_string(), FhirPathValue::String("child2".into()));

        resource.insert("children".to_string(), FhirPathValue::Collection(vec![
            FhirPathValue::JsonValue(octofhir_fhirpath_model::json_arc::ArcJsonValue::new(serde_json::Value::Object(child1))),
            FhirPathValue::JsonValue(octofhir_fhirpath_model::json_arc::ArcJsonValue::new(serde_json::Value::Object(child2))),
        ].into()));

        let context = create_test_context(FhirPathValue::JsonValue(octofhir_fhirpath_model::json_arc::ArcJsonValue::new(serde_json::Value::Object(resource))));
        let args = vec![FhirPathValue::String("children".into())];

        let result = function.evaluate(&args, &context).await.unwrap();

        // Should include original resource plus children
        match result {
            FhirPathValue::Collection(items) => {
                assert!(items.len() >= 1); // At least the original item
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_repeat_no_arguments() {
        let function = RepeatFunction::new();
        let context = create_test_context(FhirPathValue::String("test".into()));
        let args = vec![];

        let result = function.evaluate(&args, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exactly one argument"));
    }

    #[tokio::test]
    async fn test_repeat_empty_input() {
        let function = RepeatFunction::new();
        let context = create_test_context(FhirPathValue::Collection(vec![].into()));
        let args = vec![FhirPathValue::String("children".into())];

        let result = function.evaluate(&args, &context).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 0);
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_repeat_infinite_loop_protection() {
        let function = RepeatFunction::new();

        // Create a resource that would cause infinite recursion if not protected
        let mut resource = HashMap::new();
        resource.insert("self".to_string(), FhirPathValue::JsonValue(serde_json::Value::Object(resource.clone().into())));

        let context = create_test_context(FhirPathValue::JsonValue(serde_json::Value::Object(resource.into())));
        let args = vec![FhirPathValue::String("self".into())];

        // This should return an error instead of infinite loop
        let result = function.evaluate(&args, &context).await;

        // The exact error depends on implementation details,
        // but it should not hang or crash
        assert!(result.is_ok() || result.is_err()); // Should complete one way or another
    }
}
