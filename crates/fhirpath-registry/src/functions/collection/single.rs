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

//! single() function - returns the single item if collection has exactly one item

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// single() function - returns the single item if collection has exactly one item
pub struct SingleFunction;

#[async_trait]
impl AsyncFhirPathFunction for SingleFunction {
    fn name(&self) -> &str {
        "single"
    }
    fn human_friendly_name(&self) -> &str {
        "Single"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("single", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // single() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns the single item in the input collection. If the input collection does not contain exactly one item, an empty collection is returned."
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
                if items.len() == 1 {
                    Ok(items.iter().next().unwrap().clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            other => Ok(other.clone()), // Single value returns itself
        }
    }
}
