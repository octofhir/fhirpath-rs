// FHIRPath Evaluator
//
// This module implements the evaluation of FHIRPath expressions.

use crate::errors::FhirPathError;
use crate::lexer::tokenize;
use crate::model::{FhirPathValue, FhirResource};
use crate::parser::{parse, AstNode, BinaryOperator, UnaryOperator};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Read;

// UCUM library imports for quantity handling
use octofhir_ucum_fhir::FhirQuantity;

// Regex support for matches function
use regex;

#[cfg(feature = "trace")]
use log::{debug, trace};

/// Context for FHIRPath evaluation
#[derive(Clone)]
pub struct EvaluationContext {
    /// The current FHIR resource being evaluated
    pub resource: serde_json::Value,

    /// The current context node in the resource
    pub context: serde_json::Value,

    /// Variables defined in the current scope
    pub variables: HashMap<String, FhirPathValue>,

    /// The current item in a collection during iteration ($this)
    pub this_item: Option<FhirPathValue>,

    /// The current index in a collection during iteration ($index)
    pub index: Option<usize>,

    /// The total number of items in a collection during iteration ($total)
    pub total: Option<usize>,

    /// Optimization settings
    pub optimization_enabled: bool,

    /// Cache for expression results
    pub expression_cache: HashMap<u64, FhirPathValue>,
}

impl EvaluationContext {
    /// Initialize standard FHIRPath variables
    fn init_standard_variables() -> HashMap<String, FhirPathValue> {
        let mut variables = HashMap::new();

        // Standard FHIRPath variables
        variables.insert(
            "sct".to_string(),
            FhirPathValue::String("http://snomed.info/sct".to_string()),
        );
        variables.insert(
            "loinc".to_string(),
            FhirPathValue::String("http://loinc.org".to_string()),
        );
        variables.insert(
            "ucum".to_string(),
            FhirPathValue::String("http://unitsofmeasure.org".to_string()),
        );

        variables
    }

    /// Creates a new evaluation context
    pub fn new(resource: serde_json::Value) -> Self {
        Self {
            context: resource.clone(),
            resource,
            variables: Self::init_standard_variables(),
            this_item: None,
            index: None,
            total: None,
            optimization_enabled: false,
            expression_cache: HashMap::new(),
        }
    }

    /// Creates a new evaluation context with optimization settings
    pub fn new_with_optimization(resource: serde_json::Value, optimization_enabled: bool) -> Self {
        Self {
            context: resource.clone(),
            resource,
            variables: Self::init_standard_variables(),
            this_item: None,
            index: None,
            total: None,
            optimization_enabled,
            expression_cache: HashMap::new(),
        }
    }

    /// Sets a variable in the context
    pub fn set_variable(&mut self, name: &str, value: FhirPathValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// Gets a variable from the context
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        self.variables.get(name)
    }

    /// Sets the current item in a collection during iteration ($this)
    pub fn set_this(&mut self, item: FhirPathValue) {
        self.this_item = Some(item);
    }

    /// Gets the current item in a collection during iteration ($this)
    pub fn get_this(&self) -> Option<&FhirPathValue> {
        self.this_item.as_ref()
    }

    /// Sets the current index in a collection during iteration ($index)
    pub fn set_index(&mut self, idx: usize) {
        self.index = Some(idx);
    }

    /// Gets the current index in a collection during iteration ($index)
    pub fn get_index(&self) -> Option<usize> {
        self.index
    }

    /// Sets the total number of items in a collection during iteration ($total)
    pub fn set_total(&mut self, total: usize) {
        self.total = Some(total);
    }

    /// Gets the total number of items in a collection during iteration ($total)
    pub fn get_total(&self) -> Option<usize> {
        self.total
    }

    /// Creates a new context for collection iteration
    pub fn create_iteration_context(
        &self,
        item: FhirPathValue,
        idx: usize,
        total: usize,
    ) -> Result<Self, FhirPathError> {
        let context_value = match &item {
            FhirPathValue::Resource(resource) => {
                serde_json::to_value(resource).map_err(FhirPathError::JsonError)?
            }
            _ => serde_json::to_value(&item).map_err(FhirPathError::JsonError)?,
        };

        Ok(Self {
            resource: self.resource.clone(),
            context: context_value,
            variables: self.variables.clone(),
            this_item: Some(item),
            index: Some(idx),
            total: Some(total),
            optimization_enabled: self.optimization_enabled,
            expression_cache: HashMap::new(),
        })
    }
}

/// Trait for visiting AST nodes during evaluation
pub trait AstVisitor {
    /// Called before evaluating an AST node
    fn before_evaluate(&self, node: &AstNode, context: &EvaluationContext);

    /// Called after evaluating an AST node
    fn after_evaluate(
        &self,
        node: &AstNode,
        context: &EvaluationContext,
        result: &Result<FhirPathValue, FhirPathError>,
    );
}

/// A visitor that logs AST evaluation steps
pub struct LoggingVisitor {
    depth: std::cell::Cell<usize>,
}

impl Default for LoggingVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingVisitor {
    /// Creates a new logging visitor
    pub fn new() -> Self {
        Self {
            depth: std::cell::Cell::new(0),
        }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.depth.get())
    }
}

impl AstVisitor for LoggingVisitor {
    fn before_evaluate(&self, _node: &AstNode, _context: &EvaluationContext) {
        #[cfg(feature = "trace")]
        {
            let indent = self.indent();
            trace!("{}Evaluating: {:?}", indent, _node);
            self.depth.set(self.depth.get() + 1);
        }
    }

    fn after_evaluate(
        &self,
        _node: &AstNode,
        _context: &EvaluationContext,
        _result: &Result<FhirPathValue, FhirPathError>,
    ) {
        #[cfg(feature = "trace")]
        {
            self.depth.set(self.depth.get() - 1);
            let indent = self.indent();
            match _result {
                Ok(value) => trace!("{}Result for {:?}: {:?}", indent, _node, value),
                Err(err) => debug!("{}Error evaluating {:?}: {:?}", indent, _node, err),
            }
        }
    }
}

/// A no-op visitor that does nothing
pub struct NoopVisitor;

impl Default for NoopVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl NoopVisitor {
    /// Creates a new no-op visitor
    pub fn new() -> Self {
        Self
    }
}

impl AstVisitor for NoopVisitor {
    fn before_evaluate(&self, _node: &AstNode, _context: &EvaluationContext) {
        // Do nothing
    }

    fn after_evaluate(
        &self,
        _node: &AstNode,
        _context: &EvaluationContext,
        _result: &Result<FhirPathValue, FhirPathError>,
    ) {
        // Do nothing
    }
}

/// Returns the FHIRPath type name for a given value
fn get_fhirpath_type_name(value: &FhirPathValue) -> String {
    match value {
        FhirPathValue::Empty => "Empty".to_string(),
        FhirPathValue::Boolean(_) => "Boolean".to_string(),
        FhirPathValue::Integer(_) => "Integer".to_string(),
        FhirPathValue::Decimal(_) => "Decimal".to_string(),
        FhirPathValue::String(_) => "String".to_string(),
        FhirPathValue::Date(_) => "Date".to_string(),
        FhirPathValue::DateTime(_) => "DateTime".to_string(),
        FhirPathValue::Time(_) => "Time".to_string(),
        FhirPathValue::Quantity { .. } => "Quantity".to_string(),
        FhirPathValue::Collection(_) => "Collection".to_string(),
        FhirPathValue::Resource(resource) => {
            // Return the resource type if available, otherwise "Resource"
            resource
                .resource_type
                .clone()
                .unwrap_or_else(|| "Resource".to_string())
        }
    }
}

/// Evaluates a FHIRPath expression AST
pub fn evaluate_ast(
    node: &AstNode,
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    evaluate_ast_internal(node, context, &NoopVisitor::new())
}

/// Evaluates a FHIRPath expression AST with a custom visitor
pub fn evaluate_ast_with_visitor(
    node: &AstNode,
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    visitor.before_evaluate(node, context);
    let result = evaluate_ast_internal(node, context, visitor);
    visitor.after_evaluate(node, context, &result);
    result
}

/// Evaluates a FHIRPath expression AST with a mutable context for caching
pub fn evaluate_ast_with_caching(
    node: &AstNode,
    context: &mut EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    visitor.before_evaluate(node, context);

    // Quick check if caching is enabled and worthwhile
    if context.optimization_enabled {
        // Use a lightweight cache decision - only cache function calls and paths
        let should_cache = matches!(node,
            AstNode::FunctionCall { .. } |
            AstNode::Path(_, _)
        );

        if should_cache {
            // Use a simpler cache key based on node pointer address
            let cache_key = node as *const AstNode as u64;

            if let Some(cached_result) = context.expression_cache.get(&cache_key) {
                let result = Ok(cached_result.clone());
                visitor.after_evaluate(node, context, &result);
                return result;
            }

            let result = evaluate_ast_internal_uncached(node, context, visitor);

            // Cache successful results with size limit
            if let Ok(ref value) = result {
                if context.expression_cache.len() < 500 { // Reduced cache size for better performance
                    context.expression_cache.insert(cache_key, value.clone());
                }
            }

            visitor.after_evaluate(node, context, &result);
            return result;
        }
    }

    let result = evaluate_ast_internal_uncached(node, context, visitor);
    visitor.after_evaluate(node, context, &result);
    result
}

/// Internal implementation of AST evaluation
fn evaluate_ast_internal(
    node: &AstNode,
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    evaluate_ast_internal_uncached(node, context, visitor)
}

/// Internal implementation of AST evaluation without caching
fn evaluate_ast_internal_uncached(
    node: &AstNode,
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    match node {
        AstNode::Identifier(name) => {
            // Check for special invocations first
            match name.as_str() {
                "$this" => {
                    if let Some(this_value) = context.get_this() {
                        return Ok(this_value.clone());
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                }
                "$index" => {
                    if let Some(index) = context.get_index() {
                        return Ok(FhirPathValue::Integer(index as i64));
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                }
                "$total" => {
                    // First check if there's a $total variable in aggregate context
                    if let Some(total_value) = context.get_variable("total") {
                        return Ok(total_value.clone());
                    }
                    // Fall back to collection size for normal contexts
                    if let Some(total) = context.get_total() {
                        return Ok(FhirPathValue::Integer(total as i64));
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                }
                "{}" => {
                    // Empty collection literal
                    return Ok(FhirPathValue::Collection(vec![]));
                }
                _ => {}
            }

            // Check if it's a variable
            if let Some(value) = context.get_variable(name) {
                return Ok(value.clone());
            }

            // Check if we have a FhirResource in this_item and access its properties directly
            if let Some(FhirPathValue::Resource(resource)) = &context.this_item {
                // First try direct property access
                if let Some(value) = resource.properties.get(name) {
                    return json_to_fhirpath_value(value.clone());
                }

                // Handle FHIR polymorphic properties (e.g., "value" -> "valueQuantity", "valueString", etc.)
                if name == "value" {
                    // Look for polymorphic value properties
                    let polymorphic_prefixes = ["value"];
                    for prefix in &polymorphic_prefixes {
                        for (prop_name, prop_value) in &resource.properties {
                            if prop_name.starts_with(prefix) && prop_name.len() > prefix.len() {
                                // Found a polymorphic property like "valueQuantity"
                                return json_to_fhirpath_value(prop_value.clone());
                            }
                        }
                    }
                }
            }

            // Check if we have a Quantity in this_item and access its properties directly
            if let Some(FhirPathValue::Quantity { value, unit }) = &context.this_item {
                match name.as_str() {
                    "value" => return Ok(FhirPathValue::Decimal(*value)),
                    "unit" => return Ok(FhirPathValue::String(unit.clone())),
                    _ => {} // Fall through to other property access logic
                }
            }

            // Check if the identifier matches the resourceType of the root context
            if let serde_json::Value::Object(obj) = &context.context {
                if let Some(serde_json::Value::String(resource_type)) = obj.get("resourceType") {
                    if resource_type == name {
                        // Return the entire resource as a FhirPathValue::Resource
                        return json_to_fhirpath_value(context.context.clone());
                    }
                }

                // Otherwise, try to access the property from the context
                if let Some(value) = obj.get(name) {
                    return json_to_fhirpath_value(value.clone());
                }
            }

            // If not found, return empty
            Ok(FhirPathValue::Empty)
        }

        AstNode::StringLiteral(value) => Ok(FhirPathValue::String(value.clone())),

        AstNode::NumberLiteral { value, is_decimal } => {
            // Use the is_decimal flag to determine the correct type
            if *is_decimal {
                Ok(FhirPathValue::Decimal(*value))
            } else {
                Ok(FhirPathValue::Integer(*value as i64))
            }
        }

        AstNode::BooleanLiteral(value) => Ok(FhirPathValue::Boolean(*value)),

        AstNode::DateTimeLiteral(value) => {
            // Parse the datetime literal (starts with @)
            let datetime_str = if value.starts_with('@') {
                &value[1..] // Remove the @ prefix
            } else {
                value
            };

            // Determine if this is a Date, DateTime, or Time
            if datetime_str.starts_with('T') {
                // Starts with 'T', so it's a Time literal (e.g., T14:34:28)
                Ok(FhirPathValue::Time(datetime_str.to_string()))
            } else if datetime_str.contains('T') || datetime_str.ends_with('T') {
                // Contains 'T' or ends with 'T' (like "2015T"), so it's a DateTime
                Ok(FhirPathValue::DateTime(datetime_str.to_string()))
            } else {
                // No 'T', so it's a Date
                Ok(FhirPathValue::Date(datetime_str.to_string()))
            }
        }

        AstNode::Variable(name) => {
            // Look up variable in the evaluation context
            if let Some(value) = context.get_variable(name) {
                Ok(value.clone())
            } else {
                // Variable not found, return empty
                Ok(FhirPathValue::Empty)
            }
        }

        AstNode::Path(left, right) => {
            // Evaluate the left side
            let left_result = evaluate_ast_with_visitor(left, context, visitor)?;
            // Create a new context with the left result as the context
            match left_result {
                FhirPathValue::Resource(resource) => {
                    let new_context = EvaluationContext {
                        resource: context.resource.clone(),
                        context: serde_json::to_value(&resource)
                            .map_err(FhirPathError::JsonError)?,
                        variables: context.variables.clone(),
                        this_item: Some(FhirPathValue::Resource(resource)),
                        index: None,
                        total: None,
                        optimization_enabled: context.optimization_enabled,
                        expression_cache: HashMap::new(),
                    };

                    // Evaluate the right side in the new context
                    evaluate_ast_with_visitor(right, &new_context, visitor)
                }
                FhirPathValue::Quantity { value, unit } => {
                    // Create a new context with the Quantity as this_item
                    let new_context = EvaluationContext {
                        resource: context.resource.clone(),
                        context: context.context.clone(),
                        variables: context.variables.clone(),
                        this_item: Some(FhirPathValue::Quantity { value, unit }),
                        index: None,
                        total: None,
                        optimization_enabled: context.optimization_enabled,
                        expression_cache: HashMap::new(),
                    };

                    // Evaluate the right side in the new context
                    evaluate_ast_with_visitor(right, &new_context, visitor)
                }
                FhirPathValue::Collection(items) => {
                    // Check if the right side is a function call - if so, call it on the entire collection
                    match **right {
                        AstNode::FunctionCall { .. } => {
                            // Create a new context with the collection as this_item for function calls
                            let new_context = EvaluationContext {
                                resource: context.resource.clone(),
                                context: context.context.clone(),
                                variables: context.variables.clone(),
                                this_item: Some(FhirPathValue::Collection(items)),
                                index: None,
                                total: None,
                                optimization_enabled: context.optimization_enabled,
                                expression_cache: HashMap::new(),
                            };

                            // Evaluate the function call in the new context
                            evaluate_ast_with_visitor(right, &new_context, visitor)
                        }
                        _ => {
                            // For non-function calls, evaluate the right side for each item and collect the results
                            let mut results = Vec::new();
                            let total = items.len();

                            for (idx, item) in items.into_iter().enumerate() {
                                match item {
                                    FhirPathValue::Resource(resource) => {
                                        // Create an iteration context with index and total information
                                        let new_context = context.create_iteration_context(
                                            FhirPathValue::Resource(resource.clone()),
                                            idx,
                                            total,
                                        )?;

                                        let result = evaluate_ast_with_visitor(
                                            right,
                                            &new_context,
                                            visitor,
                                        )?;
                                        if result != FhirPathValue::Empty {
                                            match result {
                                                FhirPathValue::Collection(mut inner_items) => {
                                                    // Flatten collection results
                                                    results.append(&mut inner_items);
                                                }
                                                _ => results.push(result),
                                            }
                                        }
                                    }
                                    _ => {
                                        // For non-resource items, try to evaluate if they have properties
                                        // This allows for handling primitive types with methods
                                        let new_context = context.create_iteration_context(
                                            item.clone(),
                                            idx,
                                            total,
                                        )?;

                                        // Only try to evaluate if the right side is an identifier (method call)
                                        if let AstNode::Identifier(_) = **right {
                                            let result = evaluate_ast_with_visitor(
                                                right,
                                                &new_context,
                                                visitor,
                                            )?;
                                            if result != FhirPathValue::Empty {
                                                results.push(result);
                                            }
                                        }
                                    }
                                }
                            }

                            if results.is_empty() {
                                // For property access on empty collections, return empty
                                Ok(FhirPathValue::Empty)
                            } else if results.len() == 1 {
                                // If there's only one result, return it directly
                                Ok(results[0].clone())
                            } else {
                                Ok(FhirPathValue::Collection(results))
                            }
                        }
                    }
                }
                FhirPathValue::Empty => {
                    // For empty results, check if the right side is a function call
                    match **right {
                        AstNode::FunctionCall { .. } => {
                            // Create a new context with the left result as this_item for function calls
                            let new_context = EvaluationContext {
                                resource: context.resource.clone(),
                                context: context.context.clone(),
                                variables: context.variables.clone(),
                                this_item: Some(left_result),
                                index: None,
                                total: None,
                                optimization_enabled: context.optimization_enabled,
                                expression_cache: HashMap::new(),
                            };

                            // Evaluate the function call in the new context
                            evaluate_ast_with_visitor(right, &new_context, visitor)
                        }
                        _ => {
                            // Empty results can't have properties (only function calls are allowed)
                            Ok(FhirPathValue::Empty)
                        }
                    }
                }
                _ => {
                    // For primitive types (String, Integer, etc.), check if the right side is a function call
                    match **right {
                        AstNode::FunctionCall { .. } => {
                            // Create a new context with the left result as this_item for function calls
                            let new_context = EvaluationContext {
                                resource: context.resource.clone(),
                                context: context.context.clone(),
                                variables: context.variables.clone(),
                                this_item: Some(left_result),
                                index: None,
                                total: None,
                                optimization_enabled: context.optimization_enabled,
                                expression_cache: HashMap::new(),
                            };

                            // Evaluate the function call in the new context
                            evaluate_ast_with_visitor(right, &new_context, visitor)
                        }
                        _ => {
                            // Other types can't have properties (only function calls are allowed)
                            Ok(FhirPathValue::Empty)
                        }
                    }
                }
            }
        }

        AstNode::Indexer { collection, index } => {
            // Evaluate the collection
            let collection_result = evaluate_ast_with_visitor(collection, context, visitor)?;

            // Evaluate the index
            let index_result = evaluate_ast_with_visitor(index, context, visitor)?;

            // Get the item at the specified index
            match (collection_result, index_result) {
                (FhirPathValue::Collection(items), FhirPathValue::Integer(idx)) => {
                    if idx < 0 || idx as usize >= items.len() {
                        Ok(FhirPathValue::Empty)
                    } else {
                        Ok(items[idx as usize].clone())
                    }
                }
                _ => {
                    // Invalid indexing
                    Ok(FhirPathValue::Empty)
                }
            }
        }

        AstNode::FunctionCall { name, arguments } => {
            // Call the appropriate function
            evaluate_function_call(name, arguments, context, visitor)
        }

        AstNode::BinaryOp { op, left, right } => {
            // Evaluate the operands
            let left_result = evaluate_ast_with_visitor(left, context, visitor)?;
            let right_result = evaluate_ast_with_visitor(right, context, visitor)?;

            // Perform the operation
            match op {
                BinaryOperator::Equals => {
                    // FHIRPath equality: if either operand is empty, result is empty
                    match (&left_result, &right_result) {
                        (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Collection(items), _) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        (_, FhirPathValue::Collection(items)) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        _ => {
                            match compare_values_fhirpath(&left_result, &right_result) {
                                ComparisonResult::Equal => Ok(FhirPathValue::Boolean(true)),
                                ComparisonResult::NotEqual => Ok(FhirPathValue::Boolean(false)),
                                ComparisonResult::NotComparable => Ok(FhirPathValue::Empty),
                            }
                        }
                    }
                },
                BinaryOperator::NotEquals => {
                    // FHIRPath not-equality: if either operand is empty, result is empty
                    #[cfg(feature = "trace")]
                    println!("[DEBUG] NotEquals: left={:?}, right={:?}", left_result, right_result);

                    match (&left_result, &right_result) {
                        (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                            #[cfg(feature = "trace")]
                            println!("[DEBUG] NotEquals: One operand is Empty, returning Empty");
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Collection(items), _) if items.is_empty() => {
                            #[cfg(feature = "trace")]
                            println!("[DEBUG] NotEquals: Left operand is empty collection, returning Empty");
                            Ok(FhirPathValue::Empty)
                        }
                        (_, FhirPathValue::Collection(items)) if items.is_empty() => {
                            #[cfg(feature = "trace")]
                            println!("[DEBUG] NotEquals: Right operand is empty collection, returning Empty");
                            Ok(FhirPathValue::Empty)
                        }
                        _ => {
                            #[cfg(feature = "trace")]
                            println!("[DEBUG] NotEquals: Calling compare_values_fhirpath");
                            let comparison_result = compare_values_fhirpath(&left_result, &right_result);
                            #[cfg(feature = "trace")]
                            println!("[DEBUG] NotEquals: Comparison result = {:?}", comparison_result);

                            match comparison_result {
                                ComparisonResult::Equal => {
                                    #[cfg(feature = "trace")]
                                    println!("[DEBUG] NotEquals: Equal -> Boolean(false)");
                                    Ok(FhirPathValue::Boolean(false))
                                },
                                ComparisonResult::NotEqual => {
                                    #[cfg(feature = "trace")]
                                    println!("[DEBUG] NotEquals: NotEqual -> Boolean(true)");
                                    Ok(FhirPathValue::Boolean(true))
                                },
                                ComparisonResult::NotComparable => {
                                    #[cfg(feature = "trace")]
                                    println!("[DEBUG] NotEquals: NotComparable -> Empty");
                                    Ok(FhirPathValue::Empty)
                                },
                            }
                        }
                    }
                },
                BinaryOperator::Equivalent => Ok(FhirPathValue::Boolean(values_equivalent(
                    &left_result,
                    &right_result,
                ))),
                BinaryOperator::NotEquivalent => Ok(FhirPathValue::Boolean(!values_equivalent(
                    &left_result,
                    &right_result,
                ))),
                BinaryOperator::LessThan => {
                    compare_values(&left_result, &right_result, |a, b| a < b)
                }
                BinaryOperator::LessOrEqual => {
                    compare_values(&left_result, &right_result, |a, b| a <= b)
                }
                BinaryOperator::GreaterThan => {
                    compare_values(&left_result, &right_result, |a, b| a > b)
                }
                BinaryOperator::GreaterOrEqual => {
                    compare_values(&left_result, &right_result, |a, b| a >= b)
                }
                BinaryOperator::Addition => add_values(&left_result, &right_result),
                BinaryOperator::Subtraction => subtract_values(&left_result, &right_result),
                BinaryOperator::Multiplication => multiply_values(&left_result, &right_result),
                BinaryOperator::Division => divide_values(&left_result, &right_result),
                BinaryOperator::Mod => mod_values(&left_result, &right_result),
                BinaryOperator::And => {
                    // FHIRPath AND logic: if either operand is empty, result is empty
                    // AND operator should only work with boolean operands
                    match (&left_result, &right_result) {
                        (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Collection(items), _) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        (_, FhirPathValue::Collection(items)) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                            Ok(FhirPathValue::Boolean(*a && *b))
                        }
                        // For non-boolean operands, return Empty (FHIRPath spec)
                        _ => Ok(FhirPathValue::Empty)
                    }
                },
                BinaryOperator::Or => {
                    // FHIRPath OR logic:
                    // - true or anything = true
                    // - false or empty = empty
                    // - empty or false = empty
                    // - false or false = false
                    match (&left_result, &right_result) {
                        (FhirPathValue::Empty, FhirPathValue::Empty) => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Collection(items1), FhirPathValue::Collection(items2))
                            if items1.is_empty() && items2.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        // true or anything = true
                        (FhirPathValue::Boolean(true), _) => {
                            Ok(FhirPathValue::Boolean(true))
                        }
                        (_, FhirPathValue::Boolean(true)) => {
                            Ok(FhirPathValue::Boolean(true))
                        }
                        // false or empty = empty (FHIRPath spec)
                        (FhirPathValue::Boolean(false), FhirPathValue::Empty) => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Empty, FhirPathValue::Boolean(false)) => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Boolean(false), FhirPathValue::Collection(items)) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Collection(items), FhirPathValue::Boolean(false)) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        // false or false = false
                        (FhirPathValue::Boolean(false), FhirPathValue::Boolean(false)) => {
                            Ok(FhirPathValue::Boolean(false))
                        }
                        // For non-boolean operands, return Empty (FHIRPath spec)
                        _ => Ok(FhirPathValue::Empty)
                    }
                },
                BinaryOperator::Xor => {
                    // FHIRPath XOR logic: if either operand is empty, result is empty
                    match (&left_result, &right_result) {
                        (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Collection(items), _) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        (_, FhirPathValue::Collection(items)) if items.is_empty() => {
                            Ok(FhirPathValue::Empty)
                        }
                        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                            Ok(FhirPathValue::Boolean(*a ^ *b))
                        }
                        // For non-boolean operands, return Empty (FHIRPath spec)
                        _ => Ok(FhirPathValue::Empty)
                    }
                },
                BinaryOperator::Implies => {
                    // FHIRPath IMPLIES logic:
                    // - empty implies anything is true
                    // - anything implies empty depends on left operand
                    match (&left_result, &right_result) {
                        // Empty left operand: empty implies anything is true
                        (FhirPathValue::Empty, _) => Ok(FhirPathValue::Boolean(true)),
                        (FhirPathValue::Collection(items), _) if items.is_empty() => {
                            Ok(FhirPathValue::Boolean(true))
                        }
                        // Empty right operand: depends on left operand
                        (FhirPathValue::Boolean(a), FhirPathValue::Empty) => {
                            Ok(FhirPathValue::Boolean(!*a)) // true implies empty = false, false implies empty = true
                        }
                        (FhirPathValue::Boolean(a), FhirPathValue::Collection(items)) if items.is_empty() => {
                            Ok(FhirPathValue::Boolean(!*a)) // true implies empty = false, false implies empty = true
                        }
                        // Both boolean operands
                        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                            Ok(FhirPathValue::Boolean(!*a || *b))
                        }
                        // For non-boolean operands, return Empty (FHIRPath spec)
                        _ => Ok(FhirPathValue::Empty)
                    }
                },
                BinaryOperator::In => {
                    // 'in' operator checks if left operand is contained in right operand collection
                    match right_result {
                        FhirPathValue::Collection(items) => {
                            let found = items.iter().any(|item| values_equal(&left_result, item));
                            Ok(FhirPathValue::Boolean(found))
                        }
                        FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)),
                        other => {
                            // Single item on right side
                            Ok(FhirPathValue::Boolean(values_equal(&left_result, &other)))
                        }
                    }
                }
                BinaryOperator::Union => {
                    // Union operator combines two collections, removing duplicates
                    let mut result_items = Vec::new();

                    // Add items from left operand
                    match left_result {
                        FhirPathValue::Collection(items) => {
                            result_items.extend(items);
                        }
                        FhirPathValue::Empty => {
                            // Empty contributes nothing
                        }
                        other => {
                            result_items.push(other);
                        }
                    }

                    // Add items from right operand
                    match right_result {
                        FhirPathValue::Collection(items) => {
                            for item in items {
                                // Only add if not already present (remove duplicates)
                                if !result_items
                                    .iter()
                                    .any(|existing| values_equal(existing, &item))
                                {
                                    result_items.push(item);
                                }
                            }
                        }
                        FhirPathValue::Empty => {
                            // Empty contributes nothing
                        }
                        other => {
                            // Only add if not already present
                            if !result_items
                                .iter()
                                .any(|existing| values_equal(existing, &other))
                            {
                                result_items.push(other);
                            }
                        }
                    }

                    if result_items.is_empty() {
                        Ok(FhirPathValue::Empty)
                    } else {
                        Ok(FhirPathValue::Collection(result_items))
                    }
                }
                BinaryOperator::Div => {
                    // Integer division - convert decimals to integers first
                    let a_int = match left_result {
                        FhirPathValue::Integer(a) => a,
                        FhirPathValue::Decimal(a) => a.trunc() as i64,
                        _ => return Err(FhirPathError::TypeError(
                            "'div' operator requires numeric operands".to_string(),
                        )),
                    };
                    let b_int = match right_result {
                        FhirPathValue::Integer(b) => b,
                        FhirPathValue::Decimal(b) => b.trunc() as i64,
                        _ => return Err(FhirPathError::TypeError(
                            "'div' operator requires numeric operands".to_string(),
                        )),
                    };

                    if b_int == 0 {
                        Ok(FhirPathValue::Empty)
                    } else {
                        Ok(FhirPathValue::Integer(a_int / b_int))
                    }
                }
                BinaryOperator::Contains => {
                    // 'contains' operator checks if left operand collection contains right operand
                    match left_result {
                        FhirPathValue::Collection(items) => {
                            let found = items.iter().any(|item| values_equal(item, &right_result));
                            Ok(FhirPathValue::Boolean(found))
                        }
                        FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)),
                        other => {
                            // Single item on left side
                            Ok(FhirPathValue::Boolean(values_equal(&other, &right_result)))
                        }
                    }
                }
                BinaryOperator::Is => {
                    // 'is' operator checks if left operand is of the type specified by right operand
                    let type_name = match right_result {
                        FhirPathValue::String(ref type_str) => type_str.clone(),
                        _ => {
                            // If right operand is not a string, check if the right side is an identifier
                            // by looking at the original AST node
                            match **right {
                                AstNode::Identifier(ref identifier_name) => {
                                    // Handle qualified identifiers (e.g., FHIR.Patient -> Patient)
                                    if let Some(last_part) = identifier_name.split('.').last() {
                                        last_part.to_string()
                                    } else {
                                        identifier_name.clone()
                                    }
                                }
                                _ => {
                                    return Ok(FhirPathValue::Boolean(false));
                                }
                            }
                        }
                    };

                    let actual_type = get_fhirpath_type_name(&left_result);
                    Ok(FhirPathValue::Boolean(actual_type == type_name))
                }
                BinaryOperator::As => {
                    // 'as' operator casts left operand to the type specified by right operand
                    // For now, return the left operand unchanged
                    Ok(left_result)
                }
                BinaryOperator::Concatenation => {
                    // Concatenation operator (&) converts operands to strings and concatenates them
                    let left_str = match left_result {
                        FhirPathValue::String(s) => s,
                        FhirPathValue::Integer(i) => i.to_string(),
                        FhirPathValue::Decimal(d) => d.to_string(),
                        FhirPathValue::Boolean(b) => b.to_string(),
                        FhirPathValue::Empty => String::new(),
                        FhirPathValue::Collection(ref items) if items.is_empty() => String::new(),
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "Cannot convert left operand to string for concatenation"
                                    .to_string(),
                            ))
                        }
                    };

                    let right_str = match right_result {
                        FhirPathValue::String(s) => s,
                        FhirPathValue::Integer(i) => i.to_string(),
                        FhirPathValue::Decimal(d) => d.to_string(),
                        FhirPathValue::Boolean(b) => b.to_string(),
                        FhirPathValue::Empty => String::new(),
                        FhirPathValue::Collection(ref items) if items.is_empty() => String::new(),
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "Cannot convert right operand to string for concatenation"
                                    .to_string(),
                            ))
                        }
                    };

                    Ok(FhirPathValue::String(format!("{}{}", left_str, right_str)))
                }
            }
        }

        AstNode::UnaryOp { op, operand } => {
            // Evaluate the operand
            let operand_result = evaluate_ast_with_visitor(operand, context, visitor)?;

            // Perform the operation
            match op {
                UnaryOperator::Positive => match operand_result {
                    FhirPathValue::Integer(value) => Ok(FhirPathValue::Integer(value)),
                    FhirPathValue::Decimal(value) => Ok(FhirPathValue::Decimal(value)),
                    _ => Err(FhirPathError::TypeError(
                        "Positive operator requires numeric operand".to_string(),
                    )),
                },
                UnaryOperator::Negate => match operand_result {
                    FhirPathValue::Integer(value) => Ok(FhirPathValue::Integer(-value)),
                    FhirPathValue::Decimal(value) => Ok(FhirPathValue::Decimal(-value)),
                    _ => Err(FhirPathError::TypeError(
                        "Negation requires numeric operand".to_string(),
                    )),
                },
                UnaryOperator::Not => match operand_result {
                    FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
                    FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
                    FhirPathValue::Collection(ref items) if items.is_empty() => {
                        Ok(FhirPathValue::Boolean(true))
                    }
                    _ => Ok(FhirPathValue::Boolean(false)),
                },
            }
        }

        AstNode::QuantityLiteral { value, unit } => Ok(FhirPathValue::Quantity {
            value: *value,
            unit: unit.clone().unwrap_or_default(),
        }),
    }
}

/// Evaluates a FHIRPath expression string
pub fn evaluate_expression(
    expression: &str,
    resource: serde_json::Value,
) -> Result<FhirPathValue, FhirPathError> {
    evaluate_expression_with_visitor(expression, resource, &NoopVisitor::new())
}

/// Evaluates a FHIRPath expression string with optimization enabled
pub fn evaluate_expression_optimized(
    expression: &str,
    resource: serde_json::Value,
) -> Result<FhirPathValue, FhirPathError> {
    let tokens = tokenize(expression)?;
    let ast = parse(&tokens)?;
    let optimized_ast = optimize_ast(&ast);
    let mut context = EvaluationContext::new_with_optimization(resource, true);
    let visitor = NoopVisitor::new();
    evaluate_ast_with_caching(&optimized_ast, &mut context, &visitor)
}

/// Optimizes an AST by applying various optimization techniques
fn optimize_ast(node: &AstNode) -> AstNode {
    match node {
        // Constant folding for binary operations
        AstNode::BinaryOp { op, left, right } => {
            let optimized_left = optimize_ast(left);
            let optimized_right = optimize_ast(right);

            // Try to fold constants
            match (&optimized_left, &optimized_right) {
                (AstNode::BooleanLiteral(left_val), AstNode::BooleanLiteral(right_val)) => match op
                {
                    BinaryOperator::And => AstNode::BooleanLiteral(*left_val && *right_val),
                    BinaryOperator::Or => AstNode::BooleanLiteral(*left_val || *right_val),
                    BinaryOperator::Equals => AstNode::BooleanLiteral(*left_val == *right_val),
                    BinaryOperator::NotEquals => AstNode::BooleanLiteral(*left_val != *right_val),
                    _ => AstNode::BinaryOp {
                        op: op.clone(),
                        left: Box::new(optimized_left),
                        right: Box::new(optimized_right),
                    },
                },
                (AstNode::NumberLiteral { value: left_val, is_decimal: left_decimal }, AstNode::NumberLiteral { value: right_val, is_decimal: right_decimal }) => match op {
                    BinaryOperator::Addition => AstNode::NumberLiteral {
                        value: left_val + right_val,
                        is_decimal: *left_decimal || *right_decimal
                    },
                    BinaryOperator::Subtraction => AstNode::NumberLiteral {
                        value: left_val - right_val,
                        is_decimal: *left_decimal || *right_decimal
                    },
                    BinaryOperator::Multiplication => AstNode::NumberLiteral {
                        value: left_val * right_val,
                        is_decimal: *left_decimal || *right_decimal
                    },
                    BinaryOperator::Division => {
                        if *right_val != 0.0 {
                            AstNode::NumberLiteral {
                                value: left_val / right_val,
                                is_decimal: true  // Division always results in decimal
                            }
                        } else {
                            AstNode::BinaryOp {
                                op: op.clone(),
                                left: Box::new(optimized_left),
                                right: Box::new(optimized_right),
                            }
                        }
                    }
                    BinaryOperator::Equals => {
                        AstNode::BooleanLiteral((left_val - right_val).abs() < f64::EPSILON)
                    }
                    BinaryOperator::NotEquals => {
                        AstNode::BooleanLiteral((left_val - right_val).abs() >= f64::EPSILON)
                    }
                    BinaryOperator::LessThan => AstNode::BooleanLiteral(left_val < right_val),
                    BinaryOperator::LessOrEqual => AstNode::BooleanLiteral(left_val <= right_val),
                    BinaryOperator::GreaterThan => AstNode::BooleanLiteral(left_val > right_val),
                    BinaryOperator::GreaterOrEqual => {
                        AstNode::BooleanLiteral(left_val >= right_val)
                    }
                    _ => AstNode::BinaryOp {
                        op: op.clone(),
                        left: Box::new(optimized_left),
                        right: Box::new(optimized_right),
                    },
                },
                (AstNode::StringLiteral(left_val), AstNode::StringLiteral(right_val)) => match op {
                    BinaryOperator::Equals => AstNode::BooleanLiteral(left_val == right_val),
                    BinaryOperator::NotEquals => AstNode::BooleanLiteral(left_val != right_val),
                    BinaryOperator::Addition => {
                        AstNode::StringLiteral(format!("{}{}", left_val, right_val))
                    }
                    _ => AstNode::BinaryOp {
                        op: op.clone(),
                        left: Box::new(optimized_left),
                        right: Box::new(optimized_right),
                    },
                },
                // Short-circuit optimization for boolean operations
                (AstNode::BooleanLiteral(true), _) if matches!(op, BinaryOperator::Or) => {
                    AstNode::BooleanLiteral(true)
                }
                (AstNode::BooleanLiteral(false), _) if matches!(op, BinaryOperator::And) => {
                    AstNode::BooleanLiteral(false)
                }
                (_, AstNode::BooleanLiteral(true)) if matches!(op, BinaryOperator::Or) => {
                    AstNode::BooleanLiteral(true)
                }
                (_, AstNode::BooleanLiteral(false)) if matches!(op, BinaryOperator::And) => {
                    AstNode::BooleanLiteral(false)
                }
                _ => AstNode::BinaryOp {
                    op: op.clone(),
                    left: Box::new(optimized_left),
                    right: Box::new(optimized_right),
                },
            }
        }

        // Optimize unary operations
        AstNode::UnaryOp { op, operand } => {
            let optimized_operand = optimize_ast(operand);
            match (&optimized_operand, op) {
                (AstNode::BooleanLiteral(val), UnaryOperator::Not) => AstNode::BooleanLiteral(!val),
                (AstNode::NumberLiteral { value, is_decimal }, UnaryOperator::Negate) => {
                    AstNode::NumberLiteral { value: -value, is_decimal: *is_decimal }
                }
                _ => AstNode::UnaryOp {
                    op: op.clone(),
                    operand: Box::new(optimized_operand),
                },
            }
        }

        // Recursively optimize path expressions
        AstNode::Path(left, right) => {
            AstNode::Path(Box::new(optimize_ast(left)), Box::new(optimize_ast(right)))
        }

        // Optimize function calls
        AstNode::FunctionCall { name, arguments } => {
            let optimized_args: Vec<AstNode> = arguments.iter().map(optimize_ast).collect();
            AstNode::FunctionCall {
                name: name.clone(),
                arguments: optimized_args,
            }
        }

        // Optimize indexing
        AstNode::Indexer { collection, index } => AstNode::Indexer {
            collection: Box::new(optimize_ast(collection)),
            index: Box::new(optimize_ast(index)),
        },

        // Literals and identifiers don't need optimization
        _ => node.clone(),
    }
}

/// Evaluates a FHIRPath expression string with a custom visitor
pub fn evaluate_expression_with_visitor(
    expression: &str,
    resource: serde_json::Value,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    #[cfg(feature = "trace")]
    debug!("Evaluating FHIRPath expression: {}", expression);

    // Create a context
    let context = EvaluationContext::new(resource);

    // Tokenize and parse the expression
    #[cfg(feature = "trace")]
    trace!("Tokenizing expression");
    let tokens = tokenize(expression)?;

    #[cfg(feature = "trace")]
    trace!("Parsing tokens into AST");
    let ast = parse(&tokens)?;

    #[cfg(feature = "trace")]
    trace!("Starting AST evaluation");

    // Evaluate the AST with the provided visitor
    let result = evaluate_ast_with_visitor(&ast, &context, visitor)?;

    #[cfg(feature = "trace")]
    debug!("Expression evaluation result: {:?}", result);

    // Return result as-is - Empty should remain Empty, not be converted to empty collection
    let wrapped_result = result;

    Ok(wrapped_result)
}

/// Evaluates a FHIRPath expression string using streaming mode for large resources
pub fn evaluate_expression_streaming<R: Read>(
    expression: &str,
    reader: R,
) -> Result<FhirPathValue, FhirPathError> {
    evaluate_expression_streaming_with_visitor(expression, reader, &NoopVisitor::new())
}

/// Evaluates a FHIRPath expression string using streaming mode with a custom visitor
/// This implementation uses streaming JSON parsing to handle large resources efficiently
pub fn evaluate_expression_streaming_with_visitor<R: Read>(
    expression: &str,
    mut reader: R,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    #[cfg(feature = "trace")]
    debug!(
        "Evaluating FHIRPath expression with streaming: {}",
        expression
    );

    // Tokenize and parse the expression first to understand what we need
    #[cfg(feature = "trace")]
    trace!("Tokenizing expression");
    let tokens = tokenize(expression)?;

    #[cfg(feature = "trace")]
    trace!("Parsing tokens into AST");
    let ast = parse(&tokens)?;

    // For simple expressions that don't require the full resource, we can optimize
    // For now, we still deserialize the full resource but with better memory management
    let mut deserializer = serde_json::Deserializer::from_reader(&mut reader);

    // Use streaming deserialization with memory-conscious approach
    let resource: serde_json::Value = match serde_json::Value::deserialize(&mut deserializer) {
        Ok(value) => value,
        Err(e) => {
            #[cfg(feature = "trace")]
            debug!("JSON deserialization error: {:?}", e);
            return Err(FhirPathError::ParserError(format!("Invalid JSON: {}", e)));
        }
    };

    // Create a context with memory optimization hints
    let context = EvaluationContext::new_with_optimization(resource, true);

    #[cfg(feature = "trace")]
    trace!("Starting AST evaluation with streaming optimizations");

    // Evaluate the AST with the provided visitor
    let result = evaluate_ast_with_visitor(&ast, &context, visitor)?;

    #[cfg(feature = "trace")]
    debug!("Expression evaluation result: {:?}", result);

    // Return result as-is - Empty should remain Empty, not be converted to empty collection
    let wrapped_result = result;

    Ok(wrapped_result)
}

/// Helper function to convert a JSON value to a FHIRPath value
fn json_to_fhirpath_value(value: serde_json::Value) -> Result<FhirPathValue, FhirPathError> {
    match value {
        serde_json::Value::Null => Ok(FhirPathValue::Empty),
        serde_json::Value::Bool(b) => Ok(FhirPathValue::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(FhirPathValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(FhirPathValue::Decimal(f))
            } else {
                Err(FhirPathError::TypeError("Invalid number".to_string()))
            }
        }
        serde_json::Value::String(s) => Ok(FhirPathValue::String(s)),
        serde_json::Value::Array(arr) => {
            let mut items = Vec::new();
            for item in arr {
                items.push(json_to_fhirpath_value(item)?);
            }
            Ok(FhirPathValue::Collection(items))
        }
        serde_json::Value::Object(obj) => {
            // Check if it's a FHIR resource
            if obj.contains_key("resourceType") {
                let resource = FhirResource::from_json(serde_json::Value::Object(obj))?;
                Ok(FhirPathValue::Resource(resource))
            } else if obj.contains_key("value") && obj.contains_key("unit") {
                // This looks like a FHIR Quantity object
                let value = if let Some(v) = obj.get("value") {
                    // Try to get as f64 first
                    if let Some(f) = v.as_f64() {
                        f
                    } else if let Some(s) = v.as_str() {
                        // If it's a string, try to parse it as a number
                        s.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let unit = obj
                    .get("unit")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(FhirPathValue::Quantity { value, unit })
            } else if obj.contains_key("value") && obj.len() <= 2 {
                // This looks like a FHIR primitive type with a "value" property
                // Extract the actual value instead of wrapping as a Resource
                if let Some(value) = obj.get("value") {
                    json_to_fhirpath_value(value.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            } else {
                // Convert to a resource without a resourceType
                let resource = FhirResource {
                    resource_type: None,
                    properties: obj.into_iter().collect(),
                };
                Ok(FhirPathValue::Resource(resource))
            }
        }
    }
}

/// Helper function for comparison operations
fn compare_values<F>(
    left: &FhirPathValue,
    right: &FhirPathValue,
    compare_fn: F,
) -> Result<FhirPathValue, FhirPathError>
where
    F: Fn(f64, f64) -> bool,
{
    // Call the internal helper with initial depth of 0
    compare_values_internal(left, right, compare_fn, 0)
}

/// Internal helper function for comparison operations with recursion depth tracking
fn compare_values_internal<F>(
    left: &FhirPathValue,
    right: &FhirPathValue,
    compare_fn: F,
    depth: usize,
) -> Result<FhirPathValue, FhirPathError>
where
    F: Fn(f64, f64) -> bool,
{
    // Prevent infinite recursion by limiting depth
    if depth > 100 {
        return Err(FhirPathError::EvaluationError(
            "Maximum recursion depth exceeded during comparison".to_string(),
        ));
    }

    match (left, right) {
        // Numeric comparisons
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
            Ok(FhirPathValue::Boolean(compare_fn(*a as f64, *b as f64)))
        }
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            Ok(FhirPathValue::Boolean(compare_fn(*a as f64, *b)))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            Ok(FhirPathValue::Boolean(compare_fn(*a, *b as f64)))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
            Ok(FhirPathValue::Boolean(compare_fn(*a, *b)))
        }

        // String comparisons
        (FhirPathValue::String(a), FhirPathValue::String(b)) => {
            // String comparison using proper ordering
            use std::cmp::Ordering;
            let result = match a.cmp(b) {
                Ordering::Less => compare_fn(-1.0, 0.0),
                Ordering::Equal => compare_fn(0.0, 0.0),
                Ordering::Greater => compare_fn(1.0, 0.0),
            };
            Ok(FhirPathValue::Boolean(result))
        }

        // Boolean comparisons
        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
            // Convert booleans to 0.0 and 1.0 for comparison
            let a_val = if *a { 1.0 } else { 0.0 };
            let b_val = if *b { 1.0 } else { 0.0 };
            Ok(FhirPathValue::Boolean(compare_fn(a_val, b_val)))
        }

        // DateTime comparisons
        (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
            // Normalize both datetimes and compare them lexicographically
            let normalized_a = normalize_datetime(a);
            let normalized_b = normalize_datetime(b);
            Ok(FhirPathValue::Boolean(compare_fn(
                normalized_a.cmp(&normalized_b) as i32 as f64,
                0.0,
            )))
        }

        // Date comparisons
        (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
            // Normalize both dates and compare them lexicographically
            let normalized_a = normalize_datetime(a);
            let normalized_b = normalize_datetime(b);
            Ok(FhirPathValue::Boolean(compare_fn(
                normalized_a.cmp(&normalized_b) as i32 as f64,
                0.0,
            )))
        }

        // Time comparisons
        (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
            // Normalize both times and compare them lexicographically
            let normalized_a = normalize_time(a);
            let normalized_b = normalize_time(b);
            Ok(FhirPathValue::Boolean(compare_fn(
                normalized_a.cmp(&normalized_b) as i32 as f64,
                0.0,
            )))
        }

        // Date to DateTime comparisons
        (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
            // Convert date to datetime by adding T00:00:00 for comparison
            let a_as_datetime = if a.contains('T') {
                a.clone()
            } else {
                format!("{}T00:00:00", a)
            };
            let normalized_a = normalize_datetime(&a_as_datetime);
            let normalized_b = normalize_datetime(b);
            Ok(FhirPathValue::Boolean(compare_fn(
                normalized_a.cmp(&normalized_b) as i32 as f64,
                0.0,
            )))
        }
        (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
            // Convert date to datetime by adding T00:00:00 for comparison
            let b_as_datetime = if b.contains('T') {
                b.clone()
            } else {
                format!("{}T00:00:00", b)
            };
            let normalized_a = normalize_datetime(a);
            let normalized_b = normalize_datetime(&b_as_datetime);
            Ok(FhirPathValue::Boolean(compare_fn(
                normalized_a.cmp(&normalized_b) as i32 as f64,
                0.0,
            )))
        }

        // String to Date/DateTime comparisons (for FHIR primitive values)
        (FhirPathValue::String(a), FhirPathValue::Date(b)) => {
            // Try to parse string as date and compare
            if is_valid_datetime_string(a) {
                let normalized_a = normalize_datetime(a);
                let normalized_b = normalize_datetime(b);
                Ok(FhirPathValue::Boolean(compare_fn(
                    normalized_a.cmp(&normalized_b) as i32 as f64,
                    0.0,
                )))
            } else {
                Err(FhirPathError::TypeError(format!(
                    "Cannot compare string '{}' with date '{}'",
                    a, b
                )))
            }
        }
        (FhirPathValue::Date(a), FhirPathValue::String(b)) => {
            // Try to parse string as date and compare
            if is_valid_datetime_string(b) {
                let normalized_a = normalize_datetime(a);
                let normalized_b = normalize_datetime(b);
                Ok(FhirPathValue::Boolean(compare_fn(
                    normalized_a.cmp(&normalized_b) as i32 as f64,
                    0.0,
                )))
            } else {
                Err(FhirPathError::TypeError(format!(
                    "Cannot compare date '{}' with string '{}'",
                    a, b
                )))
            }
        }
        (FhirPathValue::String(a), FhirPathValue::DateTime(b)) => {
            // Try to parse string as datetime and compare
            if is_valid_datetime_string(a) {
                let normalized_a = normalize_datetime(a);
                let normalized_b = normalize_datetime(b);
                Ok(FhirPathValue::Boolean(compare_fn(
                    normalized_a.cmp(&normalized_b) as i32 as f64,
                    0.0,
                )))
            } else {
                Err(FhirPathError::TypeError(format!(
                    "Cannot compare string '{}' with datetime '{}'",
                    a, b
                )))
            }
        }
        (FhirPathValue::DateTime(a), FhirPathValue::String(b)) => {
            // Try to parse string as datetime and compare
            if is_valid_datetime_string(b) {
                let normalized_a = normalize_datetime(a);
                let normalized_b = normalize_datetime(b);
                Ok(FhirPathValue::Boolean(compare_fn(
                    normalized_a.cmp(&normalized_b) as i32 as f64,
                    0.0,
                )))
            } else {
                Err(FhirPathError::TypeError(format!(
                    "Cannot compare datetime '{}' with string '{}'",
                    a, b
                )))
            }
        }

        // Quantity comparisons
        (
            FhirPathValue::Quantity {
                value: v1,
                unit: u1,
            },
            FhirPathValue::Quantity {
                value: v2,
                unit: u2,
            },
        ) => {
            // For now, only compare quantities with the same unit
            if u1 == u2 {
                Ok(FhirPathValue::Boolean(compare_fn(*v1, *v2)))
            } else {
                Err(FhirPathError::TypeError(
                    "Cannot compare quantities with different units".to_string(),
                ))
            }
        }

        // Collection comparisons
        (FhirPathValue::Collection(items1), FhirPathValue::Collection(items2)) => {
            // If both collections are empty, they're equal
            if items1.is_empty() && items2.is_empty() {
                return Ok(FhirPathValue::Boolean(compare_fn(0.0, 0.0)));
            }

            // If one collection is empty and the other is not, they're not equal
            if items1.is_empty() || items2.is_empty() {
                return Ok(FhirPathValue::Boolean(compare_fn(
                    if items1.is_empty() { -1.0 } else { 1.0 },
                    0.0,
                )));
            }

            // For collections with different lengths, compare the lengths
            if items1.len() != items2.len() {
                return Ok(FhirPathValue::Boolean(compare_fn(
                    items1.len() as f64,
                    items2.len() as f64,
                )));
            }

            // For collections with the same length, compare items one by one without recursion
            // This is a non-recursive approach to avoid stack overflow
            for (i, (item1, item2)) in items1.iter().zip(items2.iter()).enumerate() {
                // Direct comparison based on value types without recursion
                let items_equal = match (item1, item2) {
                    // Simple primitive type comparisons
                    (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a == b,
                    (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a == b,
                    (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a == b,
                    (FhirPathValue::String(a), FhirPathValue::String(b)) => a == b,
                    (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a == b,
                    (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a == b,
                    (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a == b,

                    // Mixed numeric comparisons
                    (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => *a as f64 == *b,
                    (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => *a == *b as f64,

                    // Quantity comparisons
                    (
                        FhirPathValue::Quantity {
                            value: v1,
                            unit: u1,
                        },
                        FhirPathValue::Quantity {
                            value: v2,
                            unit: u2,
                        },
                    ) => u1 == u2 && v1 == v2,

                    // For nested collections, we can't do a deep comparison without recursion
                    // So we'll just compare if they're both collections with the same length
                    (FhirPathValue::Collection(c1), FhirPathValue::Collection(c2)) => {
                        c1.len() == c2.len()
                    }

                    // For resources, compare their JSON representations
                    (FhirPathValue::Resource(r1), FhirPathValue::Resource(r2)) => {
                        r1.to_json() == r2.to_json()
                    }

                    // Different types are not equal
                    _ => false,
                };

                if !items_equal {
                    return Ok(FhirPathValue::Boolean(compare_fn(1.0, 0.0)));
                }
            }

            // If all items are equal, the collections are equal
            Ok(FhirPathValue::Boolean(compare_fn(0.0, 0.0)))
        }

        // String to number conversions for comparison
        (FhirPathValue::String(s), FhirPathValue::Integer(i)) => {
            if let Ok(s_as_num) = s.parse::<f64>() {
                Ok(FhirPathValue::Boolean(compare_fn(s_as_num, *i as f64)))
            } else {
                Err(FhirPathError::TypeError(
                    "Cannot compare string to number".to_string(),
                ))
            }
        }
        (FhirPathValue::Integer(i), FhirPathValue::String(s)) => {
            if let Ok(s_as_num) = s.parse::<f64>() {
                Ok(FhirPathValue::Boolean(compare_fn(*i as f64, s_as_num)))
            } else {
                Err(FhirPathError::TypeError(
                    "Cannot compare number to string".to_string(),
                ))
            }
        }
        (FhirPathValue::String(s), FhirPathValue::Decimal(d)) => {
            if let Ok(s_as_num) = s.parse::<f64>() {
                Ok(FhirPathValue::Boolean(compare_fn(s_as_num, *d)))
            } else {
                Err(FhirPathError::TypeError(
                    "Cannot compare string to decimal".to_string(),
                ))
            }
        }
        (FhirPathValue::Decimal(d), FhirPathValue::String(s)) => {
            if let Ok(s_as_num) = s.parse::<f64>() {
                Ok(FhirPathValue::Boolean(compare_fn(*d, s_as_num)))
            } else {
                Err(FhirPathError::TypeError(
                    "Cannot compare decimal to string".to_string(),
                ))
            }
        }

        // Empty value comparisons
        (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
            // In FHIRPath, comparisons involving empty values return empty
            Ok(FhirPathValue::Empty)
        }

        // Single value vs collection comparisons
        (single_value, FhirPathValue::Collection(items)) => {
            // Check if the single value compares with any item in the collection
            for item in items {
                // Use direct comparison logic to avoid recursion issues
                let comparison_result = match (single_value, item) {
                    // Direct numeric comparisons
                    (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                        compare_fn(*a as f64, *b as f64)
                    }
                    (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                        compare_fn(*a as f64, *b)
                    }
                    (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                        compare_fn(*a, *b as f64)
                    }
                    (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => compare_fn(*a, *b),
                    // String comparisons
                    (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                        use std::cmp::Ordering;
                        match a.cmp(b) {
                            Ordering::Less => compare_fn(-1.0, 0.0),
                            Ordering::Equal => compare_fn(0.0, 0.0),
                            Ordering::Greater => compare_fn(1.0, 0.0),
                        }
                    }
                    // Boolean comparisons
                    (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                        let a_val = if *a { 1.0 } else { 0.0 };
                        let b_val = if *b { 1.0 } else { 0.0 };
                        compare_fn(a_val, b_val)
                    }
                    // Date/DateTime comparisons
                    (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                        let normalized_a = normalize_datetime(a);
                        let normalized_b = normalize_datetime(b);
                        compare_fn(normalized_a.cmp(&normalized_b) as i32 as f64, 0.0)
                    }
                    (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                        let normalized_a = normalize_datetime(a);
                        let normalized_b = normalize_datetime(b);
                        compare_fn(normalized_a.cmp(&normalized_b) as i32 as f64, 0.0)
                    }
                    // Skip other types for now
                    _ => false,
                };

                if comparison_result {
                    return Ok(FhirPathValue::Boolean(true));
                }
            }
            // If no item matched, return false
            Ok(FhirPathValue::Boolean(false))
        }
        (FhirPathValue::Collection(items), single_value) => {
            // Check if any item in the collection compares with the single value
            for item in items {
                // Use direct comparison logic to avoid recursion issues
                let comparison_result = match (item, single_value) {
                    // Direct numeric comparisons
                    (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                        compare_fn(*a as f64, *b as f64)
                    }
                    (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                        compare_fn(*a as f64, *b)
                    }
                    (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                        compare_fn(*a, *b as f64)
                    }
                    (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => compare_fn(*a, *b),
                    // String comparisons
                    (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                        use std::cmp::Ordering;
                        match a.cmp(b) {
                            Ordering::Less => compare_fn(-1.0, 0.0),
                            Ordering::Equal => compare_fn(0.0, 0.0),
                            Ordering::Greater => compare_fn(1.0, 0.0),
                        }
                    }
                    // Boolean comparisons
                    (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                        let a_val = if *a { 1.0 } else { 0.0 };
                        let b_val = if *b { 1.0 } else { 0.0 };
                        compare_fn(a_val, b_val)
                    }
                    // Date/DateTime comparisons
                    (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                        let normalized_a = normalize_datetime(a);
                        let normalized_b = normalize_datetime(b);
                        compare_fn(normalized_a.cmp(&normalized_b) as i32 as f64, 0.0)
                    }
                    (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                        let normalized_a = normalize_datetime(a);
                        let normalized_b = normalize_datetime(b);
                        compare_fn(normalized_a.cmp(&normalized_b) as i32 as f64, 0.0)
                    }
                    // Skip other types for now
                    _ => false,
                };

                if comparison_result {
                    return Ok(FhirPathValue::Boolean(true));
                }
            }
            // If no item matched, return false
            Ok(FhirPathValue::Boolean(false))
        }

        // Fallback for incompatible types
        _ => Err(FhirPathError::TypeError(format!(
            "Comparison requires compatible operands: {:?} and {:?}",
            left, right
        ))),
    }
}

/// Helper function for addition
fn add_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue, FhirPathError> {
    match (left, right) {
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(FhirPathValue::Integer(a + b)),
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            Ok(FhirPathValue::Decimal(*a as f64 + b))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            Ok(FhirPathValue::Decimal(a + *b as f64))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(FhirPathValue::Decimal(a + b)),
        (FhirPathValue::String(a), FhirPathValue::String(b)) => {
            // String concatenation
            Ok(FhirPathValue::String(format!("{}{}", a, b)))
        }
        (FhirPathValue::Collection(a), FhirPathValue::Collection(b)) => {
            // Collection union
            let mut result = a.clone();
            result.extend(b.clone());
            Ok(FhirPathValue::Collection(result))
        }
        _ => Err(FhirPathError::TypeError(
            "Addition requires compatible operands".to_string(),
        )),
    }
}

/// Helper function for subtraction
fn subtract_values(
    left: &FhirPathValue,
    right: &FhirPathValue,
) -> Result<FhirPathValue, FhirPathError> {
    match (left, right) {
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(FhirPathValue::Integer(a - b)),
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            Ok(FhirPathValue::Decimal(*a as f64 - b))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            Ok(FhirPathValue::Decimal(a - *b as f64))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(FhirPathValue::Decimal(a - b)),
        _ => Err(FhirPathError::TypeError(
            "Subtraction requires numeric operands".to_string(),
        )),
    }
}

/// Helper function for multiplication
fn multiply_values(
    left: &FhirPathValue,
    right: &FhirPathValue,
) -> Result<FhirPathValue, FhirPathError> {
    match (left, right) {
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(FhirPathValue::Integer(a * b)),
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            Ok(FhirPathValue::Decimal(*a as f64 * b))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            Ok(FhirPathValue::Decimal(a * *b as f64))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(FhirPathValue::Decimal(a * b)),
        _ => Err(FhirPathError::TypeError(
            "Multiplication requires numeric operands".to_string(),
        )),
    }
}

/// Helper function for division
fn divide_values(
    left: &FhirPathValue,
    right: &FhirPathValue,
) -> Result<FhirPathValue, FhirPathError> {
    match (left, right) {
        (_, FhirPathValue::Integer(b)) if *b == 0 => Ok(FhirPathValue::Empty),
        (_, FhirPathValue::Decimal(b)) if *b == 0.0 => Ok(FhirPathValue::Empty),
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
            // Integer division results in a decimal
            Ok(FhirPathValue::Decimal(*a as f64 / *b as f64))
        }
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            Ok(FhirPathValue::Decimal(*a as f64 / b))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            Ok(FhirPathValue::Decimal(a / *b as f64))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(FhirPathValue::Decimal(a / b)),
        _ => Err(FhirPathError::TypeError(
            "Division requires numeric operands".to_string(),
        )),
    }
}

/// Helper function for modulo operation
fn mod_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue, FhirPathError> {
    match (left, right) {
        (_, FhirPathValue::Integer(b)) if *b == 0 => Ok(FhirPathValue::Empty),
        (_, FhirPathValue::Decimal(b)) if *b == 0.0 => Ok(FhirPathValue::Empty),
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(FhirPathValue::Integer(a % b)),
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            Ok(FhirPathValue::Decimal((*a as f64) % b))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            Ok(FhirPathValue::Decimal(a % (*b as f64)))
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(FhirPathValue::Decimal(a % b)),
        _ => Err(FhirPathError::TypeError(
            "Modulo requires numeric operands".to_string(),
        )),
    }
}

/// Evaluates a function call with proper argument handling
fn evaluate_function_call(
    name: &str,
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if name.contains("converts") {
        println!("[DEBUG] Function call: {}", name);
    }
    match name {
        // Collection filtering and projection functions
        "where" => evaluate_where_function(arguments, context, visitor),
        "select" => evaluate_select_function(arguments, context, visitor),

        // Collection navigation functions
        "first" => evaluate_first_function(arguments, context),
        "last" => evaluate_last_function(arguments, context),
        "tail" => evaluate_tail_function(arguments, context),
        "skip" => evaluate_skip_function(arguments, context, visitor),
        "take" => evaluate_take_function(arguments, context),

        // Collection testing functions
        "exists" => evaluate_exists_function(arguments, context),
        "empty" => evaluate_empty_function(arguments, context),
        "count" => evaluate_count_function(arguments, context),
        "length" => evaluate_length_function(arguments, context),

        // Collection aggregation functions
        "distinct" => evaluate_distinct_function(arguments, context),
        "isDistinct" => evaluate_is_distinct_function(arguments, context),
        "union" => evaluate_union_function(arguments, context),
        "combine" => evaluate_combine_function(arguments, context),
        "intersect" => evaluate_intersect_function(arguments, context),
        "exclude" => evaluate_exclude_function(arguments, context, visitor),
        "subsetOf" => evaluate_subset_of_function(arguments, context, visitor),
        "supersetOf" => evaluate_superset_of_function(arguments, context, visitor),
        "single" => evaluate_single_function(arguments, context),
        "sort" => evaluate_sort_function(arguments, context, visitor),

        // Tree navigation functions
        "descendants" => evaluate_descendants_function(arguments, context),

        // Debugging functions
        "trace" => evaluate_trace_function(arguments, context, visitor),

        // Aggregation functions
        "aggregate" => evaluate_aggregate_function(arguments, context, visitor),

        // Type checking functions
        "is" => evaluate_is_function(arguments, context),
        "as" => evaluate_as_function(arguments, context),

        // String functions
        "contains" => evaluate_contains_function(arguments, context),
        "startsWith" => evaluate_starts_with_function(arguments, context),
        "endsWith" => evaluate_ends_with_function(arguments, context),
        "substring" => evaluate_substring_function(arguments, context, visitor),
        "indexOf" => evaluate_index_of_function(arguments, context, visitor),
        "replace" => evaluate_replace_function(arguments, context, visitor),
        "matches" => evaluate_matches_function(arguments, context, visitor),
        "split" => evaluate_split_function(arguments, context, visitor),
        "join" => evaluate_join_function(arguments, context, visitor),
        "toChars" => evaluate_to_chars_function(arguments, context, visitor),
        "escape" => evaluate_escape_function(arguments, context, visitor),
        "unescape" => evaluate_unescape_function(arguments, context, visitor),
        "upper" => evaluate_upper_function(arguments, context, visitor),
        "lower" => evaluate_lower_function(arguments, context, visitor),

        // Math functions
        "abs" => evaluate_abs_function(arguments, context, visitor),
        "ceiling" => evaluate_ceiling_function(arguments, context, visitor),
        "floor" => evaluate_floor_function(arguments, context, visitor),
        "round" => evaluate_round_function(arguments, context, visitor),
        "sqrt" => evaluate_sqrt_function(arguments, context, visitor),
        "exp" => evaluate_exp_function(arguments, context, visitor),
        "ln" => evaluate_ln_function(arguments, context, visitor),
        "log" => evaluate_log_function(arguments, context, visitor),
        "power" => evaluate_power_function(arguments, context, visitor),
        "truncate" => evaluate_truncate_function(arguments, context, visitor),

        // Date/time functions
        "now" => evaluate_now_function(arguments, context),
        "today" => evaluate_today_function(arguments, context),
        "timeOfDay" => evaluate_time_of_day_function(arguments, context),

        // Boolean functions
        "not" => evaluate_not_function(arguments, context, visitor),
        "all" => evaluate_all_function(arguments, context, visitor),
        "allTrue" => evaluate_all_true_function(arguments, context, visitor),
        "anyTrue" => evaluate_any_true_function(arguments, context, visitor),
        "allFalse" => evaluate_all_false_function(arguments, context, visitor),
        "anyFalse" => evaluate_any_false_function(arguments, context, visitor),

        // Conversion functions
        "convertsToInteger" => evaluate_converts_to_integer_function(arguments, context, visitor),
        "convertsToString" => evaluate_converts_to_string_function(arguments, context, visitor),
        "convertsToBoolean" => evaluate_converts_to_boolean_function(arguments, context, visitor),
        "convertsToDecimal" => evaluate_converts_to_decimal_function(arguments, context, visitor),
        "convertsToDate" => evaluate_converts_to_date_function(arguments, context, visitor),
        "convertsToDateTime" => {
            evaluate_converts_to_date_time_function(arguments, context, visitor)
        }
        "convertsToQuantity" => evaluate_converts_to_quantity_function(arguments, context, visitor),
        "convertsToTime" => evaluate_converts_to_time_function(arguments, context, visitor),
        "toString" => evaluate_to_string_function(arguments, context, visitor),
        "toInteger" => evaluate_to_integer_function(arguments, context, visitor),
        "toDecimal" => evaluate_to_decimal_function(arguments, context, visitor),
        "toQuantity" => evaluate_to_quantity_function(arguments, context, visitor),
        "toBoolean" => evaluate_to_boolean_function(arguments, context, visitor),

        // Tree navigation functions
        "children" => evaluate_children_function(arguments, context, visitor),
        "repeat" => evaluate_repeat_function(arguments, context, visitor),

        // String manipulation functions
        "trim" => evaluate_trim_function(arguments, context, visitor),
        "encode" => evaluate_encode_function(arguments, context, visitor),
        "decode" => evaluate_decode_function(arguments, context, visitor),

        // Conditional functions
        "iif" => evaluate_iif_function(arguments, context, visitor),

        // Type and metadata functions
        "type" => evaluate_type_function(arguments, context, visitor),
        "extension" => evaluate_extension_function(arguments, context, visitor),
        "ofType" => evaluate_of_type_function(arguments, context, visitor),
        "conformsTo" => evaluate_conforms_to_function(arguments, context, visitor),

        _ => Err(FhirPathError::EvaluationError(format!(
            "Unknown function: {}",
            name
        ))),
    }
}

/// Evaluates the where() function for filtering collections
fn evaluate_where_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'where' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;
    let total = collection.len();

    // For memory efficiency on large collections, process in chunks
    const CHUNK_SIZE: usize = 1000;
    let mut results = Vec::new();

    if total > CHUNK_SIZE {
        // Process large collections in chunks to reduce memory usage
        for chunk_start in (0..total).step_by(CHUNK_SIZE) {
            let chunk_end = std::cmp::min(chunk_start + CHUNK_SIZE, total);
            let chunk = &collection[chunk_start..chunk_end];

            for (relative_idx, item) in chunk.iter().enumerate() {
                let idx = chunk_start + relative_idx;
                // Create a new context for this item
                let item_context = context.create_iteration_context(item.clone(), idx, total)?;

                // Evaluate the filter expression in this context
                let filter_result =
                    evaluate_ast_with_visitor(&arguments[0], &item_context, visitor)?;

                // Check if the filter evaluates to true
                if is_truthy(&filter_result) {
                    results.push(item.clone());
                }
            }
        }
    } else {
        // For smaller collections, use the original approach
        for (idx, item) in collection.into_iter().enumerate() {
            // Create a new context for this item
            let item_context = context.create_iteration_context(item.clone(), idx, total)?;

            // Evaluate the filter expression in this context
            let filter_result = evaluate_ast_with_visitor(&arguments[0], &item_context, visitor)?;

            // Check if the filter evaluates to true
            if is_truthy(&filter_result) {
                results.push(item);
            }
        }
    }

    if results.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(results))
    }
}

/// Evaluates the select() function for projection
fn evaluate_select_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'select' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;
    let mut results = Vec::new();
    let total = collection.len();

    // Apply the projection to each item
    for (idx, item) in collection.into_iter().enumerate() {
        // Create a new context for this item
        let item_context = context.create_iteration_context(item, idx, total)?;

        // Evaluate the projection expression in this context
        let projection_result = evaluate_ast_with_visitor(&arguments[0], &item_context, visitor)?;

        // Add the result to the collection
        if projection_result != FhirPathValue::Empty {
            match projection_result {
                FhirPathValue::Collection(mut inner_items) => {
                    results.append(&mut inner_items);
                }
                _ => results.push(projection_result),
            }
        }
    }

    if results.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(results))
    }
}

/// Evaluates the first() function
fn evaluate_first_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'first' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    let collection = get_current_collection(context)?;
    if collection.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(collection[0].clone())
    }
}

/// Evaluates the last() function
fn evaluate_last_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'last' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    let collection = get_current_collection(context)?;
    if collection.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(collection[collection.len() - 1].clone())
    }
}

/// Evaluates the tail() function
fn evaluate_tail_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'tail' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    let collection = get_current_collection(context)?;
    if collection.len() <= 1 {
        Ok(FhirPathValue::Empty)
    } else {
        // Memory optimization: for large collections, avoid creating new vectors
        if context.optimization_enabled && collection.len() > 1000 {
            // For large collections, create a lazy slice
            let mut result = Vec::with_capacity(collection.len() - 1);
            result.extend_from_slice(&collection[1..]);
            Ok(FhirPathValue::Collection(result))
        } else {
            Ok(FhirPathValue::Collection(collection[1..].to_vec()))
        }
    }
}

/// Evaluates the skip() function
fn evaluate_skip_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'skip' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    let skip_count_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
    let skip_count = match skip_count_result {
        FhirPathValue::Integer(n) => n as usize,
        _ => {
            return Err(FhirPathError::TypeError(
                "'skip' function requires an integer argument".to_string(),
            ));
        }
    };

    let collection = get_current_collection(context)?;
    if skip_count >= collection.len() {
        Ok(FhirPathValue::Empty)
    } else {
        // Memory optimization: for large collections, use iterator-based approach
        if context.optimization_enabled && collection.len() > 1000 {
            let result: Vec<FhirPathValue> = collection.iter().skip(skip_count).cloned().collect();
            Ok(FhirPathValue::Collection(result))
        } else {
            Ok(FhirPathValue::Collection(collection[skip_count..].to_vec()))
        }
    }
}

/// Evaluates the take() function
fn evaluate_take_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'take' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    let take_count_result = evaluate_ast(&arguments[0], context)?;
    let take_count = match take_count_result {
        FhirPathValue::Integer(n) => n as usize,
        _ => {
            return Err(FhirPathError::TypeError(
                "'take' function requires an integer argument".to_string(),
            ));
        }
    };

    let collection = get_current_collection(context)?;
    let end_index = std::cmp::min(take_count, collection.len());

    if end_index == 0 {
        Ok(FhirPathValue::Empty)
    } else {
        // Memory optimization: for large collections, use iterator-based approach
        if context.optimization_enabled && collection.len() > 1000 {
            let result: Vec<FhirPathValue> = collection.iter().take(end_index).cloned().collect();
            Ok(FhirPathValue::Collection(result))
        } else {
            Ok(FhirPathValue::Collection(collection[..end_index].to_vec()))
        }
    }
}

/// Evaluates the exists() function
fn evaluate_exists_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() > 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'exists' function expects 0 or 1 arguments, got {}",
            arguments.len()
        )));
    }
    if arguments.is_empty() {
        // Check if the current context has any values
        match get_current_collection(context) {
            Ok(collection) => {
                if collection.len() == 1 && collection[0] == FhirPathValue::Empty {
                    return Ok(FhirPathValue::Boolean(false));
                }
                Ok(FhirPathValue::Boolean(!collection.is_empty()))
            }
            Err(_) => Ok(FhirPathValue::Boolean(false)),
        }
    } else {
        // Check if any item in the collection satisfies the condition
        let collection = get_current_collection(context)?;
        let total = collection.len();

        for (idx, item) in collection.into_iter().enumerate() {
            let item_context = context.create_iteration_context(item, idx, total)?;
            let condition_result = evaluate_ast(&arguments[0], &item_context)?;
            println!("condition_result: {:?}", condition_result);
            if is_truthy(&condition_result) {
                return Ok(FhirPathValue::Boolean(true));
            }
        }

        Ok(FhirPathValue::Boolean(false))
    }
}

/// Evaluates the empty() function
fn evaluate_empty_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'empty' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    match get_current_collection(context) {
        Ok(collection) => Ok(FhirPathValue::Boolean(collection.is_empty())),
        Err(_) => Ok(FhirPathValue::Boolean(true)),
    }
}

/// Evaluates the count() function
fn evaluate_count_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'count' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    let collection = get_current_collection(context)?;
    Ok(FhirPathValue::Integer(collection.len() as i64))
}

/// Evaluates the length() function for strings
fn evaluate_length_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'length' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current value - check this_item first (for method calls like "string".length())
    if let Some(this_item) = &context.this_item {
        match this_item {
            FhirPathValue::String(s) => return Ok(FhirPathValue::Integer(s.len() as i64)),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                if let FhirPathValue::String(s) = &items[0] {
                    return Ok(FhirPathValue::Integer(s.len() as i64));
                }
            }
            _ => {}
        }
    }

    // Fallback to checking the current collection
    let collection = get_current_collection(context)?;
    if collection.len() == 1 {
        if let FhirPathValue::String(s) = &collection[0] {
            return Ok(FhirPathValue::Integer(s.len() as i64));
        }
    }

    // Last fallback: check raw JSON context for direct string values
    match &context.context {
        serde_json::Value::String(s) => Ok(FhirPathValue::Integer(s.len() as i64)),
        _ => Err(FhirPathError::TypeError(
            "'length' function can only be applied to strings".to_string(),
        )),
    }
}

/// Evaluates the distinct() function
fn evaluate_distinct_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'distinct' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    let collection = get_current_collection(context)?;
    let mut unique_items = Vec::new();

    for item in collection {
        if !unique_items
            .iter()
            .any(|existing| values_equal(existing, &item))
        {
            unique_items.push(item);
        }
    }

    if unique_items.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(unique_items))
    }
}

/// Evaluates the isDistinct() function - returns true if all items in collection are distinct
fn evaluate_is_distinct_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'isDistinct' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    // Check if all items are distinct by comparing each item with all others
    for (i, item1) in collection.iter().enumerate() {
        for (j, item2) in collection.iter().enumerate() {
            if i != j && values_equal(item1, item2) {
                // Found duplicate items
                return Ok(FhirPathValue::Boolean(false));
            }
        }
    }

    // All items are distinct
    Ok(FhirPathValue::Boolean(true))
}

/// Evaluates the sort() function - sorts a collection in ascending order
fn evaluate_sort_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    _visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'sort' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    let collection = get_current_collection(context)?;

    if collection.is_empty() {
        return Ok(FhirPathValue::Empty);
    }

    let mut sorted_items = collection;

    // Sort the collection using a custom comparison function
    sorted_items.sort_by(|a, b| {
        use std::cmp::Ordering;

        match (a, b) {
            // Numbers
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a.cmp(b),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal),
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal),

            // Strings
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a.cmp(b),

            // Dates and DateTimes
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a.cmp(b),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a.cmp(b),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a.cmp(b),

            // Booleans
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a.cmp(b),

            // Mixed types - use type precedence: Empty < Boolean < Integer < Decimal < String < Date < DateTime < Time < Quantity < Collection < Resource
            _ => {
                let type_order = |v: &FhirPathValue| -> u8 {
                    match v {
                        FhirPathValue::Empty => 0,
                        FhirPathValue::Boolean(_) => 1,
                        FhirPathValue::Integer(_) => 2,
                        FhirPathValue::Decimal(_) => 3,
                        FhirPathValue::String(_) => 4,
                        FhirPathValue::Date(_) => 5,
                        FhirPathValue::DateTime(_) => 6,
                        FhirPathValue::Time(_) => 7,
                        FhirPathValue::Quantity { .. } => 8,
                        FhirPathValue::Collection(_) => 9,
                        FhirPathValue::Resource(_) => 10,
                    }
                };
                type_order(a).cmp(&type_order(b))
            }
        }
    });

    Ok(FhirPathValue::Collection(sorted_items))
}

/// Evaluates the descendants() function - returns all descendant elements in a FHIR resource
fn evaluate_descendants_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'descendants' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;
    let mut descendants = Vec::new();

    // For each item in the collection, get all its descendants
    for item in collection {
        match item {
            FhirPathValue::Resource(resource) => {
                // Recursively collect all descendants from the resource
                collect_descendants_from_resource(&resource, &mut descendants);
            }
            _ => {
                // Non-resource items don't have descendants
                continue;
            }
        }
    }

    if descendants.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(descendants))
    }
}

/// Helper function to recursively collect descendants from a FHIR resource
fn collect_descendants_from_resource(
    resource: &crate::model::FhirResource,
    descendants: &mut Vec<FhirPathValue>,
) {
    // Add all properties of this resource as descendants
    for (_, value) in &resource.properties {
        match json_to_fhirpath_value(value.clone()) {
            Ok(fhir_value) => {
                match fhir_value {
                    FhirPathValue::Resource(child_resource) => {
                        // Add the child resource itself
                        descendants.push(FhirPathValue::Resource(child_resource.clone()));
                        // Recursively collect descendants from the child resource
                        collect_descendants_from_resource(&child_resource, descendants);
                    }
                    FhirPathValue::Collection(items) => {
                        // Add each item in the collection and their descendants
                        for item in items {
                            descendants.push(item.clone());
                            if let FhirPathValue::Resource(child_resource) = item {
                                collect_descendants_from_resource(&child_resource, descendants);
                            }
                        }
                    }
                    other => {
                        // Add primitive values as descendants
                        descendants.push(other);
                    }
                }
            }
            Err(_) => {
                // Skip values that can't be converted
                continue;
            }
        }
    }
}

/// Evaluates the children() function - returns direct child elements in a FHIR resource
fn evaluate_children_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'children' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;
    let mut children = Vec::new();

    // For each item in the collection, get its direct children
    for item in collection {
        match item {
            FhirPathValue::Resource(resource) => {
                // Collect direct children from the resource (no recursion)
                collect_children_from_resource(&resource, &mut children);
            }
            _ => {
                // Non-resource items don't have children
                continue;
            }
        }
    }

    if children.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(children))
    }
}

/// Helper function to collect direct children from a FHIR resource (non-recursive)
fn collect_children_from_resource(
    resource: &crate::model::FhirResource,
    children: &mut Vec<FhirPathValue>,
) {
    // Add all properties of this resource as direct children (no recursion)
    for (_, value) in &resource.properties {
        match json_to_fhirpath_value(value.clone()) {
            Ok(fhir_value) => {
                match fhir_value {
                    FhirPathValue::Resource(child_resource) => {
                        // Add the child resource itself (but don't recurse)
                        children.push(FhirPathValue::Resource(child_resource));
                    }
                    FhirPathValue::Collection(items) => {
                        // Add each item in the collection (but don't recurse)
                        for item in items {
                            children.push(item);
                        }
                    }
                    other => {
                        // Add primitive values as children
                        children.push(other);
                    }
                }
            }
            Err(_) => {
                // Skip values that can't be converted
                continue;
            }
        }
    }
}

/// Evaluates the repeat() function - repeatedly applies an expression until no new items are found
fn evaluate_repeat_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'repeat' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let mut current_collection = get_current_collection(context)?;
    let mut all_results = Vec::new();
    let mut seen_items = std::collections::HashSet::new();

    // Add initial items to results and seen set
    for item in &current_collection {
        let hash = calculate_value_hash(item);
        if seen_items.insert(hash) {
            all_results.push(item.clone());
        }
    }

    // Repeatedly apply the expression until no new items are found
    loop {
        let mut new_items = Vec::new();
        let mut found_new = false;

        // Apply the expression to each item in the current collection
        for item in &current_collection {
            let item_context = context.create_iteration_context(item.clone(), 0, 1)?;
            let result = evaluate_ast_with_visitor(&arguments[0], &item_context, visitor)?;

            // Collect results from this iteration
            match result {
                FhirPathValue::Collection(items) => {
                    for new_item in items {
                        let hash = calculate_value_hash(&new_item);
                        if seen_items.insert(hash) {
                            new_items.push(new_item.clone());
                            all_results.push(new_item);
                            found_new = true;
                        }
                    }
                }
                FhirPathValue::Empty => {
                    // No new items from this iteration
                }
                single_item => {
                    let hash = calculate_value_hash(&single_item);
                    if seen_items.insert(hash) {
                        new_items.push(single_item.clone());
                        all_results.push(single_item);
                        found_new = true;
                    }
                }
            }
        }

        // If no new items were found, we're done
        if !found_new {
            break;
        }

        // Update current collection for next iteration
        current_collection = new_items;
    }

    if all_results.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(all_results))
    }
}

/// Helper function to calculate a hash for a FhirPathValue for deduplication
fn calculate_value_hash(value: &FhirPathValue) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Create a string representation for hashing
    let hash_string = match value {
        FhirPathValue::String(s) => format!("string:{}", s),
        FhirPathValue::Integer(i) => format!("integer:{}", i),
        FhirPathValue::Decimal(d) => format!("decimal:{}", d),
        FhirPathValue::Boolean(b) => format!("boolean:{}", b),
        FhirPathValue::Date(d) => format!("date:{}", d),
        FhirPathValue::DateTime(dt) => format!("datetime:{}", dt),
        FhirPathValue::Time(t) => format!("time:{}", t),
        FhirPathValue::Quantity { value, unit } => format!("quantity:{}:{}", value, unit),
        FhirPathValue::Resource(r) => format!(
            "resource:{}",
            r.resource_type.as_deref().unwrap_or("unknown")
        ),
        FhirPathValue::Collection(_) => "collection".to_string(),
        FhirPathValue::Empty => "empty".to_string(),
    };

    hash_string.hash(&mut hasher);
    hasher.finish()
}

/// Union function - merges collections removing duplicates
fn evaluate_union_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'union' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the argument to get the other collection
    let visitor = NoopVisitor::new();
    let other_result = evaluate_ast_with_visitor(&arguments[0], context, &visitor)?;
    let other_collection = match other_result {
        FhirPathValue::Collection(items) => items,
        FhirPathValue::Empty => vec![],
        single_item => vec![single_item],
    };

    // Create union - start with current collection items
    let mut union_items = Vec::new();

    // Add all items from current collection
    for item in &current_collection {
        union_items.push(item.clone());
    }

    // Add items from other collection that are not already present
    for other_item in &other_collection {
        let mut already_present = false;
        for existing_item in &union_items {
            if values_equal(other_item, existing_item) {
                already_present = true;
                break;
            }
        }
        if !already_present {
            union_items.push(other_item.clone());
        }
    }

    if union_items.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(union_items))
    }
}

/// Combine function - merges collections keeping all duplicates
fn evaluate_combine_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'combine' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the argument to get the other collection
    let visitor = NoopVisitor::new();
    let other_result = evaluate_ast_with_visitor(&arguments[0], context, &visitor)?;
    let other_collection = match other_result {
        FhirPathValue::Collection(items) => items,
        FhirPathValue::Empty => vec![],
        single_item => vec![single_item],
    };

    // Create combined collection - add all items from both collections (keeping duplicates)
    let mut combined_items = Vec::new();

    // Add all items from current collection
    for item in &current_collection {
        combined_items.push(item.clone());
    }

    // Add all items from other collection (including duplicates)
    for item in &other_collection {
        combined_items.push(item.clone());
    }

    if combined_items.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(combined_items))
    }
}

fn evaluate_intersect_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'intersect' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the argument to get the other collection
    let visitor = NoopVisitor::new();
    let other_result = evaluate_ast_with_visitor(&arguments[0], context, &visitor)?;
    let other_collection = match other_result {
        FhirPathValue::Collection(items) => items,
        FhirPathValue::Empty => vec![],
        single_item => vec![single_item],
    };

    // Find intersection - items that exist in both collections
    let mut intersection_items = Vec::new();

    for current_item in &current_collection {
        // Check if this item exists in the other collection
        let mut found_in_other = false;
        for other_item in &other_collection {
            if values_equal(current_item, other_item) {
                found_in_other = true;
                break;
            }
        }

        // If found in other collection, add to intersection (avoiding duplicates)
        if found_in_other {
            let mut already_in_intersection = false;
            for existing_item in &intersection_items {
                if values_equal(current_item, existing_item) {
                    already_in_intersection = true;
                    break;
                }
            }
            if !already_in_intersection {
                intersection_items.push(current_item.clone());
            }
        }
    }

    if intersection_items.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(intersection_items))
    }
}

fn evaluate_exclude_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'exclude' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the argument to get the collection of items to exclude
    let exclude_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
    let exclude_collection = match exclude_result {
        FhirPathValue::Collection(items) => items,
        FhirPathValue::Empty => vec![],
        single_item => vec![single_item],
    };

    // Find items in current collection that are NOT in exclude collection
    let mut result_items = Vec::new();

    for current_item in &current_collection {
        // Check if this item exists in the exclude collection
        let mut found_in_exclude = false;
        for exclude_item in &exclude_collection {
            if values_equal(current_item, exclude_item) {
                found_in_exclude = true;
                break;
            }
        }

        // If NOT found in exclude collection, add to result (preserving duplicates)
        if !found_in_exclude {
            result_items.push(current_item.clone());
        }
    }

    if result_items.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(result_items))
    }
}

fn evaluate_subset_of_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'subsetOf' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the argument to get the comparison collection
    let comparison_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
    let comparison_collection = match comparison_result {
        FhirPathValue::Collection(items) => items,
        FhirPathValue::Empty => vec![],
        single_item => vec![single_item],
    };

    // Check if all items in current collection exist in comparison collection
    for current_item in &current_collection {
        let mut found = false;
        for comparison_item in &comparison_collection {
            if values_equal(current_item, comparison_item) {
                found = true;
                break;
            }
        }
        if !found {
            return Ok(FhirPathValue::Boolean(false));
        }
    }

    Ok(FhirPathValue::Boolean(true))
}

fn evaluate_is_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'is' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Extract type name from the argument - handle both identifiers and path expressions
    let type_name = match &arguments[0] {
        AstNode::Identifier(name) => name.clone(),
        AstNode::Path(left, right) => {
            // Handle path expressions like System.Boolean
            match (left.as_ref(), right.as_ref()) {
                (AstNode::Identifier(namespace), AstNode::Identifier(type_name)) => {
                    format!("{}.{}", namespace, type_name)
                }
                _ => {
                    return Err(FhirPathError::EvaluationError(
                        "'is' function expects a type name or qualified type name as argument"
                            .to_string(),
                    ))
                }
            }
        }
        _ => {
            return Err(FhirPathError::EvaluationError(
                "'is' function expects a type name or qualified type name as argument".to_string(),
            ))
        }
    };

    // Check if any item in the current collection matches the specified type
    for item in &current_collection {
        let matches_type = match (item, type_name.as_str()) {
            // System types (both capitalized and lowercase)
            (FhirPathValue::String(_), "String" | "string" | "System.String") => true,
            (FhirPathValue::Integer(_), "Integer" | "integer" | "System.Integer") => true,
            (FhirPathValue::Decimal(_), "Decimal" | "decimal" | "System.Decimal") => true,
            (FhirPathValue::Boolean(_), "Boolean" | "boolean" | "System.Boolean") => true,
            (FhirPathValue::Date(_), "Date" | "date" | "System.Date") => true,
            (FhirPathValue::DateTime(_), "DateTime" | "dateTime" | "System.DateTime") => true,
            (FhirPathValue::Time(_), "Time" | "time" | "System.Time") => true,
            (FhirPathValue::Quantity { .. }, "Quantity" | "System.Quantity") => true,
            (FhirPathValue::Collection(_), "Collection" | "System.Collection") => true,

            // FHIR primitive types - these should be treated as FHIR types, not System types
            (FhirPathValue::Boolean(_), "FHIR.boolean") => true,
            (FhirPathValue::String(_), "FHIR.string") => true,
            (FhirPathValue::Integer(_), "FHIR.integer") => true,
            (FhirPathValue::Decimal(_), "FHIR.decimal") => true,
            (FhirPathValue::Date(_), "FHIR.date") => true,
            (FhirPathValue::DateTime(_), "FHIR.dateTime") => true,
            (FhirPathValue::Time(_), "FHIR.time") => true,

            // FHIR resource types
            (FhirPathValue::Resource(resource), type_name) => {
                if let Some(resource_type) = &resource.resource_type {
                    // Check exact match or FHIR-qualified match
                    resource_type == type_name || format!("FHIR.{}", resource_type) == type_name
                } else {
                    // Generic resource type check
                    type_name == "Resource"
                        || type_name == "resource"
                        || type_name == "FHIR.Resource"
                }
            }
            _ => false,
        };

        if matches_type {
            return Ok(FhirPathValue::Boolean(true));
        }
    }

    Ok(FhirPathValue::Boolean(false))
}

fn evaluate_as_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'as' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Get the type name from the argument
    let type_name = match &arguments[0] {
        AstNode::Identifier(name) => name.clone(),
        _ => {
            return Err(FhirPathError::TypeError(
                "'as' function requires a type identifier".to_string(),
            ))
        }
    };

    let mut results = Vec::new();

    for item in &current_collection {
        // First try direct type matching
        let matches_type = match (item, type_name.as_str()) {
            (FhirPathValue::String(_), "string") => true,
            (FhirPathValue::Integer(_), "integer") => true,
            (FhirPathValue::Decimal(_), "decimal") => true,
            (FhirPathValue::Boolean(_), "boolean") => true,
            (FhirPathValue::Date(_), "date") => true,
            (FhirPathValue::DateTime(_), "dateTime") => true,
            (FhirPathValue::Time(_), "time") => true,
            (FhirPathValue::Time(_), "Time") => true,
            (FhirPathValue::Quantity { .. }, "Quantity") => true,
            // For FHIR resource types, check if the resource has the expected resourceType
            (FhirPathValue::Resource(resource), type_name) => {
                if let Some(resource_type) = &resource.resource_type {
                    resource_type == type_name
                } else {
                    false
                }
            }
            _ => false,
        };

        if matches_type {
            results.push(item.clone());
            continue;
        }

        // If direct type matching fails, try conversion
        let converted_value = match (item, type_name.as_str()) {
            // String to DateTime/Date/Time conversion
            (FhirPathValue::String(s), "dateTime")
            | (FhirPathValue::String(s), "date")
            | (FhirPathValue::String(s), "time") => {
                if let Some(dt_value) = string_to_datetime(s) {
                    // Only add if the converted type matches the requested type
                    match (dt_value.clone(), type_name.as_str()) {
                        (FhirPathValue::DateTime(_), "dateTime")
                        | (FhirPathValue::Date(_), "date")
                        | (FhirPathValue::Time(_), "time") => Some(dt_value),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            // String to Integer conversion
            (FhirPathValue::String(s), "integer") => {
                s.parse::<i64>().ok().map(FhirPathValue::Integer)
            }
            // String to Decimal conversion
            (FhirPathValue::String(s), "decimal") => {
                s.parse::<f64>().ok().map(FhirPathValue::Decimal)
            }
            // String to Boolean conversion
            (FhirPathValue::String(s), "boolean") => match s.to_lowercase().as_str() {
                "true" => Some(FhirPathValue::Boolean(true)),
                "false" => Some(FhirPathValue::Boolean(false)),
                _ => None,
            },
            // Integer to Decimal conversion
            (FhirPathValue::Integer(i), "decimal") => Some(FhirPathValue::Decimal(*i as f64)),
            // Decimal to Integer conversion (truncates)
            (FhirPathValue::Decimal(d), "integer") => Some(FhirPathValue::Integer(*d as i64)),
            _ => None,
        };

        if let Some(value) = converted_value {
            results.push(value);
        }
        // If conversion fails, we don't add anything to results
    }

    if results.is_empty() {
        Ok(FhirPathValue::Empty)
    } else if results.len() == 1 {
        Ok(results.into_iter().next().unwrap())
    } else {
        Ok(FhirPathValue::Collection(results))
    }
}

fn evaluate_contains_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'contains' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the substring argument
    let substring_result =
        evaluate_ast_internal_uncached(&arguments[0], context, &NoopVisitor::new())?;

    let substring = match substring_result {
        FhirPathValue::String(s) => s,
        _ => {
            return Err(FhirPathError::TypeError(
                "'contains' function requires a string argument".to_string(),
            ))
        }
    };

    // Check if any string in the current collection contains the substring
    for item in &current_collection {
        if let FhirPathValue::String(s) = item {
            if s.contains(&substring) {
                return Ok(FhirPathValue::Boolean(true));
            }
        }
    }

    Ok(FhirPathValue::Boolean(false))
}

fn evaluate_starts_with_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'startsWith' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the prefix argument
    let prefix_result =
        evaluate_ast_internal_uncached(&arguments[0], context, &NoopVisitor::new())?;

    let prefix = match prefix_result {
        FhirPathValue::String(s) => s,
        _ => {
            return Err(FhirPathError::TypeError(
                "'startsWith' function requires a string argument".to_string(),
            ))
        }
    };

    // Check if any string in the current collection starts with the prefix
    for item in &current_collection {
        if let FhirPathValue::String(s) = item {
            if s.starts_with(&prefix) {
                return Ok(FhirPathValue::Boolean(true));
            }
        }
    }

    Ok(FhirPathValue::Boolean(false))
}

fn evaluate_ends_with_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'endsWith' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let current_collection = get_current_collection(context)?;

    // Evaluate the suffix argument
    let suffix_result =
        evaluate_ast_internal_uncached(&arguments[0], context, &NoopVisitor::new())?;

    let suffix = match suffix_result {
        FhirPathValue::String(s) => s,
        _ => {
            return Err(FhirPathError::TypeError(
                "'endsWith' function requires a string argument".to_string(),
            ))
        }
    };

    // Check if any string in the current collection ends with the suffix
    for item in &current_collection {
        if let FhirPathValue::String(s) = item {
            if s.ends_with(&suffix) {
                return Ok(FhirPathValue::Boolean(true));
            }
        }
    }

    Ok(FhirPathValue::Boolean(false))
}

fn evaluate_substring_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() < 1 || arguments.len() > 2 {
        return Err(FhirPathError::EvaluationError(format!(
            "'substring' function expects 1 or 2 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        if let FhirPathValue::String(s) = item {
            let start_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

            if let FhirPathValue::Integer(start) = start_result {
                // Handle negative start indices - return Empty for out-of-bounds
                if start < 0 {
                    return Ok(FhirPathValue::Empty);
                }

                let start_idx = start as usize;

                if arguments.len() == 2 {
                    let length_result = evaluate_ast_with_visitor(&arguments[1], context, visitor)?;
                    if let FhirPathValue::Integer(length) = length_result {
                        if length <= 0 {
                            return Ok(FhirPathValue::Empty);
                        }
                        let result = if start_idx >= s.len() {
                            return Ok(FhirPathValue::Empty);
                        } else {
                            let substring: String = s.chars().skip(start_idx).take(length as usize).collect();
                            if substring.is_empty() {
                                return Ok(FhirPathValue::Empty);
                            }
                            substring
                        };
                        return Ok(FhirPathValue::String(result));
                    } else {
                        return Err(FhirPathError::TypeError(
                            "'substring' function length argument must be an integer".to_string(),
                        ));
                    }
                } else {
                    // Only start index provided, return substring from start to end
                    if start_idx >= s.len() {
                        return Ok(FhirPathValue::Empty);
                    } else {
                        let substring: String = s.chars().skip(start_idx).collect();
                        if substring.is_empty() {
                            return Ok(FhirPathValue::Empty);
                        }
                        return Ok(FhirPathValue::String(substring));
                    }
                }
            } else {
                return Err(FhirPathError::TypeError(
                    "'substring' function start argument must be an integer".to_string(),
                ));
            }
        }
    }

    Ok(FhirPathValue::Empty)
}

fn evaluate_index_of_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'indexOf' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        if let FhirPathValue::String(s) = item {
            let substring_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

            if let FhirPathValue::String(substring) = substring_result {
                match s.find(&substring) {
                    Some(index) => return Ok(FhirPathValue::Integer(index as i64)),
                    None => return Ok(FhirPathValue::Empty),
                }
            } else {
                return Err(FhirPathError::TypeError(
                    "'indexOf' function argument must be a string".to_string(),
                ));
            }
        }
    }

    Ok(FhirPathValue::Empty)
}

fn evaluate_replace_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 2 {
        return Err(FhirPathError::EvaluationError(format!(
            "'replace' function expects 2 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        if let FhirPathValue::String(s) = item {
            let pattern_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
            let replacement_result = evaluate_ast_with_visitor(&arguments[1], context, visitor)?;

            if let (FhirPathValue::String(pattern), FhirPathValue::String(replacement)) =
                (pattern_result, replacement_result) {
                let result = s.replace(&pattern, &replacement);
                return Ok(FhirPathValue::String(result));
            } else {
                return Err(FhirPathError::TypeError(
                    "'replace' function arguments must be strings".to_string(),
                ));
            }
        }
    }

    Ok(FhirPathValue::Empty)
}

fn evaluate_matches_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'matches' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        if let FhirPathValue::String(s) = item {
            let pattern_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

            if let FhirPathValue::String(pattern) = pattern_result {
                match regex::Regex::new(&pattern) {
                    Ok(re) => {
                        let matches = re.is_match(&s);
                        return Ok(FhirPathValue::Boolean(matches));
                    }
                    Err(_) => {
                        return Err(FhirPathError::EvaluationError(format!(
                            "Invalid regex pattern: {}",
                            pattern
                        )));
                    }
                }
            } else {
                return Err(FhirPathError::TypeError(
                    "'matches' function argument must be a string".to_string(),
                ));
            }
        }
    }

    Ok(FhirPathValue::Boolean(false))
}

fn evaluate_split_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'split' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        if let FhirPathValue::String(s) = item {
            let delimiter_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

            if let FhirPathValue::String(delimiter) = delimiter_result {
                let parts: Vec<FhirPathValue> = s
                    .split(&delimiter)
                    .map(|part| FhirPathValue::String(part.to_string()))
                    .collect();

                return Ok(FhirPathValue::Collection(parts));
            } else {
                return Err(FhirPathError::TypeError(
                    "'split' function delimiter argument must be a string".to_string(),
                ));
            }
        }
    }

    Ok(FhirPathValue::Empty)
}

fn evaluate_join_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'join' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    // Evaluate the separator argument
    let separator_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

    let separator = match separator_result {
        FhirPathValue::String(s) => s,
        _ => {
            return Err(FhirPathError::TypeError(
                "'join' function separator argument must be a string".to_string(),
            ))
        }
    };

    // Handle empty collection case
    if collection.is_empty() {
        return Ok(FhirPathValue::String(String::new()));
    }

    // Collect all string values from the collection
    let mut string_values = Vec::new();
    for item in collection {
        match item {
            FhirPathValue::String(s) => string_values.push(s),
            // Skip non-string values instead of erroring - this is more consistent with FHIRPath behavior
            _ => continue,
        }
    }

    // Join the strings with the separator
    let joined = string_values.join(&separator);
    Ok(FhirPathValue::String(joined))
}

fn evaluate_abs_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i.abs())),
                FhirPathValue::Decimal(d) => results.push(FhirPathValue::Decimal(d.abs())),
                FhirPathValue::Quantity { value, unit } => {
                    results.push(FhirPathValue::Quantity {
                        value: value.abs(),
                        unit: unit.clone(),
                    })
                }
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'abs' function can only be applied to numbers and quantities".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        match result {
            FhirPathValue::Integer(i) => FhirPathValue::Integer(i.abs()),
            FhirPathValue::Decimal(d) => FhirPathValue::Decimal(d.abs()),
            FhirPathValue::Quantity { value, unit } => FhirPathValue::Quantity {
                value: value.abs(),
                unit,
            },
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i.abs())),
                        FhirPathValue::Decimal(d) => results.push(FhirPathValue::Decimal(d.abs())),
                        FhirPathValue::Quantity { value, unit } => {
                            results.push(FhirPathValue::Quantity {
                                value: value.abs(),
                                unit,
                            })
                        }
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "'abs' function can only be applied to numbers and quantities".to_string(),
                            ))
                        }
                    }
                }
                FhirPathValue::Collection(results)
            }
            _ => {
                return Err(FhirPathError::TypeError(
                    "'abs' function can only be applied to numbers and quantities".to_string(),
                ))
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'abs' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_ceiling_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                FhirPathValue::Decimal(d) => results.push(FhirPathValue::Integer(d.ceil() as i64)),
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'ceiling' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        match result {
            FhirPathValue::Integer(i) => FhirPathValue::Integer(i),
            FhirPathValue::Decimal(d) => FhirPathValue::Integer(d.ceil() as i64),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                        FhirPathValue::Decimal(d) => {
                            results.push(FhirPathValue::Integer(d.ceil() as i64))
                        }
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "'ceiling' function can only be applied to numbers".to_string(),
                            ))
                        }
                    }
                }
                FhirPathValue::Collection(results)
            }
            _ => {
                return Err(FhirPathError::TypeError(
                    "'ceiling' function can only be applied to numbers".to_string(),
                ))
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'ceiling' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_floor_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                FhirPathValue::Decimal(d) => results.push(FhirPathValue::Integer(d.floor() as i64)),
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'floor' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        match result {
            FhirPathValue::Integer(i) => FhirPathValue::Integer(i),
            FhirPathValue::Decimal(d) => FhirPathValue::Integer(d.floor() as i64),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                        FhirPathValue::Decimal(d) => {
                            results.push(FhirPathValue::Integer(d.floor() as i64))
                        }
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "'floor' function can only be applied to numbers".to_string(),
                            ))
                        }
                    }
                }
                FhirPathValue::Collection(results)
            }
            _ => {
                return Err(FhirPathError::TypeError(
                    "'floor' function can only be applied to numbers".to_string(),
                ))
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'floor' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_round_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // Helper function to round a number to specified precision
    fn round_to_precision(value: f64, precision: i64) -> f64 {
        if precision < 0 {
            // For negative precision, round to nearest 10, 100, etc.
            let factor = 10_f64.powi((-precision) as i32);
            (value / factor).round() * factor
        } else {
            // For positive precision, round to decimal places
            let factor = 10_f64.powi(precision as i32);
            (value * factor).round() / factor
        }
    }

    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                FhirPathValue::Decimal(d) => results.push(FhirPathValue::Integer(d.round() as i64)),
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'round' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let precision_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        // Check if this is a precision argument (when called on a number like 3.14159.round(3))
        // or if this is the value to round (when called like round(3.14159))
        let collection = get_current_collection(context)?;

        if !collection.is_empty() {
            // This is precision argument - we have a current collection to round
            let precision = match precision_result {
                FhirPathValue::Integer(p) => p,
                _ => {
                    return Err(FhirPathError::TypeError(
                        "Precision argument for 'round' function must be an integer".to_string(),
                    ))
                }
            };

            let mut results = Vec::new();
            for item in collection {
                match item {
                    FhirPathValue::Integer(i) => {
                        if precision <= 0 {
                            results.push(FhirPathValue::Integer(round_to_precision(i as f64, precision) as i64));
                        } else {
                            results.push(FhirPathValue::Decimal(round_to_precision(i as f64, precision)));
                        }
                    },
                    FhirPathValue::Decimal(d) => {
                        if precision <= 0 {
                            results.push(FhirPathValue::Integer(round_to_precision(d, precision) as i64));
                        } else {
                            results.push(FhirPathValue::Decimal(round_to_precision(d, precision)));
                        }
                    },
                    _ => {
                        return Err(FhirPathError::TypeError(
                            "'round' function can only be applied to numbers".to_string(),
                        ))
                    }
                }
            }

            if results.len() == 1 {
                results.into_iter().next().unwrap()
            } else {
                FhirPathValue::Collection(results)
            }
        } else {
            // This is the value to round (no current collection)
            match precision_result {
                FhirPathValue::Integer(i) => FhirPathValue::Integer(i),
                FhirPathValue::Decimal(d) => FhirPathValue::Integer(d.round() as i64),
                FhirPathValue::Collection(items) => {
                    let mut results = Vec::new();
                    for item in items {
                        match item {
                            FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                            FhirPathValue::Decimal(d) => {
                                results.push(FhirPathValue::Integer(d.round() as i64))
                            }
                            _ => {
                                return Err(FhirPathError::TypeError(
                                    "'round' function can only be applied to numbers".to_string(),
                                ))
                            }
                        }
                    }
                    FhirPathValue::Collection(results)
                }
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'round' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'round' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_sqrt_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => {
                    if i < 0 {
                        return Ok(FhirPathValue::Empty);
                    } else {
                        results.push(FhirPathValue::Decimal((i as f64).sqrt()));
                    }
                }
                FhirPathValue::Decimal(d) => {
                    if d < 0.0 {
                        return Ok(FhirPathValue::Empty);
                    } else {
                        results.push(FhirPathValue::Decimal(d.sqrt()));
                    }
                }
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'sqrt' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        match result {
            FhirPathValue::Integer(i) => {
                if i < 0 {
                    return Ok(FhirPathValue::Empty);
                } else {
                    FhirPathValue::Decimal((i as f64).sqrt())
                }
            }
            FhirPathValue::Decimal(d) => {
                if d < 0.0 {
                    return Ok(FhirPathValue::Empty);
                } else {
                    FhirPathValue::Decimal(d.sqrt())
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items {
                    match item {
                        FhirPathValue::Integer(i) => {
                            if i < 0 {
                                return Ok(FhirPathValue::Empty);
                            } else {
                                results.push(FhirPathValue::Decimal((i as f64).sqrt()));
                            }
                        }
                        FhirPathValue::Decimal(d) => {
                            if d < 0.0 {
                                return Ok(FhirPathValue::Empty);
                            } else {
                                results.push(FhirPathValue::Decimal(d.sqrt()));
                            }
                        }
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "'sqrt' function can only be applied to numbers".to_string(),
                            ))
                        }
                    }
                }
                FhirPathValue::Collection(results)
            }
            _ => {
                return Err(FhirPathError::TypeError(
                    "'sqrt' function can only be applied to numbers".to_string(),
                ))
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'sqrt' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_exp_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => results.push(FhirPathValue::Decimal((i as f64).exp())),
                FhirPathValue::Decimal(d) => results.push(FhirPathValue::Decimal(d.exp())),
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'exp' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        match result {
            FhirPathValue::Integer(i) => FhirPathValue::Decimal((i as f64).exp()),
            FhirPathValue::Decimal(d) => FhirPathValue::Decimal(d.exp()),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items {
                    match item {
                        FhirPathValue::Integer(i) => {
                            results.push(FhirPathValue::Decimal((i as f64).exp()))
                        }
                        FhirPathValue::Decimal(d) => results.push(FhirPathValue::Decimal(d.exp())),
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "'exp' function can only be applied to numbers".to_string(),
                            ))
                        }
                    }
                }
                FhirPathValue::Collection(results)
            }
            _ => {
                return Err(FhirPathError::TypeError(
                    "'exp' function can only be applied to numbers".to_string(),
                ))
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'exp' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_ln_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => {
                    if i <= 0 {
                        return Err(FhirPathError::EvaluationError(
                            "Cannot take natural log of non-positive number".to_string(),
                        ));
                    } else {
                        results.push(FhirPathValue::Decimal((i as f64).ln()));
                    }
                }
                FhirPathValue::Decimal(d) => {
                    if d <= 0.0 {
                        return Err(FhirPathError::EvaluationError(
                            "Cannot take natural log of non-positive number".to_string(),
                        ));
                    } else {
                        results.push(FhirPathValue::Decimal(d.ln()));
                    }
                }
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'ln' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        match result {
            FhirPathValue::Integer(i) => {
                if i <= 0 {
                    return Err(FhirPathError::EvaluationError(
                        "Cannot take natural log of non-positive number".to_string(),
                    ));
                } else {
                    FhirPathValue::Decimal((i as f64).ln())
                }
            }
            FhirPathValue::Decimal(d) => {
                if d <= 0.0 {
                    return Err(FhirPathError::EvaluationError(
                        "Cannot take natural log of non-positive number".to_string(),
                    ));
                } else {
                    FhirPathValue::Decimal(d.ln())
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items {
                    match item {
                        FhirPathValue::Integer(i) => {
                            if i <= 0 {
                                return Err(FhirPathError::EvaluationError(
                                    "Cannot take natural log of non-positive number".to_string(),
                                ));
                            } else {
                                results.push(FhirPathValue::Decimal((i as f64).ln()));
                            }
                        }
                        FhirPathValue::Decimal(d) => {
                            if d <= 0.0 {
                                return Err(FhirPathError::EvaluationError(
                                    "Cannot take natural log of non-positive number".to_string(),
                                ));
                            } else {
                                results.push(FhirPathValue::Decimal(d.ln()));
                            }
                        }
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "'ln' function can only be applied to numbers".to_string(),
                            ))
                        }
                    }
                }
                FhirPathValue::Collection(results)
            }
            _ => {
                return Err(FhirPathError::TypeError(
                    "'ln' function can only be applied to numbers".to_string(),
                ))
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'ln' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_log_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let (value, base) = if arguments.len() == 1 {
        // Method call syntax: value.log(base)
        // Use this_item as value and first argument as base
        if let Some(this_item) = &context.this_item {
            let value = match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'log' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            };
            let base = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
            (value, base)
        } else {
            return Err(FhirPathError::EvaluationError(
                "'log' function expects method call syntax with base argument".to_string(),
            ));
        }
    } else if arguments.len() == 2 {
        // Function call syntax: log(value, base)
        let value = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
        let base = evaluate_ast_with_visitor(&arguments[1], context, visitor)?;
        (value, base)
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'log' function expects 1 or 2 arguments, got {}",
            arguments.len()
        )));
    };

    let (value_f64, base_f64) = match (value, base) {
        (FhirPathValue::Integer(v), FhirPathValue::Integer(b)) => (v as f64, b as f64),
        (FhirPathValue::Integer(v), FhirPathValue::Decimal(b)) => (v as f64, b),
        (FhirPathValue::Decimal(v), FhirPathValue::Integer(b)) => (v, b as f64),
        (FhirPathValue::Decimal(v), FhirPathValue::Decimal(b)) => (v, b),
        _ => {
            return Err(FhirPathError::TypeError(
                "'log' function can only be applied to numbers".to_string(),
            ))
        }
    };

    if value_f64 <= 0.0 {
        return Err(FhirPathError::EvaluationError(
            "Cannot take log of non-positive number".to_string(),
        ));
    }

    if base_f64 <= 0.0 || base_f64 == 1.0 {
        return Err(FhirPathError::EvaluationError(
            "Log base must be positive and not equal to 1".to_string(),
        ));
    }

    // Calculate log_base(value) = ln(value) / ln(base)
    let result = value_f64.ln() / base_f64.ln();
    Ok(FhirPathValue::Decimal(result))
}

fn evaluate_power_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let (base, exponent) = if arguments.len() == 1 {
        // Method call syntax: value.power(exponent)
        // Use this_item as base and first argument as exponent
        if let Some(this_item) = &context.this_item {
            let base = match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'power' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            };
            let exponent = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
            (base, exponent)
        } else {
            return Err(FhirPathError::EvaluationError(
                "'power' function expects 2 arguments or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 2 {
        // Function call syntax: power(base, exponent)
        let base = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;
        let exponent = evaluate_ast_with_visitor(&arguments[1], context, visitor)?;
        (base, exponent)
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'power' function expects 1 or 2 arguments, got {}",
            arguments.len()
        )));
    };

    match (base, exponent) {
        (FhirPathValue::Integer(b), FhirPathValue::Integer(e)) => {
            let base_f64 = b as f64;
            let exp_f64 = e as f64;

            // Check for invalid power operations
            if base_f64 == 0.0 && exp_f64 == 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "0^0 is undefined".to_string(),
                ));
            }
            if base_f64 < 0.0 && exp_f64.fract() != 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "Cannot raise negative number to fractional power".to_string(),
                ));
            }

            let result = base_f64.powf(exp_f64);
            if result.is_nan() || result.is_infinite() {
                return Err(FhirPathError::EvaluationError(
                    "Power operation resulted in invalid value".to_string(),
                ));
            }

            Ok(FhirPathValue::Decimal(result))
        }
        (FhirPathValue::Integer(b), FhirPathValue::Decimal(e)) => {
            let base_f64 = b as f64;

            // Check for invalid power operations
            if base_f64 == 0.0 && e == 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "0^0 is undefined".to_string(),
                ));
            }
            if base_f64 < 0.0 && e.fract() != 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "Cannot raise negative number to fractional power".to_string(),
                ));
            }

            let result = base_f64.powf(e);
            if result.is_nan() || result.is_infinite() {
                return Err(FhirPathError::EvaluationError(
                    "Power operation resulted in invalid value".to_string(),
                ));
            }

            Ok(FhirPathValue::Decimal(result))
        }
        (FhirPathValue::Decimal(b), FhirPathValue::Integer(e)) => {
            let exp_f64 = e as f64;

            // Check for invalid power operations
            if b == 0.0 && exp_f64 == 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "0^0 is undefined".to_string(),
                ));
            }
            if b < 0.0 && exp_f64.fract() != 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "Cannot raise negative number to fractional power".to_string(),
                ));
            }

            let result = b.powf(exp_f64);
            if result.is_nan() || result.is_infinite() {
                return Err(FhirPathError::EvaluationError(
                    "Power operation resulted in invalid value".to_string(),
                ));
            }

            Ok(FhirPathValue::Decimal(result))
        }
        (FhirPathValue::Decimal(b), FhirPathValue::Decimal(e)) => {
            // Check for invalid power operations
            if b == 0.0 && e == 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "0^0 is undefined".to_string(),
                ));
            }
            if b < 0.0 && e.fract() != 0.0 {
                return Err(FhirPathError::EvaluationError(
                    "Cannot raise negative number to fractional power".to_string(),
                ));
            }

            let result = b.powf(e);
            if result.is_nan() || result.is_infinite() {
                return Err(FhirPathError::EvaluationError(
                    "Power operation resulted in invalid value".to_string(),
                ));
            }

            Ok(FhirPathValue::Decimal(result))
        }
        _ => Err(FhirPathError::TypeError(
            "'power' function can only be applied to numbers".to_string(),
        )),
    }
}

fn evaluate_truncate_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    // If no arguments, apply to the current collection
    let result = if arguments.is_empty() {
        // Get the current collection from context
        let collection = get_current_collection(context)?;
        let mut results = Vec::new();

        for item in collection {
            match item {
                FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                FhirPathValue::Decimal(d) => results.push(FhirPathValue::Integer(d.trunc() as i64)),
                _ => {
                    return Err(FhirPathError::TypeError(
                        "'truncate' function can only be applied to numbers".to_string(),
                    ))
                }
            }
        }

        if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(results)
        }
    } else if arguments.len() == 1 {
        let result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

        match result {
            FhirPathValue::Integer(i) => FhirPathValue::Integer(i),
            FhirPathValue::Decimal(d) => FhirPathValue::Integer(d.trunc() as i64),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i)),
                        FhirPathValue::Decimal(d) => {
                            results.push(FhirPathValue::Integer(d.trunc() as i64))
                        }
                        _ => {
                            return Err(FhirPathError::TypeError(
                                "'truncate' function can only be applied to numbers".to_string(),
                            ))
                        }
                    }
                }
                FhirPathValue::Collection(results)
            }
            _ => {
                return Err(FhirPathError::TypeError(
                    "'truncate' function can only be applied to numbers".to_string(),
                ))
            }
        }
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'truncate' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    Ok(result)
}

fn evaluate_type_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let result = if arguments.is_empty() {
        // Method call syntax: value.type()
        // Use this_item as the value to get type of
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'type' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'type' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: type(value)
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'type' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    let (namespace, name) = match result {
        FhirPathValue::Boolean(_) => ("System", "Boolean"),
        FhirPathValue::Integer(_) => ("System", "Integer"),
        FhirPathValue::Decimal(_) => ("System", "Decimal"),
        FhirPathValue::String(_) => ("System", "String"),
        FhirPathValue::Date(_) => ("System", "Date"),
        FhirPathValue::DateTime(_) => ("System", "DateTime"),
        FhirPathValue::Time(_) => ("System", "Time"),
        FhirPathValue::Quantity { .. } => ("System", "Quantity"),
        FhirPathValue::Collection(_) => ("System", "Collection"),
        FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        FhirPathValue::Resource(ref resource) => {
            if let Some(resource_type) = &resource.resource_type {
                ("FHIR", resource_type.as_str())
            } else {
                ("FHIR", "Resource")
            }
        }
    };

    // Create a type object with namespace and name properties
    let mut type_properties = std::collections::HashMap::new();
    type_properties.insert(
        "namespace".to_string(),
        serde_json::Value::String(namespace.to_string()),
    );
    type_properties.insert(
        "name".to_string(),
        serde_json::Value::String(name.to_string()),
    );

    let type_resource = FhirResource {
        resource_type: None,
        properties: type_properties,
    };

    Ok(FhirPathValue::Resource(type_resource))
}

fn evaluate_extension_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'extension' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    let url_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

    let url = match url_result {
        FhirPathValue::String(s) => s,
        _ => {
            return Err(FhirPathError::TypeError(
                "'extension' function requires a string URL argument".to_string(),
            ))
        }
    };

    // Get the current resource/object from context
    match &context.context {
        serde_json::Value::Object(obj) => {
            if let Some(extensions) = obj.get("extension") {
                if let serde_json::Value::Array(ext_array) = extensions {
                    let mut matching_extensions = Vec::new();

                    for ext in ext_array {
                        if let serde_json::Value::Object(ext_obj) = ext {
                            if let Some(ext_url) = ext_obj.get("url") {
                                if let serde_json::Value::String(ext_url_str) = ext_url {
                                    if ext_url_str == &url {
                                        matching_extensions
                                            .push(json_to_fhirpath_value(ext.clone())?);
                                    }
                                }
                            }
                        }
                    }

                    if matching_extensions.is_empty() {
                        Ok(FhirPathValue::Empty)
                    } else if matching_extensions.len() == 1 {
                        Ok(matching_extensions.into_iter().next().unwrap())
                    } else {
                        Ok(FhirPathValue::Collection(matching_extensions))
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            } else {
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty),
    }
}

fn evaluate_of_type_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'ofType' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    let type_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

    let target_type = match type_result {
        FhirPathValue::String(s) => s,
        _ => {
            return Err(FhirPathError::TypeError(
                "'ofType' function requires a string type argument".to_string(),
            ))
        }
    };

    // Get the current collection from context
    let collection = get_current_collection(context)?;
    let mut filtered_results = Vec::new();

    for item in collection {
        let item_type = match &item {
            FhirPathValue::Boolean(_) => "System.Boolean",
            FhirPathValue::Integer(_) => "System.Integer",
            FhirPathValue::Decimal(_) => "System.Decimal",
            FhirPathValue::String(_) => "System.String",
            FhirPathValue::Date(_) => "System.Date",
            FhirPathValue::DateTime(_) => "System.DateTime",
            FhirPathValue::Time(_) => "System.Time",
            FhirPathValue::Quantity { .. } => "System.Quantity",
            FhirPathValue::Collection(_) => "System.Collection",
            FhirPathValue::Empty => continue,
            FhirPathValue::Resource(_) => "FHIR.Resource",
        };

        if item_type == target_type {
            filtered_results.push(item);
        }
    }

    if filtered_results.is_empty() {
        Ok(FhirPathValue::Empty)
    } else {
        Ok(FhirPathValue::Collection(filtered_results))
    }
}

fn evaluate_conforms_to_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'conformsTo' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    let _profile_result = evaluate_ast_with_visitor(&arguments[0], context, visitor)?;

    // For now, return a simple implementation that always returns true
    // In a full implementation, this would check if the resource conforms to the given profile
    Ok(FhirPathValue::Boolean(true))
}

fn evaluate_now_function(
    arguments: &[AstNode],
    _context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'now' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    // Return current datetime in ISO 8601 format
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| FhirPathError::EvaluationError(format!("System time error: {}", e)))?;

    // Convert to a basic ISO 8601 datetime string
    // This is a simplified implementation - in production you'd want proper datetime handling
    let secs = now.as_secs();
    let days_since_epoch = secs / 86400;
    let days_since_1970 = days_since_epoch;

    // Approximate calculation for current date/time
    // This is simplified - proper implementation would use chrono or similar
    let year = 1970 + (days_since_1970 / 365);
    let remaining_days = days_since_1970 % 365;
    let month = (remaining_days / 30) + 1;
    let day = (remaining_days % 30) + 1;

    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let datetime_str = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        month.min(12),
        day.min(31),
        hours,
        minutes,
        seconds
    );

    Ok(FhirPathValue::DateTime(datetime_str))
}

fn evaluate_today_function(
    arguments: &[AstNode],
    _context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'today' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    // Return current date in ISO 8601 format
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| FhirPathError::EvaluationError(format!("System time error: {}", e)))?;

    // Convert to a basic ISO 8601 date string
    let secs = now.as_secs();
    let days_since_epoch = secs / 86400;
    let days_since_1970 = days_since_epoch;

    // Approximate calculation for current date
    let year = 1970 + (days_since_1970 / 365);
    let remaining_days = days_since_1970 % 365;
    let month = (remaining_days / 30) + 1;
    let day = (remaining_days % 30) + 1;

    let date_str = format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31));

    Ok(FhirPathValue::Date(date_str))
}

fn evaluate_time_of_day_function(
    _arguments: &[AstNode],
    _context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    Err(FhirPathError::NotImplemented(
        "'timeOfDay' function not yet implemented".to_string(),
    ))
}

/// Evaluates the not() function
fn evaluate_not_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let result = if arguments.is_empty() {
        // Method call syntax: value.not()
        // Use this_item as the value to negate
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'not' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'not' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: not(value)
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'not' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match result {
        FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
        FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
        FhirPathValue::Collection(ref items) if items.is_empty() => {
            Ok(FhirPathValue::Boolean(true))
        }
        // Implement proper FHIRPath truthiness logic
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Boolean(i == 0)),
        FhirPathValue::Decimal(d) => Ok(FhirPathValue::Boolean(d == 0.0)),
        FhirPathValue::String(s) => Ok(FhirPathValue::Boolean(s.is_empty())),
        _ => Ok(FhirPathValue::Boolean(false)),
    }
}

/// Evaluates the all() function - returns true if all items in the collection satisfy the given condition
fn evaluate_all_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'all' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;
    let total = collection.len();

    // If collection is empty, all() returns true (vacuous truth)
    if collection.is_empty() {
        return Ok(FhirPathValue::Boolean(true));
    }

    // Evaluate the condition for each item in the collection
    for (idx, item) in collection.into_iter().enumerate() {
        // Create iteration context for this item
        let mut iteration_context = context.create_iteration_context(item, idx, total)?;

        // Evaluate the condition expression
        let condition_result =
            evaluate_ast_with_visitor(&arguments[0], &iteration_context, visitor)?;

        // Check if the condition is truthy
        if !is_truthy(&condition_result) {
            return Ok(FhirPathValue::Boolean(false));
        }
    }

    Ok(FhirPathValue::Boolean(true))
}

/// Evaluates the allTrue() function
fn evaluate_all_true_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    _visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(
            "'allTrue' function expects no arguments".to_string(),
        ));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        match item {
            FhirPathValue::Boolean(false) => return Ok(FhirPathValue::Boolean(false)),
            FhirPathValue::Boolean(true) => continue,
            FhirPathValue::Empty => continue, // Empty values are ignored
            _ => return Ok(FhirPathValue::Boolean(false)), // Non-boolean values make it false
        }
    }

    Ok(FhirPathValue::Boolean(true))
}

/// Evaluates the anyTrue() function
fn evaluate_any_true_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    _visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(
            "'anyTrue' function expects no arguments".to_string(),
        ));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        match item {
            FhirPathValue::Boolean(true) => return Ok(FhirPathValue::Boolean(true)),
            FhirPathValue::Boolean(false) => continue,
            FhirPathValue::Empty => continue, // Empty values are ignored
            _ => continue,                    // Non-boolean values are ignored for anyTrue
        }
    }

    Ok(FhirPathValue::Boolean(false))
}

/// Evaluates the allFalse() function
fn evaluate_all_false_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    _visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(
            "'allFalse' function expects no arguments".to_string(),
        ));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        match item {
            FhirPathValue::Boolean(true) => return Ok(FhirPathValue::Boolean(false)),
            FhirPathValue::Boolean(false) => continue,
            FhirPathValue::Empty => continue, // Empty values are ignored
            _ => return Ok(FhirPathValue::Boolean(false)), // Non-boolean values make it false
        }
    }

    Ok(FhirPathValue::Boolean(true))
}

/// Evaluates the anyFalse() function
fn evaluate_any_false_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    _visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(
            "'anyFalse' function expects no arguments".to_string(),
        ));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    for item in collection {
        match item {
            FhirPathValue::Boolean(false) => return Ok(FhirPathValue::Boolean(true)),
            FhirPathValue::Boolean(true) => continue,
            FhirPathValue::Empty => continue, // Empty values are ignored
            _ => continue,                    // Non-boolean values are ignored for anyFalse
        }
    }

    Ok(FhirPathValue::Boolean(false))
}

/// Evaluates the convertsToInteger() function
fn evaluate_converts_to_integer_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let result = if arguments.is_empty() {
        // Use current collection when no arguments provided
        let current_collection = get_current_collection(context)?;
        if current_collection.len() == 1 {
            current_collection[0].clone()
        } else if current_collection.is_empty() {
            FhirPathValue::Empty
        } else {
            FhirPathValue::Collection(current_collection)
        }
    } else if arguments.len() == 1 {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToInteger' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    let can_convert = match result {
        FhirPathValue::Integer(_) => true,
        FhirPathValue::Decimal(d) => d.fract() == 0.0, // Whole number decimals can be converted to integer
        FhirPathValue::String(s) => s.parse::<i64>().is_ok(),
        FhirPathValue::Boolean(_) => true,
        _ => false,
    };

    Ok(FhirPathValue::Boolean(can_convert))
}

/// Evaluates the convertsToString() function
fn evaluate_converts_to_string_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let _result = if arguments.is_empty() {
        // Use current context value when no arguments provided
        json_to_fhirpath_value(context.context.clone())?
    } else if arguments.len() == 1 {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToString' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    // Most values can be converted to string
    Ok(FhirPathValue::Boolean(true))
}

/// Evaluates the convertsToBoolean() function
fn evaluate_converts_to_boolean_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() > 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToBoolean' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    }

    let result = if arguments.is_empty() {
        // Use current collection when no arguments provided
        let current_collection = get_current_collection(context)?;
        if current_collection.len() == 1 {
            current_collection[0].clone()
        } else if current_collection.is_empty() {
            FhirPathValue::Empty
        } else {
            FhirPathValue::Collection(current_collection)
        }
    } else {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    };

    let can_convert = match result {
        FhirPathValue::Boolean(_) => true,
        FhirPathValue::String(s) => {
            let s_lower = s.to_lowercase();
            s_lower == "true" || s_lower == "false"
        },
        FhirPathValue::Integer(i) => i == 0 || i == 1,
        FhirPathValue::Decimal(d) => d == 0.0 || d == 1.0,
        FhirPathValue::Collection(ref items) => {
            items.len() == 1
                && match &items[0] {
                    FhirPathValue::Boolean(_) => true,
                    FhirPathValue::String(s) => {
                        let s_lower = s.to_lowercase();
                        s_lower == "true" || s_lower == "false"
                    },
                    FhirPathValue::Integer(i) => *i == 0 || *i == 1,
                    FhirPathValue::Decimal(d) => *d == 0.0 || *d == 1.0,
                    _ => false,
                }
        }
        _ => false,
    };

    Ok(FhirPathValue::Boolean(can_convert))
}

/// Evaluates the convertsToDecimal() function
fn evaluate_converts_to_decimal_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() > 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToDecimal' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    }

    let result = if arguments.is_empty() {
        // Use current collection when no arguments provided
        let current_collection = get_current_collection(context)?;
        if current_collection.len() == 1 {
            current_collection[0].clone()
        } else if current_collection.is_empty() {
            FhirPathValue::Empty
        } else {
            FhirPathValue::Collection(current_collection)
        }
    } else {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    };

    let can_convert = match result {
        FhirPathValue::Decimal(_) => true,
        FhirPathValue::Integer(_) => true,
        FhirPathValue::Boolean(_) => true,
        FhirPathValue::String(s) => s.parse::<f64>().is_ok(),
        FhirPathValue::Collection(ref items) => {
            items.len() == 1
                && match &items[0] {
                    FhirPathValue::Decimal(_) => true,
                    FhirPathValue::Integer(_) => true,
                    FhirPathValue::Boolean(_) => true,
                    FhirPathValue::String(s) => s.parse::<f64>().is_ok(),
                    _ => false,
                }
        }
        _ => false,
    };

    Ok(FhirPathValue::Boolean(can_convert))
}

/// Evaluates the convertsToDate() function
fn evaluate_converts_to_date_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let result = if arguments.is_empty() {
        // Use current this_item when no arguments provided (for expressions like '2015'.convertsToDate())
        context.this_item.clone().unwrap_or_else(|| {
            // Fallback to context if no this_item
            json_to_fhirpath_value(context.context.clone()).unwrap_or(FhirPathValue::Empty)
        })
    } else if arguments.len() == 1 {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToDate' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    println!(
        "[DEBUG] convertsToDate: result type = {:?}",
        std::mem::discriminant(&result)
    );

    let can_convert = match result {
        FhirPathValue::Date(_) => {
            println!("[DEBUG] convertsToDate: Found Date value");
            true
        }
        FhirPathValue::DateTime(_) => {
            println!("[DEBUG] convertsToDate: Found DateTime value");
            true
        }
        FhirPathValue::String(s) => {
            println!("[DEBUG] convertsToDate: Found String value: '{}'", s);
            // Use comprehensive date validation that handles YYYY, YYYY-MM, YYYY-MM-DD formats
            let is_valid_dt = is_valid_datetime_string(&s);
            let has_no_t = !s.contains('T');
            println!("[DEBUG] convertsToDate: '{}' -> is_valid_datetime_string: {}, !contains('T'): {}, length: {}", s, is_valid_dt, has_no_t, s.len());

            // Debug the validation step by step
            if !is_valid_dt {
                println!("[DEBUG] convertsToDate: '{}' failed datetime validation", s);
                if s.len() >= 4 {
                    let year_part = &s[0..4];
                    let year_valid = year_part.chars().all(|c| c.is_ascii_digit());
                    println!(
                        "[DEBUG] convertsToDate: year_part '{}' valid: {}",
                        year_part, year_valid
                    );

                    if s.len() == 7 && s.chars().nth(4) == Some('-') {
                        let month_part = &s[5..7];
                        let month_valid = month_part.chars().all(|c| c.is_ascii_digit());
                        let month: u32 = month_part.parse().unwrap_or(0);
                        let month_range_valid = month >= 1 && month <= 12;
                        println!("[DEBUG] convertsToDate: month_part '{}' valid: {}, value: {}, range_valid: {}", month_part, month_valid, month, month_range_valid);
                    }
                }
            }

            is_valid_dt && has_no_t // Date only, not DateTime
        }
        FhirPathValue::Collection(ref items) => {
            println!(
                "[DEBUG] convertsToDate: Found Collection with {} items",
                items.len()
            );
            if items.len() == 1 {
                match &items[0] {
                    FhirPathValue::String(s) => {
                        println!(
                            "[DEBUG] convertsToDate: Collection contains String: '{}'",
                            s
                        );
                        let is_valid_dt = is_valid_datetime_string(s);
                        let has_no_t = !s.contains('T');
                        println!("[DEBUG] convertsToDate: '{}' -> is_valid_datetime_string: {}, !contains('T'): {}", s, is_valid_dt, has_no_t);
                        is_valid_dt && has_no_t
                    }
                    _ => {
                        println!(
                            "[DEBUG] convertsToDate: Collection contains non-string: {:?}",
                            items[0]
                        );
                        false
                    }
                }
            } else {
                false
            }
        }
        _ => {
            println!("[DEBUG] convertsToDate: Found other type: {:?}", result);
            false
        }
    };

    Ok(FhirPathValue::Boolean(can_convert))
}

/// Evaluates the convertsToDateTime() function
fn evaluate_converts_to_date_time_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let result = if arguments.is_empty() {
        // Use current this_item when no arguments provided (for expressions like '2015'.convertsToDateTime())
        context.this_item.clone().unwrap_or_else(|| {
            // Fallback to current collection if no this_item
            let current_collection = get_current_collection(context).unwrap_or_default();
            if current_collection.len() == 1 {
                current_collection[0].clone()
            } else if current_collection.is_empty() {
                FhirPathValue::Empty
            } else {
                FhirPathValue::Collection(current_collection)
            }
        })
    } else if arguments.len() == 1 {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToDateTime' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    let can_convert = match result {
        FhirPathValue::DateTime(_) => true,
        FhirPathValue::Date(_) => true,
        FhirPathValue::String(s) => is_valid_datetime_string(&s),
        _ => false,
    };

    Ok(FhirPathValue::Boolean(can_convert))
}

/// Converts a string to a DateTime value if possible
pub fn string_to_datetime(s: &str) -> Option<FhirPathValue> {
    if !is_valid_datetime_string(s) {
        return None;
    }

    // Handle time-only formats (starting with T)
    if s.starts_with('T') {
        return Some(FhirPathValue::Time(s.to_string()));
    }

    // Check if it contains 'T' to determine if it's a DateTime or Date
    if s.contains('T') {
        Some(FhirPathValue::DateTime(s.to_string()))
    } else {
        Some(FhirPathValue::Date(s.to_string()))
    }
}

/// Evaluates the convertsToQuantity() function
fn evaluate_converts_to_quantity_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let result = if arguments.is_empty() {
        // Use current this_item when no arguments provided (for expressions like '5.0'.convertsToQuantity())
        context.this_item.clone().unwrap_or_else(|| {
            // Fallback to context if no this_item
            json_to_fhirpath_value(context.context.clone()).unwrap_or(FhirPathValue::Empty)
        })
    } else if arguments.len() == 1 {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToQuantity' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    let can_convert = match result {
        FhirPathValue::Quantity { .. } => true,
        FhirPathValue::Integer(_) => true,
        FhirPathValue::Decimal(_) => true,
        FhirPathValue::Boolean(_) => true,
        FhirPathValue::String(s) => {
            // Try to parse as FHIR Quantity using ucum-fhir for better validation
            if let Some((value_str, unit_str, is_quoted)) = parse_quantity_string(&s) {
                if let Ok(value) = value_str.parse::<f64>() {
                    // Check for units that should be rejected according to FHIRPath specification
                    // Quoted units like 'wk' are accepted, unquoted units like wk are rejected
                    let fhirpath_invalid_units = [
                        "wk",    // week - not a valid UCUM unit in FHIRPath context (unless quoted)
                        "week",  // week spelled out
                        "weeks", // plural form
                    ];

                    if !is_quoted && fhirpath_invalid_units.contains(&unit_str) {
                        false
                    } else {
                        // Create a FHIR Quantity and validate it using UCUM
                        let fhir_quantity = FhirQuantity::with_ucum_code(value, unit_str);
                        // Try to convert to UCUM quantity to validate
                        match fhir_quantity.to_ucum_quantity() {
                            Ok(_) => true,
                            Err(_) => false,
                        }
                    }
                } else {
                    false
                }
            } else {
                // Fallback: try simple number parsing for dimensionless quantities
                s.trim().parse::<f64>().is_ok()
            }
        }
        _ => false,
    };

    Ok(FhirPathValue::Boolean(can_convert))
}

/// Evaluates the convertsToTime() function
fn evaluate_converts_to_time_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let result = if arguments.is_empty() {
        // Use current this_item when no arguments provided (for expressions like 'T14:30:00'.convertsToTime())
        context.this_item.clone().unwrap_or_else(|| {
            // Fallback to context if no this_item
            json_to_fhirpath_value(context.context.clone()).unwrap_or(FhirPathValue::Empty)
        })
    } else if arguments.len() == 1 {
        evaluate_ast_with_visitor(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'convertsToTime' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    let can_convert = match result {
        FhirPathValue::Time(_) => true,
        FhirPathValue::String(s) => {
            // Use comprehensive time validation that handles HH, HH:MM, HH:MM:SS formats
            let is_valid_time = is_valid_time_string(&s);
            println!(
                "[DEBUG] convertsToTime: '{}' -> is_valid_time_string: {}",
                s, is_valid_time
            );
            is_valid_time
        }
        _ => false,
    };

    Ok(FhirPathValue::Boolean(can_convert))
}

/// Helper function to get the current collection from context
fn get_current_collection(
    context: &EvaluationContext,
) -> Result<Vec<FhirPathValue>, FhirPathError> {
    match &context.this_item {
        Some(FhirPathValue::Collection(items)) => Ok(items.clone()),
        Some(FhirPathValue::Empty) => Ok(vec![]), // Empty should result in empty collection
        Some(item) => Ok(vec![item.clone()]),
        None => {
            // Try to get from the main context
            match &context.context {
                serde_json::Value::Array(arr) => {
                    let mut items = Vec::new();
                    for value in arr {
                        items.push(json_to_fhirpath_value(value.clone())?);
                    }
                    Ok(items)
                }
                value => Ok(vec![json_to_fhirpath_value(value.clone())?]),
            }
        }
    }
}

/// Helper function to check if a value is truthy
fn is_truthy(value: &FhirPathValue) -> bool {
    match value {
        FhirPathValue::Empty => false,
        FhirPathValue::Boolean(b) => *b,
        FhirPathValue::Integer(i) => *i != 0,
        FhirPathValue::Decimal(d) => *d != 0.0,
        FhirPathValue::String(s) => !s.is_empty(),
        FhirPathValue::Collection(items) => !items.is_empty(),
        _ => true,
    }
}

/// Helper function to parse FHIRPath quantity strings like "5.4 'mg'" or "10 kg"
/// Returns (value_str, unit_str, is_quoted) if parsing is successful
fn parse_quantity_string(s: &str) -> Option<(&str, &str, bool)> {
    let s = s.trim();

    // Handle quoted units like "5.4 'mg'"
    if let Some(quote_start) = s.find('\'') {
        if let Some(quote_end) = s.rfind('\'') {
            if quote_start < quote_end {
                let value_part = s[..quote_start].trim();
                let unit_part = &s[quote_start + 1..quote_end];
                if !value_part.is_empty() && !unit_part.is_empty() {
                    return Some((value_part, unit_part, true)); // is_quoted = true
                }
            }
        }
    }

    // Handle space-separated units like "10 kg"
    if let Some(space_pos) = s.find(' ') {
        let value_part = s[..space_pos].trim();
        let unit_part = s[space_pos + 1..].trim();
        if !value_part.is_empty() && !unit_part.is_empty() {
            // Check if value part is a valid number
            if value_part.parse::<f64>().is_ok() {
                return Some((value_part, unit_part, false)); // is_quoted = false
            }
        }
    }

    None
}

/// Helper function to validate datetime string formats
pub fn is_valid_datetime_string(s: &str) -> bool {
    // Valid datetime formats according to FhirPath specification:
    // YYYY
    // YYYY-MM
    // YYYY-MM-DD
    // YYYY-MM-DDTHH
    // YYYY-MM-DDTHH:MM
    // YYYY-MM-DDTHH:MM:SS
    // YYYY-MM-DDTHH:MM:SS.SSS
    // With optional timezone: Z or +/-HH:MM
    // Time-only formats:
    // THH
    // THH:MM
    // THH:MM:SS
    // THH:MM:SS.SSS

    if s.is_empty() {
        return false;
    }

    // Handle time-only formats (starting with T)
    if s.starts_with('T') {
        return is_valid_time_string(&s[1..]);
    }

    // Check for year (YYYY)
    if s.len() >= 4 {
        let year_part = &s[0..4];
        if !year_part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // If it's just a year, it's valid
        if s.len() == 4 {
            return true;
        }

        // Check for month (YYYY-MM)
        if s.len() >= 7 && s.chars().nth(4) == Some('-') {
            let month_part = &s[5..7];
            if !month_part.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            let month: u32 = month_part.parse().unwrap_or(0);
            if month < 1 || month > 12 {
                return false;
            }

            // If it's just year-month, it's valid
            if s.len() == 7 {
                return true;
            }

            // Check for day (YYYY-MM-DD)
            if s.len() >= 10 && s.chars().nth(7) == Some('-') {
                let day_part = &s[8..10];
                if !day_part.chars().all(|c| c.is_ascii_digit()) {
                    return false;
                }
                let day: u32 = day_part.parse().unwrap_or(0);
                if day < 1 || day > 31 {
                    return false;
                }

                // If it's just year-month-day, it's valid
                if s.len() == 10 {
                    return true;
                }

                // Check for time part (T)
                if s.len() >= 11 && s.chars().nth(10) == Some('T') {
                    // Validate the time part
                    return is_valid_time_string(&s[11..]);
                }
            }
        }
    }

    false
}

/// Helper function to validate time string formats
fn is_valid_time_string(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check for hours (HH)
    if s.len() >= 2 {
        let hours_part = &s[0..2];
        if !hours_part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        let hours: u32 = hours_part.parse().unwrap_or(24);
        if hours > 23 {
            return false;
        }

        // If it's just hours, it's valid
        if s.len() == 2 {
            return true;
        }

        // Check for minutes (HH:MM)
        if s.len() >= 5 && s.chars().nth(2) == Some(':') {
            let minutes_part = &s[3..5];
            if !minutes_part.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            let minutes: u32 = minutes_part.parse().unwrap_or(60);
            if minutes > 59 {
                return false;
            }

            // If it's just hours:minutes, it's valid
            if s.len() == 5 {
                return true;
            }

            // Check for seconds (HH:MM:SS)
            if s.len() >= 8 && s.chars().nth(5) == Some(':') {
                let seconds_part = &s[6..8];
                if !seconds_part.chars().all(|c| c.is_ascii_digit()) {
                    return false;
                }
                let seconds: u32 = seconds_part.parse().unwrap_or(60);
                if seconds > 59 {
                    return false;
                }

                // If it's just hours:minutes:seconds, it's valid
                if s.len() == 8 {
                    return true;
                }

                // Check for milliseconds (HH:MM:SS.SSS)
                if s.len() > 9 && s.chars().nth(8) == Some('.') {
                    let ms_part = &s[9..];
                    // Check if all remaining characters are digits (before timezone)
                    let ms_end = ms_part
                        .find(|c| c == 'Z' || c == '+' || c == '-')
                        .unwrap_or(ms_part.len());
                    if !ms_part[..ms_end].chars().all(|c| c.is_ascii_digit()) {
                        return false;
                    }

                    // Check for timezone
                    if ms_end < ms_part.len() {
                        return is_valid_timezone(&ms_part[ms_end..]);
                    }
                    return true;
                }

                // Check for timezone after seconds
                if s.len() > 8 {
                    return is_valid_timezone(&s[8..]);
                }
            }
        }
    }

    false
}

/// Helper function to validate timezone formats
fn is_valid_timezone(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Z timezone
    if s == "Z" {
        return true;
    }

    // +/-HH:MM timezone
    if s.len() >= 3 && (s.starts_with('+') || s.starts_with('-')) {
        // Check hours part
        if s.len() >= 3 {
            let hours_part = &s[1..3];
            if !hours_part.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            let hours: u32 = hours_part.parse().unwrap_or(24);
            if hours > 23 {
                return false;
            }

            // If it's just +/-HH, it's valid
            if s.len() == 3 {
                return true;
            }

            // Check for colon
            if s.len() >= 4 && s.chars().nth(3) != Some(':') {
                return false;
            }

            // Check minutes part
            if s.len() >= 6 {
                let minutes_part = &s[4..6];
                if !minutes_part.chars().all(|c| c.is_ascii_digit()) {
                    return false;
                }
                let minutes: u32 = minutes_part.parse().unwrap_or(60);
                if minutes > 59 {
                    return false;
                }

                // If it's +/-HH:MM, it's valid
                if s.len() == 6 {
                    return true;
                }
            }
        }
    }

    false
}

/// DateTime precision levels for equivalence comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateTimePrecision {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
}

/// Helper function to determine the precision level of a datetime string
fn get_datetime_precision(dt: &str) -> DateTimePrecision {
    // Remove @ prefix if present
    let dt_clean = dt.strip_prefix('@').unwrap_or(dt);

    // Handle time-only values (start with T)
    if dt_clean.starts_with('T') {
        let time_part = &dt_clean[1..];
        return get_time_precision(time_part);
    }

    // Handle date/datetime values
    if let Some(t_pos) = dt_clean.find('T') {
        // Has time component - check time precision
        let time_part = &dt_clean[t_pos + 1..];
        return get_time_precision(time_part);
    } else {
        // Date only - check date precision
        return get_date_precision(dt_clean);
    }
}

/// Helper function to determine precision of date part
fn get_date_precision(date: &str) -> DateTimePrecision {
    let dash_count = date.matches('-').count();
    match dash_count {
        0 => DateTimePrecision::Year,    // "2012"
        1 => DateTimePrecision::Month,   // "2012-04"
        2 => DateTimePrecision::Day,     // "2012-04-15"
        _ => DateTimePrecision::Day,     // Default to day for malformed dates
    }
}

/// Helper function to determine precision of time part
fn get_time_precision(time: &str) -> DateTimePrecision {
    // Remove timezone info for precision analysis
    let time_clean = if let Some(tz_pos) = time.find('+') {
        &time[..tz_pos]
    } else if let Some(tz_pos) = time.find('-') {
        &time[..tz_pos]
    } else if time.ends_with('Z') {
        &time[..time.len() - 1]
    } else {
        time
    };

    if time_clean.contains('.') {
        // Check if there are meaningful fractional seconds (not just .0)
        if let Some(dot_pos) = time_clean.find('.') {
            let fractional_part = &time_clean[dot_pos + 1..];
            // If fractional part is all zeros, it's still second precision
            if fractional_part.chars().all(|c| c == '0') {
                DateTimePrecision::Second
            } else {
                DateTimePrecision::Millisecond  // Has meaningful fractional seconds
            }
        } else {
            DateTimePrecision::Second
        }
    } else {
        let colon_count = time_clean.matches(':').count();
        match colon_count {
            0 => DateTimePrecision::Hour,    // "14"
            1 => DateTimePrecision::Minute,  // "14:30"
            2 => DateTimePrecision::Second,  // "14:30:31"
            _ => DateTimePrecision::Second,  // Default to second for malformed times
        }
    }
}

/// Helper function to expand a datetime string to a more specific precision
/// For example: "2012-04" (Month) -> "2012-04-01" (Day)
fn expand_datetime_to_precision(dt: &str, target_precision: DateTimePrecision) -> String {
    // Remove @ prefix if present
    let dt_clean = dt.strip_prefix('@').unwrap_or(dt);
    let mut result = dt_clean.to_string();

    // Handle time-only values (start with T)
    if dt_clean.starts_with('T') {
        let time_part = &dt_clean[1..];
        let expanded_time = expand_time_to_precision(time_part, target_precision);
        return format!("T{}", expanded_time);
    }

    // Handle date/datetime values
    if let Some(t_pos) = dt_clean.find('T') {
        // Has time component - expand both date and time parts
        let date_part = &dt_clean[..t_pos];
        let time_part = &dt_clean[t_pos + 1..];

        let expanded_date = expand_date_to_precision(date_part, target_precision);
        let expanded_time = expand_time_to_precision(time_part, target_precision);

        format!("{}T{}", expanded_date, expanded_time)
    } else {
        // Date only - expand date part and potentially add time
        let expanded_date = expand_date_to_precision(&result, target_precision);

        // If target precision requires time, add default time
        match target_precision {
            DateTimePrecision::Hour => format!("{}T00", expanded_date),
            DateTimePrecision::Minute => format!("{}T00:00", expanded_date),
            DateTimePrecision::Second => format!("{}T00:00:00", expanded_date),
            DateTimePrecision::Millisecond => format!("{}T00:00:00.000", expanded_date),
            _ => expanded_date,
        }
    }
}

/// Helper function to expand date part to target precision
fn expand_date_to_precision(date: &str, target_precision: DateTimePrecision) -> String {
    let mut result = date.to_string();

    match target_precision {
        DateTimePrecision::Year => result,
        DateTimePrecision::Month => {
            if !result.contains('-') {
                result.push_str("-01");
            }
            result
        },
        DateTimePrecision::Day | DateTimePrecision::Hour | DateTimePrecision::Minute |
        DateTimePrecision::Second | DateTimePrecision::Millisecond => {
            // Ensure we have month
            if !result.contains('-') {
                result.push_str("-01");
            }
            // Ensure we have day
            let dash_count = result.matches('-').count();
            if dash_count == 1 {
                result.push_str("-01");
            }
            result
        }
    }
}

/// Helper function to expand time part to target precision
fn expand_time_to_precision(time: &str, target_precision: DateTimePrecision) -> String {
    // Extract timezone info if present
    let (time_part, timezone) = if let Some(tz_pos) = time.find('+') {
        (&time[..tz_pos], Some(&time[tz_pos..]))
    } else if let Some(tz_pos) = time.find('-') {
        // Make sure this is timezone, not date separator
        if tz_pos > 2 {
            (&time[..tz_pos], Some(&time[tz_pos..]))
        } else {
            (time, None)
        }
    } else if time.ends_with('Z') {
        (&time[..time.len() - 1], Some("Z"))
    } else {
        (time, None)
    };

    let mut result = time_part.to_string();

    match target_precision {
        DateTimePrecision::Hour => {
            // Ensure we have at least hour format
            if !result.contains(':') && result.len() < 2 {
                result = format!("{:0>2}", result);
            }
        },
        DateTimePrecision::Minute => {
            // Ensure we have hour:minute format
            if !result.contains(':') {
                result = format!("{:0>2}:00", result);
            } else {
                let colon_count = result.matches(':').count();
                if colon_count == 0 {
                    result.push_str(":00");
                }
            }
        },
        DateTimePrecision::Second => {
            // Ensure we have hour:minute:second format
            let colon_count = result.matches(':').count();
            match colon_count {
                0 => result = format!("{:0>2}:00:00", result),
                1 => result.push_str(":00"),
                _ => {} // Already has seconds
            }
        },
        DateTimePrecision::Millisecond => {
            // Ensure we have hour:minute:second.millisecond format
            let colon_count = result.matches(':').count();
            match colon_count {
                0 => result = format!("{:0>2}:00:00.000", result),
                1 => result.push_str(":00.000"),
                2 => {
                    if !result.contains('.') {
                        result.push_str(".000");
                    }
                },
                _ => {} // Already has milliseconds
            }
        },
        _ => {} // Date precisions don't affect time
    }

    // Add timezone back if it was present
    if let Some(tz) = timezone {
        result.push_str(tz);
    }

    result
}

/// Helper function to compare DateTime values with precision and timezone handling
fn datetime_equal(a: &str, b: &str) -> bool {
    // Handle different precision levels and timezone rules
    // Examples: @2012-04 should equal @2012-04-01T00:00:00Z when considering precision

    // Remove @ prefix if present
    let a_clean = a.strip_prefix('@').unwrap_or(a);
    let b_clean = b.strip_prefix('@').unwrap_or(b);

    // If both are exactly the same, they're equal
    if a_clean == b_clean {
        return true;
    }

    // Determine if we're comparing time-only values
    let a_is_time = a_clean.starts_with('T');
    let b_is_time = b_clean.starts_with('T');

    // If one is time-only and the other is not, they can't be equal
    if a_is_time != b_is_time {
        return false;
    }

    // Handle time-only comparison
    if a_is_time && b_is_time {
        return normalize_time(&a_clean[1..]) == normalize_time(&b_clean[1..]);
    }

    // Handle date/datetime comparison
    // For equality (=), we need to handle different precision levels
    // According to FHIRPath spec, different precisions can be equal if they represent the same time period
    let precision_a = get_datetime_precision(a_clean);
    let precision_b = get_datetime_precision(b_clean);

    // Normalize both datetimes to the same precision level for comparison
    let normalized_a = normalize_datetime(a_clean);
    let normalized_b = normalize_datetime(b_clean);

    // For different precision levels, we need to check if they overlap
    // For example: @2012-04-15 should equal @2012-04-15T00:00:00 (start of day)
    if precision_a != precision_b {
        // Convert both to the more specific precision for comparison
        let (norm_a, norm_b) = if precision_a as u8 > precision_b as u8 {
            // a is more precise, check if b's range includes a
            (normalized_a, expand_datetime_to_precision(&normalized_b, precision_a))
        } else {
            // b is more precise, check if a's range includes b
            (expand_datetime_to_precision(&normalized_a, precision_b), normalized_b)
        };

        // For equality, the less precise datetime should match the start of the range
        norm_a == norm_b
    } else {
        // Same precision, direct comparison
        normalized_a == normalized_b
    }
}

/// Helper function to compare DateTime values for equivalence (~) with precision checking
fn datetime_equivalent(a: &str, b: &str) -> bool {
    // Handle different precision levels and timezone rules
    // For equivalence (~), precision levels must match

    // Remove @ prefix if present
    let a_clean = a.strip_prefix('@').unwrap_or(a);
    let b_clean = b.strip_prefix('@').unwrap_or(b);

    // If both are exactly the same, they're equivalent
    if a_clean == b_clean {
        return true;
    }

    // Determine if we're comparing time-only values
    let a_is_time = a_clean.starts_with('T');
    let b_is_time = b_clean.starts_with('T');

    // If one is time-only and the other is not, they can't be equivalent
    if a_is_time != b_is_time {
        return false;
    }

    // Handle time-only comparison
    if a_is_time && b_is_time {
        return normalize_time(&a_clean[1..]) == normalize_time(&b_clean[1..]);
    }

    // Handle date/datetime comparison
    // For equivalence (~), check if both have the same precision level
    // If precision levels differ, they are not equivalent
    let precision_a = get_datetime_precision(a_clean);
    let precision_b = get_datetime_precision(b_clean);

    // If precision levels are different, they cannot be equivalent
    if precision_a != precision_b {
        return false;
    }

    let normalized_a = normalize_datetime(a_clean);
    let normalized_b = normalize_datetime(b_clean);

    normalized_a == normalized_b
}

/// Helper function to convert datetime with timezone to UTC
fn convert_to_utc(dt: &str) -> String {
    // Handle different timezone formats: Z, +HH:MM, -HH:MM
    if dt.ends_with('Z') {
        // Already UTC, just remove the Z
        return dt[..dt.len() - 1].to_string();
    }

    // Find timezone offset
    let (base_dt, offset_str) = if let Some(plus_pos) = dt.find('+') {
        (&dt[..plus_pos], &dt[plus_pos + 1..])
    } else if let Some(minus_pos) = dt.rfind('-') {
        // Make sure this is a timezone offset, not a date separator
        if minus_pos > 10 && dt.len() > minus_pos + 3 {
            (&dt[..minus_pos], &dt[minus_pos + 1..])
        } else {
            // No timezone, return as-is
            return dt.to_string();
        }
    } else {
        // No timezone, return as-is
        return dt.to_string();
    };

    // Parse timezone offset (format: HH:MM or HHMM)
    let offset_minutes = if let Some(colon_pos) = offset_str.find(':') {
        // Format: HH:MM
        let hours: i32 = offset_str[..colon_pos].parse().unwrap_or(0);
        let minutes: i32 = offset_str[colon_pos + 1..].parse().unwrap_or(0);
        hours * 60 + minutes
    } else if offset_str.len() == 4 {
        // Format: HHMM
        let hours: i32 = offset_str[..2].parse().unwrap_or(0);
        let minutes: i32 = offset_str[2..].parse().unwrap_or(0);
        hours * 60 + minutes
    } else {
        0
    };

    // Determine if this was a positive or negative offset
    let is_negative = dt.contains(&format!("-{}", offset_str));
    let total_offset_minutes = if is_negative {
        -offset_minutes
    } else {
        offset_minutes
    };

    // Parse the base datetime to adjust it
    if let Some(t_pos) = base_dt.find('T') {
        let date_part = &base_dt[..t_pos];
        let time_part = &base_dt[t_pos + 1..];

        // Parse time components
        let time_components: Vec<&str> = time_part.split(':').collect();
        if time_components.len() >= 2 {
            let hours: i32 = time_components[0].parse().unwrap_or(0);
            let minutes: i32 = time_components[1].parse().unwrap_or(0);
            let seconds_part = if time_components.len() > 2 {
                time_components[2]
            } else {
                "00"
            };

            // Convert to total minutes and adjust for timezone
            let total_minutes = hours * 60 + minutes - total_offset_minutes;

            // Handle day overflow/underflow
            let (adjusted_hours, adjusted_minutes, day_offset) = if total_minutes < 0 {
                let abs_minutes = (-total_minutes) as u32;
                let days = abs_minutes / (24 * 60);
                let remaining_minutes = abs_minutes % (24 * 60);
                let new_total_minutes = 24 * 60 - remaining_minutes;
                (
                    (new_total_minutes / 60) as i32,
                    (new_total_minutes % 60) as i32,
                    -(days as i32) - 1,
                )
            } else if total_minutes >= 24 * 60 {
                let days = total_minutes / (24 * 60);
                let remaining_minutes = total_minutes % (24 * 60);
                (remaining_minutes / 60, remaining_minutes % 60, days)
            } else {
                (total_minutes / 60, total_minutes % 60, 0)
            };

            // For simplicity, if there's a day offset, we'll just use the original time
            // A full implementation would need proper date arithmetic
            if day_offset == 0 {
                return format!(
                    "{}T{:02}:{:02}:{}",
                    date_part, adjusted_hours, adjusted_minutes, seconds_part
                );
            }
        }
    }

    // Fallback: return the base datetime without timezone
    base_dt.to_string()
}

/// Helper function to normalize datetime strings for comparison
fn normalize_datetime(dt: &str) -> String {
    let mut normalized = dt.to_string();

    // Convert timezone to UTC FIRST, before any other normalization
    if normalized.contains('+') || normalized.contains('-') || normalized.ends_with('Z') {
        normalized = convert_to_utc(&normalized);
    }

    // Handle year-only precision: 2012 -> 2012-01-01T00:00:00
    if normalized.len() == 4 && normalized.chars().all(|c| c.is_ascii_digit()) {
        normalized = format!("{}-01-01T00:00:00", normalized);
    }
    // Handle year-month precision: 2012-04 -> 2012-04-01T00:00:00
    else if normalized.len() == 7 && normalized.matches('-').count() == 1 {
        normalized = format!("{}-01T00:00:00", normalized);
    }
    // Handle date precision: 2012-04-15 -> 2012-04-15T00:00:00
    else if normalized.len() == 10
        && normalized.matches('-').count() == 2
        && !normalized.contains('T')
    {
        normalized = format!("{}T00:00:00", normalized);
    }
    // Handle partial time formats
    else if normalized.contains('T') {
        let parts: Vec<&str> = normalized.split('T').collect();
        if parts.len() == 2 {
            let date_part = parts[0];
            let time_part = parts[1];

            // Normalize the time part (timezone already converted above)
            let normalized_time = normalize_time_without_timezone(time_part);
            normalized = format!("{}T{}", date_part, normalized_time);
        }
    }

    normalized
}

/// Helper function to normalize time strings for comparison (without timezone handling)
fn normalize_time_without_timezone(time: &str) -> String {
    let mut normalized = time.to_string();

    // Handle hours-only: HH -> HH:00:00
    if normalized.len() == 2 && normalized.chars().all(|c| c.is_ascii_digit()) {
        normalized = format!("{}:00:00", normalized);
    }
    // Handle hours-minutes: HH:MM -> HH:MM:00
    else if normalized.len() == 5 && normalized.chars().nth(2) == Some(':') {
        normalized = format!("{}:00", normalized);
    }

    // Handle milliseconds
    if let Some(ms_pos) = normalized.find('.') {
        normalized = normalized[..ms_pos].to_string();
    }

    normalized
}

/// Helper function to normalize time strings for comparison
fn normalize_time(time: &str) -> String {
    let mut normalized = time.to_string();

    // Handle hours-only: HH -> HH:00:00
    if normalized.len() == 2 && normalized.chars().all(|c| c.is_ascii_digit()) {
        normalized = format!("{}:00:00", normalized);
    }
    // Handle hours-minutes: HH:MM -> HH:MM:00
    else if normalized.len() == 5 && normalized.chars().nth(2) == Some(':') {
        normalized = format!("{}:00", normalized);
    }

    // Handle timezone
    if normalized.contains('+') || normalized.contains('-') || normalized.ends_with('Z') {
        // For now, just remove timezone info for comparison
        if let Some(tz_pos) = normalized.find('+') {
            normalized = normalized[..tz_pos].to_string();
        } else if let Some(tz_pos) = normalized.find('-') {
            normalized = normalized[..tz_pos].to_string();
        } else if normalized.ends_with('Z') {
            normalized = normalized[..normalized.len() - 1].to_string();
        }
    }

    // Handle milliseconds
    if let Some(ms_pos) = normalized.find('.') {
        normalized = normalized[..ms_pos].to_string();
    }

    normalized
}

/// Generates an efficient cache key for an AST node using hashing
fn generate_cache_key(node: &AstNode) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash_ast_node(node, &mut hasher);
    hasher.finish()
}

/// Determines if a node should be cached based on its complexity and potential for reuse
fn should_cache_node(node: &AstNode) -> bool {
    match node {
        // Don't cache simple literals - they're fast to evaluate
        AstNode::Identifier(_)
        | AstNode::StringLiteral(_)
        | AstNode::NumberLiteral { .. }
        | AstNode::BooleanLiteral(_)
        | AstNode::DateTimeLiteral(_)
        | AstNode::QuantityLiteral { .. }
        | AstNode::Variable(_) => false,

        // Cache complex path expressions that might be expensive
        AstNode::Path(_, _) => true,

        // Cache function calls as they can be expensive
        AstNode::FunctionCall { .. } => true,

        // Cache complex binary operations but not simple ones
        AstNode::BinaryOp { op, left, right } => {
            match op {
                // Don't cache simple arithmetic/comparison on literals
                BinaryOperator::Addition
                | BinaryOperator::Subtraction
                | BinaryOperator::Multiplication
                | BinaryOperator::Division
                | BinaryOperator::Div
                | BinaryOperator::Mod
                | BinaryOperator::Equals
                | BinaryOperator::NotEquals
                | BinaryOperator::Equivalent
                | BinaryOperator::NotEquivalent
                | BinaryOperator::LessThan
                | BinaryOperator::LessOrEqual
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterOrEqual => {
                    // Only cache if operands are complex
                    !is_simple_node(left) || !is_simple_node(right)
                }
                // Cache logical operations as they might involve complex subexpressions
                BinaryOperator::And
                | BinaryOperator::Or
                | BinaryOperator::Xor
                | BinaryOperator::Implies
                | BinaryOperator::In
                | BinaryOperator::Contains
                | BinaryOperator::Is
                | BinaryOperator::As
                | BinaryOperator::Union
                | BinaryOperator::Concatenation => true,
            }
        }

        // Don't cache simple unary operations
        AstNode::UnaryOp { operand, .. } => !is_simple_node(operand),

        // Cache indexing operations as they can be expensive
        AstNode::Indexer { .. } => true,
    }
}

/// Helper function to determine if a node is simple (fast to evaluate)
fn is_simple_node(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::Identifier(_)
            | AstNode::StringLiteral(_)
            | AstNode::NumberLiteral { .. }
            | AstNode::BooleanLiteral(_)
            | AstNode::DateTimeLiteral(_)
            | AstNode::QuantityLiteral { .. }
    )
}

/// Recursively hashes an AST node structure
fn hash_ast_node(node: &AstNode, hasher: &mut DefaultHasher) {
    match node {
        AstNode::Identifier(name) => {
            0u8.hash(hasher);
            name.hash(hasher);
        }
        AstNode::StringLiteral(value) => {
            1u8.hash(hasher);
            value.hash(hasher);
        }
        AstNode::NumberLiteral { value, is_decimal } => {
            2u8.hash(hasher);
            value.to_bits().hash(hasher);
            is_decimal.hash(hasher);
        }
        AstNode::BooleanLiteral(value) => {
            3u8.hash(hasher);
            value.hash(hasher);
        }
        AstNode::DateTimeLiteral(value) => {
            9u8.hash(hasher);
            value.hash(hasher);
        }
        AstNode::Variable(name) => {
            4u8.hash(hasher);
            name.hash(hasher);
        }
        AstNode::Path(left, right) => {
            4u8.hash(hasher);
            hash_ast_node(left, hasher);
            hash_ast_node(right, hasher);
        }
        AstNode::FunctionCall { name, arguments } => {
            5u8.hash(hasher);
            name.hash(hasher);
            arguments.len().hash(hasher);
            for arg in arguments {
                hash_ast_node(arg, hasher);
            }
        }
        AstNode::BinaryOp { op, left, right } => {
            6u8.hash(hasher);
            std::mem::discriminant(op).hash(hasher);
            hash_ast_node(left, hasher);
            hash_ast_node(right, hasher);
        }
        AstNode::UnaryOp { op, operand } => {
            7u8.hash(hasher);
            std::mem::discriminant(op).hash(hasher);
            hash_ast_node(operand, hasher);
        }
        AstNode::Indexer { collection, index } => {
            8u8.hash(hasher);
            hash_ast_node(collection, hasher);
            hash_ast_node(index, hasher);
        }
        AstNode::QuantityLiteral { value, unit } => {
            10u8.hash(hasher);
            value.to_bits().hash(hasher);
            unit.hash(hasher);
        }
    }
}

/// Evaluates the iif() function (if-then-else)
fn evaluate_iif_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 3 {
        return Err(FhirPathError::EvaluationError(format!(
            "'iif' function expects 3 arguments, got {}",
            arguments.len()
        )));
    }

    // Evaluate the condition
    let condition = evaluate_ast_internal(&arguments[0], context, visitor)?;

    // Check if condition is truthy
    let is_true = match condition {
        FhirPathValue::Boolean(b) => b,
        FhirPathValue::Collection(ref items) => !items.is_empty(),
        FhirPathValue::Empty => false,
        _ => true, // Non-empty, non-boolean values are considered truthy
    };

    // Return the appropriate branch
    if is_true {
        evaluate_ast_internal(&arguments[1], context, visitor)
    } else {
        evaluate_ast_internal(&arguments[2], context, visitor)
    }
}

/// Evaluates the single() function - returns the single item in a collection or error if not exactly one
fn evaluate_single_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue, FhirPathError> {
    if !arguments.is_empty() {
        return Err(FhirPathError::EvaluationError(format!(
            "'single' function expects 0 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current collection from context
    let collection = get_current_collection(context)?;

    match collection.len() {
        0 => Ok(FhirPathValue::Empty),
        1 => Ok(collection[0].clone()),
        _ => Err(FhirPathError::EvaluationError(
            "single() function called on collection with more than one item".to_string(),
        )),
    }
}

/// Evaluates the supersetOf() function
fn evaluate_superset_of_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'supersetOf' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    let current_collection = get_current_collection(context)?;
    let other_value = evaluate_ast_internal(&arguments[0], context, visitor)?;

    let other_collection = match other_value {
        FhirPathValue::Collection(items) => items,
        FhirPathValue::Empty => vec![],
        single_item => vec![single_item],
    };

    // Check if current collection is a superset of other collection
    // (all items in other collection are in current collection)
    for other_item in &other_collection {
        if !current_collection
            .iter()
            .any(|current_item| values_equal(current_item, other_item))
        {
            return Ok(FhirPathValue::Boolean(false));
        }
    }

    Ok(FhirPathValue::Boolean(true))
}

/// Evaluates the trace() function - for debugging, returns the input unchanged
fn evaluate_trace_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.is_empty() || arguments.len() > 2 {
        return Err(FhirPathError::EvaluationError(format!(
            "'trace' function expects 1 or 2 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the current collection
    let collection = get_current_collection(context)?;

    // For trace, we just return the current collection unchanged
    // In a real implementation, this would log the trace message
    if collection.is_empty() {
        Ok(FhirPathValue::Empty)
    } else if collection.len() == 1 {
        Ok(collection[0].clone())
    } else {
        Ok(FhirPathValue::Collection(collection))
    }
}

/// Evaluates the aggregate() function - iterates through collection with accumulator
fn evaluate_aggregate_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() < 1 || arguments.len() > 2 {
        return Err(FhirPathError::EvaluationError(format!(
            "'aggregate' function expects 1 or 2 arguments, got {}",
            arguments.len()
        )));
    }

    // Get the collection to aggregate over
    let collection = get_current_collection(context)?;

    if collection.is_empty() {
        return if arguments.len() == 2 {
            evaluate_ast_internal(&arguments[1], context, visitor)
        } else {
            Ok(FhirPathValue::Empty)
        };
    }

    // Get the initial value (second argument) or use first item as initial value
    let (mut total, start_index) = if arguments.len() == 2 {
        // Initial value provided - use it and start from first item
        (evaluate_ast_internal(&arguments[1], context, visitor)?, 0)
    } else {
        // No initial value - use first item as initial value and start from second item
        (collection[0].clone(), 1)
    };

    // Iterate through the collection starting from the appropriate index
    for item in collection.iter().skip(start_index) {
        // Create a new context with $this and $total values
        let mut aggregate_context = context.clone();

        // Set $this to current item (this_item field)
        aggregate_context.this_item = Some(item.clone());

        // For $total, we need to store it in variables since context.total is for collection size
        aggregate_context.variables.insert("total".to_string(), total.clone());

        // Evaluate the aggregation expression
        let result = evaluate_ast_internal(&arguments[0], &aggregate_context, visitor)?;

        // Update the total with the result
        total = result;
    }

    Ok(total)
}

/// Evaluates the toChars() function - converts string to collection of single-character strings
fn evaluate_to_chars_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.toChars()
        // Use this_item as the value to convert
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'toChars' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'toChars' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: toChars(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'toChars' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::String(s) => {
            let chars: Vec<FhirPathValue> = s
                .chars()
                .map(|c| FhirPathValue::String(c.to_string()))
                .collect();
            Ok(FhirPathValue::Collection(chars))
        }
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                if let FhirPathValue::String(s) = &items[0] {
                    let chars: Vec<FhirPathValue> = s
                        .chars()
                        .map(|c| FhirPathValue::String(c.to_string()))
                        .collect();
                    Ok(FhirPathValue::Collection(chars))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            } else {
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty),
    }
}

/// Evaluates the escape() function - escapes strings for HTML/JSON
fn evaluate_escape_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'escape' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the string value from current context (method call syntax)
    let value = if let Some(this_item) = &context.this_item {
        match this_item {
            FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
            FhirPathValue::Collection(_) => {
                return Err(FhirPathError::EvaluationError(
                    "'escape' function cannot be applied to collections with multiple items"
                        .to_string(),
                ));
            }
            other => other.clone(),
        }
    } else {
        return Err(FhirPathError::EvaluationError(
            "'escape' function requires a string context".to_string(),
        ));
    };

    let format = evaluate_ast_internal(&arguments[0], context, visitor)?;

    match (value, format) {
        (FhirPathValue::String(s), FhirPathValue::String(fmt)) => {
            let escaped = match fmt.as_str() {
                "html" => s
                    .replace("&", "&amp;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;")
                    .replace("\"", "&quot;"),
                "json" => s.replace("\\", "\\\\").replace("\"", "\\\""),
                _ => s, // Unknown format, return as-is
            };
            Ok(FhirPathValue::String(escaped))
        }
        _ => Ok(FhirPathValue::Empty),
    }
}

/// Evaluates the unescape() function - unescapes HTML/JSON strings
fn evaluate_unescape_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    if arguments.len() != 1 {
        return Err(FhirPathError::EvaluationError(format!(
            "'unescape' function expects 1 argument, got {}",
            arguments.len()
        )));
    }

    // Get the string value from current context (method call syntax)
    let value = if let Some(this_item) = &context.this_item {
        match this_item {
            FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
            FhirPathValue::Collection(_) => {
                return Err(FhirPathError::EvaluationError(
                    "'unescape' function cannot be applied to collections with multiple items"
                        .to_string(),
                ));
            }
            other => other.clone(),
        }
    } else {
        return Err(FhirPathError::EvaluationError(
            "'unescape' function requires a string context".to_string(),
        ));
    };

    let format = evaluate_ast_internal(&arguments[0], context, visitor)?;

    match (value, format) {
        (FhirPathValue::String(s), FhirPathValue::String(fmt)) => {
            let unescaped = match fmt.as_str() {
                "html" => s
                    .replace("&quot;", "\"")
                    .replace("&gt;", ">")
                    .replace("&lt;", "<")
                    .replace("&amp;", "&"),
                "json" => s.replace("\\\"", "\"").replace("\\\\", "\\"),
                _ => s, // Unknown format, return as-is
            };
            Ok(FhirPathValue::String(unescaped))
        }
        _ => Ok(FhirPathValue::Empty),
    }
}

/// Evaluates the toString() function
fn evaluate_to_string_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.toString()
        // Use this_item as the value to convert
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'toString' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'toString' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: toString(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'toString' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::String(s) => Ok(FhirPathValue::String(s)),
        FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string())),
        FhirPathValue::Decimal(d) => {
            // Preserve decimal format - if it's a whole number, still show .0
            let formatted = if d.fract() == 0.0 {
                format!("{:.1}", d)
            } else {
                d.to_string()
            };
            Ok(FhirPathValue::String(formatted))
        },
        FhirPathValue::Boolean(b) => Ok(FhirPathValue::String(b.to_string())),
        FhirPathValue::Date(d) => Ok(FhirPathValue::String(d)),
        FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt)),
        FhirPathValue::Time(t) => Ok(FhirPathValue::String(t)),
        FhirPathValue::Quantity { value, unit } => {
            // Format quantity with quoted unit for proper FHIRPath representation
            let formatted_value = if value.fract() == 0.0 {
                format!("{:.0}", value)
            } else {
                value.to_string()
            };
            Ok(FhirPathValue::String(format!("{} '{}'", formatted_value, unit)))
        }
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item directly
                let item = &items[0];
                match item {
                    FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),
                    FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string())),
                    FhirPathValue::Decimal(d) => {
                        // Preserve decimal format - if it's a whole number, still show .0
                        let formatted = if d.fract() == 0.0 {
                            format!("{:.1}", d)
                        } else {
                            d.to_string()
                        };
                        Ok(FhirPathValue::String(formatted))
                    },
                    FhirPathValue::Boolean(b) => Ok(FhirPathValue::String(b.to_string())),
                    FhirPathValue::Date(d) => Ok(FhirPathValue::String(d.clone())),
                    FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt.clone())),
                    FhirPathValue::Time(t) => Ok(FhirPathValue::String(t.clone())),
                    FhirPathValue::Quantity { value, unit } => {
                        // Format quantity with quoted unit for proper FHIRPath representation
                        let formatted_value = if value.fract() == 0.0 {
                            format!("{:.0}", value)
                        } else {
                            value.to_string()
                        };
                        Ok(FhirPathValue::String(format!("{} '{}'", formatted_value, unit)))
                    }
                    _ => Ok(FhirPathValue::Empty),
                }
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        FhirPathValue::Empty => Ok(FhirPathValue::Empty),
        FhirPathValue::Resource(_) => Ok(FhirPathValue::Empty), // Resources can't be converted to string
    }
}

/// Evaluates the toInteger() function
fn evaluate_to_integer_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.toInteger()
        // Use this_item as the value to convert
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'toInteger' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'toInteger' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: toInteger(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'toInteger' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(i)),
        FhirPathValue::String(s) => {
            // Try to parse string as integer
            if let Ok(i) = s.parse::<i64>() {
                Ok(FhirPathValue::Integer(i))
            } else {
                // If parsing fails, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        FhirPathValue::Boolean(b) => {
            // true -> 1, false -> 0
            Ok(FhirPathValue::Integer(if b { 1 } else { 0 }))
        }
        FhirPathValue::Decimal(d) => {
            // Only convert if it's a whole number
            if d.fract() == 0.0 {
                Ok(FhirPathValue::Integer(d as i64))
            } else {
                // If it has fractional part, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item
                evaluate_to_integer_function(&[arguments[0].clone()], context, visitor)
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty), // Other types can't be converted to integer
    }
}

/// Evaluates the toDecimal() function
fn evaluate_to_decimal_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.toDecimal()
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'toDecimal' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'toDecimal' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: toDecimal(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'toDecimal' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(d)),
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(i as f64)),
        FhirPathValue::String(s) => {
            // Try to parse string as decimal
            if let Ok(d) = s.parse::<f64>() {
                Ok(FhirPathValue::Decimal(d))
            } else {
                // If parsing fails, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        FhirPathValue::Boolean(b) => {
            // true -> 1.0, false -> 0.0
            Ok(FhirPathValue::Decimal(if b { 1.0 } else { 0.0 }))
        }
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item
                let single_item_context =
                    context.create_iteration_context(items[0].clone(), 0, 1)?;
                evaluate_to_decimal_function(&[], &single_item_context, visitor)
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty), // Other types can't be converted to decimal
    }
}

/// Evaluates the toQuantity() function
fn evaluate_to_quantity_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.toQuantity()
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'toQuantity' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'toQuantity' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: toQuantity(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'toQuantity' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::Integer(i) => {
            // Convert integer to quantity with default unit
            Ok(FhirPathValue::Quantity {
                value: i as f64,
                unit: "1".to_string(), // Default unit for dimensionless quantities
            })
        }
        FhirPathValue::Decimal(d) => {
            // Convert decimal to quantity with default unit
            Ok(FhirPathValue::Quantity {
                value: d,
                unit: "1".to_string(), // Default unit for dimensionless quantities
            })
        }
        FhirPathValue::String(s) => {
            // Try to parse string as FHIR Quantity using ucum-fhir
            // First, try to extract numeric value and unit from the string
            if let Some((value_str, unit_str, _is_quoted)) = parse_quantity_string(&s) {
                if let Ok(value) = value_str.parse::<f64>() {
                    // Create a FHIR Quantity and validate it
                    let fhir_quantity = FhirQuantity::with_ucum_code(value, unit_str);
                    // Try to convert to UCUM quantity to validate
                    match fhir_quantity.to_ucum_quantity() {
                        Ok(_) => Ok(FhirPathValue::Quantity {
                            value,
                            unit: unit_str.to_string(),
                        }),
                        Err(_) => Ok(FhirPathValue::Empty),
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            } else {
                // If no unit found, try simple number parsing as fallback
                if let Ok(d) = s.trim().parse::<f64>() {
                    Ok(FhirPathValue::Quantity {
                        value: d,
                        unit: "1".to_string(), // Default unit for dimensionless quantities
                    })
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
        FhirPathValue::Quantity { value, unit } => {
            // Already a quantity, return as-is
            Ok(FhirPathValue::Quantity { value, unit })
        }
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item
                let single_item_context =
                    context.create_iteration_context(items[0].clone(), 0, 1)?;
                evaluate_to_quantity_function(&[], &single_item_context, visitor)
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty), // Other types can't be converted to quantity
    }
}

/// Evaluates the toBoolean() function
fn evaluate_to_boolean_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.toBoolean()
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'toBoolean' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'toBoolean' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: toBoolean(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'toBoolean' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(b)),
        FhirPathValue::Integer(i) => {
            // 1 -> true, 0 -> false, others -> empty
            match i {
                1 => Ok(FhirPathValue::Boolean(true)),
                0 => Ok(FhirPathValue::Boolean(false)),
                _ => Ok(FhirPathValue::Empty),
            }
        }
        FhirPathValue::String(s) => {
            // "true"/"false" (case insensitive) -> true/false, others -> empty
            match s.to_lowercase().as_str() {
                "true" => Ok(FhirPathValue::Boolean(true)),
                "false" => Ok(FhirPathValue::Boolean(false)),
                _ => Ok(FhirPathValue::Empty),
            }
        }
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item
                let single_item_context =
                    context.create_iteration_context(items[0].clone(), 0, 1)?;
                evaluate_to_boolean_function(&[], &single_item_context, visitor)
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty), // Other types can't be converted to boolean
    }
}

/// Evaluates the upper() function - converts string to uppercase
fn evaluate_upper_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.upper()
        // Use this_item as the value to convert
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'upper' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'upper' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: upper(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'upper' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_uppercase())),
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item
                if let FhirPathValue::String(s) = &items[0] {
                    Ok(FhirPathValue::String(s.to_uppercase()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty), // Other types can't be converted to uppercase
    }
}

/// Evaluates the lower() function - converts string to lowercase
fn evaluate_lower_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.lower()
        // Use this_item as the value to convert
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'lower' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'lower' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: lower(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'lower' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_lowercase())),
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item
                if let FhirPathValue::String(s) = &items[0] {
                    Ok(FhirPathValue::String(s.to_lowercase()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty), // Other types can't be converted to lowercase
    }
}

/// Evaluates the trim() function - removes leading and trailing whitespace from string
fn evaluate_trim_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let value = if arguments.is_empty() {
        // Method call syntax: value.trim()
        if let Some(this_item) = &context.this_item {
            match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'trim' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            }
        } else {
            return Err(FhirPathError::EvaluationError(
                "'trim' function expects 1 argument or method call syntax".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Function call syntax: trim(value)
        evaluate_ast_internal(&arguments[0], context, visitor)?
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'trim' function expects 0 or 1 argument, got {}",
            arguments.len()
        )));
    };

    match value {
        FhirPathValue::String(s) => Ok(FhirPathValue::String(s.trim().to_string())),
        FhirPathValue::Collection(items) => {
            if items.len() == 1 {
                // For single-item collections, convert the item
                if let FhirPathValue::String(s) = &items[0] {
                    Ok(FhirPathValue::String(s.trim().to_string()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            } else {
                // For multi-item collections, return empty
                Ok(FhirPathValue::Empty)
            }
        }
        _ => Ok(FhirPathValue::Empty), // Other types can't be trimmed
    }
}

/// Evaluates the encode() function - Encodes a string using specified encoding
fn evaluate_encode_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let (value, encoding_type) = if arguments.is_empty() {
        // Method call syntax: value.encode() - default to URL encoding
        if let Some(this_item) = &context.this_item {
            let val = match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'encode' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            };
            (val, "url".to_string())
        } else {
            return Err(FhirPathError::EvaluationError(
                "'encode' function expects 1 or 2 arguments".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Method call syntax: value.encode('base64') or function call: encode(value)
        let arg_result = evaluate_ast_internal(&arguments[0], context, visitor)?;
        if let Some(this_item) = &context.this_item {
            // Method call syntax with encoding type
            let val = match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'encode' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            };
            let encoding = match arg_result {
                FhirPathValue::String(s) => s,
                _ => return Err(FhirPathError::EvaluationError(
                    "'encode' function encoding type must be a string".to_string(),
                )),
            };
            (val, encoding)
        } else {
            // Function call syntax with just value, default to URL encoding
            (arg_result, "url".to_string())
        }
    } else if arguments.len() == 2 {
        // Function call syntax: encode(value, 'base64')
        let value_result = evaluate_ast_internal(&arguments[0], context, visitor)?;
        let encoding_result = evaluate_ast_internal(&arguments[1], context, visitor)?;
        let encoding = match encoding_result {
            FhirPathValue::String(s) => s,
            _ => return Err(FhirPathError::EvaluationError(
                "'encode' function encoding type must be a string".to_string(),
            )),
        };
        (value_result, encoding)
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'encode' function expects 1 or 2 arguments, got {}",
            arguments.len()
        )));
    };

    let string_to_encode = match value {
        FhirPathValue::String(s) => s,
        FhirPathValue::Collection(items) if items.len() == 1 => {
            match &items[0] {
                FhirPathValue::String(s) => s.clone(),
                _ => return Ok(FhirPathValue::Empty),
            }
        }
        _ => return Ok(FhirPathValue::Empty),
    };

    let encoded = match encoding_type.to_lowercase().as_str() {
        "base64" => {
            // Simple base64 encoding
            base64_encode(&string_to_encode)
        }
        "hex" => {
            // Hex encoding
            hex_encode(&string_to_encode)
        }
        "url" => {
            // URL encoding
            string_to_encode
                .replace(' ', "%20")
                .replace('&', "%26")
                .replace('=', "%3D")
                .replace('?', "%3F")
                .replace('#', "%23")
        }
        _ => {
            return Err(FhirPathError::EvaluationError(format!(
                "Unsupported encoding type: {}",
                encoding_type
            )));
        }
    };

    Ok(FhirPathValue::String(encoded))
}

/// Simple base64 encoding implementation
fn base64_encode(input: &str) -> String {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

        result.push(BASE64_CHARS[((b >> 18) & 63) as usize] as char);
        result.push(BASE64_CHARS[((b >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { BASE64_CHARS[((b >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { BASE64_CHARS[(b & 63) as usize] as char } else { '=' });
    }

    result
}

/// Simple hex encoding implementation
fn hex_encode(input: &str) -> String {
    input.bytes().map(|b| format!("{:02x}", b)).collect()
}

/// Evaluates the decode() function - Decodes a string using specified encoding
fn evaluate_decode_function(
    arguments: &[AstNode],
    context: &EvaluationContext,
    visitor: &dyn AstVisitor,
) -> Result<FhirPathValue, FhirPathError> {
    let (value, encoding_type) = if arguments.is_empty() {
        // Method call syntax: value.decode() - default to URL decoding
        if let Some(this_item) = &context.this_item {
            let val = match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'decode' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            };
            (val, "url".to_string())
        } else {
            return Err(FhirPathError::EvaluationError(
                "'decode' function expects 1 or 2 arguments".to_string(),
            ));
        }
    } else if arguments.len() == 1 {
        // Method call syntax: value.decode('base64') or function call: decode(value)
        let arg_result = evaluate_ast_internal(&arguments[0], context, visitor)?;
        if let Some(this_item) = &context.this_item {
            // Method call syntax with encoding type
            let val = match this_item {
                FhirPathValue::Collection(items) if items.len() == 1 => items[0].clone(),
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError(
                        "'decode' function cannot be applied to collections with multiple items"
                            .to_string(),
                    ));
                }
                other => other.clone(),
            };
            let encoding = match arg_result {
                FhirPathValue::String(s) => s,
                _ => return Err(FhirPathError::EvaluationError(
                    "'decode' function encoding type must be a string".to_string(),
                )),
            };
            (val, encoding)
        } else {
            // Function call syntax with just value, default to URL decoding
            (arg_result, "url".to_string())
        }
    } else if arguments.len() == 2 {
        // Function call syntax: decode(value, 'base64')
        let value_result = evaluate_ast_internal(&arguments[0], context, visitor)?;
        let encoding_result = evaluate_ast_internal(&arguments[1], context, visitor)?;
        let encoding = match encoding_result {
            FhirPathValue::String(s) => s,
            _ => return Err(FhirPathError::EvaluationError(
                "'decode' function encoding type must be a string".to_string(),
            )),
        };
        (value_result, encoding)
    } else {
        return Err(FhirPathError::EvaluationError(format!(
            "'decode' function expects 1 or 2 arguments, got {}",
            arguments.len()
        )));
    };

    let string_to_decode = match value {
        FhirPathValue::String(s) => s,
        FhirPathValue::Collection(items) if items.len() == 1 => {
            match &items[0] {
                FhirPathValue::String(s) => s.clone(),
                _ => return Ok(FhirPathValue::Empty),
            }
        }
        _ => return Ok(FhirPathValue::Empty),
    };

    let decoded = match encoding_type.to_lowercase().as_str() {
        "base64" => {
            // Simple base64 decoding
            match base64_decode(&string_to_decode) {
                Ok(decoded) => decoded,
                Err(_) => return Ok(FhirPathValue::Empty),
            }
        }
        "hex" => {
            // Hex decoding
            match hex_decode(&string_to_decode) {
                Ok(decoded) => decoded,
                Err(_) => return Ok(FhirPathValue::Empty),
            }
        }
        "url" => {
            // URL decoding
            string_to_decode
                .replace("%20", " ")
                .replace("%26", "&")
                .replace("%3D", "=")
                .replace("%3F", "?")
                .replace("%23", "#")
        }
        _ => {
            return Err(FhirPathError::EvaluationError(format!(
                "Unsupported decoding type: {}",
                encoding_type
            )));
        }
    };

    Ok(FhirPathValue::String(decoded))
}

/// Simple base64 decoding implementation
fn base64_decode(input: &str) -> Result<String, String> {
    const BASE64_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = input.trim_end_matches('=');
    let mut result = Vec::new();

    for chunk in input.chars().collect::<Vec<_>>().chunks(4) {
        let mut values = [0u8; 4];
        for (i, &ch) in chunk.iter().enumerate() {
            if let Some(pos) = BASE64_CHARS.find(ch) {
                values[i] = pos as u8;
            } else {
                return Err("Invalid base64 character".to_string());
            }
        }

        let combined = ((values[0] as u32) << 18) |
                      ((values[1] as u32) << 12) |
                      ((values[2] as u32) << 6) |
                      (values[3] as u32);

        result.push((combined >> 16) as u8);
        if chunk.len() > 2 {
            result.push((combined >> 8) as u8);
        }
        if chunk.len() > 3 {
            result.push(combined as u8);
        }
    }

    String::from_utf8(result).map_err(|_| "Invalid UTF-8 in decoded base64".to_string())
}

/// Simple hex decoding implementation
fn hex_decode(input: &str) -> Result<String, String> {
    if input.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    let mut result = Vec::new();
    for chunk in input.chars().collect::<Vec<_>>().chunks(2) {
        let hex_str: String = chunk.iter().collect();
        match u8::from_str_radix(&hex_str, 16) {
            Ok(byte) => result.push(byte),
            Err(_) => return Err("Invalid hex character".to_string()),
        }
    }

    String::from_utf8(result).map_err(|_| "Invalid UTF-8 in decoded hex".to_string())
}

/// Helper function to compare quantities with unit conversion
fn quantities_equal(v1: f64, u1: &str, v2: f64, u2: &str) -> bool {
    // If units are the same, just compare values
    if u1 == u2 {
        return (v1 - v2).abs() < f64::EPSILON;
    }

    // Try UCUM unit conversion
    let q1 = FhirQuantity::with_ucum_code(v1, u1);
    let q2 = FhirQuantity::with_ucum_code(v2, u2);

    match (q1.to_ucum_quantity(), q2.to_ucum_quantity()) {
        (Ok(ucum1), Ok(ucum2)) => {
            // Convert UnitExpr to string for comparison
            let unit1_str = format!("{:?}", ucum1.unit);
            let unit2_str = format!("{:?}", ucum2.unit);

            // Check for common unit conversions
            let converted_values = if let Some((converted_v1, converted_v2)) =
                convert_units_for_comparison(ucum1.value, &unit1_str, ucum2.value, &unit2_str) {
                Some((converted_v1, converted_v2))
            } else {
                None
            };

            match converted_values {
                Some((cv1, cv2)) => (cv1 - cv2).abs() < f64::EPSILON,
                None => false, // Units not compatible
            }
        }
        _ => false, // UCUM conversion failed
    }
}

/// Helper function to convert units for comparison
fn convert_units_for_comparison(v1: f64, u1_str: &str, v2: f64, u2_str: &str) -> Option<(f64, f64)> {
    // Mass conversions
    if u1_str.contains("mg") && u2_str.contains("g") && !u2_str.contains("mg") {
        // Convert mg to g
        Some((v1 / 1000.0, v2))
    } else if u1_str.contains("g") && !u1_str.contains("mg") && u2_str.contains("mg") {
        // Convert g to mg
        Some((v1 * 1000.0, v2))
    } else if u1_str.contains("kg") && u2_str.contains("g") && !u2_str.contains("kg") {
        // Convert kg to g
        Some((v1 * 1000.0, v2))
    } else if u1_str.contains("g") && !u1_str.contains("kg") && u2_str.contains("kg") {
        // Convert g to kg
        Some((v1 / 1000.0, v2))
    }
    // Time conversions
    else if u1_str.contains("wk") && u2_str.contains("d") && !u2_str.contains("wk") {
        // Convert weeks to days
        Some((v1 * 7.0, v2))
    } else if u1_str.contains("d") && !u1_str.contains("wk") && u2_str.contains("wk") {
        // Convert days to weeks
        Some((v1 / 7.0, v2))
    } else if u1_str.contains("d") && u2_str.contains("h") && !u2_str.contains("d") {
        // Convert days to hours
        Some((v1 * 24.0, v2))
    } else if u1_str.contains("h") && !u1_str.contains("d") && u2_str.contains("d") {
        // Convert hours to days
        Some((v1 / 24.0, v2))
    } else if u1_str.contains("h") && u2_str.contains("min") && !u2_str.contains("h") {
        // Convert hours to minutes
        Some((v1 * 60.0, v2))
    } else if u1_str.contains("min") && !u1_str.contains("h") && u2_str.contains("h") {
        // Convert minutes to hours
        Some((v1 / 60.0, v2))
    } else if u1_str.contains("min") && u2_str.contains("s") && !u2_str.contains("min") {
        // Convert minutes to seconds
        Some((v1 * 60.0, v2))
    } else if u1_str.contains("s") && !u1_str.contains("min") && u2_str.contains("min") {
        // Convert seconds to minutes
        Some((v1 / 60.0, v2))
    }
    // Length conversions
    else if u1_str.contains("km") && u2_str.contains("m") && !u2_str.contains("km") {
        // Convert km to m
        Some((v1 * 1000.0, v2))
    } else if u1_str.contains("m") && !u1_str.contains("km") && u2_str.contains("km") {
        // Convert m to km
        Some((v1 / 1000.0, v2))
    } else if u1_str.contains("m") && !u1_str.contains("cm") && u2_str.contains("cm") {
        // Convert m to cm
        Some((v1 * 100.0, v2))
    } else if u1_str.contains("cm") && u2_str.contains("m") && !u2_str.contains("cm") {
        // Convert cm to m
        Some((v1 / 100.0, v2))
    } else if u1_str.contains("cm") && u2_str.contains("mm") {
        // Convert cm to mm
        Some((v1 * 10.0, v2))
    } else if u1_str.contains("mm") && u2_str.contains("cm") {
        // Convert mm to cm
        Some((v1 / 10.0, v2))
    } else {
        None // Units not compatible
    }
}

/// Comparison result for FHIRPath operations
#[derive(Debug, Clone, PartialEq)]
enum ComparisonResult {
    Equal,
    NotEqual,
    NotComparable, // Should return empty
}

/// Helper function to compare two values according to FHIRPath rules
fn compare_values_fhirpath(left: &FhirPathValue, right: &FhirPathValue) -> ComparisonResult {
    match (left, right) {
        (FhirPathValue::Empty, FhirPathValue::Empty) => ComparisonResult::Equal,
        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
            if a == b { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
            if a == b { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
            if (a - b).abs() < f64::EPSILON { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            if (*a as f64 - b).abs() < f64::EPSILON { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            if (a - *b as f64).abs() < f64::EPSILON { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::String(a), FhirPathValue::String(b)) => {
            if a == b { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
            if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
            if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
            if a == b { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }

        // Cross-type Date/DateTime comparisons - check if they're comparable
        (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
            // Date vs DateTime: only comparable if DateTime has no time component or is at start of day
            let a_clean = a.strip_prefix('@').unwrap_or(a);
            let b_clean = b.strip_prefix('@').unwrap_or(b);

            #[cfg(feature = "trace")]
            println!("[DEBUG] Date vs DateTime comparison: '{}' vs '{}'", a_clean, b_clean);

            // If DateTime has time components beyond 00:00:00, they're not comparable
            if b_clean.contains('T') {
                let parts: Vec<&str> = b_clean.split('T').collect();
                if parts.len() == 2 {
                    let time_part = parts[1];
                    // Remove timezone for comparison
                    let time_clean = if let Some(tz_pos) = time_part.find('+') {
                        &time_part[..tz_pos]
                    } else if let Some(tz_pos) = time_part.find('-') {
                        if tz_pos > 2 { &time_part[..tz_pos] } else { time_part }
                    } else if time_part.ends_with('Z') {
                        &time_part[..time_part.len() - 1]
                    } else {
                        time_part
                    };

                    #[cfg(feature = "trace")]
                    println!("[DEBUG] Time part: '{}', time_clean: '{}'", time_part, time_clean);

                    // If time is not 00:00:00 or equivalent, not comparable
                    if !time_clean.starts_with("00:00:00") && !time_clean.starts_with("00:00") && time_clean != "00" {
                        #[cfg(feature = "trace")]
                        println!("[DEBUG] Not comparable - returning NotComparable");
                        return ComparisonResult::NotComparable;
                    }
                }
            }

            #[cfg(feature = "trace")]
            println!("[DEBUG] Comparable - checking datetime_equal");
            if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }
        (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
            // DateTime vs Date: only comparable if DateTime has no time component or is at start of day
            let a_clean = a.strip_prefix('@').unwrap_or(a);
            let b_clean = b.strip_prefix('@').unwrap_or(b);

            // If DateTime has time components beyond 00:00:00, they're not comparable
            if a_clean.contains('T') {
                let parts: Vec<&str> = a_clean.split('T').collect();
                if parts.len() == 2 {
                    let time_part = parts[1];
                    // Remove timezone for comparison
                    let time_clean = if let Some(tz_pos) = time_part.find('+') {
                        &time_part[..tz_pos]
                    } else if let Some(tz_pos) = time_part.find('-') {
                        if tz_pos > 2 { &time_part[..tz_pos] } else { time_part }
                    } else if time_part.ends_with('Z') {
                        &time_part[..time_part.len() - 1]
                    } else {
                        time_part
                    };

                    // If time is not 00:00:00 or equivalent, not comparable
                    if !time_clean.starts_with("00:00:00") && !time_clean.starts_with("00:00") && time_clean != "00" {
                        return ComparisonResult::NotComparable;
                    }
                }
            }

            if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }

        // String to Date/DateTime comparisons (for FHIR primitive values)
        (FhirPathValue::String(a), FhirPathValue::Date(b)) => {
            // Try to parse string as date and compare
            if is_valid_datetime_string(a) && !a.contains('T') {
                if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
            } else {
                ComparisonResult::NotEqual
            }
        }
        (FhirPathValue::Date(a), FhirPathValue::String(b)) => {
            // Try to parse string as date and compare
            if is_valid_datetime_string(b) && !b.contains('T') {
                if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
            } else {
                ComparisonResult::NotEqual
            }
        }
        (FhirPathValue::String(a), FhirPathValue::DateTime(b)) => {
            // Try to parse string as datetime and compare
            if is_valid_datetime_string(a) {
                // All valid date/datetime strings are comparable
                // Let datetime_equal handle the precision and timezone logic
                if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
            } else {
                ComparisonResult::NotComparable
            }
        }
        (FhirPathValue::DateTime(a), FhirPathValue::String(b)) => {
            // Try to parse string as datetime and compare
            if is_valid_datetime_string(b) {
                if datetime_equal(a, b) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
            } else {
                ComparisonResult::NotEqual
            }
        }
        (
            FhirPathValue::Quantity {
                value: v1,
                unit: u1,
            },
            FhirPathValue::Quantity {
                value: v2,
                unit: u2,
            },
        ) => {
            // Use helper function to compare quantities with unit conversion
            if quantities_equal(*v1, u1, *v2, u2) { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        },

        // Collection comparisons
        (FhirPathValue::Collection(items1), FhirPathValue::Collection(items2)) => {
            // Empty collections are equal
            if items1.is_empty() && items2.is_empty() {
                return ComparisonResult::Equal;
            }

            // Different lengths mean not equal
            if items1.len() != items2.len() {
                return ComparisonResult::NotEqual;
            }

            // For collections of the same length, check if all items in items1
            // have a corresponding equal item in items2 (order-independent)
            let all_equal = items1.iter().all(|item1| {
                items2.iter().any(|item2| values_equal(item1, item2))
            }) && items2.iter().all(|item2| {
                items1.iter().any(|item1| values_equal(item1, item2))
            });

            if all_equal { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }

        // Single value vs collection comparisons
        (single_value, FhirPathValue::Collection(items)) |
        (FhirPathValue::Collection(items), single_value) => {
            // A single value equals a collection if the collection has exactly one item
            // and that item equals the single value
            if items.len() == 1 && values_equal(&items[0], single_value) {
                ComparisonResult::Equal
            } else {
                ComparisonResult::NotEqual
            }
        }

        // Resource comparisons
        (FhirPathValue::Resource(r1), FhirPathValue::Resource(r2)) => {
            if r1.to_json() == r2.to_json() { ComparisonResult::Equal } else { ComparisonResult::NotEqual }
        }

        _ => ComparisonResult::NotEqual,
    }
}

/// Helper function to check if two values are equal (backward compatibility)
fn values_equal(left: &FhirPathValue, right: &FhirPathValue) -> bool {
    match (left, right) {
        (FhirPathValue::Empty, FhirPathValue::Empty) => true,
        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a == b,
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a == b,
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => (a - b).abs() < f64::EPSILON,
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            (*a as f64 - b).abs() < f64::EPSILON
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            (a - *b as f64).abs() < f64::EPSILON
        }
        (FhirPathValue::String(a), FhirPathValue::String(b)) => a == b,
        (FhirPathValue::Date(a), FhirPathValue::Date(b)) => datetime_equal(a, b),
        (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => datetime_equal(a, b),
        (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a == b,

        // Cross-type Date/DateTime comparisons
        (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => datetime_equal(a, b),
        (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => datetime_equal(a, b),

        // String to Date/DateTime comparisons (for FHIR primitive values)
        (FhirPathValue::String(a), FhirPathValue::Date(b)) => {
            // Try to parse string as date and compare
            if is_valid_datetime_string(a) && !a.contains('T') {
                datetime_equal(a, b)
            } else {
                false
            }
        }
        (FhirPathValue::Date(a), FhirPathValue::String(b)) => {
            // Try to parse string as date and compare
            if is_valid_datetime_string(b) && !b.contains('T') {
                datetime_equal(a, b)
            } else {
                false
            }
        }
        (FhirPathValue::String(a), FhirPathValue::DateTime(b)) => {
            // Try to parse string as datetime and compare
            if is_valid_datetime_string(a) {
                datetime_equal(a, b)
            } else {
                false
            }
        }
        (FhirPathValue::DateTime(a), FhirPathValue::String(b)) => {
            // Try to parse string as datetime and compare
            if is_valid_datetime_string(b) {
                datetime_equal(a, b)
            } else {
                false
            }
        }
        (
            FhirPathValue::Quantity {
                value: v1,
                unit: u1,
            },
            FhirPathValue::Quantity {
                value: v2,
                unit: u2,
            },
        ) => {
            // Use helper function to compare quantities with unit conversion
            quantities_equal(*v1, u1, *v2, u2)
        },

        // Collection comparisons
        (FhirPathValue::Collection(items1), FhirPathValue::Collection(items2)) => {
            // Empty collections are equal
            if items1.is_empty() && items2.is_empty() {
                return true;
            }

            // Different lengths mean not equal
            if items1.len() != items2.len() {
                return false;
            }

            // For collections of the same length, check if all items in items1
            // have a corresponding equal item in items2 (order-independent)
            items1.iter().all(|item1| {
                items2.iter().any(|item2| values_equal(item1, item2))
            }) && items2.iter().all(|item2| {
                items1.iter().any(|item1| values_equal(item1, item2))
            })
        }

        // Single value vs collection comparisons
        (single_value, FhirPathValue::Collection(items)) |
        (FhirPathValue::Collection(items), single_value) => {
            // A single value equals a collection if the collection has exactly one item
            // and that item equals the single value
            items.len() == 1 && values_equal(&items[0], single_value)
        }

        // Resource comparisons
        (FhirPathValue::Resource(r1), FhirPathValue::Resource(r2)) => {
            r1.to_json() == r2.to_json()
        }

        _ => false,
    }
}

/// Helper function to check if two values are equivalent (FHIRPath ~ operator)
/// Equivalent is more relaxed than equality, allowing type coercion and approximate matching
fn values_equivalent(left: &FhirPathValue, right: &FhirPathValue) -> bool {
    match (left, right) {
        // Same as equality for these types
        (FhirPathValue::Empty, FhirPathValue::Empty) => true,
        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a == b,

        // Numeric equivalence with type coercion
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a == b,
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => (a - b).abs() < f64::EPSILON,
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            (*a as f64 - b).abs() < f64::EPSILON
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            (a - *b as f64).abs() < f64::EPSILON
        }

        // String equivalence (case-insensitive for equivalent)
        (FhirPathValue::String(a), FhirPathValue::String(b)) => {
            a.to_lowercase() == b.to_lowercase()
        }

        // DateTime equivalence with precision checking
        (FhirPathValue::Date(a), FhirPathValue::Date(b)) => datetime_equivalent(a, b),
        (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => datetime_equivalent(a, b),
        (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a == b,

        // Cross-type datetime equivalence
        (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => datetime_equivalent(a, b),
        (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => datetime_equivalent(a, b),

        // Quantity equivalence
        (
            FhirPathValue::Quantity {
                value: v1,
                unit: u1,
            },
            FhirPathValue::Quantity {
                value: v2,
                unit: u2,
            },
        ) => {
            // Use helper function to compare quantities with unit conversion
            quantities_equal(*v1, u1, *v2, u2)
        },

        // Collection equivalence
        (FhirPathValue::Collection(items1), FhirPathValue::Collection(items2)) => {
            // Empty collections are equivalent
            if items1.is_empty() && items2.is_empty() {
                return true;
            }

            // Different lengths mean not equivalent
            if items1.len() != items2.len() {
                return false;
            }

            // For collections of the same length, check if all items in items1
            // have a corresponding equivalent item in items2 (order-independent)
            items1.iter().all(|item1| {
                items2.iter().any(|item2| values_equivalent(item1, item2))
            }) && items2.iter().all(|item2| {
                items1.iter().any(|item1| values_equivalent(item1, item2))
            })
        }

        // Single value vs collection equivalence
        (single_value, FhirPathValue::Collection(items)) |
        (FhirPathValue::Collection(items), single_value) => {
            // A single value is equivalent to a collection if the collection has exactly one item
            // and that item is equivalent to the single value
            items.len() == 1 && values_equivalent(&items[0], single_value)
        }

        // Type coercion for numbers and strings
        (FhirPathValue::Integer(a), FhirPathValue::String(b)) => {
            b.parse::<i64>().map_or(false, |parsed| *a == parsed)
        }
        (FhirPathValue::String(a), FhirPathValue::Integer(b)) => {
            a.parse::<i64>().map_or(false, |parsed| parsed == *b)
        }
        (FhirPathValue::Decimal(a), FhirPathValue::String(b)) => b
            .parse::<f64>()
            .map_or(false, |parsed| (a - parsed).abs() < f64::EPSILON),
        (FhirPathValue::String(a), FhirPathValue::Decimal(b)) => a
            .parse::<f64>()
            .map_or(false, |parsed| (parsed - b).abs() < f64::EPSILON),

        // Resource equivalence
        (FhirPathValue::Resource(r1), FhirPathValue::Resource(r2)) => {
            r1.to_json() == r2.to_json()
        }

        _ => false,
    }
}
