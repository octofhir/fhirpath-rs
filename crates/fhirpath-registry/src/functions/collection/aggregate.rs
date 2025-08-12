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

//! aggregate() function implementation

use crate::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult, LambdaEvaluationContext,
    LambdaFunction,
};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_ast::ExpressionNode;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
use std::hash::BuildHasherDefault;

type VarMap =
    std::collections::HashMap<String, FhirPathValue, BuildHasherDefault<rustc_hash::FxHasher>>;

/// aggregate() function - aggregates values using a lambda expression
pub struct AggregateFunction;

impl FhirPathFunction for AggregateFunction {
    fn name(&self) -> &str {
        "aggregate"
    }
    fn human_friendly_name(&self) -> &str {
        "Aggregate"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "aggregate",
                vec![
                    ParameterInfo::required("aggregator", TypeInfo::Any),
                    ParameterInfo::optional("init", TypeInfo::Any),
                ],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // This should not be called for lambda functions - use evaluate_with_lambda instead
        Err(FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "aggregate() should use lambda evaluation".to_string(),
        })
    }
}

#[async_trait::async_trait(?Send)]
impl LambdaFunction for AggregateFunction {
    async fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() || args.len() > 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(2),
                actual: args.len(),
            });
        }

        let aggregator_expr = &args[0];

        // Get the collection to aggregate
        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single => vec![single],
        };

        // Get initial value (second argument or empty)
        let mut total = if args.len() > 1 {
            // Evaluate the initial value expression
            let init_result = (context.evaluator)(&args[1], &context.context.input).await?;

            // Unwrap single-item collections (FHIRPath semantics)
            match init_result {
                FhirPathValue::Collection(ref items) if items.len() == 1 => {
                    items.first().unwrap().clone()
                }
                other => other,
            }
        } else {
            FhirPathValue::Empty
        };

        // Aggregate over each item
        for item in items.iter() {
            // Create enhanced evaluator with $this and $total variables
            let result = if let Some(enhanced_evaluator) = context.enhanced_evaluator {
                let mut additional_vars: VarMap =
                    std::collections::HashMap::with_hasher(BuildHasherDefault::<
                        rustc_hash::FxHasher,
                    >::default());

                // Include all variables from outer context
                for (name, value) in &context.context.variables {
                    additional_vars.insert(name.clone(), value.clone());
                }

                // Set $this to current item and $total to accumulated value (parser strips $ prefix)
                additional_vars.insert("this".to_string(), (*item).clone());
                additional_vars.insert("total".to_string(), total.clone());

                enhanced_evaluator(aggregator_expr, item, &additional_vars).await?
            } else {
                // Fall back to regular evaluator (won't have $total support)
                (context.evaluator)(aggregator_expr, item).await?
            };

            // Update total with the result
            total = result;
        }

        Ok(total)
    }
}
