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

//! first() function - returns the first item in the collection

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// first() function - returns the first item in the collection
pub struct FirstFunction;

#[async_trait]
impl AsyncFhirPathFunction for FirstFunction {
    fn name(&self) -> &str {
        "first"
    }
    fn human_friendly_name(&self) -> &str {
        "First"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("first", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // first() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns a collection containing only the first item in the input collection. This function is equivalent to `item(0)`. Returns empty (`{ }`) if the input collection has no items."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(items.iter().next().unwrap().clone())
                }
            }
            other => Ok(other.clone()), // Single value is its own first
        }
    }
}
