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

//! Unified implies() function implementation

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

/// Unified implies() function implementation
/// 
/// Boolean implication operator. Returns true unless the first operand is true and the second is false.
/// In logic: A implies B is equivalent to (!A or B)
/// Syntax: implies(other)
pub struct UnifiedImpliesFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedImpliesFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("implies", FunctionCategory::BooleanLogic)
            .display_name("Implies")
            .description("Boolean implication operator: A implies B is equivalent to (!A or B)")
            .example("true.implies(false) // false")
            .example("false.implies(anything) // true")
            .example("Patient.active.implies(Patient.name.exists())")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::Boolean])
            .output_type(TypePattern::Boolean)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Constant)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("implies(${1:condition})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["implies", "implication", "logic", "conditional"])
            .usage_pattern(
                "Logical implication",
                "condition.implies(consequence)",
                "Boolean logic and conditional expressions"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedImpliesFunction {
    fn name(&self) -> &str {
        "implies"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - exactly 1 required
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let consequence = &args[0];
        
        // Handle collections by evaluating element-wise
        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    let result = self.evaluate_implies(item, consequence)?;
                    results.push(result);
                }
                Ok(FhirPathValue::collection(results))
            }
            single_item => {
                let result = self.evaluate_implies(single_item, consequence)?;
                Ok(result)
            }
        }
    }
}

impl UnifiedImpliesFunction {
    /// Evaluate implication: A implies B ≡ (!A or B)
    fn evaluate_implies(&self, antecedent: &FhirPathValue, consequence: &FhirPathValue) -> FunctionResult<FhirPathValue> {
        let a_bool = self.to_boolean(antecedent)?;
        let b_bool = self.to_boolean(consequence)?;
        
        // A implies B is equivalent to (!A or B)
        // Truth table:
        // A=false, B=false -> true  (false implies false is true)
        // A=false, B=true  -> true  (false implies true is true)
        // A=true,  B=false -> false (true implies false is false)
        // A=true,  B=true  -> true  (true implies true is true)
        let result = !a_bool || b_bool;
        
        Ok(FhirPathValue::Boolean(result))
    }
    
    /// Convert FhirPathValue to boolean using FHIRPath boolean conversion rules
    fn to_boolean(&self, value: &FhirPathValue) -> FunctionResult<bool> {
        match value {
            FhirPathValue::Boolean(b) => Ok(*b),
            FhirPathValue::Integer(i) => Ok(*i != 0),
            FhirPathValue::Decimal(d) => Ok(!d.is_zero()),
            FhirPathValue::String(s) => Ok(!s.is_empty()),
            FhirPathValue::Collection(items) => Ok(!items.is_empty()),
            FhirPathValue::Empty => Ok(false),
            _ => {
                // For other types, consider them truthy if they exist
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_implies_true_true() {
        let func = UnifiedImpliesFunction::new();
        let context = create_test_context(FhirPathValue::Boolean(true));
        
        let args = vec![FhirPathValue::Boolean(true)];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true)); // true implies true = true
    }
    
    #[tokio::test]
    async fn test_implies_true_false() {
        let func = UnifiedImpliesFunction::new();
        let context = create_test_context(FhirPathValue::Boolean(true));
        
        let args = vec![FhirPathValue::Boolean(false)];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false)); // true implies false = false
    }
    
    #[tokio::test]
    async fn test_implies_false_true() {
        let func = UnifiedImpliesFunction::new();
        let context = create_test_context(FhirPathValue::Boolean(false));
        
        let args = vec![FhirPathValue::Boolean(true)];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true)); // false implies true = true
    }
    
    #[tokio::test]
    async fn test_implies_false_false() {
        let func = UnifiedImpliesFunction::new();
        let context = create_test_context(FhirPathValue::Boolean(false));
        
        let args = vec![FhirPathValue::Boolean(false)];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true)); // false implies false = true
    }
    
    #[tokio::test]
    async fn test_implies_with_integers() {
        let func = UnifiedImpliesFunction::new();
        let context = create_test_context(FhirPathValue::Integer(1)); // truthy
        
        let args = vec![FhirPathValue::Integer(0)]; // falsy
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false)); // true implies false = false
    }
    
    #[tokio::test]
    async fn test_implies_with_strings() {
        let func = UnifiedImpliesFunction::new();
        let context = create_test_context(FhirPathValue::String("hello".into())); // truthy
        
        let args = vec![FhirPathValue::String("".into())]; // falsy (empty string)
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false)); // true implies false = false
    }
    
    #[tokio::test]
    async fn test_implies_collection() {
        let func = UnifiedImpliesFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(false),
        ]);
        let context = create_test_context(collection);
        
        let args = vec![FhirPathValue::Boolean(true)];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Boolean(true)));  // true implies true = true
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Boolean(true)));  // false implies true = true
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedImpliesFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "implies");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::BooleanLogic);
    }
}