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

//! Decimal conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use regex::Regex;
use std::sync::OnceLock;

/// ToDecimal function: converts input to Decimal
pub struct ToDecimalFunction;

impl ToDecimalFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toDecimal", OperationType::Function)
            .description("If the input collection contains a single item, this function will return a single decimal if the item is convertible")
            .example("'1.5'.toDecimal()")
            .example("true.toDecimal()")
            .example("42.toDecimal()")
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn decimal_regex() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            // Match: optional sign, digits, optional decimal point and digits
            Regex::new(r"^(\+|-)?\d+(\.\d+)?$").unwrap()
        })
    }

    fn convert_to_decimal(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            // Already a decimal
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),
            
            // Integer conversion
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(Decimal::from(*i))),
            
            // Boolean conversion
            FhirPathValue::Boolean(b) => {
                if *b {
                    Ok(FhirPathValue::Decimal(Decimal::from(1)))
                } else {
                    Ok(FhirPathValue::Decimal(Decimal::from(0)))
                }
            }
            
            // String conversion with validation
            FhirPathValue::String(s) => {
                let trimmed = s.trim();
                if Self::decimal_regex().is_match(trimmed) {
                    match trimmed.parse::<Decimal>() {
                        Ok(d) => Ok(FhirPathValue::Decimal(d)),
                        Err(_) => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection on parse error
                    }
                } else {
                    Ok(FhirPathValue::Collection(vec![].into())) // Empty collection for invalid format
                }
            }
            
            // Empty input
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
            
            // Collection handling
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Collection(vec![].into()))
                } else if c.len() == 1 {
                    Self::convert_to_decimal(c.first().unwrap())
                } else {
                    // Multiple items - return empty collection per FHIRPath spec
                    Ok(FhirPathValue::Collection(vec![].into()))
                }
            }
            
            // Unsupported types
            _ => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection for unsupported types
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToDecimalFunction {
    fn identifier(&self) -> &str {
        "toDecimal"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| ToDecimalFunction::create_metadata());
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

        Self::convert_to_decimal(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_decimal(&context.input))
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

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_to_decimal() {
        let func = ToDecimalFunction::new();

        // Test with decimal
        let ctx = create_test_context(FhirPathValue::Decimal(Decimal::from(5)));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(5)));

        // Test with integer
        let ctx = create_test_context(FhirPathValue::Integer(42));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(42)));

        // Test with boolean true
        let ctx = create_test_context(FhirPathValue::Boolean(true));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(1)));

        // Test with boolean false
        let ctx = create_test_context(FhirPathValue::Boolean(false));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(0)));

        // Test with string that can be parsed as decimal
        let ctx = create_test_context(FhirPathValue::String("123.45".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from_str_exact("123.45").unwrap()));

        // Test with string that cannot be parsed as decimal
        let ctx = create_test_context(FhirPathValue::String("abc".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));

        // Test with empty
        let ctx = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));

        // Test with valid decimal string with whitespace
        let ctx = create_test_context(FhirPathValue::String("  123.45  ".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from_str_exact("123.45").unwrap()));

        // Test with positive signed decimal
        let ctx = create_test_context(FhirPathValue::String("+123.45".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from_str_exact("123.45").unwrap()));

        // Test with negative decimal
        let ctx = create_test_context(FhirPathValue::String("-123.45".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from_str_exact("-123.45").unwrap()));

        // Test with invalid format (multiple dots)
        let ctx = create_test_context(FhirPathValue::String("123.45.67".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));

        // Test with multiple items collection
        let ctx = create_test_context(FhirPathValue::Collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ].into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));
    }

    #[tokio::test]
    async fn test_to_decimal_sync() {
        let func = ToDecimalFunction::new();
        let ctx = create_test_context(FhirPathValue::Decimal(Decimal::from(5)));
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(5)));
        assert!(func.supports_sync());
    }
}
