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

//! Unified repeat() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

/// Unified repeat() function implementation
/// 
/// Repeatedly applies an expression to the input until the result is empty or unchanged.
/// Syntax: repeat(expression)
pub struct UnifiedRepeatFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedRepeatFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("repeat", FunctionCategory::Utilities)
            .display_name("Repeat")
            .description("Repeatedly applies an expression until the result is empty or unchanged")
            .example("Patient.partOf.repeat(partOf)")
            .example("organization.repeat(partOf).name")
            .execution_mode(ExecutionMode::Sync) // Simple case handling
            .input_types(vec![TypePattern::Any])
            .output_type(TypePattern::Any)
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Quadratic) // Can be very expensive
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("repeat(${1:expression})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["recursive", "iterate", "loop", "traverse"])
            .usage_pattern(
                "Recursive traversal",
                "Patient.partOf.repeat(partOf)",
                "Following recursive relationships"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl LambdaFhirPathFunction for UnifiedRepeatFunction {
    fn name(&self) -> &str {
        "repeat"
    }
    
    fn human_friendly_name(&self) -> &str {
        "Repeat"
    }
    
    fn signature(&self) -> &FunctionSignature {
        &self.signature
    }
    
    fn documentation(&self) -> &str {
        "Repeatedly applies an expression until the result is empty or unchanged"
    }
    
    fn is_pure(&self) -> bool {
        true
    }
    
    async fn evaluate_with_expressions(
        &self,
        args: &[ExpressionArgument],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - exactly 1 required (expression)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let expression_arg = &args[0];
        
        // Handle the case where input is empty
        if matches!(context.context.input, FhirPathValue::Empty) {
            return Ok(FhirPathValue::Empty);
        }
        
        // Convert input to collection for iteration
        let input_items = match &context.context.input {
            FhirPathValue::Collection(collection) => {
                collection.clone().into_iter().collect::<Vec<_>>()
            }
            single_value => vec![single_value.clone()],
        };
        
        // Initialize result collection and seen tracker for cycle detection
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        
        // Add initial input items to result
        for item in &input_items {
            let item_key = format!("{:?}", item);
            if seen.insert(item_key) {
                result.push(item.clone());
            }
        }
        
        let mut current_items = input_items;
        const MAX_ITERATIONS: usize = 1000; // Prevent infinite loops
        let mut iterations = 0;
        
        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Maximum iteration limit reached to prevent infinite loops".to_string(),
                });
            }
            
            let mut next_items = Vec::new();
            let mut found_new = false;
            
            // Apply the expression to each current item
            for current_item in &current_items {
                // Create a variable scope for this iteration
                let mut scope = VariableScope::new();
                scope.set("this".to_string(), current_item.clone());
                
                // Create a new evaluation context with the current item as input
                let item_context = EvaluationContext {
                    input: current_item.clone(),
                    root: context.context.root.clone(),
                    variables: context.context.variables.clone(),
                    model_provider: context.context.model_provider.clone(),
                };
                
                // Evaluate the expression based on its type
                let evaluation_result = match expression_arg {
                    ExpressionArgument::Value(value) => {
                        // Pre-evaluated value - return as is
                        Ok(value.clone())
                    }
                    ExpressionArgument::Expression(expr) => {
                        // Evaluate the expression with the current item context
                        let evaluator = context.evaluator;
                        evaluator(expr, &scope, &item_context).await
                    }
                };
                
                match evaluation_result {
                    Ok(expr_result) => {
                        // Convert result to collection if needed
                        let result_items = match expr_result {
                            FhirPathValue::Collection(collection) => {
                                collection.into_iter().collect::<Vec<_>>()
                            }
                            FhirPathValue::Empty => vec![],
                            single_value => vec![single_value],
                        };
                        
                        // Add new items that haven't been seen before
                        for new_item in result_items {
                            let item_key = format!("{:?}", new_item);
                            if seen.insert(item_key) {
                                next_items.push(new_item.clone());
                                result.push(new_item);
                                found_new = true;
                            }
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            
            // If no new items were found, stop iteration
            if !found_new || next_items.is_empty() {
                break;
            }
            
            // Continue with the new items in the next iteration
            current_items = next_items;
        }
        
        // Return the result
        match result.len() {
            0 => Ok(FhirPathValue::Empty),
            1 => Ok(result.into_iter().next().unwrap()),
            _ => Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from_vec(result)
            )),
        }
    }
}