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

//! empty() function - returns true if the collection is empty

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};

/// empty() function - returns true if the collection is empty
pub struct EmptyFunction;

#[async_trait]
impl AsyncFhirPathFunction for EmptyFunction {
    fn name(&self) -> &str {
        "empty"
    }
    fn human_friendly_name(&self) -> &str {
        "Empty"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("empty", vec![], TypeInfo::Boolean));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // empty() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns `true` if the input collection is empty (`{ }`) and `false` otherwise."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let is_empty = match &context.input {
            FhirPathValue::Empty => true,
            FhirPathValue::Collection(items) => items.is_empty(),
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            is_empty,
        )]))
    }
}
