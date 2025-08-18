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

//! SubsetOf function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// SubsetOf function: returns true if the input collection is a subset of the other collection
pub struct SubsetOfFunction;

impl Default for SubsetOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SubsetOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("subsetOf", OperationType::Function)
            .description("Returns true if the input collection is a subset of the other collection (all items in the input collection are also in the other collection). Empty collection is a subset of any collection.")
            .example("Patient.name.given.subsetOf(Patient.name.family)")
            .example("Bundle.entry.subsetOf(Bundle.contained)")
            .parameter("other", TypeConstraint::Specific(FhirPathType::Collection), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SubsetOfFunction {
    fn identifier(&self) -> &str {
        "subsetOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(SubsetOfFunction::create_metadata);
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

        // Fallback to async evaluation (though subsetOf is always sync)
        self.evaluate_subset_of(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_subset_of(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SubsetOfFunction {
    fn evaluate_subset_of(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument (the superset collection)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "subsetOf() requires exactly one collection argument".to_string(),
            });
        }

        let other = &args[0];

        // Convert both inputs to collections
        let left_items = self.to_collection_items(&context.input);
        let right_items = self.to_collection_items(other);

        // Empty collection is a subset of any collection
        if left_items.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }

        // Check if all items from left collection are in right collection using FHIRPath equality
        for item in &left_items {
            if !right_items
                .iter()
                .any(|right_item| item.fhirpath_equals(right_item))
            {
                // Found an item in left that's not in right - not a subset
                return Ok(FhirPathValue::Boolean(false));
            }
        }

        // All items in left are in right - it's a subset
        Ok(FhirPathValue::Boolean(true))
    }

    /// Convert a FhirPathValue to a vector of items (flattening if it's a collection)
    fn to_collection_items(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.clone_for_mutation(),
            FhirPathValue::Empty => vec![],
            _ => vec![value.clone()],
        }
    }

    /// Convert a FhirPathValue to a comparable key for subset detection
    fn value_to_comparable_key(&self, value: &FhirPathValue) -> Result<String> {
        match value {
            FhirPathValue::String(s) => Ok(format!("string:{}", s.as_ref())),
            FhirPathValue::Integer(i) => Ok(format!("integer:{i}")),
            FhirPathValue::Decimal(d) => Ok(format!("decimal:{d}")),
            FhirPathValue::Boolean(b) => Ok(format!("boolean:{b}")),
            FhirPathValue::Date(d) => Ok(format!("date:{d}")),
            FhirPathValue::DateTime(dt) => Ok(format!("datetime:{dt}")),
            FhirPathValue::Time(t) => Ok(format!("time:{t}")),
            FhirPathValue::JsonValue(json) => {
                Ok(format!("json:{}", json.to_string().unwrap_or_default()))
            }
            FhirPathValue::Collection(_) => {
                // Collections are compared structurally - convert to JSON representation
                Ok(format!(
                    "collection:{}",
                    sonic_rs::to_string(value).map_err(|_| {
                        FhirPathError::InvalidArguments {
                            message: "Cannot serialize collection for comparison".to_string(),
                        }
                    })?
                ))
            }
            FhirPathValue::Empty => Ok("empty".to_string()),
            FhirPathValue::Quantity(q) => Ok(format!("quantity:{q}")),
            FhirPathValue::Resource(r) => {
                let id = r
                    .as_json_value()
                    .get_property("id")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                Ok(format!("resource:{id}"))
            }
            FhirPathValue::TypeInfoObject { name, .. } => Ok(format!("typeinfo:{name}")),
        }
    }
}
