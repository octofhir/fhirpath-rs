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

//! Union function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Union function: returns the union of two collections, removing duplicates
pub struct UnionFunction;

impl Default for UnionFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl UnionFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("union", OperationType::Function)
            .description("Returns the union of the input collection and the other collection, removing duplicates. Also available as the | operator.")
            .example("Patient.name.given.union(Patient.name.family)")
            .example("Bundle.entry.union(Bundle.contained)")
            .parameter("other", TypeConstraint::Specific(FhirPathType::Collection), false)
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for UnionFunction {
    fn identifier(&self) -> &str {
        "union"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(UnionFunction::create_metadata);
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

        // Fallback to async evaluation (though union is always sync)
        self.evaluate_union(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_union(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UnionFunction {
    fn evaluate_union(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument (the other collection)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "union() requires exactly one collection argument".to_string(),
            });
        }

        let other = &args[0];

        // Convert both inputs to collections
        let left_items = self.to_collection_items(&context.input);
        let right_items = self.to_collection_items(other);

        // Use FHIRPath equality for deduplication
        let mut result_items = Vec::new();

        // Add items from left collection first
        for item in &left_items {
            if !result_items
                .iter()
                .any(|existing: &FhirPathValue| existing.fhirpath_equals(item))
            {
                result_items.push(item.clone());
            }
        }

        // Add items from right collection, skipping duplicates
        for item in &right_items {
            if !result_items
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
