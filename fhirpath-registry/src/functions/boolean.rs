//! Boolean logic functions

use crate::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult, LambdaEvaluationContext,
    LambdaFunction,
};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_ast::ExpressionNode;
use fhirpath_model::{FhirPathValue, TypeInfo};
use std::collections::HashMap;

/// not() function - logical negation
pub struct NotFunction;

impl FhirPathFunction for NotFunction {
    fn name(&self) -> &str {
        "not"
    }
    fn human_friendly_name(&self) -> &str {
        "Not"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("not", vec![], TypeInfo::Boolean));
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)]))
            }
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection is false, not becomes true
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        true,
                    )]))
                } else if items.len() == 1 {
                    match items.iter().next() {
                        Some(FhirPathValue::Boolean(b)) => {
                            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)]))
                        }
                        _ => Ok(FhirPathValue::Empty),
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                true,
            )])),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

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

impl LambdaFunction for AllFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext,
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
            let result = (context.evaluator)(criteria, item)?;

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

/// any() function - returns true if criteria is true for any item
pub struct AnyFunction;

impl FhirPathFunction for AnyFunction {
    fn name(&self) -> &str {
        "any"
    }
    fn human_friendly_name(&self) -> &str {
        "Any"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "any",
                vec![ParameterInfo::optional("criteria", TypeInfo::Any)],
                TypeInfo::Boolean,
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
        if args.is_empty() {
            // No criteria - check if any items exist (non-empty means some exist)
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                !context.input.is_empty(),
            )]))
        } else {
            // TODO: Implement any with criteria - need lambda evaluation
            Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "any() with criteria requires lambda evaluation support".to_string(),
            })
        }
    }
}

/// allTrue() function - returns true if all items in collection are true
pub struct AllTrueFunction;

impl FhirPathFunction for AllTrueFunction {
    fn name(&self) -> &str {
        "allTrue"
    }
    fn human_friendly_name(&self) -> &str {
        "All True"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("allTrue", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    true,
                )]));
            } // Empty collection is vacuously true
            single => {
                // Single item - check if it's a boolean true
                match single {
                    FhirPathValue::Boolean(b) => {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*b)]));
                    }
                    _ => {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                            false,
                        )]));
                    }
                }
            }
        };

        // All items must be boolean true
        for item in items.iter() {
            match item {
                FhirPathValue::Boolean(true) => continue,
                _ => {
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        false,
                    )]));
                }
            }
        }
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            true,
        )]))
    }
}

/// isDistinct() function - returns true if the collection contains no duplicates
pub struct IsDistinctFunction;

/// Discriminant key for fast duplicate detection
#[derive(Hash, Eq, PartialEq)]
enum ValueDiscriminant {
    Empty,
    Boolean(bool),
    Integer(i64),
    Decimal(String), // Use string representation for decimal
    String(String),
    Date(String),
    DateTime(String),
    Time(String),
    Quantity(String, Option<String>), // value and unit
    Code(String, Option<String>),     // code and system
    Collection(usize),                // Collection with size
    Resource(String),                 // Resource type name
    Lambda,
}

impl IsDistinctFunction {
    /// Create a discriminant for fast equality checking
    fn create_discriminant(value: &FhirPathValue) -> ValueDiscriminant {
        match value {
            FhirPathValue::Empty => ValueDiscriminant::Empty,
            FhirPathValue::Boolean(b) => ValueDiscriminant::Boolean(*b),
            FhirPathValue::Integer(i) => ValueDiscriminant::Integer(*i),
            FhirPathValue::Decimal(d) => ValueDiscriminant::Decimal(d.to_string()),
            FhirPathValue::String(s) => ValueDiscriminant::String(s.clone()),
            FhirPathValue::Date(d) => ValueDiscriminant::Date(d.to_string()),
            FhirPathValue::DateTime(dt) => ValueDiscriminant::DateTime(dt.to_string()),
            FhirPathValue::Time(t) => ValueDiscriminant::Time(t.to_string()),
            FhirPathValue::Quantity(q) => {
                ValueDiscriminant::Quantity(q.value.to_string(), q.unit.clone())
            }
            // FhirPathValue::Code variant doesn't exist yet
            // FhirPathValue::Code(c) => ValueDiscriminant::Code(
            //     c.code.clone(),
            //     c.system.clone(),
            // ),
            FhirPathValue::Collection(items) => ValueDiscriminant::Collection(items.len()),
            FhirPathValue::Resource(r) => {
                ValueDiscriminant::Resource(r.resource_type().unwrap_or("Unknown").to_string())
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                ValueDiscriminant::String(format!("TypeInfo({}.{})", namespace, name))
            }
            // FhirPathValue::Lambda variant doesn't exist yet
            // FhirPathValue::Lambda(_) => ValueDiscriminant::Lambda,
        }
    }

    /// Optimized duplicate detection using hash-like approach
    /// Time complexity: O(n) average case, O(n²) worst case
    /// Space complexity: O(n)
    #[inline]
    fn has_no_duplicates<'a, I>(items: I) -> bool
    where
        I: Iterator<Item = &'a FhirPathValue>,
    {
        // Use a HashMap with custom discriminant keys for fast duplicate detection
        let mut seen: HashMap<ValueDiscriminant, Vec<&'a FhirPathValue>> = HashMap::new();

        for item in items {
            let discriminant = Self::create_discriminant(item);

            // Check if we've seen this discriminant before
            if let Some(existing_items) = seen.get_mut(&discriminant) {
                // We have a potential match - need to check for actual equality
                // within items that have the same discriminant
                for existing_item in existing_items.iter() {
                    if item == *existing_item {
                        return false; // Duplicate found
                    }
                }
                existing_items.push(item);
            } else {
                seen.insert(discriminant, vec![item]);
            }
        }

        true // No duplicates found
    }
}

impl FhirPathFunction for IsDistinctFunction {
    fn name(&self) -> &str {
        "isDistinct"
    }
    fn human_friendly_name(&self) -> &str {
        "Is Distinct"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("isDistinct", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let is_distinct = match &context.input {
            FhirPathValue::Empty => true, // Empty collection has no duplicates
            FhirPathValue::Collection(items) => Self::has_no_duplicates(items.iter()),
            _ => true, // Single value is always distinct
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            is_distinct,
        )]))
    }
}
