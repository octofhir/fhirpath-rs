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

//! toString() function - converts value to string

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// toString() function - converts value to string
pub struct ToStringFunction;

#[async_trait]
impl AsyncFhirPathFunction for ToStringFunction {
    fn name(&self) -> &str {
        "toString"
    }
    fn human_friendly_name(&self) -> &str {
        "To String"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toString", vec![], TypeInfo::String)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toString() is a pure type conversion function
    }

    fn documentation(&self) -> &str {
        "Returns the value as a String. Note that this function will only work if the input is convertible to a String, and will return empty if the input cannot be converted to a String."
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
            FhirPathValue::String(s) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                s.clone(),
            )])),
            FhirPathValue::Integer(i) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    i.to_string().into(),
                )]))
            }
            FhirPathValue::Decimal(d) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    d.to_string().into(),
                )]))
            }
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    b.to_string().into(),
                )]))
            }
            FhirPathValue::Date(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                d.to_string().into(),
            )])),
            FhirPathValue::DateTime(dt) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    dt.to_string().into(),
                )]))
            }
            FhirPathValue::Time(t) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                t.to_string().into(),
            )])),
            FhirPathValue::Quantity(q) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    q.to_string().into(),
                )]))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
