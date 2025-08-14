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

//! Type casting 'as' operator implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::UnifiedFhirPathOperator;
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use rust_decimal::prelude::ToPrimitive;

/// Type casting 'as' operator implementation
/// Attempts to cast a value to a specific type according to FHIRPath type system
pub struct UnifiedAsOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedAsOperator {
    /// Create a new 'as' type casting operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "as",
            OperatorCategory::Type,
            10, // FHIRPath spec: 'is' and 'as' have precedence #10
            Associativity::Left,
        )
        .display_name("Type Cast (as)")
        .description("Attempts to cast a value to a specific type according to FHIRPath type system.")
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Linear)
        .example("5 as Decimal", "Type casting (5.0)")
        .example("'123' as Integer", "String to integer conversion")
        .example("5 as String", "Integer to string conversion")
        .keywords(vec!["as", "cast", "convert", "type", "coerce"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }

    /// Attempt to cast a FHIRPath value to the specified type
    fn cast_value(&self, value: &FhirPathValue, type_name: &str) -> Option<FhirPathValue> {
        use FhirPathValue::*;
        
        let target_type = type_name.to_lowercase();
        
        match (value, target_type.as_str()) {
            // Same type - return as-is
            (Boolean(_), "boolean") => Some(value.clone()),
            (Integer(_), "integer") => Some(value.clone()),
            (Decimal(_), "decimal") => Some(value.clone()),
            (String(_), "string") => Some(value.clone()),
            (Date(_), "date") => Some(value.clone()),
            (DateTime(_), "datetime") => Some(value.clone()),
            (Time(_), "time") => Some(value.clone()),
            (Quantity(_), "quantity") => Some(value.clone()),
            (Collection(_), "collection") => Some(value.clone()),
            
            // Integer to Decimal (widening conversion)
            (Integer(i), "decimal") => Some(FhirPathValue::Decimal(rust_decimal::Decimal::from(*i))),
            
            // Decimal to Integer (narrowing conversion - only if no fractional part)
            (Decimal(d), "integer") => {
                if d.fract().is_zero() {
                    d.to_i64().map(|i| FhirPathValue::Integer(i))
                } else {
                    None // Cannot convert decimal with fractional part to integer
                }
            }
            
            // String conversions
            (String(s), "integer") => {
                s.trim().parse::<i64>().ok().map(|i| FhirPathValue::Integer(i))
            }
            (String(s), "decimal") => {
                s.trim().parse::<rust_decimal::Decimal>().ok().map(|d| FhirPathValue::Decimal(d))
            }
            (String(s), "boolean") => {
                match s.trim().to_lowercase().as_str() {
                    "true" => Some(FhirPathValue::Boolean(true)),
                    "false" => Some(FhirPathValue::Boolean(false)),
                    _ => None,
                }
            }
            
            // To String conversions
            (Integer(i), "string") => Some(FhirPathValue::String(i.to_string().into())),
            (Decimal(d), "string") => Some(FhirPathValue::String(d.to_string().into())),
            (Boolean(b), "string") => Some(FhirPathValue::String(b.to_string().into())),
            (Date(d), "string") => Some(FhirPathValue::String(d.format("%Y-%m-%d").to_string().into())),
            (DateTime(dt), "string") => Some(FhirPathValue::String(dt.to_rfc3339().into())),
            (Time(t), "string") => Some(FhirPathValue::String(t.format("%H:%M:%S").to_string().into())),
            
            // Any type - always succeeds
            (_, "any") => Some(value.clone()),
            
            // Collection handling
            (Collection(_), target_type) => {
                // For collections, we return the collection as-is if it matches,
                // or None if it doesn't
                if target_type == "collection" {
                    Some(value.clone())
                } else {
                    None
                }
            }
            
            // Empty collection handling
            (Empty, "collection") => Some(value.clone()),
            (Empty, _) => None,
            
            // Default case - no conversion available
            _ => None,
        }
    }

    /// Parse type name from FHIRPath value (should be a string)
    fn extract_type_name(&self, type_value: &FhirPathValue) -> Option<String> {
        match type_value {
            FhirPathValue::String(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

impl Default for UnifiedAsOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedAsOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Handle collection on left side
        match &left {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }
                
                if let Some(type_name) = self.extract_type_name(&right) {
                    // For collections, attempt to cast each item
                    let mut cast_results = Vec::new();
                    
                    for item in items.iter() {
                        if let Some(cast_item) = self.cast_value(item, &type_name) {
                            cast_results.push(cast_item);
                        } else {
                            // If any item cannot be cast, return empty
                            return Ok(FhirPathValue::Empty);
                        }
                    }
                    
                    // If all items were successfully cast
                    if cast_results.len() == 1 {
                        // Single item - return unwrapped
                        Ok(cast_results.into_iter().next().unwrap())
                    } else {
                        // Multiple items - return as collection
                        Ok(FhirPathValue::Collection(Collection::from_vec(cast_results)))
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            
            // Single value
            _ => {
                if let Some(type_name) = self.extract_type_name(&right) {
                    if let Some(cast_value) = self.cast_value(&left, &type_name) {
                        Ok(cast_value)
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::{FhirPathValue, Collection};
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_as_same_type() {
        let operator = UnifiedAsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer as Integer
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(5));

        // String as String
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("String".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));
    }

    #[tokio::test]
    async fn test_as_numeric_conversions() {
        let operator = UnifiedAsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer as Decimal
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::String("Decimal".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(5)));

        // Decimal as Integer (whole number)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::from(7)),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(7));

        // Decimal with fractional part as Integer should fail
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(75, 1)), // 7.5
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_as_string_conversions() {
        let operator = UnifiedAsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // String to Integer
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("123".into()),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(123));

        // String to Boolean
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("true".into()),
                FhirPathValue::String("Boolean".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Integer to String
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(42),
                FhirPathValue::String("String".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("42".into()));

        // Boolean to String
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::String("String".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("true".into()));
    }

    #[tokio::test]
    async fn test_as_invalid_conversions() {
        let operator = UnifiedAsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Invalid string to integer
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("not-a-number".into()),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Invalid string to boolean
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("maybe".into()),
                FhirPathValue::String("Boolean".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_as_collections() {
        let operator = UnifiedAsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Collection of integers as Decimal
        let collection = Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(collection),
                FhirPathValue::String("Decimal".into()),
                &context,
            )
            .await
            .unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items.iter().next().unwrap(), &FhirPathValue::Decimal(Decimal::from(1)));
                assert_eq!(items.iter().nth(1).unwrap(), &FhirPathValue::Decimal(Decimal::from(2)));
            }
            _ => panic!("Expected collection result"),
        }

        // Single item collection unwraps
        let collection = Collection::from_vec(vec![FhirPathValue::Integer(5)]);
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(collection),
                FhirPathValue::String("Decimal".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(5)));
    }

    #[tokio::test]
    async fn test_as_any_type() {
        let operator = UnifiedAsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Any value as Any
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("test".into()),
                FhirPathValue::String("Any".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("test".into()));
    }

    #[tokio::test]
    async fn test_as_empty_collection() {
        let operator = UnifiedAsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Empty as Integer
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedAsOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "as");
        assert_eq!(metadata.basic.display_name, "Type Cast (as)");
        assert_eq!(metadata.basic.precedence, 10);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Type);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // Type casting is not commutative
    }

    #[test]
    fn test_cast_value_logic() {
        let operator = UnifiedAsOperator::new();
        
        // Test basic casts
        assert_eq!(
            operator.cast_value(&FhirPathValue::Integer(5), "Integer"),
            Some(FhirPathValue::Integer(5))
        );
        
        assert_eq!(
            operator.cast_value(&FhirPathValue::Integer(5), "Decimal"),
            Some(FhirPathValue::Decimal(Decimal::from(5)))
        );
        
        assert_eq!(
            operator.cast_value(&FhirPathValue::String("123".into()), "Integer"),
            Some(FhirPathValue::Integer(123))
        );
        
        assert_eq!(
            operator.cast_value(&FhirPathValue::Integer(42), "String"),
            Some(FhirPathValue::String("42".into()))
        );
        
        // Test failed casts
        assert_eq!(
            operator.cast_value(&FhirPathValue::String("not-a-number".into()), "Integer"),
            None
        );
        
        // Test Decimal with fractional part to Integer
        assert_eq!(
            operator.cast_value(&FhirPathValue::Decimal(Decimal::new(75, 1)), "Integer"),
            None
        );
        
        // Test Decimal without fractional part to Integer
        assert_eq!(
            operator.cast_value(&FhirPathValue::Decimal(Decimal::from(7)), "Integer"),
            Some(FhirPathValue::Integer(7))
        );
    }
}