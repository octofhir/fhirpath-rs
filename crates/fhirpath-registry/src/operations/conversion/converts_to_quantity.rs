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

//! Quantity conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ConvertsToQuantity function: returns true if the input can be converted to Quantity
pub struct ConvertsToQuantityFunction;

impl Default for ConvertsToQuantityFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertsToQuantityFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToQuantity", OperationType::Function)
            .description("Returns true if the input can be converted to Quantity")
            .example("'1.5\'cm\''.convertsToQuantity()")
            .example("1.convertsToQuantity()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_quantity(value: &FhirPathValue) -> Result<bool> {
        match value {
            // Already a quantity
            FhirPathValue::Quantity(_) => Ok(true),

            // Numbers can be converted to quantities (dimensionless)
            FhirPathValue::Integer(_) => Ok(true),
            FhirPathValue::Decimal(_) => Ok(true),

            // Booleans can be converted (0 or 1)
            FhirPathValue::Boolean(_) => Ok(true),

            // Strings can potentially be parsed as quantities
            FhirPathValue::String(_) => {
                // Try to parse as a quantity using FhirPathValue's built-in method
                Ok(value.to_quantity_value().is_some())
            }

            // Empty collection returns empty result
            FhirPathValue::Empty => Ok(true),

            // Handle collections
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true) // Empty collection returns empty result
                } else if c.len() == 1 {
                    Self::can_convert_to_quantity(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message: "convertsToQuantity() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            }

            // Other types cannot be converted
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToQuantityFunction {
    fn identifier(&self) -> &str {
        "convertsToQuantity"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConvertsToQuantityFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(_args, context) {
            return result;
        }

        match Self::can_convert_to_quantity(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match Self::can_convert_to_quantity(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
