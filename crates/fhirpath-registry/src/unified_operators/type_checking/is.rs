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

//! Type checking 'is' operator implementation with enhanced metadata

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

/// Type checking 'is' operator implementation
/// Tests if a value is of a specific type according to FHIRPath type system
pub struct UnifiedIsOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedIsOperator {
    /// Create a new 'is' type checking operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "is",
            OperatorCategory::Type,
            10, // FHIRPath spec: 'is' and 'as' have precedence #10
            Associativity::Left,
        )
        .display_name("Type Check (is)")
        .description("Tests whether a value is of a specific type according to FHIRPath type system.")
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("5 is Integer", "Type checking (true)")
        .example("'hello' is String", "Type checking (true)")
        .example("5 is String", "Type checking (false)")
        .keywords(vec!["is", "type", "check", "instanceof", "typeof"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }

    /// Check if a FHIRPath value matches the specified type
    fn check_type(&self, value: &FhirPathValue, type_name: &str) -> bool {
        use FhirPathValue::*;
        
        // Normalize type name - remove System. prefix and convert to lowercase
        let normalized_type = type_name
            .strip_prefix("System.")
            .or_else(|| type_name.strip_prefix("system."))
            .unwrap_or(type_name)
            .to_lowercase();
        
        match (value, normalized_type.as_str()) {
            // Basic types
            (Boolean(_), "boolean") => true,
            (Integer(_), "integer") => true,
            (Decimal(_), "decimal") => true,
            (String(_), "string") => true,
            (Date(_), "date") => true,
            (DateTime(_), "datetime") => true,
            (Time(_), "time") => true,
            (Quantity(_), "quantity") => true,
            
            // Collection types
            (Collection(_), "collection") => true,
            (Empty, "collection") => true, // Empty is a valid collection
            
            // Resource types
            (Resource(resource), type_name) => {
                // Check if the resource type matches
                resource.resource_type()
                    .map(|rt| rt.to_lowercase() == type_name.to_lowercase())
                    .unwrap_or(false)
            }
            
            // Numeric type hierarchy - Integer is also a Decimal in FHIRPath
            (Integer(_), "decimal") => true,
            
            // Any type
            (_, "any") => true,
            
            // Empty collection handling
            (Empty, _) => false,
            
            // Default case - no match
            _ => false,
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

impl Default for UnifiedIsOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedIsOperator {
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
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                }
                
                // For collections, check if all items are of the specified type
                if let Some(type_name) = self.extract_type_name(&right) {
                    let all_match = items.iter().all(|item| self.check_type(item, &type_name));
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(all_match)]))
                } else {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
                }
            }
            
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
            
            // Single value
            _ => {
                if let Some(type_name) = self.extract_type_name(&right) {
                    let matches = self.check_type(&left, &type_name);
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(matches)]))
                } else {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
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
    async fn test_is_basic_types() {
        let operator = UnifiedIsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer is Integer
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // String is String
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("String".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Boolean is Boolean
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::String("Boolean".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_is_type_mismatch() {
        let operator = UnifiedIsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer is not String
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::String("String".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // String is not Integer
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_is_type_hierarchy() {
        let operator = UnifiedIsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer is Decimal (type hierarchy)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::String("Decimal".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Any value is Any
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("test".into()),
                FhirPathValue::String("Any".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_is_collections() {
        let operator = UnifiedIsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Collection of integers is Collection
        let collection = Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(collection),
                FhirPathValue::String("Collection".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Collection of integers - all are Integer
        let collection = Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(collection),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Mixed collection - not all are Integer
        let collection = Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::String("hello".into()),
        ]);
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(collection),
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_is_empty_collection() {
        let operator = UnifiedIsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Empty collection
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::String("Integer".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_is_invalid_type_name() {
        let operator = UnifiedIsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Non-string type name should return false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(123), // Invalid type name
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedIsOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "is");
        assert_eq!(metadata.basic.display_name, "Type Check (is)");
        assert_eq!(metadata.basic.precedence, 10);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Type);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // Type checking is not commutative
    }

    #[test]
    fn test_type_checking_logic() {
        let operator = UnifiedIsOperator::new();
        
        // Test basic type checking
        assert!(operator.check_type(&FhirPathValue::Integer(5), "Integer"));
        assert!(operator.check_type(&FhirPathValue::Integer(5), "integer")); // Case insensitive
        assert!(operator.check_type(&FhirPathValue::String("hello".into()), "String"));
        assert!(operator.check_type(&FhirPathValue::Boolean(true), "Boolean"));
        assert!(operator.check_type(&FhirPathValue::Decimal(Decimal::ONE), "Decimal"));
        
        // Test type hierarchy
        assert!(operator.check_type(&FhirPathValue::Integer(5), "Decimal"));
        
        // Test Any type
        assert!(operator.check_type(&FhirPathValue::Integer(5), "Any"));
        assert!(operator.check_type(&FhirPathValue::String("test".into()), "any"));
        
        // Test mismatches
        assert!(!operator.check_type(&FhirPathValue::Integer(5), "String"));
        assert!(!operator.check_type(&FhirPathValue::String("hello".into()), "Integer"));
        
        // Test empty
        assert!(!operator.check_type(&FhirPathValue::Empty, "Integer"));
        
        // Test collection type
        let collection = Collection::new();
        assert!(operator.check_type(&FhirPathValue::Collection(collection), "Collection"));
        assert!(operator.check_type(&FhirPathValue::Empty, "Collection")); // Empty is a collection
    }
}