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

//! Simple test to demonstrate lambda function concept

#[cfg(test)]
mod tests {
    use crate::unified_implementations::aggregates::EnhancedAggregateFunction;
    use crate::expression_argument::{ExpressionArgument, VariableScope};
    use crate::lambda_function::{LambdaEvaluationContext, LambdaFhirPathFunction};
    use crate::function::{EvaluationContext, FunctionError, FunctionResult};
    use octofhir_fhirpath_ast::{BinaryOpData, BinaryOperator, ExpressionNode};
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_enhanced_aggregate_function_creation() {
        let func = EnhancedAggregateFunction::new();
        
        // Verify the function was created correctly
        assert_eq!(func.name(), "aggregate");
        assert!(func.supports_lambda_expressions());
        assert_eq!(func.lambda_argument_indices(), vec![0]);
        
        // Verify signature
        let signature = func.signature();
        assert_eq!(signature.min_arity, 1);
        assert_eq!(signature.max_arity, Some(2));
        assert_eq!(signature.name, "aggregate");
    }

    #[tokio::test]
    async fn test_variable_scope() {
        let mut scope = VariableScope::new();
        
        // Test setting and getting variables
        scope = scope
            .with_this(FhirPathValue::Integer(42))
            .with_total(FhirPathValue::Integer(100))
            .with_index(5);
        
        // Test variable access
        assert_eq!(scope.get_owned("this"), Some(FhirPathValue::Integer(42)));
        assert_eq!(scope.get_owned("total"), Some(FhirPathValue::Integer(100)));
        assert_eq!(scope.get_owned("index"), Some(FhirPathValue::Integer(5)));
        assert_eq!(scope.get_owned("nonexistent"), None);
        
        // Test variables map conversion
        let vars_map = scope.to_variables_map();
        assert!(vars_map.contains_key("this"));
        assert!(vars_map.contains_key("total"));
        assert!(vars_map.contains_key("index"));
        assert_eq!(vars_map.len(), 3);
    }

    #[tokio::test]
    async fn test_expression_argument_types() {
        // Test ExpressionArgument creation
        let value_arg = ExpressionArgument::value(FhirPathValue::Integer(42));
        let expr_arg = ExpressionArgument::expression(ExpressionNode::Variable("test".to_string()));
        
        // Test type checking
        assert!(value_arg.is_value());
        assert!(!value_arg.is_expression());
        
        assert!(expr_arg.is_expression());
        assert!(!expr_arg.is_value());
        
        // Test value extraction
        assert_eq!(value_arg.as_value(), Some(&FhirPathValue::Integer(42)));
        assert!(expr_arg.as_value().is_none());
    }

    #[tokio::test]
    async fn test_lambda_function_basic_properties() {
        let func = EnhancedAggregateFunction::new();
        
        // Test lambda function properties
        assert_eq!(func.name(), "aggregate");
        assert_eq!(func.human_friendly_name(), "Aggregate Function");
        assert!(func.supports_lambda_expressions());
        assert!(!func.is_pure()); // aggregate might have side effects from expressions
        
        // Test lambda argument indices
        let lambda_indices = func.lambda_argument_indices();
        assert_eq!(lambda_indices, vec![0]); // First argument is lambda expression
    }

    #[test]
    fn test_aggregate_function_signature_details() {
        let func = EnhancedAggregateFunction::new();
        let signature = func.signature();
        
        // Test signature details
        assert_eq!(signature.name, "aggregate");
        assert_eq!(signature.min_arity, 1);
        assert_eq!(signature.max_arity, Some(2));
        assert_eq!(signature.parameters.len(), 2);
        
        // Test parameters
        assert_eq!(signature.parameters[0].name, "aggregator");
        assert!(signature.parameters[0].required);
        
        assert_eq!(signature.parameters[1].name, "init");
        assert!(!signature.parameters[1].required);
    }

    #[test]
    fn test_function_documentation() {
        let func = EnhancedAggregateFunction::new();
        let doc = func.documentation();
        
        // Verify documentation exists and contains key information
        assert!(!doc.is_empty());
        assert!(doc.contains("$this"));
        assert!(doc.contains("$total"));
        assert!(doc.contains("$index"));
        assert!(doc.contains("aggregate"));
    }
}