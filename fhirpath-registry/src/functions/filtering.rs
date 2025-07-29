//! Filtering and selection functions

use crate::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult, LambdaEvaluationContext,
    LambdaFunction,
};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_ast::ExpressionNode;
use fhirpath_model::{FhirPathValue, TypeInfo};
use std::hash::BuildHasherDefault;

type VarMap =
    std::collections::HashMap<String, FhirPathValue, BuildHasherDefault<rustc_hash::FxHasher>>;

/// where() function - filters collection based on criteria
pub struct WhereFunction;

impl FhirPathFunction for WhereFunction {
    fn name(&self) -> &str {
        "where"
    }
    fn human_friendly_name(&self) -> &str {
        "Where"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "where",
                vec![ParameterInfo::required("criteria", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // This should not be called for lambda functions - use evaluate_with_lambda instead
        Err(FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "where() should use lambda evaluation".to_string(),
        })
    }
}

impl LambdaFunction for WhereFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: 0,
            });
        }

        let criteria = &args[0];

        // Get the collection to iterate over
        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])), // Empty collection returns empty
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // Apply criteria to each item with index support
        for (index, item) in items.iter().enumerate() {
            let result = if let Some(enhanced_evaluator) = context.enhanced_evaluator {
                // Use enhanced evaluator with $index variable injection
                let mut additional_vars: VarMap =
                    std::collections::HashMap::with_hasher(BuildHasherDefault::<
                        rustc_hash::FxHasher,
                    >::default());
                additional_vars.insert("$index".to_string(), FhirPathValue::Integer(index as i64));

                enhanced_evaluator(criteria, item, &additional_vars)?
            } else {
                // Fall back to regular evaluator
                (context.evaluator)(criteria, item)?
            };

            // Check if criteria evaluates to true
            let is_true = match result {
                FhirPathValue::Boolean(true) => true,
                FhirPathValue::Collection(coll) => {
                    // Collection is true if it contains at least one true value
                    coll.iter()
                        .any(|v| matches!(v, FhirPathValue::Boolean(true)))
                }
                FhirPathValue::Empty => false, // Empty is considered false
                _ => false,                    // All other values are considered false
            };

            if is_true {
                results.push((*item).clone());
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

/// select() function - transforms collection using expression
pub struct SelectFunction;

impl FhirPathFunction for SelectFunction {
    fn name(&self) -> &str {
        "select"
    }
    fn human_friendly_name(&self) -> &str {
        "Select"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "select",
                vec![ParameterInfo::required("expression", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // This should not be called for lambda functions - use evaluate_with_lambda instead
        Err(FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "select() should use lambda evaluation".to_string(),
        })
    }
}

impl LambdaFunction for SelectFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: 0,
            });
        }

        let expression = &args[0];

        // Get the collection to iterate over
        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])), // Empty collection returns empty
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // Apply expression to each item with index support
        for (index, item) in items.iter().enumerate() {
            let result = if let Some(enhanced_evaluator) = context.enhanced_evaluator {
                // Use enhanced evaluator with $index variable injection
                let mut additional_vars: VarMap =
                    std::collections::HashMap::with_hasher(BuildHasherDefault::<
                        rustc_hash::FxHasher,
                    >::default());
                additional_vars.insert("$index".to_string(), FhirPathValue::Integer(index as i64));

                enhanced_evaluator(expression, item, &additional_vars)?
            } else {
                // Fall back to regular evaluator
                (context.evaluator)(expression, item)?
            };

            // Add result to collection, flattening collections
            match result {
                FhirPathValue::Collection(coll) => {
                    for item in coll {
                        results.push(item);
                    }
                }
                FhirPathValue::Empty => {
                    // Skip empty results
                }
                other => results.push(other),
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

/// take() function - takes first n elements
pub struct TakeFunction;

impl FhirPathFunction for TakeFunction {
    fn name(&self) -> &str {
        "take"
    }
    fn human_friendly_name(&self) -> &str {
        "Take"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "take",
                vec![ParameterInfo::required("num", TypeInfo::Integer)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let num = match &args[0] {
            FhirPathValue::Integer(n) => *n as usize,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        let items = context.input.clone().to_collection();
        let result: Vec<FhirPathValue> = items.into_iter().take(num).collect();
        Ok(FhirPathValue::collection(result))
    }
}

/// skip() function - skips first n elements
pub struct SkipFunction;

impl FhirPathFunction for SkipFunction {
    fn name(&self) -> &str {
        "skip"
    }
    fn human_friendly_name(&self) -> &str {
        "Skip"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "skip",
                vec![ParameterInfo::required("num", TypeInfo::Integer)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let num = match &args[0] {
            FhirPathValue::Integer(n) => *n as usize,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        let items = context.input.clone().to_collection();
        let result: Vec<FhirPathValue> = items.into_iter().skip(num).collect();
        Ok(FhirPathValue::collection(result))
    }
}

/// ofType() function - filters collection to items of specified type
pub struct OfTypeFunction;

impl FhirPathFunction for OfTypeFunction {
    fn name(&self) -> &str {
        "ofType"
    }
    fn human_friendly_name(&self) -> &str {
        "OfType"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "ofType",
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        let type_name = match &args[0] {
            FhirPathValue::String(t) => t,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Get the collection to filter
        let items = match &context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])),
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // Filter items by type
        for item in items {
            if self.matches_type(item, type_name) {
                results.push((*item).clone());
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

impl OfTypeFunction {
    /// Check if a value matches the specified type name
    fn matches_type(&self, value: &FhirPathValue, type_name: &str) -> bool {
        match value {
            FhirPathValue::Boolean(_) => {
                matches!(
                    type_name,
                    "Boolean" | "System.Boolean" | "boolean" | "FHIR.boolean"
                )
            }
            FhirPathValue::Integer(_) => {
                matches!(
                    type_name,
                    "Integer" | "System.Integer" | "integer" | "FHIR.integer"
                )
            }
            FhirPathValue::Decimal(_) => {
                matches!(
                    type_name,
                    "Decimal" | "System.Decimal" | "decimal" | "FHIR.decimal"
                )
            }
            FhirPathValue::String(_) => {
                matches!(
                    type_name,
                    "String"
                        | "System.String"
                        | "string"
                        | "FHIR.string"
                        | "uri"
                        | "FHIR.uri"
                        | "uuid"
                        | "FHIR.uuid"
                        | "code"
                        | "FHIR.code"
                        | "id" 
                        | "FHIR.id"
                )
            }
            FhirPathValue::Date(_) => {
                matches!(type_name, "Date" | "System.Date" | "date" | "FHIR.date")
            }
            FhirPathValue::DateTime(_) => {
                matches!(
                    type_name,
                    "DateTime" | "System.DateTime" | "dateTime" | "FHIR.dateTime"
                )
            }
            FhirPathValue::Time(_) => {
                matches!(type_name, "Time" | "System.Time" | "time" | "FHIR.time")
            }
            FhirPathValue::Quantity { .. } => {
                matches!(type_name, "Quantity" | "System.Quantity" | "FHIR.Quantity")
            }
            FhirPathValue::Resource(resource) => {
                // Check FHIR resource type - support both with and without FHIR prefix
                if let Some(resource_type) = resource.resource_type() {
                    resource_type == type_name
                        || type_name == format!("FHIR.{}", resource_type)
                        || type_name == format!("FHIR.`{}`", resource_type)
                        // Handle case-insensitive matching for common FHIR resources
                        || resource_type.to_lowercase() == type_name.to_lowercase()
                } else {
                    false
                }
            }
            FhirPathValue::Collection(_) => {
                matches!(type_name, "Collection")
            }
            FhirPathValue::TypeInfoObject { .. } => {
                matches!(type_name, "TypeInfo" | "System.TypeInfo")
            }
            FhirPathValue::Empty => false,
        }
    }
}
