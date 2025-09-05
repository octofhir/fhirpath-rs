//! Collection functions implementation for FHIRPath
//! 
//! Implements core collection manipulation functions including navigation,
//! aggregation, set operations, and logic functions.

use super::{FunctionRegistry, FunctionCategory, FunctionContext};
use crate::core::{FhirPathValue, Result, error_code::{FP0053}};
use crate::{register_function};
use std::collections::HashSet;

/// Utility functions for collection operations
pub struct CollectionUtils;

impl CollectionUtils {
    /// Generate a hash key for a FhirPathValue for use in collections
    pub fn value_hash_key(value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(s) => format!("str:{}", s),
            FhirPathValue::Integer(i) => format!("int:{}", i),
            FhirPathValue::Decimal(d) => format!("dec:{}", d),
            FhirPathValue::Boolean(b) => format!("bool:{}", b),
            FhirPathValue::Date(d) => format!("date:{}", d.to_string()),
            FhirPathValue::DateTime(dt) => format!("datetime:{}", dt.to_string()),
            FhirPathValue::Time(t) => format!("time:{}", t.to_string()),
            _ => format!("complex:{:?}", value),
        }
    }

    /// Remove duplicates from a collection while preserving order
    pub fn remove_duplicates(collection: &[FhirPathValue]) -> Vec<FhirPathValue> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for item in collection {
            let hash_key = Self::value_hash_key(item);
            if seen.insert(hash_key) {
                result.push(item.clone());
            }
        }

        result
    }

    /// Combine two collections with union semantics (no duplicates)
    pub fn union_collections(
        first: &[FhirPathValue],
        second: &[FhirPathValue],
    ) -> Vec<FhirPathValue> {
        let mut result = first.to_vec();
        let mut seen = HashSet::new();

        // Add all items from first collection to seen set
        for item in first {
            seen.insert(Self::value_hash_key(item));
        }

        // Add items from second collection that aren't already present
        for item in second {
            let hash_key = Self::value_hash_key(item);
            if seen.insert(hash_key) {
                result.push(item.clone());
            }
        }

        result
    }

    /// Find intersection of two collections
    pub fn intersect_collections(
        first: &[FhirPathValue],
        second: &[FhirPathValue],
    ) -> Vec<FhirPathValue> {
        let mut second_set = HashSet::new();
        for item in second {
            second_set.insert(Self::value_hash_key(item));
        }

        let mut result = Vec::new();
        let mut result_set = HashSet::new();

        for item in first {
            let hash_key = Self::value_hash_key(item);
            if second_set.contains(&hash_key) && result_set.insert(hash_key) {
                result.push(item.clone());
            }
        }

        result
    }
}

impl FunctionRegistry {
    pub fn register_collection_functions(&self) -> Result<()> {
        self.register_first_function()?;
        self.register_last_function()?;
        self.register_tail_function()?;
        self.register_skip_function()?;
        self.register_take_function()?;
        self.register_count_function()?;
        self.register_single_function()?;
        self.register_distinct_function()?;
        self.register_union_function()?;
        self.register_intersect_function()?;
        
        // Register lambda functions with metadata (deferred implementation)
        self.register_where_function_metadata()?;
        self.register_select_function_metadata()?;
        self.register_all_function_metadata()?;
        
        // Register missing collection functions
        self.register_alltrue_function()?;
        self.register_combine_function()?;
        self.register_isdistinct_function()?;
        self.register_exclude_function()?;
        self.register_aggregate_function()?;
        self.register_sort_function()?;
        self.register_subsetof_function()?;
        self.register_supersetof_function()?;
        self.register_trace_function()?;
        self.register_repeat_function()?;
        
        Ok(())
    }

    fn register_first_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "first",
            category: FunctionCategory::Collection,
            description: "Returns the first item in a collection, or empty if the collection is empty",
            parameters: [],
            return_type: "any",
            examples: ["Patient.name.first()", "Bundle.entry.first().resource"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    Ok(vec![])
                } else {
                    Ok(vec![context.input[0].clone()])
                }
            }
        )
    }

    fn register_last_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "last",
            category: FunctionCategory::Collection,
            description: "Returns the last item in a collection, or empty if the collection is empty",
            parameters: [],
            return_type: "any",
            examples: ["Patient.name.last()", "Bundle.entry.last().resource"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    Ok(vec![])
                } else {
                    Ok(vec![context.input.last().unwrap().clone()])
                }
            }
        )
    }

    fn register_tail_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "tail",
            category: FunctionCategory::Collection,
            description: "Returns all items except the first in a collection",
            parameters: [],
            return_type: "collection",
            examples: ["Patient.name.tail()", "Bundle.entry.tail()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    Ok(vec![])
                } else {
                    Ok(context.input[1..].to_vec())
                }
            }
        )
    }

    fn register_skip_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "skip",
            category: FunctionCategory::Collection,
            description: "Returns all items except the first n items in a collection",
            parameters: ["num": Some("integer".to_string()) => "Number of items to skip"],
            return_type: "collection",
            examples: ["Patient.name.skip(1)", "Bundle.entry.skip(2)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.is_empty() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "skip() requires exactly one integer argument".to_string()
                    ));
                }

                let skip_count = match &context.arguments[0] {
                    FhirPathValue::Integer(n) => {
                        if *n < 0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "skip() argument must be non-negative".to_string()
                            ));
                        }
                        *n as usize
                    }
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "skip() requires an integer argument".to_string()
                        ));
                    }
                };

                if skip_count >= context.input.len() {
                    Ok(vec![])
                } else {
                    Ok(context.input[skip_count..].to_vec())
                }
            }
        )
    }

    fn register_take_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "take",
            category: FunctionCategory::Collection,
            description: "Returns the first n items in a collection",
            parameters: ["num": Some("integer".to_string()) => "Number of items to take"],
            return_type: "collection",
            examples: ["Patient.name.take(2)", "Bundle.entry.take(5)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.is_empty() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "take() requires exactly one integer argument".to_string()
                    ));
                }

                let take_count = match &context.arguments[0] {
                    FhirPathValue::Integer(n) => {
                        if *n < 0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "take() argument must be non-negative".to_string()
                            ));
                        }
                        *n as usize
                    }
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "take() requires an integer argument".to_string()
                        ));
                    }
                };

                if take_count == 0 {
                    Ok(vec![])
                } else if take_count >= context.input.len() {
                    Ok(context.input.to_vec())
                } else {
                    Ok(context.input[..take_count].to_vec())
                }
            }
        )
    }

    fn register_count_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "count",
            category: FunctionCategory::Collection,
            description: "Returns the number of items in a collection",
            parameters: [],
            return_type: "integer",
            examples: ["Patient.name.count()", "Bundle.entry.count()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                Ok(vec![FhirPathValue::Integer(context.input.len() as i64)])
            }
        )
    }

    fn register_single_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "single",
            category: FunctionCategory::Collection,
            description: "Returns the single item in a collection, or error if the collection is empty or has more than one item",
            parameters: [],
            return_type: "any",
            examples: ["Patient.id.single()", "Patient.active.single()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                match context.input.len() {
                    0 => Ok(vec![]),
                    1 => Ok(vec![context.input[0].clone()]),
                    _ => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "single() can only be called on collections with 0 or 1 items".to_string()
                    ))
                }
            }
        )
    }

    fn register_distinct_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "distinct",
            category: FunctionCategory::Collection,
            description: "Returns a collection with duplicate items removed",
            parameters: [],
            return_type: "collection",
            examples: ["Patient.name.family.distinct()", "Bundle.entry.resource.resourceType.distinct()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                Ok(CollectionUtils::remove_duplicates(context.input))
            }
        )
    }

    fn register_union_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "union",
            category: FunctionCategory::Collection,
            description: "Returns the union of two collections, removing duplicates",
            parameters: ["other": Some("collection".to_string()) => "Collection to union with"],
            return_type: "collection",
            examples: ["Patient.name.given.union(Patient.name.family)", "Bundle.entry.resource.union($otherBundle.entry.resource)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.is_empty() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "union() requires exactly one collection argument".to_string()
                    ));
                }

                // Arguments are the second collection 
                Ok(CollectionUtils::union_collections(context.input, context.arguments))
            }
        )
    }

    fn register_intersect_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "intersect",
            category: FunctionCategory::Collection,
            description: "Returns the intersection of two collections",
            parameters: ["other": Some("collection".to_string()) => "Collection to intersect with"],
            return_type: "collection",
            examples: ["Patient.name.given.intersect(Patient.name.family)", "Bundle.entry.resource.intersect($otherBundle.entry.resource)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.is_empty() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "intersect() requires exactly one collection argument".to_string()
                    ));
                }

                Ok(CollectionUtils::intersect_collections(context.input, context.arguments))
            }
        )
    }

    // Lambda functions with metadata registration but deferred implementation
    // These require lambda expression evaluation system to be built first
    
    fn register_where_function_metadata(&self) -> Result<()> {
        register_function!(
            self,
            sync "where",
            category: FunctionCategory::Collection,
            description: "Returns items from the collection where the given expression evaluates to true",
            parameters: ["criteria": Some("expression".to_string()) => "Boolean expression to filter by"],
            return_type: "collection",
            examples: ["Patient.name.where(use = 'official')", "Bundle.entry.where(resource.active = true)"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "where() function requires lambda expression evaluation system (not yet implemented)".to_string()
                ))
            }
        )
    }

    fn register_select_function_metadata(&self) -> Result<()> {
        register_function!(
            self,
            sync "select",
            category: FunctionCategory::Collection,
            description: "Projects each item in the collection through the given expression",
            parameters: ["projection": Some("expression".to_string()) => "Expression to apply to each item"],
            return_type: "collection",
            examples: ["Patient.name.select(family + ', ' + given.first())", "Bundle.entry.select(resource.id)"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "select() function requires lambda expression evaluation system (not yet implemented)".to_string()
                ))
            }
        )
    }

    fn register_all_function_metadata(&self) -> Result<()> {
        register_function!(
            self,
            sync "all",
            category: FunctionCategory::Collection,
            description: "Returns true if the given expression evaluates to true for all items in the collection",
            parameters: ["criteria": Some("expression".to_string()) => "Boolean expression to evaluate for each item"],
            return_type: "boolean",
            examples: ["Patient.name.all(use = 'official')", "Bundle.entry.all(resource.exists())"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "all() function requires lambda expression evaluation system (not yet implemented)".to_string()
                ))
            }
        )
    }

    fn register_alltrue_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "allTrue",
            category: FunctionCategory::Collection,
            description: "Returns true if all items in the collection are boolean true",
            parameters: [],
            return_type: "boolean",
            examples: ["(true | true | true).allTrue()", "(Patient.active | Patient.active).allTrue()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![FhirPathValue::Boolean(true)]); // Empty collection is vacuously true
                }
                
                for value in context.input {
                    match value {
                        FhirPathValue::Boolean(false) => return Ok(vec![FhirPathValue::Boolean(false)]),
                        FhirPathValue::Boolean(true) => continue,
                        FhirPathValue::Empty => return Ok(vec![]), // Presence of empty makes result empty
                        _ => return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "allTrue() can only be applied to boolean values".to_string()
                        ))
                    }
                }
                
                Ok(vec![FhirPathValue::Boolean(true)])
            }
        )
    }

    fn register_combine_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "combine",
            category: FunctionCategory::Collection,
            description: "Combines two collections into a single collection with duplicates removed",
            parameters: ["other": Some("collection".to_string()) => "Collection to combine with"],
            return_type: "collection",
            examples: ["(1 | 2).combine(2 | 3)", "Patient.name.combine(Patient.address)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "combine() requires exactly one argument".to_string()
                    ));
                }

                let second_collection = match &context.arguments[0] {
                    FhirPathValue::Collection(items) => items.clone(),
                    single_item => vec![single_item.clone()],
                };

                let result = CollectionUtils::union_collections(context.input, &second_collection);
                Ok(result)
            }
        )
    }

    fn register_isdistinct_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "isDistinct",
            category: FunctionCategory::Collection,
            description: "Returns true if all items in the collection are unique",
            parameters: [],
            return_type: "boolean",
            examples: ["(1 | 2 | 3).isDistinct()", "Patient.name.given.isDistinct()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let mut seen = HashSet::new();
                
                for value in context.input {
                    let hash_key = CollectionUtils::value_hash_key(value);
                    if !seen.insert(hash_key) {
                        return Ok(vec![FhirPathValue::Boolean(false)]);
                    }
                }
                
                Ok(vec![FhirPathValue::Boolean(true)])
            }
        )
    }

    fn register_exclude_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "exclude",
            category: FunctionCategory::Collection,
            description: "Returns a collection with all items from the input except those that match the argument",
            parameters: ["other": Some("collection".to_string()) => "Collection of items to exclude"],
            return_type: "collection",
            examples: ["(1 | 2 | 3).exclude(2)", "Patient.telecom.exclude(Patient.telecom.where(system = 'email'))"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "exclude() requires exactly one argument".to_string()
                    ));
                }

                let exclude_items = match &context.arguments[0] {
                    FhirPathValue::Collection(items) => items.clone(),
                    single_item => vec![single_item.clone()],
                };

                // Build set of items to exclude
                let mut exclude_set = HashSet::new();
                for item in &exclude_items {
                    exclude_set.insert(CollectionUtils::value_hash_key(item));
                }

                // Filter input collection
                let result: Vec<FhirPathValue> = context.input
                    .iter()
                    .filter(|value| !exclude_set.contains(&CollectionUtils::value_hash_key(value)))
                    .cloned()
                    .collect();

                Ok(result)
            }
        )
    }

    fn register_aggregate_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "aggregate",
            category: FunctionCategory::Collection,
            description: "Performs aggregation over a collection using lambda expressions (placeholder implementation)",
            parameters: [
                "initial": Some("any".to_string()) => "Initial value for aggregation",
                "expression": Some("expression".to_string()) => "Lambda expression for aggregation"
            ],
            return_type: "any",
            examples: ["(1 | 2 | 3).aggregate($total + $this, 0)"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "aggregate() function requires lambda expression evaluation system (not yet implemented)".to_string()
                ))
            }
        )
    }

    fn register_sort_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "sort",
            category: FunctionCategory::Collection,
            description: "Returns the collection sorted by the specified criteria (placeholder implementation)",
            parameters: ["criteria": Some("expression".to_string()) => "Expression to sort by (optional)"],
            return_type: "collection",
            examples: ["Patient.name.sort()", "Bundle.entry.sort(resource.id)"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "sort() function requires lambda expression evaluation system (not yet implemented)".to_string()
                ))
            }
        )
    }

    fn register_subsetof_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "subsetOf",
            category: FunctionCategory::Collection,
            description: "Returns true if this collection is a subset of the other collection",
            parameters: ["other": Some("collection".to_string()) => "Collection to compare against"],
            return_type: "boolean",
            examples: ["(1 | 2).subsetOf(1 | 2 | 3)", "Patient.name.given.subsetOf(Patient.name.family)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "subsetOf() requires exactly one argument".to_string()
                    ));
                }

                let other_collection = match &context.arguments[0] {
                    FhirPathValue::Collection(items) => items.clone(),
                    single_item => vec![single_item.clone()],
                };

                // Build set of items in the other collection
                let mut other_set = HashSet::new();
                for item in &other_collection {
                    other_set.insert(CollectionUtils::value_hash_key(item));
                }

                // Check if all items in input are in other collection
                for value in context.input {
                    if !other_set.contains(&CollectionUtils::value_hash_key(value)) {
                        return Ok(vec![FhirPathValue::Boolean(false)]);
                    }
                }

                Ok(vec![FhirPathValue::Boolean(true)])
            }
        )
    }

    fn register_supersetof_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "supersetOf",
            category: FunctionCategory::Collection,
            description: "Returns true if this collection is a superset of the other collection",
            parameters: ["other": Some("collection".to_string()) => "Collection to compare against"],
            return_type: "boolean",
            examples: ["(1 | 2 | 3).supersetOf(1 | 2)", "Patient.telecom.supersetOf(Patient.telecom.where(system = 'phone'))"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "supersetOf() requires exactly one argument".to_string()
                    ));
                }

                let other_collection = match &context.arguments[0] {
                    FhirPathValue::Collection(items) => items.clone(),
                    single_item => vec![single_item.clone()],
                };

                // Build set of items in this collection
                let mut this_set = HashSet::new();
                for item in context.input {
                    this_set.insert(CollectionUtils::value_hash_key(item));
                }

                // Check if all items in other collection are in this collection
                for value in &other_collection {
                    if !this_set.contains(&CollectionUtils::value_hash_key(value)) {
                        return Ok(vec![FhirPathValue::Boolean(false)]);
                    }
                }

                Ok(vec![FhirPathValue::Boolean(true)])
            }
        )
    }

    fn register_trace_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "trace",
            category: FunctionCategory::Utility,
            description: "Logs the input value and passes it through unchanged for debugging",
            parameters: ["name": Some("string".to_string()) => "Name to use in trace output (optional)"],
            return_type: "any",
            examples: ["Patient.name.trace('names')", "Bundle.entry.trace().resource"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let trace_name = if !context.arguments.is_empty() {
                    match &context.arguments[0] {
                        FhirPathValue::String(s) => s.clone(),
                        _ => "trace".to_string(),
                    }
                } else {
                    "trace".to_string()
                };

                // Print trace information to stderr for debugging
                eprintln!("TRACE[{}]: {} items", trace_name, context.input.len());
                for (i, value) in context.input.iter().enumerate() {
                    eprintln!("  [{}]: {:?}", i, value);
                }

                // Pass through input unchanged
                Ok(context.input.to_vec())
            }
        )
    }

    fn register_repeat_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "repeat",
            category: FunctionCategory::Collection,
            description: "Repeatedly evaluates a lambda expression until no new unique items are found, preventing infinite loops",
            parameters: ["expression": Some("expression".to_string()) => "Lambda expression to repeat recursively"],
            return_type: "collection",
            examples: ["ValueSet.expansion.repeat(contains)", "Questionnaire.repeat(item)", "Bundle.entry.resource.repeat(reference.resolve())"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                // The repeat() function requires lambda evaluation support
                // For now, we'll return an error indicating lambda support is required
                // The actual implementation should be handled by the evaluator engine
                // with proper lambda expression evaluation and cycle detection
                
                if context.arguments.is_empty() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "repeat() requires exactly one lambda expression argument".to_string()
                    ));
                }
                
                // This is a placeholder - the real implementation needs:
                // 1. Lambda expression parsing from arguments 
                // 2. Recursive evaluation with cycle detection
                // 3. Queue-based processing to prevent stack overflow
                // 4. Integration with the evaluator's lambda support
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "repeat() function with lambda expressions requires full evaluator integration. Use the FhirPathEngine.evaluate() method which supports lambda evaluation.".to_string()
                ))
            }
        )
    }

}

// mod collection_tests;