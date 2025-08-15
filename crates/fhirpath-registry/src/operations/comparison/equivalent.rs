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

//! Equivalent operator (~) implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity, Associativity
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Equivalent operator (~)
/// Returns true if the collections are equivalent, ignoring order and using special string equivalence rules
#[derive(Debug, Clone)]
pub struct EquivalentOperation;

impl EquivalentOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("~", OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        })
            .description("Equivalence comparison operator - true if collections are equivalent, ignoring order and using special string equivalence rules")
            .example("'Hello' ~ 'hello'")
            .example("'Hello World' ~ 'Hello    World'")
            .example("{1, 2} ~ {2, 1}")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    pub fn are_equivalent(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        match (left, right) {
            // Empty collections are equivalent
            (FhirPathValue::Empty, FhirPathValue::Empty) => Ok(true),
            // If either is empty but not both, return false
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(false),
            
            // Single items
            (FhirPathValue::String(l), FhirPathValue::String(r)) => {
                Ok(Self::string_equivalent(l, r))
            }
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => Ok(l == r),
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(Self::decimal_equivalent(*l, *r))
            }
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                Ok(Self::decimal_equivalent(Decimal::from(*l), *r))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                Ok(Self::decimal_equivalent(*l, Decimal::from(*r)))
            }
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => Ok(l == r),
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => {
                Ok(Self::date_equivalent(&l.to_string(), &r.to_string()))
            }
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => {
                Ok(Self::datetime_equivalent(&l.to_string(), &r.to_string()))
            }
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => {
                Ok(Self::time_equivalent(&l.to_string(), &r.to_string()))
            }
            
            // Collections
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                Self::collections_equivalent(l.as_arc().as_ref(), r.as_arc().as_ref())
            }
            
            // Convert single items to collections and compare
            (left_val, FhirPathValue::Collection(r)) => {
                let left_as_collection = vec![left_val.clone()];
                Self::collections_equivalent(&left_as_collection, r.as_arc().as_ref())
            }
            (FhirPathValue::Collection(l), right_val) => {
                let right_as_collection = vec![right_val.clone()];
                Self::collections_equivalent(l.as_arc().as_ref(), &right_as_collection)
            }
            
            // Different types
            _ => Ok(false),
        }
    }

    fn string_equivalent(left: &str, right: &str) -> bool {
        // String equivalence: case-insensitive, normalized whitespace
        let left_normalized = Self::normalize_string(left);
        let right_normalized = Self::normalize_string(right);
        left_normalized.eq_ignore_ascii_case(&right_normalized)
    }

    fn normalize_string(s: &str) -> String {
        // Normalize whitespace: all whitespace characters treated as single space
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn decimal_equivalent(left: Decimal, right: Decimal) -> bool {
        // For equivalence, we use a small epsilon for comparison
        let diff = (left - right).abs();
        diff <= Decimal::new(1, 10) // 0.1 precision
    }

    fn date_equivalent(left: &str, right: &str) -> bool {
        // For equivalence, different precision levels can be equivalent if they overlap
        // For now, implement simple string comparison 
        // TODO: Implement proper date precision comparison
        left == right
    }

    fn datetime_equivalent(left: &str, right: &str) -> bool {
        // TODO: Implement proper datetime precision comparison
        left == right
    }

    fn time_equivalent(left: &str, right: &str) -> bool {
        // TODO: Implement proper time precision comparison  
        left == right
    }

    fn collections_equivalent(left: &[FhirPathValue], right: &[FhirPathValue]) -> Result<bool> {
        // Collections are equivalent if they have same items, order doesn't matter
        if left.len() != right.len() {
            return Ok(false);
        }

        // For each item in left, find equivalent item in right
        for left_item in left {
            let mut found = false;
            for right_item in right {
                if Self::are_equivalent(left_item, right_item)? {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[async_trait]
impl FhirPathOperation for EquivalentOperation {
    fn identifier(&self) -> &str {
        "~"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            EquivalentOperation::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 2, 
                actual: args.len() 
            });
        }

        let result = Self::are_equivalent(&args[0], &args[1])?;
        Ok(FhirPathValue::Boolean(result))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 2, 
                actual: args.len() 
            }));
        }

        match Self::are_equivalent(&args[0], &args[1]) {
            Ok(result) => Some(Ok(FhirPathValue::Boolean(result))),
            Err(e) => Some(Err(e)),
        }
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

    fn create_test_context() -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
    }

    #[tokio::test]
    async fn test_equivalent_strings_case_insensitive() {
        let op = EquivalentOperation::new();
        let ctx = create_test_context();

        // Test "Hello" ~ "hello" (true)
        let args = vec![
            FhirPathValue::String("Hello".into()),
            FhirPathValue::String("hello".into())
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_equivalent_strings_whitespace_normalized() {
        let op = EquivalentOperation::new();
        let ctx = create_test_context();

        // Test "Hello World" ~ "Hello    World" (true)
        let args = vec![
            FhirPathValue::String("Hello World".into()),
            FhirPathValue::String("Hello    World".into())
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_equivalent_integers() {
        let op = EquivalentOperation::new();
        let ctx = create_test_context();

        // Test 5 ~ 5 (true)
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(5)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test 5 ~ 3 (false)
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(3)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_equivalent_collections_order_ignored() {
        let op = EquivalentOperation::new();
        let ctx = create_test_context();

        // Test {1, 2} ~ {2, 1} (true)
        let args = vec![
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2)
            ]),
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(1)
            ])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_equivalent_empty_collections() {
        let op = EquivalentOperation::new();
        let ctx = create_test_context();

        // Test {} ~ {} (true)
        let args = vec![FhirPathValue::Empty, FhirPathValue::Empty];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test {} ~ {1} (false)
        let args = vec![
            FhirPathValue::Empty,
            FhirPathValue::Collection(vec![FhirPathValue::Integer(1)])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_equivalent_different_types() {
        let op = EquivalentOperation::new();
        let ctx = create_test_context();

        // Test "5" ~ 5 (false - different types)
        let args = vec![
            FhirPathValue::String("5".into()),
            FhirPathValue::Integer(5)
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_sync_evaluation() {
        let op = EquivalentOperation::new();
        let ctx = create_test_context();

        let args = vec![
            FhirPathValue::String("Hello".into()),
            FhirPathValue::String("hello".into())
        ];
        let result = op.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}