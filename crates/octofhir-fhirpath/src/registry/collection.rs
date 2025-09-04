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

}

// mod collection_tests;