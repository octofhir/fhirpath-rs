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

//! min() function - finds minimum value in a collection

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::*;

/// min() function - finds minimum value in a collection
pub struct MinFunction;

#[async_trait]
impl AsyncFhirPathFunction for MinFunction {
    fn name(&self) -> &str {
        "min"
    }
    fn human_friendly_name(&self) -> &str {
        "Minimum"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("min", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // min() is a pure mathematical function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let items = match &context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single => vec![single],
        };

        if items.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let mut min_value: Option<FhirPathValue> = None;

        for item in items {
            match item {
                FhirPathValue::Empty => continue, // Skip empty values
                _ => {
                    if let Some(ref current_min) = min_value {
                        // Compare values
                        if let Ok(ordering) = self.compare_values(item, current_min) {
                            if ordering == std::cmp::Ordering::Less {
                                min_value = Some(item.clone());
                            }
                        }
                    } else {
                        min_value = Some(item.clone());
                    }
                }
            }
        }

        match min_value {
            Some(value) => Ok(value),
            None => Ok(FhirPathValue::Empty),
        }
    }
}

impl MinFunction {
    fn compare_values(
        &self,
        a: &FhirPathValue,
        b: &FhirPathValue,
    ) -> Result<std::cmp::Ordering, FunctionError> {
        match (a, b) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Ok(Decimal::from(*a).cmp(b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => Ok(a.cmp(&Decimal::from(*b))),
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Ok(a.cmp(b)),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Ok(a.cmp(b)),
            _ => Err(FunctionError::InvalidArgumentType {
                name: "min".to_string(),
                index: 0,
                expected: "Comparable types".to_string(),
                actual: format!("Cannot compare {a:?} and {b:?}"),
            }),
        }
    }
}
