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

//! toInteger() function - converts value to integer

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// toInteger() function - converts value to integer
pub struct ToIntegerFunction;

#[async_trait]
impl AsyncFhirPathFunction for ToIntegerFunction {
    fn name(&self) -> &str {
        "toInteger"
    }
    fn human_friendly_name(&self) -> &str {
        "To Integer"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toInteger", vec![], TypeInfo::Integer)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toInteger() is a pure type conversion function
    }

    fn documentation(&self) -> &str {
        "Returns the value as an Integer if it is a valid representation of an integer. If the input is not convertible to an Integer, the result is empty."
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
            FhirPathValue::Integer(i) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)]))
            }
            FhirPathValue::String(s) => {
                // According to FHIRPath spec, strings with decimal points cannot be converted to integers
                if s.contains('.') {
                    Ok(FhirPathValue::Empty)
                } else {
                    match s.trim().parse::<i64>() {
                        Ok(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i)])),
                        Err(_) => Ok(FhirPathValue::Empty),
                    }
                }
            }
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(
                    if *b { 1 } else { 0 },
                )]))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
