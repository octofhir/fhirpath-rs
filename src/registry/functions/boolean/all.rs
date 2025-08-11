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

//! all() function - returns true if criteria is true for all items

use crate::ast::ExpressionNode;
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult, LambdaEvaluationContext,
    LambdaFunction,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// all() function - returns true if criteria is true for all items
pub struct AllFunction;

impl FhirPathFunction for AllFunction {
    fn name(&self) -> &str {
        "all"
    }
    fn human_friendly_name(&self) -> &str {
        "All"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "all",
                vec![ParameterInfo::optional("criteria", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn documentation(&self) -> &str {
        "Returns `true` if for every element in the input collection, `criteria` evaluates to `true`. Otherwise, the result is `false`. If the input collection is empty (`{ }`), the result is `true`."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        if args.is_empty() {
            // No criteria - check if all items exist (non-empty means all exist)
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                !context.input.is_empty(),
            )]))
        } else {
            // This should not be called for lambda functions - use evaluate_with_lambda instead
            Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "all() with criteria should use lambda evaluation".to_string(),
            })
        }
    }
}

#[async_trait::async_trait(?Send)]
impl LambdaFunction for AllFunction {
    async fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() {
            // No criteria - check if all items exist (non-empty means all exist)
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                !context.context.input.is_empty(),
            )]));
        }

        let criteria = &args[0];

        // Get the collection to iterate over
        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    true,
                )]));
            } // Empty collection is vacuously true
            single => vec![single], // Single item treated as collection
        };

        // Check if criteria is true for all items
        for item in items {
            let result = (context.evaluator)(criteria, item).await?;

            // Convert result to boolean
            let is_true = match result {
                FhirPathValue::Boolean(b) => b,
                FhirPathValue::Collection(ref coll) if coll.len() == 1 => {
                    match coll.get(0) {
                        Some(FhirPathValue::Boolean(b)) => *b,
                        Some(_) => true, // Non-empty, non-boolean value is truthy
                        None => false,
                    }
                }
                FhirPathValue::Empty => false,
                _ => true, // Non-empty value is truthy
            };

            if !is_true {
                return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    false,
                )]));
            }
        }

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            true,
        )]))
    }
}
