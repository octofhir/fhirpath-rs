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

//! String concatenation '&' operator implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::UnifiedFhirPathOperator;
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// String concatenation '&' operator implementation
/// Concatenates two values as strings according to FHIRPath specification
pub struct UnifiedConcatenationOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedConcatenationOperator {
    /// Create a new string concatenation operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "&",
            OperatorCategory::String,
            5, // FHIRPath spec: +, -, & have precedence #5
            Associativity::Left,
        )
        .display_name("String Concatenation (&)")
        .description("Concatenates two values as strings, treating empty values as empty strings.")
        .complexity(OperatorComplexity::Linear)
        .memory_usage(OperatorMemoryUsage::Linear)
        .example("'Hello' & ' World'", "String concatenation ('Hello World')")
        .example("'Value: ' & 42", "Value to string concatenation ('Value: 42')")
        .example("'test' & {}", "Empty concatenation ('test')")
        .keywords(vec!["&", "concatenation", "string", "append", "join"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
    
    /// Convert a FHIRPath value to its string representation for concatenation
    fn to_string_for_concat(&self, value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::Empty => String::new(),
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Date(d) => d.format("%Y-%m-%d").to_string(),
            FhirPathValue::DateTime(dt) => dt.to_rfc3339(),
            FhirPathValue::Time(t) => t.format("%H:%M:%S").to_string(),
            FhirPathValue::Quantity(q) => {
                match &q.unit {
                    Some(unit) if !unit.is_empty() => format!("{} {}", q.value, unit),
                    _ => q.value.to_string(),
                }
            }
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    String::new()
                } else {
                    // For collections, convert to comma-separated string
                    c.iter()
                        .map(|item| self.to_string_for_concat(item))
                        .collect::<Vec<_>>()
                        .join(",")
                }
            }
            FhirPathValue::Resource(r) => {
                // For resources, use resource type or a generic representation
                r.resource_type().unwrap_or("Resource").to_string()
            }
            FhirPathValue::JsonValue(j) => {
                // For JSON values, serialize to string
                j.to_string()
            }
            FhirPathValue::TypeInfoObject { .. } => {
                // For type info objects, use a generic representation
                "TypeInfo".to_string()
            }
        }
    }
}

impl Default for UnifiedConcatenationOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedConcatenationOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Convert both operands to strings
        let left_str = self.to_string_for_concat(&left);
        let right_str = self.to_string_for_concat(&right);
        
        // Concatenate and return as string
        let result = left_str + &right_str;
        Ok(FhirPathValue::String(result.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::{FhirPathValue, Collection, Quantity};
    use rust_decimal::Decimal;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    #[tokio::test]
    async fn test_concatenation_strings() {
        let operator = UnifiedConcatenationOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 'Hello' & ' World' = 'Hello World'
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Hello".into()),
                FhirPathValue::String(" World".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Hello World".into()));

        // 'test' & '' = 'test'
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("test".into()),
                FhirPathValue::String("".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("test".into()));
    }

    #[tokio::test]
    async fn test_concatenation_mixed_types() {
        let operator = UnifiedConcatenationOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 'Value: ' & 42 = 'Value: 42'
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Value: ".into()),
                FhirPathValue::Integer(42),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Value: 42".into()));

        // true & ' is correct' = 'true is correct'
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::String(" is correct".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("true is correct".into()));

        // 3.14 & ' pi' = '3.14 pi'
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(314, 2)),
                FhirPathValue::String(" pi".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("3.14 pi".into()));
    }

    #[tokio::test]
    async fn test_concatenation_with_empty() {
        let operator = UnifiedConcatenationOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 'test' & {} = 'test'
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("test".into()),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("test".into()));

        // {} & 'hello' = 'hello'
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));

        // {} & {} = ''
        let result = operator
            .evaluate_binary(FhirPathValue::Empty, FhirPathValue::Empty, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("".into()));
    }

    #[tokio::test]
    async fn test_concatenation_datetime_types() {
        let operator = UnifiedConcatenationOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Date concatenation
        let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Date: ".into()),
                FhirPathValue::Date(date),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Date: 2023-12-25".into()));

        // Time concatenation
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Time: ".into()),
                FhirPathValue::Time(time),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Time: 14:30:00".into()));
    }

    #[tokio::test]
    async fn test_concatenation_quantities() {
        let operator = UnifiedConcatenationOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Quantity with unit
        let quantity = Quantity::new(Decimal::from(5), Some("kg".into()));
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Weight: ".into()),
                FhirPathValue::Quantity(quantity.into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Weight: 5 kg".into()));

        // Quantity without unit
        let quantity = Quantity::new(Decimal::from(10), None);
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Count: ".into()),
                FhirPathValue::Quantity(quantity.into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Count: 10".into()));
    }

    #[tokio::test]
    async fn test_concatenation_collections() {
        let operator = UnifiedConcatenationOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Collection concatenation
        let collection = Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Numbers: ".into()),
                FhirPathValue::Collection(collection),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Numbers: 1,2,3".into()));

        // Empty collection
        let empty_collection = Collection::new();
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("Items: ".into()),
                FhirPathValue::Collection(empty_collection),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Items: ".into()));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedConcatenationOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "&");
        assert_eq!(metadata.basic.display_name, "String Concatenation (&)");
        assert_eq!(metadata.basic.precedence, 5);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::String);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // String concatenation is not commutative
    }
}