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

//! hasValue() function - checks if a value exists (not empty)

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// hasValue() function - returns true if the input is not empty
pub struct HasValueFunction;

#[async_trait]
impl AsyncFhirPathFunction for HasValueFunction {
    fn name(&self) -> &str {
        "hasValue"
    }

    fn human_friendly_name(&self) -> &str {
        "Has Value"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "hasValue",
                vec![], // No parameters
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // hasValue() returns true if the input is not empty
        let has_value = match &context.input {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(coll) => !coll.is_empty(),
            _ => true, // Any other value means it exists
        };

        Ok(FhirPathValue::Boolean(has_value))
    }
}
