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

//! As operator implementation - type casting

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{
        Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
        PerformanceComplexity, TypeConstraint,
    },
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use rust_decimal::{prelude::ToPrimitive, prelude::FromPrimitive};

/// As operator - casts value to specified type
#[derive(Debug, Clone)]
pub struct AsOperation;

impl AsOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("as", OperationType::BinaryOperator {
            precedence: 8,
            associativity: Associativity::Left,
        })
            .description("Type casting operator - casts value to specified type or returns empty if conversion fails")
            .example("'123' as Integer")
            .example("Patient.active as Boolean")
            .example("42 as String")
            .parameter("value", TypeConstraint::Any, false)
            .parameter("type", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    async fn cast_value_with_provider(
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Normalize type name - handle both FHIR.String and string formats
        let normalized_type = Self::normalize_type_name(type_name);
        
        // Handle primitive FHIRPath types first (these don't need ModelProvider)
        match normalized_type.to_lowercase().as_str() {
            "boolean" => return Self::cast_to_boolean(value),
            "integer" => return Self::cast_to_integer(value),
            "decimal" => return Self::cast_to_decimal(value),
            "string" => return Self::cast_to_string(value),
            _ => {}
        }

        // For FHIR types, use ModelProvider to check type compatibility
        let value_type = Self::extract_fhir_type(value);
        if let Some(value_type) = value_type {
            // Use ModelProvider for accurate FHIR type checking
            let is_compatible = context.model_provider.is_type_compatible(&value_type, &normalized_type).await;
            if is_compatible {
                Ok(value.clone())
            } else {
                Ok(FhirPathValue::Empty)
            }
        } else {
            // Not a FHIR resource/type
            Ok(FhirPathValue::Empty)
        }
    }

    /// Normalize type names to handle various namespace formats per FHIRPath specification
    /// Supports: String, FHIR.String, System.String, `String`, etc.
    fn normalize_type_name(type_name: &str) -> String {
        // Handle backticks first
        let cleaned = type_name.trim_matches('`');
        
        // Handle various namespace prefixes per FHIRPath specification
        if let Some(stripped) = cleaned.strip_prefix("FHIR.") {
            stripped.to_string()
        } else if let Some(stripped) = cleaned.strip_prefix("fhir.") {
            stripped.to_string()
        } else if let Some(stripped) = cleaned.strip_prefix("System.") {
            stripped.to_string()
        } else if let Some(stripped) = cleaned.strip_prefix("system.") {
            stripped.to_string()
        } else {
            cleaned.to_string()
        }
    }

    fn extract_fhir_type(value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::Resource(resource) => {
                resource.resource_type().map(|s| s.to_string())
            },
            FhirPathValue::JsonValue(json) => {
                json.as_json()
                    .get("resourceType")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            },
            _ => None,
        }
    }

    fn cast_to_boolean(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Boolean(_) => Ok(value.clone()),
            FhirPathValue::String(s) => match s.to_lowercase().as_str() {
                "true" => Ok(FhirPathValue::Boolean(true)),
                "false" => Ok(FhirPathValue::Boolean(false)),
                _ => Ok(FhirPathValue::Empty),
            },
            FhirPathValue::Integer(n) => match n {
                0 => Ok(FhirPathValue::Boolean(false)),
                1 => Ok(FhirPathValue::Boolean(true)),
                _ => Ok(FhirPathValue::Empty),
            },
            _ => Ok(FhirPathValue::Empty),
        }
    }

    fn cast_to_integer(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Integer(_) => Ok(value.clone()),
            FhirPathValue::String(s) => match s.parse::<i64>() {
                Ok(n) => Ok(FhirPathValue::Integer(n)),
                Err(_) => Ok(FhirPathValue::Empty),
            },
            FhirPathValue::Decimal(d) => {
                if d.fract() == rust_decimal::Decimal::ZERO {
                    Ok(FhirPathValue::Integer(d.to_i64().unwrap_or(0)))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Integer(if *b { 1 } else { 0 })),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    fn cast_to_decimal(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Decimal(_) => Ok(value.clone()),
            FhirPathValue::Integer(n) => {
                Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*n)))
            }
            FhirPathValue::String(s) => match s.parse::<f64>() {
                Ok(d) => Ok(FhirPathValue::Decimal(
                    rust_decimal::Decimal::from_f64(d).unwrap_or_default(),
                )),
                Err(_) => Ok(FhirPathValue::Empty),
            },
            _ => Ok(FhirPathValue::Empty),
        }
    }

    fn cast_to_string(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(_) => Ok(value.clone()),
            FhirPathValue::Integer(n) => Ok(FhirPathValue::String(n.to_string().into())),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::String(d.to_string().into())),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::String(b.to_string().into())),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

#[async_trait]
impl FhirPathOperation for AsOperation {
    fn identifier(&self) -> &str {
        "as"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 8,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| AsOperation::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let type_name = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "as operator type argument must be a string".to_string(),
                });
            }
        };

        match &context.input {
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Collection(Collection::from(vec![])))
                } else if c.len() == 1 {
                    let cast_result = Self::cast_value_with_provider(c.first().unwrap(), type_name, context).await?;
                    if matches!(cast_result, FhirPathValue::Empty) {
                        Ok(FhirPathValue::Collection(Collection::from(vec![])))
                    } else {
                        Ok(FhirPathValue::Collection(Collection::from(vec![cast_result])))
                    }
                } else {
                    // For multiple items, try to cast each and return a collection of successful casts
                    let mut results = Vec::new();
                    for item in c.iter() {
                        let cast_result = Self::cast_value_with_provider(item, type_name, context).await?;
                        if !matches!(cast_result, FhirPathValue::Empty) {
                            results.push(cast_result);
                        }
                    }
                    Ok(FhirPathValue::Collection(Collection::from(results)))
                }
            }
            single_value => {
                let cast_result = Self::cast_value_with_provider(single_value, type_name, context).await?;
                if matches!(cast_result, FhirPathValue::Empty) {
                    Ok(FhirPathValue::Collection(Collection::from(vec![])))
                } else {
                    Ok(FhirPathValue::Collection(Collection::from(vec![cast_result])))
                }
            }
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Type casting may require async ModelProvider calls for FHIR types
        None
    }

    fn supports_sync(&self) -> bool {
        false  // Type casting may require async ModelProvider calls for FHIR types
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_as_operation_basic_types() {
        let op = AsOperation::new();

        // String to Integer
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let ctx = EvaluationContext::new(FhirPathValue::String("123".into()), registry.clone(), model_provider.clone());
        let args = vec![FhirPathValue::String("Integer".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(123));

        // Integer to String
        let ctx = EvaluationContext::new(FhirPathValue::Integer(42), registry.clone(), model_provider.clone());
        let args = vec![FhirPathValue::String("String".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::String("42".into()));

        // Boolean to Integer
        let ctx = EvaluationContext::new(FhirPathValue::Boolean(true), registry.clone(), model_provider.clone());
        let args = vec![FhirPathValue::String("Integer".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // String to Boolean
        let ctx = EvaluationContext::new(FhirPathValue::String("true".into()), registry.clone(), model_provider.clone());
        let args = vec![FhirPathValue::String("Boolean".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_as_operation_failed_casts() {
        let op = AsOperation::new();

        // Invalid string to integer
        let ctx = EvaluationContext::new(FhirPathValue::String("invalid".into()), registry.clone(), model_provider.clone());
        let args = vec![FhirPathValue::String("Integer".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Object to boolean
        let ctx =
            EvaluationContext::new(FhirPathValue::JsonValue(serde_json::json!({"test": "value"})), registry.clone(), model_provider.clone());
        let args = vec![FhirPathValue::String("Boolean".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_as_operation_fhir_types() {
        let op = AsOperation::new();

        // Valid Patient resource
        let patient = serde_json::json!({
            "resourceType": "Patient",
            "id": "123"
        });
        let ctx = EvaluationContext::new(FhirPathValue::JsonValue(patient), registry.clone(), model_provider.clone());
        let args = vec![FhirPathValue::String("Patient".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();

        match result {
            FhirPathValue::JsonValue(obj) => {
                assert_eq!(
                    obj.get("resourceType").unwrap().as_str().unwrap(),
                    "Patient"
                );
            }
            _ => panic!("Expected Patient object"),
        }

        // Wrong resource type
        let args = vec![FhirPathValue::String("Organization".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_as_operation_collection() {
        let op = AsOperation::new();

        // Collection with multiple valid items
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("123".into()),
            FhirPathValue::String("456".into()),
            FhirPathValue::String("invalid".into()),
        ]);
        let ctx = EvaluationContext::new(collection, context.registry.clone(), context.model_provider.clone());
        let args = vec![FhirPathValue::String("Integer".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items.get(0).unwrap(), FhirPathValue::Integer(123));
                assert_eq!(items[1], FhirPathValue::Integer(456));
            }
            _ => panic!("Expected collection"),
        }
    }

    #[tokio::test]
    async fn test_as_operation_sync() {
        let op = AsOperation::new();
        let ctx = EvaluationContext::new(FhirPathValue::String("3.14".into()), registry.clone(), model_provider.clone());

        let args = vec![FhirPathValue::String("Decimal".into())];
        let result = op.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Decimal(3.14));
    }

    #[tokio::test]
    async fn test_as_operation_invalid_args() {
        let op = AsOperation::new();
        let ctx = EvaluationContext::new(FhirPathValue::Integer(1), registry.clone(), model_provider.clone());

        // No arguments
        let result = op.evaluate(&[], &ctx).await;
        assert!(result.is_err());

        // Wrong argument type
        let args = vec![FhirPathValue::Integer(123)];
        let result = op.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }
}
