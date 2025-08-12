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

//! combine() function - concatenates two collections

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// combine() function - concatenates two collections
pub struct CombineFunction;

#[async_trait]
impl AsyncFhirPathFunction for CombineFunction {
    fn name(&self) -> &str {
        "combine"
    }
    fn human_friendly_name(&self) -> &str {
        "Combine"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "combine",
                vec![ParameterInfo::required("other", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // combine() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns a collection that contains all items in the input collection, followed by all items in the other collection. Duplicates are not removed."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let other = &args[0];
        let mut result = context
            .input
            .clone()
            .to_collection()
            .into_iter()
            .collect::<Vec<_>>();
        result.extend(other.clone().to_collection());
        Ok(FhirPathValue::collection(result))
    }
}
