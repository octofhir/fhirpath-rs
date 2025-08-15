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

//! Trace function implementation

use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Trace function - logs a value and returns it unchanged (debugging utility)
#[derive(Debug, Clone)]
pub struct TraceFunction;

impl TraceFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("trace", OperationType::Function)
            .description("Logs the input value with optional name and projection expression, returns input unchanged")
            .example("Patient.name.trace('patient-name')")
            .example("Patient.active.trace()")
            .example("contained.where(criteria).trace('unmatched', id)")
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TraceFunction {
    fn identifier(&self) -> &str {
        "trace"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| TraceFunction::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        let name = if args.is_empty() {
            "trace".to_string()
        } else {
            match &args[0] {
                FhirPathValue::String(s) => s.to_string(),
                FhirPathValue::Collection(items) if items.len() == 1 => {
                    match items.iter().next().unwrap() {
                        FhirPathValue::String(s) => s.to_string(),
                        _ => {
                            return Err(FhirPathError::InvalidArguments {
                                message: "trace() name argument must be a string".to_string(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(FhirPathError::TypeError {
                        message: "trace() name argument must be a string".to_string(),
                    });
                }
            }
        };

        // Determine what to log - input or projection result
        let value_to_log = if args.len() >= 2 {
            // Evaluate projection expression on the input
            let projection_result = self.evaluate_expression(&args[1], context).await?;
            projection_result
        } else {
            // No projection, log the input directly
            context.input.clone()
        };

        // Log the value (using println for now, could be replaced with proper logging)
        println!("[TRACE: {}] {}", name, Self::format_value(&value_to_log));

        // Return the input unchanged (not the projection result)
        Ok(context.input.clone())
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() > 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        let name = if args.is_empty() {
            "trace".to_string()
        } else {
            match &args[0] {
                FhirPathValue::String(s) => s.to_string(),
                FhirPathValue::Collection(items) if items.len() == 1 => {
                    match items.iter().next().unwrap() {
                        FhirPathValue::String(s) => s.to_string(),
                        _ => {
                            return Some(Err(FhirPathError::InvalidArguments {
                                message: "trace() name argument must be a string".to_string(),
                            }));
                        }
                    }
                }
                _ => {
                    return Some(Err(FhirPathError::TypeError {
                        message: "trace() name argument must be a string".to_string(),
                    }));
                }
            }
        };

        // Determine what to log - input or projection result
        let value_to_log = if args.len() >= 2 {
            // For sync evaluation, use a simple synchronous evaluation of projection
            match self.evaluate_expression_sync(&args[1], context) {
                Ok(result) => result,
                Err(e) => return Some(Err(e)),
            }
        } else {
            // No projection, log the input directly
            context.input.clone()
        };

        // Log the value
        println!("[TRACE: {}] {}", name, Self::format_value(&value_to_log));

        // Return the input unchanged (not the projection result)
        Some(Ok(context.input.clone()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TraceFunction {
    // Evaluate expression (similar to repeat function approach)
    async fn evaluate_expression(
        &self,
        expr: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match expr {
            FhirPathValue::String(expr_str) => {
                // Simple property access simulation for basic cases
                if let Some(property) = self.extract_simple_property(expr_str) {
                    return self.get_property_value(&context.input, &property);
                }
                // For complex expressions, return input for now (placeholder)
                Ok(context.input.clone())
            }
            _ => Ok(context.input.clone()),
        }
    }

    // Synchronous version of evaluate_expression
    fn evaluate_expression_sync(
        &self,
        expr: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match expr {
            FhirPathValue::String(expr_str) => {
                // Simple property access simulation for basic cases
                if let Some(property) = self.extract_simple_property(expr_str) {
                    return self.get_property_value(&context.input, &property);
                }
                // For complex expressions, return input for now (placeholder)
                Ok(context.input.clone())
            }
            _ => Ok(context.input.clone()),
        }
    }

    // Extract simple property names like "id" or "name"
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
            FhirPathValue::Collection(collection) => {
                // Apply property access to each item in the collection
                let mut results = Vec::new();
                for item in collection.iter() {
                    let prop_result = self.get_property_value(item, property)?;
                    match prop_result {
                        FhirPathValue::Collection(sub_collection) => {
                            results.extend(sub_collection.into_iter());
                        }
                        other => results.push(other),
                    }
                }
                Ok(FhirPathValue::Collection(results.into()))
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
                let items: Result<Vec<_>> = arr
                    .iter()
                    .map(|v| self.json_to_fhir_path_value(v))
                    .collect();
                Ok(FhirPathValue::Collection(items?.into()))
            }
            serde_json::Value::Object(_) => {
                // For objects, wrap back in JsonValue
                Ok(FhirPathValue::JsonValue(
                    octofhir_fhirpath_model::json_arc::ArcJsonValue::new(value.clone()),
                ))
            }
            serde_json::Value::Null => Ok(FhirPathValue::Empty),
        }
    }

    fn format_value(value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::Empty => "empty()".to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::String(s) => format!("'{}'", s),
            FhirPathValue::Date(d) => format!("@{}", d),
            FhirPathValue::DateTime(dt) => format!("@{}", dt.format("%Y-%m-%dT%H:%M:%S%.3fZ")),
            FhirPathValue::Time(t) => format!("@T{}", t),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    "{}".to_string()
                } else if c.len() == 1 {
                    Self::format_value(c.first().unwrap())
                } else {
                    format!("{{ {} items }}", c.len())
                }
            }
            FhirPathValue::Quantity(_) => "{ quantity }".to_string(),
            FhirPathValue::Resource(_) => "{ resource }".to_string(),
            FhirPathValue::JsonValue(_) => "{ json }".to_string(),
            FhirPathValue::TypeInfoObject { .. } => "{ type }".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FhirPathRegistry;
    use octofhir_fhirpath_model::MockModelProvider;

    #[tokio::test]
    async fn test_trace_function() {
        let func = TraceFunction::new();

        // Test without name
        let ctx = {
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(42), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Test with name
        let args = vec![FhirPathValue::String("my-value".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Test with string value
        let ctx = {
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(
                FhirPathValue::String("hello".into()),
                registry,
                model_provider,
            )
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));
    }

    #[tokio::test]
    async fn test_trace_sync() {
        let func = TraceFunction::new();
        let ctx = {
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Boolean(true), registry, model_provider)
        };

        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_trace_invalid_args() {
        let func = TraceFunction::new();
        let ctx = {
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(1), registry, model_provider)
        };

        // Too many arguments (now accepts up to 2)
        let args = vec![
            FhirPathValue::String("name".into()),
            FhirPathValue::String("projection".into()),
            FhirPathValue::String("extra".into()),
        ];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());

        // Invalid name type
        let args = vec![FhirPathValue::Integer(123)];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_trace_with_projection() {
        let func = TraceFunction::new();

        // Test with JSON object containing an id property
        use serde_json::json;
        let json_data = json!({"id": "123", "name": "test"});
        let json_value = FhirPathValue::JsonValue(
            octofhir_fhirpath_model::json_arc::ArcJsonValue::new(json_data),
        );

        let ctx = {
            use std::sync::Arc;
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(json_value.clone(), registry, model_provider)
        };

        // Test with projection expression "id"
        let args = vec![
            FhirPathValue::String("test-trace".into()),
            FhirPathValue::String("id".into()),
        ];
        let result = func.evaluate(&args, &ctx).await.unwrap();

        // Should return the original input unchanged
        assert_eq!(result, json_value);
    }

    #[tokio::test]
    async fn test_trace_projection_sync() {
        let func = TraceFunction::new();

        // Test with JSON object containing an id property
        use serde_json::json;
        let json_data = json!({"id": "456", "active": true});
        let json_value = FhirPathValue::JsonValue(
            octofhir_fhirpath_model::json_arc::ArcJsonValue::new(json_data),
        );

        let ctx = {
            use std::sync::Arc;
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(json_value.clone(), registry, model_provider)
        };

        // Test sync evaluation with projection
        let args = vec![
            FhirPathValue::String("sync-trace".into()),
            FhirPathValue::String("active".into()),
        ];
        let result = func.try_evaluate_sync(&args, &ctx).unwrap().unwrap();

        // Should return the original input unchanged
        assert_eq!(result, json_value);
    }
}
