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

//! Unified isDistinct() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified isDistinct() function implementation
pub struct UnifiedIsDistinctFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedIsDistinctFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("isDistinct", vec![], TypeInfo::Boolean);
        
        let metadata = MetadataBuilder::collection_function("isDistinct")
            .display_name("Is Distinct")
            .description("Returns true if all elements in the collection are distinct (no duplicates)")
            .signature(signature)
            .example("(1|2|3).isDistinct()")
            .example("('a'|'b'|'a').isDistinct()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .output_is_collection(true)
            .lsp_snippet("isDistinct()")
            .keywords(vec!["isDistinct", "distinct", "unique", "duplicate", "collection"])
            .usage_pattern_with_frequency(
                "Check for duplicates",
                "(1|2|3).isDistinct()",
                "Validating uniqueness in collections",
                UsageFrequency::Common
            )
            .related_function("distinct")
            .related_function("count")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedIsDistinctFunction {
    fn name(&self) -> &str {
        "isDistinct"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate no arguments
        self.validate_args(args)?;
        
        let items = match &context.input {
            FhirPathValue::Collection(items) => items.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)])), // Empty is distinct
            single_item => vec![single_item.clone()].into(),
        };
        
        // Check if all elements are distinct
        let mut is_distinct = true;
        for (i, item) in items.iter().enumerate() {
            for (j, other_item) in items.iter().enumerate() {
                if i != j && self.values_equal(item, other_item) {
                    is_distinct = false;
                    break;
                }
            }
            if !is_distinct {
                break;
            }
        }
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(is_distinct)]))
    }
}

impl UnifiedIsDistinctFunction {
    /// Check if two values are equal
    fn values_equal(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        use octofhir_fhirpath_model::FhirPathValue::*;
        
        match (left, right) {
            (Empty, Empty) => true,
            (Integer(l), Integer(r)) => l == r,
            (Decimal(l), Decimal(r)) => l == r,
            (Integer(l), Decimal(r)) => rust_decimal::Decimal::from(*l) == *r,
            (Decimal(l), Integer(r)) => *l == rust_decimal::Decimal::from(*r),
            (String(l), String(r)) => l == r,
            (Boolean(l), Boolean(r)) => l == r,
            (Date(l), Date(r)) => l == r,
            (DateTime(l), DateTime(r)) => l == r,
            (Time(l), Time(r)) => l == r,
            (Quantity(l), Quantity(r)) => l.equals_with_conversion(r).unwrap_or(false),
            (TypeInfoObject { namespace: ln, name: lname }, TypeInfoObject { namespace: rn, name: rname }) => {
                ln == rn && lname == rname
            },
            // For collections, check recursively
            (Collection(l), Collection(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| self.values_equal(a, b))
            },
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_is_distinct_function() {
        let is_distinct_func = UnifiedIsDistinctFunction::new();
        
        // Test metadata
        assert_eq!(is_distinct_func.name(), "isDistinct");
        assert_eq!(is_distinct_func.execution_mode(), ExecutionMode::Sync);
        assert!(is_distinct_func.is_pure());
        
        // Test distinct collection (1|2|3)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));
        let result = is_distinct_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
        
        // Test non-distinct collection (1|2|1)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1),
        ]));
        let result = is_distinct_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
        
        // Test single item
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = is_distinct_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
        
        // Test empty collection
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = is_distinct_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
        
        // Test string duplicates
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("a".into()),
        ]));
        let result = is_distinct_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
    }
}