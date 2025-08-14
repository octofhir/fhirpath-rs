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

//! Unified comparable() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;

/// Unified comparable() function implementation
/// 
/// Checks if two quantities have compatible units for comparison operations.
/// Returns true if the quantities can be compared (have compatible dimensions),
/// false otherwise.
pub struct UnifiedComparableFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedComparableFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 required quantity parameter
        let signature = FunctionSignature::new(
            "comparable",
            vec![ParameterInfo::required("otherQuantity", TypeInfo::Quantity)],
            TypeInfo::Boolean,
        );

        let metadata = MetadataBuilder::new("comparable", FunctionCategory::FhirSpecific)
            .display_name("Comparable")
            .description("Checks if two quantities have compatible units for comparison")
            .example("(5 'cm').comparable(3 'mm')")
            .example("(10 'kg').comparable(5 'g')")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("comparable(${1:other_quantity})")
            .keywords(vec!["comparable", "quantity", "units", "dimensions", "ucum"])
            .usage_pattern(
                "Check quantity compatibility",
                "quantity.comparable(otherQuantity)",
                "Unit compatibility and comparison validation"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedComparableFunction {
    fn name(&self) -> &str {
        "comparable"
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
        // Validate single argument
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }

        // Input must be a Quantity
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

        // Argument must be a Quantity
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
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_model::Quantity;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_unified_comparable_function() {
        let comparable_func = UnifiedComparableFunction::new();
        
        // Test with compatible length units
        let quantity1 = FhirPathValue::Quantity(
            Quantity::new(Decimal::from(5), Some("cm".to_string())).into()
        );
        let quantity2 = FhirPathValue::Quantity(
            Quantity::new(Decimal::from(3), Some("mm".to_string())).into()
        );
        
        let context = EvaluationContext::new(quantity1);
        let args = vec![quantity2];
        
        let result = comparable_func.evaluate_sync(&args, &context).unwrap();
        
        // Should return true for compatible length units
        match result {
            FhirPathValue::Boolean(is_comparable) => {
                // The exact result depends on the Quantity implementation
                // Both cm and mm are length units, so should be comparable
                assert!(is_comparable);
            },
            _ => panic!("Expected Boolean result"),
        }
        
        // Test with incompatible units (length vs weight)
        let length_quantity = FhirPathValue::Quantity(
            Quantity::new(Decimal::from(5), Some("cm".to_string())).into()
        );
        let weight_quantity = FhirPathValue::Quantity(
            Quantity::new(Decimal::from(10), Some("kg".to_string())).into()
        );
        
        let context = EvaluationContext::new(length_quantity);
        let args = vec![weight_quantity];
        
        let result = comparable_func.evaluate_sync(&args, &context).unwrap();
        
        // Should return false for incompatible units
        match result {
            FhirPathValue::Boolean(is_comparable) => {
                // Length and weight are not comparable
                assert!(!is_comparable);
            },
            _ => panic!("Expected Boolean result"),
        }
        
        // Test metadata
        assert_eq!(comparable_func.name(), "comparable");
        assert_eq!(comparable_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(comparable_func.metadata().basic.display_name, "Comparable");
        assert!(comparable_func.metadata().basic.is_pure);
    }
    
    #[tokio::test]
    async fn test_comparable_invalid_arguments() {
        let comparable_func = UnifiedComparableFunction::new();
        
        let quantity = FhirPathValue::Quantity(
            Quantity::new(Decimal::from(5), Some("cm".to_string())).into()
        );
        let context = EvaluationContext::new(quantity);
        
        // Test with no arguments
        let result = comparable_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
        
        // Test with too many arguments
        let args = vec![
            FhirPathValue::Quantity(Quantity::new(Decimal::from(1), Some("m".to_string())).into()),
            FhirPathValue::Quantity(Quantity::new(Decimal::from(2), Some("cm".to_string())).into())
        ];
        let result = comparable_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
        
        // Test with invalid argument type
        let args = vec![FhirPathValue::String("not-a-quantity".into())];
        let result = comparable_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_comparable_invalid_input_type() {
        let comparable_func = UnifiedComparableFunction::new();
        
        // Test with non-quantity input
        let context = EvaluationContext::new(FhirPathValue::String("not-a-quantity".into()));
        let args = vec![FhirPathValue::Quantity(
            Quantity::new(Decimal::from(5), Some("cm".to_string())).into()
        )];
        
        let result = comparable_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
        
        // Should error because input is not a Quantity
        if let Err(FunctionError::InvalidArgumentType { expected, .. }) = result {
            assert_eq!(expected, "Quantity");
        } else {
            panic!("Expected InvalidArgumentType error");
        }
    }
    
    #[tokio::test]
    async fn test_comparable_same_units() {
        let comparable_func = UnifiedComparableFunction::new();
        
        // Test with same units
        let quantity1 = FhirPathValue::Quantity(
            Quantity::new(Decimal::from(5), Some("cm".to_string())).into()
        );
        let quantity2 = FhirPathValue::Quantity(
            Quantity::new(Decimal::from(10), Some("cm".to_string())).into()
        );
        
        let context = EvaluationContext::new(quantity1);
        let args = vec![quantity2];
        
        let result = comparable_func.evaluate_sync(&args, &context).unwrap();
        
        // Should return true for same units
        match result {
            FhirPathValue::Boolean(is_comparable) => {
                assert!(is_comparable);
            },
            _ => panic!("Expected Boolean result"),
        }
    }
}