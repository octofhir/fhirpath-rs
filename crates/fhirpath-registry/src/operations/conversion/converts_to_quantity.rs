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

//! Quantity conversion functions implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// ConvertsToQuantity function: returns true if the input can be converted to Quantity
pub struct ConvertsToQuantityFunction;

impl ConvertsToQuantityFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToQuantity", OperationType::Function)
            .description("Returns true if the input can be converted to Quantity")
            .example("'1.5\'cm\''.convertsToQuantity()")
            .example("1.convertsToQuantity()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_quantity(value: &FhirPathValue) -> Result<bool> {
        match value {
            // Already a quantity
            FhirPathValue::Quantity(_) => Ok(true),
            
            // Numbers can be converted to quantities (dimensionless)
            FhirPathValue::Integer(_) => Ok(true),
            FhirPathValue::Decimal(_) => Ok(true),
            
            // Booleans can be converted (0 or 1)
            FhirPathValue::Boolean(_) => Ok(true),
            
            // Strings can potentially be parsed as quantities
            FhirPathValue::String(_) => {
                // Try to parse as a quantity using FhirPathValue's built-in method
                Ok(value.to_quantity_value().is_some())
            },
            
            // Empty collection returns empty result
            FhirPathValue::Empty => Ok(true),
            
            // Handle collections
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true) // Empty collection returns empty result
                } else if c.len() == 1 {
                    Self::can_convert_to_quantity(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message: "convertsToQuantity() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            },
            
            // Other types cannot be converted
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToQuantityFunction {
    fn identifier(&self) -> &str {
        "convertsToQuantity"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ConvertsToQuantityFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(_args, context) {
            return result;
        }

        match Self::can_convert_to_quantity(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match Self::can_convert_to_quantity(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_converts_to_quantity() {
        let func = ConvertsToQuantityFunction::new();
        
        // Test with quantity
        let quantity = Quantity::unitless(Decimal::from(5i64));
        let ctx = create_test_context(FhirPathValue::from(quantity));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with integer
        let ctx = create_test_context(FhirPathValue::Integer(42));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with decimal
        let ctx = create_test_context(FhirPathValue::Decimal(Decimal::from(3i64)));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with boolean
        let ctx = create_test_context(FhirPathValue::Boolean(true));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with valid quantity string
        let ctx = create_test_context(FhirPathValue::String("5 'kg'".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with invalid quantity string
        let ctx = create_test_context(FhirPathValue::String("invalid".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty
        let ctx = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_converts_to_quantity_sync() {
        let func = ConvertsToQuantityFunction::new();
        let quantity = Quantity::unitless(Decimal::from(5));
        let ctx = create_test_context(FhirPathValue::from(quantity));
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        assert!(func.supports_sync());
    }
}
