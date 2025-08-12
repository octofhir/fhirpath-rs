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

//! toQuantity() function - converts value to quantity

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::*;

/// toQuantity() function - converts value to quantity
pub struct ToQuantityFunction;

#[async_trait]
impl AsyncFhirPathFunction for ToQuantityFunction {
    fn name(&self) -> &str {
        "toQuantity"
    }
    fn human_friendly_name(&self) -> &str {
        "To Quantity"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toQuantity", vec![], TypeInfo::Quantity)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toQuantity() is a pure type conversion function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::Quantity(q) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(
                    q.clone(),
                )]))
            }
            FhirPathValue::Integer(i) => {
                // Convert integer to quantity with unit "1" (dimensionless)
                let quantity = octofhir_fhirpath_model::Quantity::new(
                    rust_decimal::Decimal::from(*i),
                    Some("1".to_string()),
                );
                Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(
                    quantity.into(),
                )]))
            }
            FhirPathValue::Decimal(d) => {
                // Convert decimal to quantity with unit "1" (dimensionless)
                let quantity = octofhir_fhirpath_model::Quantity::new(*d, Some("1".to_string()));
                Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(
                    quantity.into(),
                )]))
            }
            FhirPathValue::String(s) => {
                // Try to parse string as quantity
                if let Ok(parsed) = s.parse::<f64>() {
                    let quantity = octofhir_fhirpath_model::Quantity::new(
                        rust_decimal::Decimal::from_f64(parsed).unwrap_or_default(),
                        Some("1".to_string()),
                    );
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(
                        quantity.into(),
                    )]))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
