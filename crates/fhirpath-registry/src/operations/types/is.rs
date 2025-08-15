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

//! Is operator implementation - type checking

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
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// Is operator - checks if value is of specified type
#[derive(Debug, Clone)]
pub struct IsOperation;

impl IsOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("is", OperationType::Function)
            .description(
                "Type checking function - returns true if the input is of the specified type",
            )
            .example("Patient.active.is(Boolean)")
            .example("Patient.name.is(Collection)")
            .example("Patient.is(Patient)")
            .parameter(
                "type",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    pub async fn check_type_with_provider(
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> Result<bool> {
        // Normalize type name - handle both FHIR.String and string formats
        let normalized_type = Self::normalize_type_name(type_name);

        // Handle primitive FHIRPath types first (these don't need ModelProvider)
        match normalized_type.to_lowercase().as_str() {
            "boolean" => return Ok(matches!(value, FhirPathValue::Boolean(_))),
            "integer" => return Ok(matches!(value, FhirPathValue::Integer(_))),
            "decimal" => return Ok(matches!(value, FhirPathValue::Decimal(_))),
            "string" => return Ok(matches!(value, FhirPathValue::String(_))),
            "date" => return Ok(matches!(value, FhirPathValue::Date(_))),
            "datetime" => return Ok(matches!(value, FhirPathValue::DateTime(_))),
            "time" => return Ok(matches!(value, FhirPathValue::Time(_))),
            "collection" => return Ok(matches!(value, FhirPathValue::Collection(_))),
            "empty" => return Ok(matches!(value, FhirPathValue::Empty)),
            _ => {}
        }

        // For FHIR types, use ModelProvider to check type compatibility
        let value_type = Self::extract_fhir_type(value);
        if let Some(value_type) = value_type {
            // Use ModelProvider for accurate FHIR type checking
            let is_compatible = context
                .model_provider
                .is_type_compatible(&value_type, &normalized_type)
                .await;
            Ok(is_compatible)
        } else {
            // Not a FHIR resource/type
            Ok(false)
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
            FhirPathValue::Resource(resource) => resource.resource_type().map(|s| s.to_string()),
            FhirPathValue::JsonValue(json) => json
                .as_json()
                .get("resourceType")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        }
    }
}

#[async_trait]
impl FhirPathOperation for IsOperation {
    fn identifier(&self) -> &str {
        "is"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| IsOperation::create_metadata());
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

        // Handle both direct strings and single-element collections containing strings or identifiers
        let type_name = match &args[0] {
            FhirPathValue::String(s) => s.as_ref().to_string(),
            FhirPathValue::Collection(items) => {
                match items.len() {
                    0 => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
                    1 => {
                        match items.first().unwrap() {
                            FhirPathValue::String(s) => s.as_ref().to_string(),
                            FhirPathValue::TypeInfoObject { namespace, name } => {
                                // For type identifiers, use just the name (e.g., "Integer" from "System.Integer")
                                name.as_ref().to_string()
                            }
                            value => match value.to_string_value() {
                                Some(s) => s,
                                None => {
                                    return Err(FhirPathError::TypeError {
                                        message: format!(
                                            "is operator type argument must be convertible to string, got {}",
                                            value.type_name()
                                        ),
                                    });
                                }
                            },
                        }
                    }
                    _ => {
                        return Err(FhirPathError::TypeError {
                            message: "is operator type argument must be a single value".to_string(),
                        });
                    }
                }
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // For type identifiers, use just the name (e.g., "Integer" from "System.Integer")
                name.as_ref().to_string()
            }
            value => match value.to_string_value() {
                Some(s) => s,
                None => {
                    return Err(FhirPathError::TypeError {
                        message: format!(
                            "is operator type argument must be convertible to string, got {}",
                            value.type_name()
                        ),
                    });
                }
            },
        };

        let result = match &context.input {
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    false
                } else if c.len() == 1 {
                    Self::check_type_with_provider(c.first().unwrap(), &type_name, context).await?
                } else {
                    type_name.to_lowercase() == "collection"
                }
            }
            single_value => {
                Self::check_type_with_provider(single_value, &type_name, context).await?
            }
        };

        Ok(FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Boolean(result),
        ])))
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Type checking requires async ModelProvider calls, so cannot be done synchronously
        None
    }

    fn supports_sync(&self) -> bool {
        false // Type checking requires async ModelProvider calls
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
    use serde_json::json;

    #[tokio::test]
    async fn test_is_operation() {
        let op = IsOperation::new();

        // Test integer
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(42), registry, model_provider)
        };
        let args = vec![FhirPathValue::String("Integer".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );

        let args = vec![FhirPathValue::String("String".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)]))
        );

        // Test string
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(
                FhirPathValue::String("hello".into()),
                registry,
                model_provider,
            )
        };
        let args = vec![FhirPathValue::String("String".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );

        // Test boolean
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Boolean(true), registry, model_provider)
        };
        let args = vec![FhirPathValue::String("Boolean".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );

        // Test empty
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
        };
        let args = vec![FhirPathValue::String("Empty".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );
    }

    #[tokio::test]
    async fn test_is_fhir_resource() {
        let op = IsOperation::new();

        // Test Patient resource
        let patient = json!({
            "resourceType": "Patient",
            "id": "123"
        });
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::JsonValue(patient), registry, model_provider)
        };

        let args = vec![FhirPathValue::String("Patient".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );

        let args = vec![FhirPathValue::String("Resource".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );

        let args = vec![FhirPathValue::String("Organization".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)]))
        );
    }

    #[tokio::test]
    async fn test_is_collection() {
        let op = IsOperation::new();

        // Test collection
        let collection =
            FhirPathValue::collection(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(collection, registry, model_provider)
        };
        let args = vec![FhirPathValue::String("Collection".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );

        // Single item collection
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(collection, registry, model_provider)
        };
        let args = vec![FhirPathValue::String("String".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );
    }

    #[tokio::test]
    async fn test_is_sync() {
        let op = IsOperation::new();
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(3.14), registry, model_provider)
        };

        let args = vec![FhirPathValue::String("Decimal".into())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );
    }

    #[tokio::test]
    async fn test_is_invalid_args() {
        let op = IsOperation::new();
        let ctx = {
            use crate::FhirPathRegistry;
            use octofhir_fhirpath_model::MockModelProvider;
            use std::sync::Arc;

            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(1), registry, model_provider)
        };

        // No arguments
        let result = op.evaluate(&[], &ctx).await;
        assert!(result.is_err());

        // Wrong argument type
        let args = vec![FhirPathValue::Integer(123)];
        let result = op.evaluate(&args, &ctx).await;
        assert!(result.is_err());

        // Collection with string should work
        let args = vec![FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("Integer".into()),
        ]))];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)]))
        );

        // Empty collection should return empty
        let args = vec![FhirPathValue::Collection(Collection::from(vec![]))];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Multiple items in collection should error
        let args = vec![FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("Integer".into()),
            FhirPathValue::String("String".into()),
        ]))];
        let result = op.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }
}
