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

//! IsDistinct function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

/// IsDistinct function: returns true if all items in the collection are distinct
pub struct IsDistinctFunction;

impl Default for IsDistinctFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl IsDistinctFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("isDistinct", OperationType::Function)
            .description("Returns true if all items in the collection are distinct (no duplicates). Returns true for empty collections and single-item collections.")
            .example("Patient.name.given.isDistinct()")
            .example("Bundle.entry.resource.id.isDistinct()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for IsDistinctFunction {
    fn identifier(&self) -> &str {
        "isDistinct"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(IsDistinctFunction::create_metadata);
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

        // Fallback to async evaluation (though isDistinct is always sync)
        self.evaluate_is_distinct(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_is_distinct(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IsDistinctFunction {
    fn evaluate_is_distinct(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments {
                message: "isDistinct() takes no arguments".to_string(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(items) => {
                // Empty or single item collections are always distinct
                if items.len() <= 1 {
                    return Ok(FhirPathValue::Boolean(true));
                }

                // Use HashSet to check for duplicates
                let mut seen = HashSet::new();
                for item in items.iter() {
                    // Convert item to a comparable representation
                    let key = self.value_to_comparable_key(item)?;
                    if !seen.insert(key) {
                        // Duplicate found
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                // All items are distinct
                Ok(FhirPathValue::Boolean(true))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => {
                // Single item is always distinct
                Ok(FhirPathValue::Boolean(true))
            }
        }
    }

    /// Convert a FhirPathValue to a comparable key for duplicate detection
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
