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

//! count() function - returns the number of elements in the collection

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};

/// count() function - returns the number of elements in the collection
pub struct CountFunction;

#[async_trait]
impl AsyncFhirPathFunction for CountFunction {
    fn name(&self) -> &str {
        "count"
    }
    fn human_friendly_name(&self) -> &str {
        "Count"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("count", vec![], TypeInfo::Integer));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // count() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns a collection with a single value which is the integer count of the number of items in the input collection. Returns 0 when the input collection is empty."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let count = match &context.input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(
            count as i64,
        )]))
    }
}
