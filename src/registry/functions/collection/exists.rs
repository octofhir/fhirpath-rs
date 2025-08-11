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

//! exists() function - returns true if the collection has any items

use crate::ast::ExpressionNode;
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult, LambdaEvaluationContext,
    LambdaFunction,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// exists() function - returns true if the collection has any items
pub struct ExistsFunction;

impl FhirPathFunction for ExistsFunction {
    fn name(&self) -> &str {
        "exists"
    }
    fn human_friendly_name(&self) -> &str {
        "Exists"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "exists",
                vec![ParameterInfo::optional("condition", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn documentation(&self) -> &str {
        "Returns `true` if the collection has any elements, and `false` otherwise. This is the opposite of `empty()`, and as such is a shorthand for `empty().not()`. If the input collection is empty (`{ }`), the result is `false`. The function can also take an optional criteria to be applied to the collection prior to the determination of the exists."
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let exists = match &context.input {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => {
                if args.is_empty() {
                    // No condition provided, just check if collection is non-empty
                    !items.is_empty()
                } else {
                    // With condition argument, this should use lambda evaluation
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "exists() with condition should use lambda evaluation".to_string(),
                    });
                }
            }
            _ => args.is_empty(), // Single value exists if no condition, or needs lambda evaluation
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            exists,
        )]))
    }
}

#[async_trait::async_trait(?Send)]
impl LambdaFunction for ExistsFunction {
    async fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        // If no arguments, just check if collection is non-empty
        if args.is_empty() {
            let exists = match &context.context.input {
                FhirPathValue::Empty => false,
                FhirPathValue::Collection(items) => !items.is_empty(),
                _ => true, // Single value exists
            };
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                exists,
            )]));
        }

        // With condition argument, evaluate it for each item
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(1),
                actual: args.len(),
            });
        }

        let condition_expr = &args[0];

        // Get the collection to iterate over
        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    false,
                )]));
            } // Empty collection
            single => vec![single], // Single item treated as collection
        };

        // Check if any item satisfies the condition
        for item in items.iter() {
            let result = (context.evaluator)(condition_expr, item).await?;

            // Check if result is truthy
            let is_truthy = match result {
                FhirPathValue::Boolean(b) => b,
                FhirPathValue::Empty => false,
                FhirPathValue::Collection(ref items) => !items.is_empty(),
                _ => true, // Most non-empty values are truthy
            };

            if is_truthy {
                return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    true,
                )]));
            }
        }

        // No item satisfied the condition
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            false,
        )]))
    }
}
