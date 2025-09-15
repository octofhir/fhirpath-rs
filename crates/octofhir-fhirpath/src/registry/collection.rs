//! Collection functions implementation for FHIRPath
//!
//! Implements core collection manipulation functions including navigation,
//! aggregation, set operations, and logic functions.

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::{
    FhirPathValue, Result,
    error_code::{FP0053, FP0155},
};
use crate::register_function;
use std::collections::HashSet;

/// Utility functions for collection operations
pub struct CollectionUtils;

impl CollectionUtils {
    /// Generate a hash key for a FhirPathValue for use in collections
    pub fn value_hash_key(value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(s) => {
                // Lenient numeric string normalization for set ops: "1" == 1
                if let Ok(i) = s.parse::<i64>() {
                    let d = rust_decimal::Decimal::from(i).normalize();
                    format!("num:{}", d)
                } else if let Ok(d) = s.parse::<rust_decimal::Decimal>() {
                    format!("num:{}", d.normalize())
                } else {
                    format!("str:{}", s)
                }
            }
            // Normalize numeric types so that 1 and 1.0 are treated equivalently
            FhirPathValue::Integer(i) => {
                let d = rust_decimal::Decimal::from(*i).normalize();
                format!("num:{}", d)
            }
            FhirPathValue::Decimal(d) => format!("num:{}", d.normalize()),
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
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        // Add unique items from first collection
        for item in first {
            let hash_key = Self::value_hash_key(item);
            if seen.insert(hash_key) {
                result.push(item.clone());
            }
        }

        // Add unique items from second collection that aren't already present
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

        // Lambda functions (where, select, all, exists, aggregate) now handled by lambda functions module

        // Register missing collection functions
        self.register_alltrue_function()?;
        self.register_combine_function()?;
        self.register_isdistinct_function()?;
        self.register_exclude_function()?;
        // aggregate() function is now implemented through lambda functions module
        self.register_sort_function()?;
        self.register_subsetof_function()?;
        self.register_supersetof_function()?;
        self.register_trace_function()?;
        self.register_repeat_function_metadata()?;
        self.register_repeat_all_function_metadata()?;

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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                match context.input.first() {
                    Some(val) => Ok(val.clone()),
                    None => Ok(FhirPathValue::empty()),
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                match context.input.last(){
                    Some(val) => Ok(val.clone()),
                    None => Ok(FhirPathValue::empty()),
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let items: Vec<FhirPathValue> = context.input.iter().skip(1).cloned().collect();
                Ok(FhirPathValue::collection(items))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                // Check if the input collection is ordered (semantic validation)
                if !context.input.is_ordered_collection() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0155,
                        "skip() function can only be applied to ordered collections. The children() function returns an unordered collection.".to_string()
                    ));
                }

                let skip_count = match context.arguments.first() {
                    Some(FhirPathValue::Integer(n)) => {
                        if *n < 0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "skip() argument must be non-negative".to_string()
                            ));
                        }
                        *n as usize
                    }
                    Some(_) => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "skip() requires an integer argument".to_string()
                        ));
                    }
                    None => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "skip() requires exactly one integer argument".to_string()
                        ));
                    }
                };

                let items: Vec<FhirPathValue> = context.input.iter().skip(skip_count).cloned().collect();
                Ok(FhirPathValue::collection(items))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let take_count = match context.arguments.first() {
                    Some(FhirPathValue::Integer(n)) => {
                        if *n < 0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "take() argument must be non-negative".to_string()
                            ));
                        }
                        *n as usize
                    }
                    Some(_) => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "take() requires an integer argument".to_string()
                        ));
                    }
                    None => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "take() requires exactly one integer argument".to_string()
                        ));
                    }
                };

                let items: Vec<FhirPathValue> = context.input.iter().take(take_count).cloned().collect();
                Ok(FhirPathValue::collection(items))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                Ok(FhirPathValue::Integer(context.input.len() as i64))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                match context.input.len() {
                    0 => Ok(FhirPathValue::empty()),
                    1 => Ok(context.input.first().unwrap().clone()),
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let items = CollectionUtils::remove_duplicates(&context.input.cloned_collection());
                Ok(FhirPathValue::collection(items))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                // Note: union() function always gets exactly one argument due to function signature
                // The context.arguments contains the evaluated result of that single argument

                let input_vec: Vec<FhirPathValue> = context.input.cloned_collection();
                let args_vec: Vec<FhirPathValue> = context.arguments.cloned_collection();
                let result = CollectionUtils::union_collections(&input_vec, &args_vec);
                Ok(FhirPathValue::collection(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let input_vec: Vec<FhirPathValue> = context.input.cloned_collection();
                let args_vec: Vec<FhirPathValue> = context.arguments.cloned_collection();
                let result = CollectionUtils::intersect_collections(&input_vec, &args_vec);
                Ok(FhirPathValue::collection(result))
            }
        )
    }

    // Lambda functions with metadata registration but deferred implementation
    fn register_alltrue_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "allTrue",
            category: FunctionCategory::Collection,
            description: "Returns true if all items in the collection are boolean true",
            parameters: [],
            return_type: "boolean",
            examples: ["(true | true | true).allTrue()", "(Patient.active | Patient.active).allTrue()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() == 0 {
                    return Ok(FhirPathValue::Boolean(true)); // Empty collection is vacuously true
                }

                for value in context.input.iter() {
                    match value {
                        FhirPathValue::Boolean(false) => return Ok(FhirPathValue::Boolean(false)),
                        FhirPathValue::Boolean(true) => continue,
                        FhirPathValue::Empty => return Ok(FhirPathValue::empty()), // Presence of empty makes result empty
                        _ => return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "allTrue() can only be applied to boolean values".to_string()
                        ))
                    }
                }

                Ok(FhirPathValue::Boolean(true))
            }
        )
    }

    fn register_combine_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "combine",
            category: FunctionCategory::Collection,
            description: "Merge the input and other collections into a single collection without eliminating duplicate values",
            parameters: ["other": Some("collection".to_string()) => "Collection to combine with"],
            return_type: "collection",
            examples: ["(1 | 2).combine(2 | 3)", "Patient.name.combine(Patient.address)"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                // Combine collections preserving duplicates (unlike union which removes them)
                let mut result: Vec<FhirPathValue> = context.input.cloned_collection();

                // Add the argument collection to the result
                let other_collection = context.arguments.cloned_collection();
                result.extend(other_collection);

                Ok(FhirPathValue::collection(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let mut seen = HashSet::new();

                for value in context.input.iter() {
                    let hash_key = CollectionUtils::value_hash_key(value);
                    if !seen.insert(hash_key) {
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                Ok(FhirPathValue::Boolean(true))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                // Build set of items to exclude from all arguments
                let mut exclude_set = HashSet::new();
                for item in context.arguments.iter() {
                    exclude_set.insert(CollectionUtils::value_hash_key(item));
                }

                // Filter input collection
                let result: Vec<FhirPathValue> = context.input
                    .iter()
                    .filter(|value| !exclude_set.contains(&CollectionUtils::value_hash_key(value)))
                    .cloned()
                    .collect();

                Ok(FhirPathValue::collection(result))
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
            implementation: |_context: &FunctionContext| -> Result<FhirPathValue> {
                Err(crate::core::FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "sort() function is implemented through metadata-aware lambda evaluation system".to_string()
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                // Note: subsetOf() function always gets exactly one argument due to function signature
                // The context.arguments contains the evaluated result of that single argument

                // Build set of items in the other collection (flatten arguments)
                let mut other_set = HashSet::new();
                for arg in context.arguments.iter() {
                    match arg {
                        FhirPathValue::Collection(coll) => {
                            for item in coll.iter() {
                                other_set.insert(CollectionUtils::value_hash_key(item));
                            }
                        }
                        FhirPathValue::Empty => {}
                        other => {
                            other_set.insert(CollectionUtils::value_hash_key(other));
                        }
                    }
                }

                // Check if all items in input are in other collection
                for value in context.input.iter() {
                    if !other_set.contains(&CollectionUtils::value_hash_key(value)) {
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                Ok(FhirPathValue::Boolean(true))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                // Note: supersetOf() function always gets exactly one argument due to function signature
                // The context.arguments contains the evaluated result of that single argument

                // Build set of items in this collection
                let mut this_set = HashSet::new();
                for item in context.input.iter() {
                    this_set.insert(CollectionUtils::value_hash_key(item));
                }

                // Check if all items in other collection are in this collection (flatten arguments)
                for arg in context.arguments.iter() {
                    match arg {
                        FhirPathValue::Collection(coll) => {
                            for value in coll.iter() {
                                if !this_set.contains(&CollectionUtils::value_hash_key(value)) {
                                    return Ok(FhirPathValue::Boolean(false));
                                }
                            }
                        }
                        FhirPathValue::Empty => {}
                        other => {
                            if !this_set.contains(&CollectionUtils::value_hash_key(other)) {
                                return Ok(FhirPathValue::Boolean(false));
                            }
                        }
                    }
                }

                Ok(FhirPathValue::Boolean(true))
            }
        )
    }

    fn register_trace_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "trace",
            category: FunctionCategory::Utility,
            description: "Logs the input value and optionally evaluates a projection expression on each item",
            parameters: [
                "name": Some("string".to_string()) => "Name to use in trace output (optional)",
                "projection": None => "Expression to evaluate on each item for output (optional)"
            ],
            return_type: "any",
            examples: ["Patient.name.trace('names')", "name.trace('test', given)"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                trace_impl(context)
            }
        )
    }

    fn register_repeat_function_metadata(&self) -> Result<()> {
        register_function!(
            self,
            sync "repeat",
            category: FunctionCategory::Collection,
            description: "Repeatedly evaluates a lambda expression until no new unique items are found, preventing infinite loops",
            parameters: ["expression": Some("expression".to_string()) => "Lambda expression to repeat recursively"],
            return_type: "collection",
            examples: ["ValueSet.expansion.repeat(contains)", "Questionnaire.repeat(item)", "Bundle.entry.resource.repeat(reference.resolve())"],
            implementation: |_context: &FunctionContext| -> Result<FhirPathValue> {
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "repeat() function requires lambda expression evaluation system (handled by evaluator engine)".to_string()
                ))
            }
        )
    }

    fn register_repeat_all_function_metadata(&self) -> Result<()> {
        register_function!(
            self,
            sync "repeatAll",
            category: FunctionCategory::Collection,
            description: "Repeatedly evaluates a lambda expression allowing duplicate items in the result, unlike repeat() which deduplicates",
            parameters: ["expression": Some("expression".to_string()) => "Lambda expression to repeat recursively"],
            return_type: "collection",
            examples: ["ValueSet.expansion.repeatAll(contains)", "Questionnaire.repeatAll(item)", "Bundle.entry.resource.repeatAll(reference.resolve())"],
            implementation: |_context: &FunctionContext| -> Result<FhirPathValue> {
                Err(crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    "repeatAll() function requires lambda expression evaluation system (handled by evaluator engine)".to_string()
                ))
            }
        )
    }

}

/// Implementation of trace function with lambda support
fn trace_impl(context: &FunctionContext) -> Result<FhirPathValue> {
    let input = &context.input;
    let args = &context.arguments;

    // Get arguments as a vector
    let args_vec = match args {
        FhirPathValue::Collection(collection) => collection.values().to_vec(),
        FhirPathValue::Empty => vec![],
        other => vec![other.clone()],
    };

    // trace() function signature:
    // trace(name: String)
    // trace(name: String, projection: Expression)

    if args_vec.is_empty() || args_vec.len() > 2 {
        return Err(crate::core::FhirPathError::evaluation_error(
            FP0053,
            "trace() function requires 1 or 2 arguments: trace(name) or trace(name, projection)"
                .to_string(),
        ));
    }

    // Get the trace name parameter
    let trace_name = match &args_vec[0] {
        FhirPathValue::String(name) => name.clone(),
        _ => {
            return Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "trace() function first argument must be a string (trace name)".to_string(),
            ));
        }
    };

    // Handle different input types
    if args_vec.len() == 1 {
        // trace(name) - just trace the input and return it
        match input {
            FhirPathValue::Empty => {
                eprintln!("TRACE[{}]: <empty>", trace_name);
                Ok(FhirPathValue::Empty)
            }
            FhirPathValue::Collection(collection) => {
                for (i, value) in collection.values().iter().enumerate() {
                    eprintln!(
                        "TRACE[{}][{}]: {}",
                        trace_name,
                        i,
                        format_trace_value(value)
                    );
                }
                Ok(input.clone())
            }
            single_value => {
                eprintln!(
                    "TRACE[{}]: {}",
                    trace_name,
                    format_trace_value(single_value)
                );
                Ok(input.clone())
            }
        }
    } else {
        // trace(name, projection) - this needs metadata system for full lambda evaluation
        // For now, just trace input and return it (projection will be handled by metadata system)
        match input {
            FhirPathValue::Empty => {
                eprintln!("TRACE[{}]: <empty> (with projection)", trace_name);
                Ok(FhirPathValue::Empty)
            }
            FhirPathValue::Collection(collection) => {
                for (i, value) in collection.values().iter().enumerate() {
                    eprintln!(
                        "TRACE[{}][{}]: {} (with projection)",
                        trace_name,
                        i,
                        format_trace_value(value)
                    );
                }
                Ok(input.clone())
            }
            single_value => {
                eprintln!(
                    "TRACE[{}]: {} (with projection)",
                    trace_name,
                    format_trace_value(single_value)
                );
                Ok(input.clone())
            }
        }
    }
}

/// Format a FhirPathValue for trace output
fn format_trace_value(value: &FhirPathValue) -> String {
    match value {
        FhirPathValue::String(s) => format!("\"{}\"(String)", s),
        FhirPathValue::Integer(i) => format!("{}(Integer)", i),
        FhirPathValue::Decimal(d) => format!("{}(Decimal)", d),
        FhirPathValue::Boolean(b) => format!("{}(Boolean)", b),
        FhirPathValue::Date(d) => format!("{}(Date)", d.to_string()),
        FhirPathValue::DateTime(dt) => format!("{}(DateTime)", dt.to_string()),
        FhirPathValue::Time(t) => format!("{}(Time)", t.to_string()),
        FhirPathValue::Quantity { value, unit, .. } => match unit {
            Some(u) => format!("{} {}(Quantity)", value, u),
            None => format!("{}(Quantity)", value),
        },
        FhirPathValue::Resource(resource) => {
            format!(
                "Resource({})",
                resource
                    .get("resourceType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            )
        }
        FhirPathValue::JsonValue(json) => {
            format!("JsonValue({})", json.to_string())
        }
        FhirPathValue::Collection(items) => {
            format!("Collection[{}]", items.len())
        }
        FhirPathValue::Id(id) => format!("{}(Id)", id),
        FhirPathValue::Base64Binary(data) => format!("Base64[{}](Base64Binary)", data.len()),
        FhirPathValue::Uri(uri) => format!("{}(Uri)", uri),
        FhirPathValue::Url(url) => format!("{}(Url)", url),
        FhirPathValue::TypeInfoObject { namespace, name } => {
            format!("{}:{}(TypeInfo)", namespace, name)
        }
        FhirPathValue::Wrapped(wrapped) => {
            format!("Wrapped({})", wrapped.get_type_info().map(|t| t.type_name.clone()).unwrap_or_else(|| "Any".to_string()))
        }
        FhirPathValue::ResourceWrapped(wrapped) => {
            format!("ResourceWrapped({})", wrapped.get_type_info().map(|t| t.type_name.clone()).unwrap_or_else(|| "Resource".to_string()))
        }
        FhirPathValue::Empty => "<empty>".to_string(),
    }
}

// mod collection_tests;
