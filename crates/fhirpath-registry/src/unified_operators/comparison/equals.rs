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

//! Equals operator (=) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorTypeSignature, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::{ComparisonOperator, UnifiedFhirPathOperator};
use octofhir_fhirpath_core::EvaluationResult;
use crate::function::EvaluationContext;
use octofhir_fhirpath_model::FhirPathValue;

/// Equals operator (=) implementation
pub struct UnifiedEqualsOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedEqualsOperator {
    /// Create a new equals operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "=",
            OperatorCategory::Comparison,
            8, // Lower precedence than arithmetic
            Associativity::Left,
        )
        .display_name("Equals")
        .description("Tests equality between two values. Supports comparison of compatible types including numbers, strings, booleans, dates, and times.")
        .commutative(true)
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("5 = 5", "Integer equality")
        .example("'hello' = 'hello'", "String equality")
        .example("true = true", "Boolean equality")
        .example("@2023-01-01 = @2023-01-01", "Date equality")
        .keywords(vec!["equals", "equality", "comparison", "same", "equal"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        // Add type signatures for all comparable types
        let mut metadata = metadata;
        metadata.types.type_signatures = vec![
            OperatorTypeSignature {
                left_type: Some("Integer".to_string()),
                right_type: "Integer".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Decimal".to_string()),
                right_type: "Decimal".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Integer".to_string()),
                right_type: "Decimal".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: false,
            },
            OperatorTypeSignature {
                left_type: Some("Decimal".to_string()),
                right_type: "Integer".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: false,
            },
            OperatorTypeSignature {
                left_type: Some("String".to_string()),
                right_type: "String".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Boolean".to_string()),
                right_type: "Boolean".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Date".to_string()),
                right_type: "Date".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("DateTime".to_string()),
                right_type: "DateTime".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Time".to_string()),
                right_type: "Time".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Quantity".to_string()),
                right_type: "Quantity".to_string(),
                result_type: "Boolean".to_string(),
                is_preferred: true,
            },
        ];

        metadata.types.default_result_type = Some("Boolean".to_string());

        metadata.usage.related_operators = vec![
            "!=".to_string(),
            "~".to_string(),
            "!~".to_string(),
            "<".to_string(),
            ">".to_string(),
            "<=".to_string(),
            ">=".to_string(),
        ];

        metadata.usage.common_mistakes = vec![
            "Remember that string comparison is case-sensitive".to_string(),
            "Use '~' for case-insensitive string equality".to_string(),
            "Empty collections are not equal to null".to_string(),
        ];

        Self { metadata }
    }

    /// Helper method for simple value equality comparison (non-recursive)
    #[allow(dead_code)]
    fn values_equal_simple(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        use octofhir_fhirpath_model::FhirPathValue::*;

        match (left, right) {
            (Integer(l), Integer(r)) => l == r,
            (Decimal(l), Decimal(r)) => l == r,
            (String(l), String(r)) => l == r,
            (Boolean(l), Boolean(r)) => l == r,
            (Date(l), Date(r)) => l == r,
            (DateTime(l), DateTime(r)) => l == r,
            (Time(l), Time(r)) => l == r,
            (Integer(l), Decimal(r)) => rust_decimal::Decimal::from(*l) == *r,
            (Decimal(l), Integer(r)) => *l == rust_decimal::Decimal::from(*r),
            (Quantity(l), Quantity(r)) => {
                // Use the Quantity's equals_with_conversion method for proper unit handling
                l.equals_with_conversion(r).unwrap_or(false)
            }
            _ => false,
        }
    }
}

impl Default for UnifiedEqualsOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl UnifiedFhirPathOperator for UnifiedEqualsOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        self.evaluate_comparison_binary(left, right, context).await
    }
}


impl ComparisonOperator for UnifiedEqualsOperator {
    fn compare(&self, ordering: std::cmp::Ordering) -> bool {
        ordering == std::cmp::Ordering::Equal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;
    use rust_decimal::Decimal;

    fn create_test_context() -> EvaluationContext {
        EvaluationContext::new(FhirPathValue::Empty)
    }

    #[tokio::test]
    async fn test_integer_equality() {
        let op = UnifiedEqualsOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_string_equality() {
        let op = UnifiedEqualsOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = op
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("world".into()),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_mixed_numeric_equality() {
        let op = UnifiedEqualsOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::from(5)),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::new(51, 1)),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_boolean_equality() {
        let op = UnifiedEqualsOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = op
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(false),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_different_types() {
        let op = UnifiedEqualsOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::String("5".into()),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_metadata() {
        let op = UnifiedEqualsOperator::new();
        let metadata = op.metadata();

        assert_eq!(metadata.basic.symbol, "=");
        assert_eq!(metadata.basic.display_name, "Equals");
        assert_eq!(metadata.basic.precedence, 8);
        assert!(metadata.basic.is_commutative);
        assert!(metadata.basic.supports_binary);
        assert!(!metadata.basic.supports_unary);

        // All type signatures should result in Boolean
        assert!(metadata.types.type_signatures.iter().all(|sig| sig.result_type == "Boolean"));
    }

    #[tokio::test]
    async fn test_quantity_equality() {
        let op = UnifiedEqualsOperator::new();
        let context = create_test_context();

        // Create two quantities with same value and unit
        let q1 = octofhir_fhirpath_model::Quantity::new(
            Decimal::from(10),
            Some("kg".to_string()),
        );
        let q2 = octofhir_fhirpath_model::Quantity::new(
            Decimal::from(10),
            Some("kg".to_string()),
        );

        let result = op
            .evaluate_binary(
                FhirPathValue::Quantity(std::sync::Arc::new(q1)),
                FhirPathValue::Quantity(std::sync::Arc::new(q2)),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(true));

        // Create two quantities with different values but same unit
        let q3 = octofhir_fhirpath_model::Quantity::new(
            Decimal::from(10),
            Some("kg".to_string()),
        );
        let q4 = octofhir_fhirpath_model::Quantity::new(
            Decimal::from(5),
            Some("kg".to_string()),
        );

        let result = op
            .evaluate_binary(
                FhirPathValue::Quantity(std::sync::Arc::new(q3)),
                FhirPathValue::Quantity(std::sync::Arc::new(q4)),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_comparison_trait_method() {
        let op = UnifiedEqualsOperator::new();

        assert!(op.compare(std::cmp::Ordering::Equal));
        assert!(!op.compare(std::cmp::Ordering::Less));
        assert!(!op.compare(std::cmp::Ordering::Greater));
    }
}
