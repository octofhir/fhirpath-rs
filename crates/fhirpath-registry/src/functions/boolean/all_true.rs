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

//! allTrue() function - returns true if all items in collection are true

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};

/// allTrue() function - returns true if all items in collection are true
pub struct AllTrueFunction;

#[async_trait]
impl AsyncFhirPathFunction for AllTrueFunction {
    fn name(&self) -> &str {
        "allTrue"
    }
    fn human_friendly_name(&self) -> &str {
        "All True"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("allTrue", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // allTrue() is a pure boolean function
    }

    fn documentation(&self) -> &str {
        "Takes a collection of Boolean values and returns `true` if all the items are `true`. If any items are `false`, the result is `false`. If the input is empty (`{ }`), the result is `true`."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    true,
                )]));
            } // Empty collection is vacuously true
            single => {
                // Single item - check if it's a boolean true
                match single {
                    FhirPathValue::Boolean(b) => {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*b)]));
                    }
                    _ => {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                            false,
                        )]));
                    }
                }
            }
        };

        // All items must be boolean true
        for item in items.iter() {
            match item {
                FhirPathValue::Boolean(true) => continue,
                _ => {
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        false,
                    )]));
                }
            }
        }
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            true,
        )]))
    }
}
