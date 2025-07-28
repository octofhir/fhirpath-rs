//! Collection operation functions

use crate::function::{FhirPathFunction, FunctionError, FunctionResult, EvaluationContext};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// count() function - returns the number of items in a collection
pub struct CountFunction;

impl FhirPathFunction for CountFunction {
    fn name(&self) -> &str { "count" }
    fn human_friendly_name(&self) -> &str { "Count" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "count",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let count = match &context.input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(count as i64)]))
    }
}

/// empty() function - returns true if the collection is empty
pub struct EmptyFunction;

impl FhirPathFunction for EmptyFunction {
    fn name(&self) -> &str { "empty" }
    fn human_friendly_name(&self) -> &str { "Empty" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "empty",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let is_empty = match &context.input {
            FhirPathValue::Empty => true,
            FhirPathValue::Collection(items) => items.is_empty(),
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(is_empty)]))
    }
}

/// exists() function - returns true if the collection has any items
pub struct ExistsFunction;

impl FhirPathFunction for ExistsFunction {
    fn name(&self) -> &str { "exists" }
    fn human_friendly_name(&self) -> &str { "Exists" }
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
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let exists = match &context.input {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => {
                if args.is_empty() {
                    // No condition provided, just check if collection is non-empty
                    !items.is_empty()
                } else {
                    // TODO: With condition argument, this needs lambda evaluation support
                    // For now, return error to indicate this functionality is not yet implemented
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "exists() with condition parameter requires lambda evaluation support (not yet implemented)".to_string(),
                    });
                }
            }
            _ => args.is_empty(), // Single value exists if no condition, or needs lambda evaluation
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(exists)]))
    }
}

/// first() function - returns the first item in the collection
pub struct FirstFunction;

impl FhirPathFunction for FirstFunction {
    fn name(&self) -> &str { "first" }
    fn human_friendly_name(&self) -> &str { "First" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "first",
                vec![],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(items.iter().next().unwrap().clone())
                }
            }
            other => Ok(other.clone()), // Single value is its own first
        }
    }
}

/// last() function - returns the last item in the collection
pub struct LastFunction;

impl FhirPathFunction for LastFunction {
    fn name(&self) -> &str { "last" }
    fn human_friendly_name(&self) -> &str { "Last" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "last",
                vec![],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(items.iter().last().unwrap().clone())
                }
            }
            other => Ok(other.clone()), // Single value is its own last
        }
    }
}

/// tail() function - returns all items except the first
pub struct TailFunction;

impl FhirPathFunction for TailFunction {
    fn name(&self) -> &str { "tail" }
    fn human_friendly_name(&self) -> &str { "Tail" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "tail",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() <= 1 {
                    Ok(FhirPathValue::Empty)
                } else {
                    let tail_items: Vec<FhirPathValue> = items.iter().skip(1).cloned().collect();
                    Ok(FhirPathValue::collection(tail_items))
                }
            }
            _ => Ok(FhirPathValue::Empty), // Single value's tail is empty
        }
    }
}

/// distinct() function - returns unique items in the collection
pub struct DistinctFunction;

impl FhirPathFunction for DistinctFunction {
    fn name(&self) -> &str { "distinct" }
    fn human_friendly_name(&self) -> &str { "Distinct" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "distinct",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = context.input.clone().to_collection();
        let mut unique = Vec::new();
        for item in items.into_iter() {
            if !unique.iter().any(|u| u == &item) {
                unique.push(item);
            }
        }
        Ok(FhirPathValue::collection(unique))
    }
}

/// single() function - returns the single item if collection has exactly one item
pub struct SingleFunction;

impl FhirPathFunction for SingleFunction {
    fn name(&self) -> &str { "single" }
    fn human_friendly_name(&self) -> &str { "Single" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "single",
                vec![],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    Ok(items.iter().next().unwrap().clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            other => Ok(other.clone()), // Single value returns itself
        }
    }
}

/// intersect() function - returns the intersection of two collections
pub struct IntersectFunction;

impl FhirPathFunction for IntersectFunction {
    fn name(&self) -> &str { "intersect" }
    fn human_friendly_name(&self) -> &str { "Intersect" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "intersect",
                vec![ParameterInfo::required("other", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let other = &args[0];
        let left = context.input.clone().to_collection();
        let right = other.clone().to_collection();

        let mut result = Vec::new();
        for item in left.into_iter() {
            if right.iter().any(|r| r == &item) && !result.iter().any(|res| res == &item) {
                result.push(item);
            }
        }
        Ok(FhirPathValue::collection(result))
    }
}

/// exclude() function - returns items in first collection but not in second
pub struct ExcludeFunction;

impl FhirPathFunction for ExcludeFunction {
    fn name(&self) -> &str { "exclude" }
    fn human_friendly_name(&self) -> &str { "Exclude" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "exclude",
                vec![ParameterInfo::required("other", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let other = &args[0];
        let left = context.input.clone().to_collection();
        let right = other.clone().to_collection();

        let mut result = Vec::new();
        for item in left.into_iter() {
            if !right.iter().any(|r| r == &item) {
                result.push(item);
            }
        }
        Ok(FhirPathValue::collection(result))
    }
}

/// combine() function - concatenates two collections
pub struct CombineFunction;

impl FhirPathFunction for CombineFunction {
    fn name(&self) -> &str { "combine" }
    fn human_friendly_name(&self) -> &str { "Combine" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "combine",
                vec![ParameterInfo::required("other", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let other = &args[0];
        let mut result = context.input.clone().to_collection().into_iter().collect::<Vec<_>>();
        result.extend(other.clone().to_collection().into_iter());
        Ok(FhirPathValue::collection(result))
    }
}

/// descendants() function - returns all descendants of nodes in the collection
pub struct DescendantsFunction;

impl FhirPathFunction for DescendantsFunction {
    fn name(&self) -> &str { "descendants" }
    fn human_friendly_name(&self) -> &str { "Descendants" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "descendants",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = context.input.clone().to_collection();
        let mut result = Vec::new();

        fn collect_descendants(value: &FhirPathValue, result: &mut Vec<FhirPathValue>) {
            match value {
                FhirPathValue::Resource(resource) => {
                    // Collect all nested values from the resource
                    for (_key, field_value) in resource.properties() {
                        // Convert JSON Value to FhirPathValue - for now, skip this complex conversion
                        // TODO: Implement proper JSON Value to FhirPathValue conversion
                    }
                }
                FhirPathValue::Collection(items) => {
                    for item in items.iter() {
                        result.push(item.clone());
                        collect_descendants(item, result);
                    }
                }
                _ => {} // Primitives have no descendants
            }
        }

        for item in items.iter() {
            collect_descendants(item, &mut result);
        }

        Ok(FhirPathValue::collection(result))
    }
}

/// aggregate() function - aggregates values using a lambda expression
pub struct AggregateFunction;

impl FhirPathFunction for AggregateFunction {
    fn name(&self) -> &str { "aggregate" }
    fn human_friendly_name(&self) -> &str { "Aggregate" }
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
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(2),
                actual: 0,
            });
        }

        // This needs lambda evaluation support
        Err(FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "aggregate() requires lambda evaluation support".to_string(),
        })
    }
}

/// sort() function - sorts the collection
pub struct SortFunction;

impl FhirPathFunction for SortFunction {
    fn name(&self) -> &str { "sort" }
    fn human_friendly_name(&self) -> &str { "Sort" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "sort",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = context.input.clone().to_collection();

        // Simple sort without selector
        if args.is_empty() {
            let mut items_vec: Vec<FhirPathValue> = items.into_iter().collect();
            // TODO: Implement sorting when PartialOrd is available for FhirPathValue
            // items_vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            Ok(FhirPathValue::collection(items_vec))
        } else {
            // Sort with selector requires lambda evaluation
            Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "sort() with selector requires lambda evaluation support".to_string(),
            })
        }
    }
}

/// length() function - returns the length of a string
pub struct LengthFunction;

impl FhirPathFunction for LengthFunction {
    fn name(&self) -> &str { "length" }
    fn human_friendly_name(&self) -> &str { "Length" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "length",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::Integer(s.len() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// subsetOf() function - returns true if the input collection is a subset of the argument collection
pub struct SubsetOfFunction;

impl FhirPathFunction for SubsetOfFunction {
    fn name(&self) -> &str { "subsetOf" }
    fn human_friendly_name(&self) -> &str { "Subset Of" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "subsetOf",
                vec![ParameterInfo::required("superset", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let superset_arg = &args[0];
        let subset = context.input.clone().to_collection();
        let superset = superset_arg.clone().to_collection();

        // Empty set is subset of any set
        if subset.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }

        // Check if every element in subset exists in superset
        let is_subset = subset.iter().all(|item| {
            superset.iter().any(|super_item| super_item == item)
        });

        Ok(FhirPathValue::Boolean(is_subset))
    }
}

/// supersetOf() function - returns true if the input collection is a superset of the argument collection
pub struct SupersetOfFunction;

impl FhirPathFunction for SupersetOfFunction {
    fn name(&self) -> &str { "supersetOf" }
    fn human_friendly_name(&self) -> &str { "Superset Of" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "supersetOf",
                vec![ParameterInfo::required("subset", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let subset_arg = &args[0];
        let superset = context.input.clone().to_collection();
        let subset = subset_arg.clone().to_collection();

        // Any set is superset of empty set
        if subset.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }

        // Check if every element in subset exists in superset
        let is_superset = subset.iter().all(|item| {
            superset.iter().any(|super_item| super_item == item)
        });

        Ok(FhirPathValue::Boolean(is_superset))
    }
}
