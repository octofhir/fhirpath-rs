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

//! Union operator (|) implementation for FHIRPath

use crate::metadata::{MetadataBuilder, OperationType};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use rustc_hash::FxHashSet;

/// Union operator (|): returns the union of two collections
pub struct UnionOperator;

impl Default for UnionOperator {
    fn default() -> Self {
        Self::new()
    }
}

impl UnionOperator {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> crate::metadata::OperationMetadata {
        MetadataBuilder::new(
            "|",
            OperationType::BinaryOperator {
                associativity: crate::metadata::Associativity::Left,
                precedence: 5, // FHIRPath union precedence
            },
        )
        .description("Returns the union of the left and right collections, removing duplicates")
        .example("Patient.name.given | Patient.name.family")
        .example("Bundle.entry | Bundle.contained")
        .build()
    }
}

#[async_trait]
impl FhirPathOperation for UnionOperator {
    fn identifier(&self) -> &str {
        "|"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            associativity: crate::metadata::Associativity::Left,
            precedence: 5,
        }
    }

    fn metadata(&self) -> &crate::metadata::OperationMetadata {
        static METADATA: once_cell::sync::Lazy<crate::metadata::OperationMetadata> =
            once_cell::sync::Lazy::new(UnionOperator::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Delegate to sync implementation
        self.evaluate_union(args, context)
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArguments {
                message: "| operator requires exactly two operands".to_string(),
            });
        }
        Ok(())
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_union(args, context))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UnionOperator {
    fn evaluate_union(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArguments {
                message: "| operator requires exactly two operands".to_string(),
            });
        }

        let left = &args[0];
        let right = &args[1];

        // Convert both operands to collections
        let left_items = match left {
            FhirPathValue::Collection(items) => items.clone(),
            other => Collection::from(vec![other.clone()]),
        };

        let right_items = match right {
            FhirPathValue::Collection(items) => items.clone(),
            other => Collection::from(vec![other.clone()]),
        };

        // Combine and deduplicate
        let mut seen = FxHashSet::default();
        let mut result = Vec::new();

        // Add left items
        for item in left_items.iter() {
            let key = format!("{item:?}"); // Simple hash key for deduplication
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        // Add right items
        for item in right_items.iter() {
            let key = format!("{item:?}");
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        Ok(FhirPathValue::Collection(Collection::from(result)))
    }
}
