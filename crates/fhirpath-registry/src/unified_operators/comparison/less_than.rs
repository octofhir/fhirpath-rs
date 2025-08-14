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

//! Less than operator (<) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::{ComparisonOperator, UnifiedFhirPathOperator};
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// Less than operator (<) implementation
pub struct UnifiedLessThanOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedLessThanOperator {
    /// Create a new less than operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "<",
            OperatorCategory::Comparison,
            9, // Higher precedence than equality
            Associativity::Left,
        )
        .display_name("Less Than")
        .description("Tests if the left operand is less than the right operand.")
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("3 < 5", "Integer comparison")
        .example("'a' < 'b'", "String comparison")
        .keywords(vec!["less", "smaller", "comparison", "order"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedLessThanOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedLessThanOperator {
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

impl ComparisonOperator for UnifiedLessThanOperator {
    fn compare(&self, ordering: std::cmp::Ordering) -> bool {
        ordering == std::cmp::Ordering::Less
    }
}