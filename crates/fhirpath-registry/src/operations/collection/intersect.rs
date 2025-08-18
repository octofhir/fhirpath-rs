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

//! Intersect function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Intersect function: returns the intersection of two collections
pub struct IntersectFunction;

impl Default for IntersectFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl IntersectFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("intersect", OperationType::Function)
            .description("Returns the intersection of the input collection and the other collection (items that appear in both collections). Duplicates are removed from the result.")
            .example("Patient.name.given.intersect(Patient.name.family)")
            .example("Bundle.entry.intersect(Bundle.contained)")
            .parameter("other", TypeConstraint::Specific(FhirPathType::Collection), false)
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for IntersectFunction {
    fn identifier(&self) -> &str {
        "intersect"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(IntersectFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync path first for performance
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        // Fallback to async evaluation (though intersect is always sync)
        self.evaluate_intersect(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_intersect(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntersectFunction {
    fn evaluate_intersect(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument (the other collection)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "intersect() requires exactly one collection argument".to_string(),
            });
        }

        let other = &args[0];

        // Convert both inputs to collections
        let left_items = self.to_collection_items(&context.input);
        let right_items = self.to_collection_items(other);

        // Find items from left collection that are also in right collection using FHIRPath equality
        let mut result_items = Vec::new();

        for item in &left_items {
            // Check if item is in right collection and not already in result
            if right_items
                .iter()
                .any(|right_item| item.fhirpath_equals(right_item))
                && !result_items
                    .iter()
                    .any(|existing: &FhirPathValue| existing.fhirpath_equals(item))
            {
                result_items.push(item.clone());
            }
        }

        Ok(FhirPathValue::normalize_collection_result(result_items))
    }

    /// Convert a FhirPathValue to a vector of items (flattening if it's a collection)
    fn to_collection_items(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.clone_for_mutation(),
            FhirPathValue::Empty => vec![],
            _ => vec![value.clone()],
        }
    }
}
