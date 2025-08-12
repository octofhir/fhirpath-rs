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

//! avg() function - averages numeric values in a collection

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::*;

/// avg() function - averages numeric values in a collection
pub struct AvgFunction;

#[async_trait]
impl AsyncFhirPathFunction for AvgFunction {
    fn name(&self) -> &str {
        "avg"
    }
    fn human_friendly_name(&self) -> &str {
        "Average"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("avg", vec![], TypeInfo::Decimal));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // avg() is a pure mathematical function
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

        let mut sum = Decimal::ZERO;
        let mut count = 0;

        for item in items {
            match item {
                FhirPathValue::Integer(i) => {
                    sum += Decimal::from(*i);
                    count += 1;
                }
                FhirPathValue::Decimal(d) => {
                    sum += d;
                    count += 1;
                }
                FhirPathValue::Empty => {
                    // Skip empty values
                }
                _ => {
                    return Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: 0,
                        expected: "Number".to_string(),
                        actual: format!("{item:?}"),
                    });
                }
            }
        }

        if count == 0 {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::Decimal(sum / Decimal::from(count)))
        }
    }
}
