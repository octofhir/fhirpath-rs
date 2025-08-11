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

//! any() function - returns true if criteria is true for any item

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;

/// any() function - returns true if criteria is true for any item
pub struct AnyFunction;

#[async_trait]
impl AsyncFhirPathFunction for AnyFunction {
    fn name(&self) -> &str {
        "any"
    }
    fn human_friendly_name(&self) -> &str {
        "Any"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "any",
                vec![ParameterInfo::optional("criteria", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn documentation(&self) -> &str {
        "Returns `true` if the criteria evaluates to `true` for any element in the input collection, otherwise `false`. If the input collection is empty (`{ }`), the result is `false`."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        if args.is_empty() {
            // No criteria - check if any items exist (non-empty means some exist)
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                !context.input.is_empty(),
            )]))
        } else {
            // TODO: Implement any with criteria - need lambda evaluation
            Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "any() with criteria requires lambda evaluation support".to_string(),
            })
        }
    }
}
