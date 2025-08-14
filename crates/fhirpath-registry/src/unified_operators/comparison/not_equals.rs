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

//! Not equals operator (!=) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorTypeSignature, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::{ComparisonOperator, UnifiedFhirPathOperator};
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// Not equals operator (!=) implementation
pub struct UnifiedNotEqualsOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedNotEqualsOperator {
    /// Create a new not equals operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "!=",
            OperatorCategory::Comparison,
            8, // Same precedence as equals
            Associativity::Left,
        )
        .display_name("Not Equals")
        .description("Tests inequality between two values. Returns true if values are not equal, false otherwise.")
        .commutative(true)
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("5 != 3", "Integer inequality")
        .example("'hello' != 'world'", "String inequality")
        .keywords(vec!["not equals", "inequality", "comparison", "different", "not equal"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        // Add type signatures for all comparable types - same as equals
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
        ];

        metadata.usage.related_operators = vec![
            "=".to_string(),
            "~".to_string(),
            "!~".to_string(),
        ];

        Self { metadata }
    }
}

impl Default for UnifiedNotEqualsOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedNotEqualsOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;

        // Use the same equality logic as equals operator, but negate the result
        let result = match (&left, &right) {
            // Same type comparisons
            (Integer(l), Integer(r)) => l != r,
            (Decimal(l), Decimal(r)) => l != r,
            (String(l), String(r)) => l != r,
            (Boolean(l), Boolean(r)) => l != r,
            (Date(l), Date(r)) => l != r,
            (DateTime(l), DateTime(r)) => l != r,
            (Time(l), Time(r)) => l != r,
            
            // Cross-type numeric comparisons
            (Integer(l), Decimal(r)) => rust_decimal::Decimal::from(*l) != *r,
            (Decimal(l), Integer(r)) => *l != rust_decimal::Decimal::from(*r),
            
            // Different types are not equal (so != returns true)
            _ => true,
        };

        Ok(Boolean(result))
    }
}

impl ComparisonOperator for UnifiedNotEqualsOperator {
    fn compare(&self, ordering: std::cmp::Ordering) -> bool {
        ordering != std::cmp::Ordering::Equal
    }
}