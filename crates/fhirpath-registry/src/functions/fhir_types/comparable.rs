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

//! comparable() function - checks if two quantities have compatible units

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};

/// comparable() function - checks if two quantities have compatible units
pub struct ComparableFunction;

#[async_trait]
impl AsyncFhirPathFunction for ComparableFunction {
    fn name(&self) -> &str {
        "comparable"
    }
    fn human_friendly_name(&self) -> &str {
        "Comparable"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "comparable",
                vec![ParameterInfo::required("other", TypeInfo::Quantity)],
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

        let this_quantity = match &context.input {
            FhirPathValue::Quantity(q) => q,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Quantity".to_string(),
                    actual: context.input.type_name().to_string(),
                });
            }
        };

        let other_quantity = match &args[0] {
            FhirPathValue::Quantity(q) => q,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Quantity".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Check if quantities have compatible dimensions using existing method
        let result = this_quantity.has_compatible_dimensions(other_quantity);
        Ok(FhirPathValue::Boolean(result))
    }
}
