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

//! SupersetOf function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// SupersetOf function: returns true if the input collection is a superset of the other collection
pub struct SupersetOfFunction;

impl Default for SupersetOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SupersetOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("supersetOf", OperationType::Function)
            .description("Returns true if the input collection is a superset of the other collection (all items in the other collection are also in the input collection). Any collection is a superset of an empty collection.")
            .example("Patient.name.given.supersetOf(Patient.name.family)")
            .example("Bundle.entry.supersetOf(Bundle.contained)")
            .parameter("other", TypeConstraint::Specific(FhirPathType::Collection), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SupersetOfFunction {
    fn identifier(&self) -> &str {
        "supersetOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(SupersetOfFunction::create_metadata);
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

        // Fallback to async evaluation (though supersetOf is always sync)
        self.evaluate_superset_of(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_superset_of(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SupersetOfFunction {
    fn evaluate_superset_of(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument (the subset collection)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "supersetOf() requires exactly one collection argument".to_string(),
            });
        }

        let other = &args[0];

        // Convert both inputs to collections
        let left_items = self.to_collection_items(&context.input);
        let right_items = self.to_collection_items(other);

        // Any collection is a superset of empty collection
        if right_items.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }

        // Check if all items from right collection are in left collection using FHIRPath equality
        for item in &right_items {
            if !left_items
                .iter()
                .any(|left_item| item.fhirpath_equals(left_item))
            {
                // Found an item in right that's not in left - not a superset
                return Ok(FhirPathValue::Boolean(false));
            }
        }

        // All items in right are in left - it's a superset
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

    /// Convert a FhirPathValue to a comparable key for superset detection
    fn value_to_comparable_key(&self, value: &FhirPathValue) -> Result<String> {
        match value {
            FhirPathValue::String(s) => Ok(format!("string:{}", s.as_ref())),
            FhirPathValue::Integer(i) => Ok(format!("integer:{i}")),
            FhirPathValue::Decimal(d) => Ok(format!("decimal:{d}")),
            FhirPathValue::Boolean(b) => Ok(format!("boolean:{b}")),
            FhirPathValue::Date(d) => Ok(format!("date:{d}")),
            FhirPathValue::DateTime(dt) => Ok(format!("datetime:{dt}")),
            FhirPathValue::Time(t) => Ok(format!("time:{t}")),
            FhirPathValue::JsonValue(json) => Ok(format!("json:{}", **json)),
            FhirPathValue::Collection(_) => {
                // Collections are compared structurally - convert to JSON representation
                Ok(format!(
                    "collection:{}",
                    serde_json::to_string(value).map_err(|_| {
                        FhirPathError::InvalidArguments {
                            message: "Cannot serialize collection for comparison".to_string(),
                        }
                    })?
                ))
            }
            FhirPathValue::Empty => Ok("empty".to_string()),
            FhirPathValue::Quantity(q) => Ok(format!("quantity:{q}")),
            FhirPathValue::Resource(r) => {
                let id = r.as_json().get("id").and_then(|v| v.as_str()).unwrap_or("");
                Ok(format!("resource:{id}"))
            }
            FhirPathValue::TypeInfoObject { name, .. } => Ok(format!("typeinfo:{name}")),
        }
    }
}
