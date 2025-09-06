//! Lambda Expression Evaluator for FHIRPath
//!
//! This module provides comprehensive lambda expression evaluation for FHIRPath
//! collection functions like `where()`, `select()`, `all()`, `repeat()`, `aggregate()`, and `sort()`.
//!
//! # Key Features
//!
//! - **Proper Variable Scoping**: Each lambda creates its own scope with $this, $index, and $total
//! - **Variable Capture**: Lambda expressions can access variables from outer scopes
//! - **Performance Optimized**: Minimal overhead for lambda invocation
//! - **Error Handling**: Comprehensive error reporting with source location tracking
//! - **Async Support**: Full async evaluation support for lambda expressions

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathValue};
use crate::evaluator::{EvaluationContext, ScopeManager};

// Simple AST matcher to detect predicates like: identifier = 'string'
fn extract_simple_literal_compare(
    expr: &ExpressionNode,
) -> Option<(Vec<String>, SimpleLiteral, SimpleOp)> {
    use crate::ast::{BinaryOperator, ExpressionNode as EN, LiteralValue};
    let (left, right, op) = match expr {
        EN::BinaryOperation(bin)
            if matches!(
                bin.operator,
                BinaryOperator::Equal | BinaryOperator::NotEqual
            ) =>
        {
            (&*bin.left, &*bin.right, bin.operator)
        }
        _ => return None,
    };
    // PropertyPath <op> literal
    if let Some(path) = extract_property_path(left) {
        if let EN::Literal(lit) = right {
            return match &lit.value {
                LiteralValue::String(s) => {
                    Some((path, SimpleLiteral::String(s.clone()), op.into()))
                }
                LiteralValue::Integer(i) => Some((path, SimpleLiteral::Integer(*i), op.into())),
                LiteralValue::Boolean(b) => Some((path, SimpleLiteral::Boolean(*b), op.into())),
                _ => None,
            };
        }
    }
    // literal <op> PropertyPath
    if let EN::Literal(lit) = left {
        if let Some(path) = extract_property_path(right) {
            return match &lit.value {
                LiteralValue::String(s) => {
                    Some((path, SimpleLiteral::String(s.clone()), op.into()))
                }
                LiteralValue::Integer(i) => Some((path, SimpleLiteral::Integer(*i), op.into())),
                LiteralValue::Boolean(b) => Some((path, SimpleLiteral::Boolean(*b), op.into())),
                _ => None,
            };
        }
    }
    None
}

#[derive(Clone)]
enum SimpleLiteral {
    String(String),
    Integer(i64),
    Boolean(bool),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SimpleOp {
    Eq,
    Ne,
}

impl From<crate::ast::BinaryOperator> for SimpleOp {
    fn from(op: crate::ast::BinaryOperator) -> Self {
        match op {
            crate::ast::BinaryOperator::NotEqual => SimpleOp::Ne,
            _ => SimpleOp::Eq,
        }
    }
}

fn extract_exists_on_path(expr: &ExpressionNode) -> Option<Vec<String>> {
    use crate::ast::ExpressionNode as EN;
    if let EN::MethodCall(mc) = expr {
        if mc.method == "exists" && mc.arguments.is_empty() {
            return extract_property_path(&mc.object);
        }
    }
    extract_property_path(expr) // bare identifier/chain truthiness
}

fn property_exists_truthy(obj: &serde_json::Value, name: &str) -> bool {
    match obj.get(name) {
        None => false,
        Some(v) if v.is_null() => false,
        Some(v) if v.is_array() => v.as_array().map(|a| !a.is_empty()).unwrap_or(false),
        Some(v) if v.is_string() => v.as_str().map(|s| !s.is_empty()).unwrap_or(false),
        Some(_) => true,
    }
}

#[inline]
fn cmp_eq_ne(eq: bool, op: SimpleOp) -> bool {
    if op == SimpleOp::Eq { eq } else { !eq }
}

fn extract_property_path(expr: &ExpressionNode) -> Option<Vec<String>> {
    use crate::ast::ExpressionNode as EN;
    fn collect(node: &ExpressionNode, acc: &mut Vec<String>) -> bool {
        match node {
            EN::Identifier(id) => {
                acc.push(id.name.clone());
                true
            }
            EN::PropertyAccess(p) => {
                if !collect(&p.object, acc) {
                    return false;
                }
                acc.push(p.property.clone());
                true
            }
            _ => false,
        }
    }
    let mut parts = Vec::new();
    if collect(expr, &mut parts) {
        Some(parts)
    } else {
        None
    }
}

fn get_json_at_path<'a>(
    mut obj: &'a serde_json::Value,
    path: &[String],
) -> Option<&'a serde_json::Value> {
    for key in path {
        obj = obj.get(key.as_str())?;
    }
    Some(obj)
}

fn json_to_fhirpath_value(v: &serde_json::Value) -> Option<FhirPathValue> {
    match v {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(FhirPathValue::Boolean(*b)),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(FhirPathValue::Integer)
            .or_else(|| n.as_f64().map(|f| FhirPathValue::String(f.to_string()))),
        serde_json::Value::String(s) => Some(FhirPathValue::String(s.clone())),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            Some(FhirPathValue::Resource(v.clone()))
        }
    }
}

/// Sort criterion for multi-criteria sorting
///
/// Represents a single sort criterion with expression and direction.
#[derive(Debug, Clone)]
pub struct SortCriterion {
    /// Expression to evaluate for sort key
    pub expression: ExpressionNode,
    /// Whether to sort in descending order
    pub descending: bool,
}

/// Sort key for comparison
///
/// Represents a sortable value extracted from FHIRPath expressions.
#[derive(Debug, Clone)]
enum SortKey {
    /// Empty value (sorts first)
    Empty,
    /// String value
    String(String),
    /// Numeric value (integer or decimal)
    Number(rust_decimal::Decimal),
    /// Boolean value
    Boolean(bool),
}

/// Sort item containing original value and sort keys
///
/// Used during sorting to associate original items with their sort keys.
#[derive(Debug)]
struct SortItem {
    /// Original collection item
    original_item: FhirPathValue,
    /// Sort keys with descending flags
    sort_keys: Vec<(SortKey, bool)>,
}

/// Lambda expression evaluator for collection functions
///
/// The LambdaEvaluator manages the evaluation of lambda expressions within
/// collection functions, providing proper variable scoping and context management.
///
/// # Examples
///
/// ```rust,no_run
/// use octofhir_fhirpath::evaluator::{LambdaEvaluator, EvaluationContext};
/// use octofhir_fhirpath::{Collection, FhirPathValue};
/// use std::sync::Arc;
///
/// # async fn example() -> octofhir_fhirpath::Result<()> {
/// let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
/// let mut lambda_evaluator = LambdaEvaluator::new(global_context);
///
/// // Create test collection: [1, 2, 3, 4, 5]
/// let collection = Collection::from_values(vec![
///     FhirPathValue::Integer(1),
///     FhirPathValue::Integer(2),
///     FhirPathValue::Integer(3),
///     FhirPathValue::Integer(4),
///     FhirPathValue::Integer(5),
/// ]);
///
/// // This would require a parsed lambda expression
/// // let result = lambda_evaluator.evaluate_where(&collection, &lambda_expr).await?;
/// # Ok(())
/// # }
/// ```
pub struct LambdaEvaluator {
    /// Scope manager for variable scoping
    scope_manager: ScopeManager,
}

impl LambdaEvaluator {
    /// Create new lambda evaluator with global context
    ///
    /// # Arguments
    /// * `global_context` - Global evaluation context containing user variables and built-ins
    pub fn new(global_context: Arc<EvaluationContext>) -> Self {
        Self {
            scope_manager: ScopeManager::new(global_context),
        }
    }

    /// Evaluate where() lambda for collection filtering
    ///
    /// Filters a collection by evaluating a lambda expression for each item.
    /// Items where the lambda expression returns a truthy value are included
    /// in the result.
    ///
    /// # Arguments
    /// * `collection` - Input collection to filter
    /// * `lambda_expr` - Lambda expression to evaluate for each item
    ///
    /// # Returns
    /// Filtered collection containing items where lambda expression is truthy
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For expression: collection.where($this > 3)
    /// // Would filter [1, 2, 3, 4, 5] to [4, 5]
    /// ```
    pub async fn evaluate_where(
        &mut self,
        collection: &Collection,
        lambda_expr: &ExpressionNode,
        evaluator: &dyn LambdaExpressionEvaluator,
    ) -> crate::core::Result<Collection> {
        let mut filtered_items = Vec::new();

        // Fast path: exists() on property path, or bare property path truthiness
        if let Some(path) = extract_exists_on_path(lambda_expr) {
            for item in collection.iter() {
                let exists = match item {
                    FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                        get_json_at_path(j, &path)
                            .map(|v| match v {
                                serde_json::Value::Null => false,
                                serde_json::Value::String(s) => !s.is_empty(),
                                serde_json::Value::Array(a) => !a.is_empty(),
                                _ => true,
                            })
                            .unwrap_or(false)
                    }
                    _ => false,
                };
                if exists {
                    filtered_items.push(item.clone());
                }
            }
            return Ok(Collection::from_values(filtered_items));
        }

        // Fast path: simple predicate property ==/!= literal (string/int/bool)
        if let Some((prop_path, expected, op)) = extract_simple_literal_compare(lambda_expr) {
            for item in collection.iter() {
                let matches = match item {
                    FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                        let val = get_json_at_path(j, &prop_path);
                        match expected {
                            SimpleLiteral::String(ref s) => val
                                .and_then(|v| v.as_str())
                                .map(|x| cmp_eq_ne(x == s, op))
                                .unwrap_or(false),
                            SimpleLiteral::Integer(i) => val
                                .and_then(|v| v.as_i64())
                                .map(|x| cmp_eq_ne(x == i, op))
                                .unwrap_or(false),
                            SimpleLiteral::Boolean(b) => val
                                .and_then(|v| v.as_bool())
                                .map(|x| cmp_eq_ne(x == b, op))
                                .unwrap_or(false),
                        }
                    }
                    _ => false,
                };
                if matches {
                    filtered_items.push(item.clone());
                }
            }
            return Ok(Collection::from_values(filtered_items));
        }

        // Reusable lambda context: inherit built-ins and captured variables once
        let mut lambda_context = self.scope_manager.create_lambda_base_context();

        for (index, item) in collection.iter().enumerate() {
            // Update context for item and index
            self.scope_manager
                .update_lambda_item(&mut lambda_context, item, index);
            // Evaluate lambda expression
            let result = evaluator
                .evaluate_expression(lambda_expr, &lambda_context)
                .await?;

            // Check if result is truthy
            if is_truthy(&result) {
                filtered_items.push(item.clone());
            }
        }

        Ok(Collection::from_values(filtered_items))
    }

    /// Evaluate select() lambda for collection transformation
    ///
    /// Transforms a collection by evaluating a lambda expression for each item.
    /// The results of the lambda expressions are collected into the result collection.
    ///
    /// # Arguments
    /// * `collection` - Input collection to transform
    /// * `lambda_expr` - Lambda expression to evaluate for each item
    ///
    /// # Returns
    /// Transformed collection containing the results of lambda expressions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For expression: Patient.name.select(family)
    /// // Would extract family names from all name elements
    /// ```
    pub async fn evaluate_select(
        &mut self,
        collection: &Collection,
        lambda_expr: &ExpressionNode,
        evaluator: &dyn LambdaExpressionEvaluator,
    ) -> crate::core::Result<Collection> {
        let mut transformed_items = Vec::new();

        // Fast path: simple projection of a property path
        if let Some(path) = extract_property_path(lambda_expr) {
            for item in collection.iter() {
                if let FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) = item {
                    if let Some(val) = get_json_at_path(j, &path) {
                        if let Some(fp) = json_to_fhirpath_value(val) {
                            transformed_items.push(fp);
                        }
                    }
                }
            }
            return Ok(Collection::from_values(transformed_items));
        }

        // Reusable lambda context
        let mut lambda_context = self.scope_manager.create_lambda_base_context();

        for (index, item) in collection.iter().enumerate() {
            self.scope_manager
                .update_lambda_item(&mut lambda_context, item, index);
            // Evaluate lambda expression
            let result = evaluator
                .evaluate_expression(lambda_expr, &lambda_context)
                .await?;

            // Add all result items to transformed collection
            transformed_items.extend(result.into_vec());
        }

        Ok(Collection::from_values(transformed_items))
    }

    /// Evaluate all() lambda for universal quantification
    ///
    /// Checks if all items in a collection satisfy a lambda expression.
    /// Returns true if the lambda expression evaluates to a truthy value
    /// for all items, or true for empty collections (vacuous truth).
    ///
    /// # Arguments
    /// * `collection` - Input collection to check
    /// * `lambda_expr` - Lambda expression to evaluate for each item
    ///
    /// # Returns
    /// True if all items satisfy the lambda expression, false otherwise
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For expression: Patient.name.all(family.exists())
    /// // Would check if all names have a family element
    /// ```
    pub async fn evaluate_all(
        &mut self,
        collection: &Collection,
        lambda_expr: &ExpressionNode,
        evaluator: &dyn LambdaExpressionEvaluator,
    ) -> crate::core::Result<bool> {
        // Empty collection returns true (vacuous truth)
        if collection.is_empty() {
            return Ok(true);
        }

        // Fast path: exists() on property path or bare property path
        if let Some(path) = extract_exists_on_path(lambda_expr) {
            for item in collection.iter() {
                let ok = match item {
                    FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                        get_json_at_path(j, &path)
                            .map(|v| match v {
                                serde_json::Value::Null => false,
                                serde_json::Value::String(s) => !s.is_empty(),
                                serde_json::Value::Array(a) => !a.is_empty(),
                                _ => true,
                            })
                            .unwrap_or(false)
                    }
                    _ => false,
                };
                if !ok {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        // Fast path: simple literal compare across all items
        if let Some((path, expected, op)) = extract_simple_literal_compare(lambda_expr) {
            for item in collection.iter() {
                let ok = match item {
                    FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                        let val = get_json_at_path(j, &path);
                        match expected {
                            SimpleLiteral::String(ref s) => val
                                .and_then(|v| v.as_str())
                                .map(|x| cmp_eq_ne(x == s, op))
                                .unwrap_or(false),
                            SimpleLiteral::Integer(i) => val
                                .and_then(|v| v.as_i64())
                                .map(|x| cmp_eq_ne(x == i, op))
                                .unwrap_or(false),
                            SimpleLiteral::Boolean(b) => val
                                .and_then(|v| v.as_bool())
                                .map(|x| cmp_eq_ne(x == b, op))
                                .unwrap_or(false),
                        }
                    }
                    _ => false,
                };
                if !ok {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        // Reusable lambda context
        let mut lambda_context = self.scope_manager.create_lambda_base_context();

        for (index, item) in collection.iter().enumerate() {
            self.scope_manager
                .update_lambda_item(&mut lambda_context, item, index);
            // Evaluate lambda expression
            let result = evaluator
                .evaluate_expression(lambda_expr, &lambda_context)
                .await?;

            // If any item is falsy, return false immediately
            if !is_truthy(&result) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Evaluate repeat() lambda for recursive projection with cycle detection
    ///
    /// Implements the FHIRPath repeat() function according to the specification:
    /// - Recursively evaluates the projection expression starting with the input collection
    /// - Adds unique nodes to the result (prevents infinite loops)
    /// - Continues until no new unique nodes are found
    /// - Uses a queue-based approach to process items iteratively
    /// - Includes cycle detection and stack overflow protection
    ///
    /// # Arguments
    /// * `collection` - Input collection to start projection from
    /// * `lambda_expr` - Lambda expression to evaluate for each item
    /// * `evaluator` - Expression evaluator for lambda execution
    /// * `max_iterations` - Optional maximum iterations (defaults to 10,000 for safety)
    /// * `max_unique_items` - Optional maximum unique items limit (defaults to 100,000)
    ///
    /// # Returns
    /// Collection containing all unique nodes found through recursive projection
    ///
    /// # Infinite Loop Prevention
    /// - Uses unique node tracking via hash set
    /// - Implements maximum iteration limits
    /// - Implements maximum unique items limits
    /// - Stops when input queue becomes empty (no new nodes)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For expression: ValueSet.expansion.repeat(contains)
    /// // Would recursively find all 'contains' nodes in nested structure
    ///
    /// // For expression: Questionnaire.repeat(item)
    /// // Would recursively find all nested 'item' elements
    /// ```
    pub async fn evaluate_repeat(
        &mut self,
        collection: &Collection,
        lambda_expr: &ExpressionNode,
        evaluator: &dyn LambdaExpressionEvaluator,
        max_iterations: Option<usize>,
        max_unique_items: Option<usize>,
    ) -> crate::core::Result<Collection> {
        use std::collections::{HashMap, VecDeque};

        let max_iterations = max_iterations.unwrap_or(10_000);
        let max_unique_items = max_unique_items.unwrap_or(100_000);

        // Unique node tracking using more robust key generation
        let mut seen_nodes = HashMap::new();
        let mut result_items = Vec::new();

        // Input queue for iterative processing (prevents stack overflow)
        // Store indices into the original collection to avoid cloning large items unnecessarily
        let mut input_queue: VecDeque<FhirPathValue> = VecDeque::new();
        for item in collection.iter() {
            input_queue.push_back(item.clone());
        }

        let mut iteration_count = 0;

        // Reusable lambda context: inherit built-ins and captured variables once
        let mut lambda_context = self.scope_manager.create_lambda_base_context();

        // Process until queue is empty or limits reached
        while !input_queue.is_empty()
            && iteration_count < max_iterations
            && result_items.len() < max_unique_items
        {
            iteration_count += 1;

            // Take the next item from queue
            let current_item = input_queue.pop_front().unwrap();

            // Generate unique key for this node (more robust than Debug format)
            let item_key = self.generate_unique_node_key(&current_item);

            // Skip if we've already seen this exact node
            if seen_nodes.contains_key(&item_key) {
                continue;
            }

            // Mark this node as seen and add to results
            seen_nodes.insert(item_key, true);
            result_items.push(current_item.clone());

            // Update context for current item (index doesn't apply to repeat)
            self.scope_manager
                .update_lambda_item(&mut lambda_context, &current_item, 0);

            // Evaluate lambda expression on current item
            let projection_result = evaluator
                .evaluate_expression(lambda_expr, &lambda_context)
                .await?;

            // Add projection results to input queue for further processing
            for new_item in projection_result.into_vec() {
                let new_item_key = self.generate_unique_node_key(&new_item);

                // Only add to queue if we haven't seen this node before
                if !seen_nodes.contains_key(&new_item_key) {
                    input_queue.push_back(new_item);
                }
            }

            // Safety check for memory usage
            if result_items.len() >= max_unique_items {
                // Log warning but don't fail - truncate results
                eprintln!(
                    "repeat() function reached maximum unique items limit ({}). Results truncated for safety.",
                    max_unique_items
                );
                break;
            }
        }

        // Report if we hit iteration limit (potential infinite loop detected)
        if iteration_count >= max_iterations {
            eprintln!(
                "repeat() function reached maximum iteration limit ({}). Potential infinite loop detected and prevented.",
                max_iterations
            );
        }

        Ok(Collection::from_values(result_items))
    }

    /// Generate a unique key for a FhirPathValue for cycle detection
    ///
    /// This creates a more robust unique identifier than Debug formatting,
    /// taking into account the structure and content of complex values.
    ///
    /// # Arguments
    /// * `value` - FhirPathValue to generate key for
    ///
    /// # Returns
    /// String key that uniquely identifies the value
    fn generate_unique_node_key(&self, value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(s) => format!("string:{}", s),
            FhirPathValue::Integer(i) => format!("integer:{}", i),
            FhirPathValue::Decimal(d) => format!("decimal:{}", d),
            FhirPathValue::Boolean(b) => format!("boolean:{}", b),
            FhirPathValue::Date(d) => format!("date:{}", d),
            FhirPathValue::DateTime(dt) => format!("datetime:{}", dt),
            FhirPathValue::Time(t) => format!("time:{}", t),
            FhirPathValue::Quantity { value, unit, .. } => {
                format!("quantity:{}:{}", value, unit.as_deref().unwrap_or("none"))
            }
            FhirPathValue::Id(id) => format!("id:{}", id),
            FhirPathValue::Base64Binary(b64) => format!("base64:{}bytes", b64.len()),
            FhirPathValue::Uri(uri) => format!("uri:{}", uri),
            FhirPathValue::Url(url) => format!("url:{}", url),
            FhirPathValue::Resource(resource) => {
                // For resources, include resourceType and id for uniqueness
                let resource_type = resource
                    .get("resourceType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let id = resource
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("no-id");
                format!("resource:{}:{}", resource_type, id)
            }
            FhirPathValue::JsonValue(json) => {
                // For JSON values, create hash of the content
                let content = serde_json::to_string(json).unwrap_or_default();
                format!("json:{}", content)
            }
            FhirPathValue::Collection(items) => {
                // For collections, hash the combination of all items
                let mut combined = String::from("collection:");
                for item in items {
                    combined.push_str(&self.generate_unique_node_key(item));
                    combined.push(':');
                }
                combined
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                format!("typeinfo:{}:{}", namespace, name)
            }
            FhirPathValue::Empty => "empty:".to_string(),
        }
    }

    /// Evaluate aggregate() lambda for collection reduction
    ///
    /// Reduces a collection to a single value using a lambda expression.
    /// The lambda has access to $total (running total) and $this (current item).
    ///
    /// # Arguments
    /// * `collection` - Input collection to aggregate
    /// * `lambda_expr` - Lambda expression to evaluate for aggregation
    /// * `initial_value` - Optional initial value for aggregation
    ///
    /// # Returns
    /// Single aggregated value
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For expression: (1 | 2 | 3).aggregate($total + $this, 0)
    /// // Would compute sum: 0 + 1 + 2 + 3 = 6
    /// ```
    pub async fn evaluate_aggregate(
        &mut self,
        collection: &Collection,
        lambda_expr: &ExpressionNode,
        initial_value: Option<FhirPathValue>,
        evaluator: &dyn LambdaExpressionEvaluator,
    ) -> crate::core::Result<FhirPathValue> {
        // Check if we have an initial value to determine processing mode
        let has_initial_value = initial_value.is_some();

        // Start with initial value or first item of collection
        let mut total = if let Some(init) = initial_value {
            init
        } else if !collection.is_empty() {
            // For 1-argument form, start with first item
            collection.first().unwrap().clone()
        } else {
            // Empty collection case
            return Ok(FhirPathValue::Collection(Vec::new()));
        };

        // Reusable lambda context: inherit built-ins and captured variables once
        let mut lambda_context = self.scope_manager.create_lambda_base_context();

        // For 1-argument form, skip first item since it's used as initial value
        let items_to_process: Vec<(usize, &FhirPathValue)> = if has_initial_value {
            collection.iter().enumerate().collect() // Process all items
        } else {
            collection.iter().enumerate().skip(1).collect() // Skip first item
        };

        for (index, item) in items_to_process {
            // Update context for item, index, and total
            self.scope_manager
                .update_lambda_item(&mut lambda_context, item, index);
            lambda_context.set_variable("$total".to_string(), total.clone());

            // Evaluate lambda expression
            let result = evaluator
                .evaluate_expression(lambda_expr, &lambda_context)
                .await?;

            // Update total with result (take first item if collection)
            total = if result.is_empty() {
                total // Keep current total if result is empty
            } else {
                result.first().unwrap().clone()
            };
        }

        Ok(total)
    }

    /// Evaluate sort() function for collection sorting
    ///
    /// Sorts a collection using natural ordering or custom lambda expressions.
    /// Supports multiple sort criteria and descending order using negative prefix.
    ///
    /// # Arguments
    /// * `collection` - Input collection to sort
    /// * `sort_criteria` - Optional vector of sort expressions (can be empty for natural sort)
    /// * `evaluator` - Expression evaluator
    ///
    /// # Returns
    /// Sorted collection
    ///
    /// # Supported Variants
    /// - `sort()` - Natural sort without criteria
    /// - `sort($this)` - Sort using lambda expression
    /// - `sort(-$this)` - Sort using lambda expression in descending order
    /// - `sort(-family, -given.first())` - Multi-criteria sort with descending support
    ///
    /// # Examples
    /// ```rust,no_run
    /// // Natural sort: (3 | 2 | 1).sort() = (1 | 2 | 3)
    /// // Descending: (1 | 2 | 3).sort(-$this) = (3 | 2 | 1)
    /// // Multi-criteria: Patient.name.sort(-family, -given.first())
    /// ```
    pub async fn evaluate_sort(
        &mut self,
        collection: &Collection,
        sort_criteria: Vec<SortCriterion>,
        evaluator: &dyn LambdaExpressionEvaluator,
    ) -> crate::core::Result<Collection> {
        // Handle empty collection
        if collection.is_empty() {
            return Ok(Collection::empty());
        }

        // Natural sort if no criteria specified
        if sort_criteria.is_empty() {
            return self.natural_sort(collection);
        }

        // Lambda sort with criteria
        self.lambda_sort(collection, sort_criteria, evaluator).await
    }

    /// Perform natural sort without lambda expressions
    ///
    /// Sorts collection items by their natural ordering (string, numeric, etc.)
    fn natural_sort(&self, collection: &Collection) -> crate::core::Result<Collection> {
        let mut items = collection.clone().into_vec();

        // Sort using natural comparison
        items.sort_by(|a, b| self.compare_values_naturally(a, b));

        Ok(Collection::from_values(items))
    }

    /// Perform lambda sort with multiple criteria
    ///
    /// Sorts collection using lambda expressions, supporting multiple sort keys
    /// and ascending/descending order for each key.
    async fn lambda_sort(
        &mut self,
        collection: &Collection,
        sort_criteria: Vec<SortCriterion>,
        evaluator: &dyn LambdaExpressionEvaluator,
    ) -> crate::core::Result<Collection> {
        // Create sort keys for each item
        let mut sort_items = Vec::new();

        // Reusable lambda context: inherit built-ins and captured variables once
        let mut lambda_context = self.scope_manager.create_lambda_base_context();

        for (index, item) in collection.iter().enumerate() {
            // Update context for current item and index
            self.scope_manager
                .update_lambda_item(&mut lambda_context, item, index);

            // Evaluate all sort criteria for this item
            let mut sort_keys = Vec::new();

            for criterion in &sort_criteria {
                // Evaluate the sort expression
                let result = evaluator
                    .evaluate_expression(&criterion.expression, &lambda_context)
                    .await?;

                // Extract sort key (use first value or empty)
                let sort_key = if result.is_empty() {
                    SortKey::Empty
                } else {
                    self.value_to_sort_key(result.first().unwrap())
                };

                sort_keys.push((sort_key, criterion.descending));
            }

            sort_items.push(SortItem {
                original_item: item.clone(),
                sort_keys,
            });
        }

        // Sort items by their sort keys
        sort_items.sort_by(|a, b| self.compare_sort_items(a, b));

        // Extract sorted items
        let sorted_items: Vec<_> = sort_items
            .into_iter()
            .map(|sort_item| sort_item.original_item)
            .collect();

        Ok(Collection::from_values(sorted_items))
    }

    /// Compare two values using natural ordering
    fn compare_values_naturally(&self, a: &FhirPathValue, b: &FhirPathValue) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (a, b) {
            // Same types - direct comparison
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a.cmp(b),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a.cmp(b),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a.cmp(b),
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a.cmp(b),

            // Mixed numeric types
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                rust_decimal::Decimal::from(*a).cmp(b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                a.cmp(&rust_decimal::Decimal::from(*b))
            }

            // Different types - use type precedence: numbers < strings < booleans < others
            (FhirPathValue::Integer(_) | FhirPathValue::Decimal(_), FhirPathValue::String(_)) => {
                Ordering::Less
            }
            (FhirPathValue::String(_), FhirPathValue::Integer(_) | FhirPathValue::Decimal(_)) => {
                Ordering::Greater
            }
            (FhirPathValue::Integer(_) | FhirPathValue::Decimal(_), FhirPathValue::Boolean(_)) => {
                Ordering::Less
            }
            (FhirPathValue::Boolean(_), FhirPathValue::Integer(_) | FhirPathValue::Decimal(_)) => {
                Ordering::Greater
            }
            (FhirPathValue::String(_), FhirPathValue::Boolean(_)) => Ordering::Less,
            (FhirPathValue::Boolean(_), FhirPathValue::String(_)) => Ordering::Greater,

            // All other cases - fallback to string comparison
            _ => format!("{:?}", a).cmp(&format!("{:?}", b)),
        }
    }

    /// Convert FhirPathValue to sortable key
    fn value_to_sort_key(&self, value: &FhirPathValue) -> SortKey {
        match value {
            FhirPathValue::String(s) => SortKey::String(s.clone()),
            FhirPathValue::Integer(i) => SortKey::Number(rust_decimal::Decimal::from(*i)),
            FhirPathValue::Decimal(d) => SortKey::Number(*d),
            FhirPathValue::Boolean(b) => SortKey::Boolean(*b),
            _ => SortKey::String(format!("{:?}", value)),
        }
    }

    /// Compare two sort items using their sort keys
    fn compare_sort_items(&self, a: &SortItem, b: &SortItem) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        // Compare each sort key in order
        for (key_a, key_b) in a.sort_keys.iter().zip(b.sort_keys.iter()) {
            let result = self.compare_sort_keys(&key_a.0, &key_b.0);

            // Apply descending order if needed
            let final_result = if key_a.1 {
                // descending
                result.reverse()
            } else {
                result
            };

            // If not equal, return this comparison result
            if final_result != Ordering::Equal {
                return final_result;
            }
        }

        // All sort keys are equal
        Ordering::Equal
    }

    /// Compare two sort keys
    fn compare_sort_keys(&self, a: &SortKey, b: &SortKey) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (a, b) {
            (SortKey::Empty, SortKey::Empty) => Ordering::Equal,
            (SortKey::Empty, _) => Ordering::Less, // Empty values sort first
            (_, SortKey::Empty) => Ordering::Greater,

            (SortKey::String(a), SortKey::String(b)) => a.cmp(b),
            (SortKey::Number(a), SortKey::Number(b)) => a.cmp(b),
            (SortKey::Boolean(a), SortKey::Boolean(b)) => a.cmp(b),

            // Mixed types - use type precedence
            (SortKey::Number(_), SortKey::String(_)) => Ordering::Less,
            (SortKey::String(_), SortKey::Number(_)) => Ordering::Greater,
            (SortKey::Number(_), SortKey::Boolean(_)) => Ordering::Less,
            (SortKey::Boolean(_), SortKey::Number(_)) => Ordering::Greater,
            (SortKey::String(_), SortKey::Boolean(_)) => Ordering::Less,
            (SortKey::Boolean(_), SortKey::String(_)) => Ordering::Greater,
        }
    }

    /// Evaluate iif() function with short-circuit evaluation
    ///
    /// Conditional function that evaluates expressions based on a boolean condition.
    /// Uses short-circuit evaluation: only evaluates the chosen branch.
    ///
    /// # Arguments
    /// * `condition_expr` - Boolean condition expression to evaluate
    /// * `true_expr` - Expression to evaluate if condition is true
    /// * `false_expr` - Optional expression to evaluate if condition is false
    /// * `evaluator` - Expression evaluator
    /// * `context` - Current evaluation context
    ///
    /// # Returns
    /// Result of the chosen branch expression
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For expression: iif(Patient.active, Patient.name, {})
    /// // Would return patient name if active, empty collection otherwise
    /// ```
    pub async fn evaluate_iif(
        condition_expr: &ExpressionNode,
        true_expr: &ExpressionNode,
        false_expr: Option<&ExpressionNode>,
        evaluator: &dyn LambdaExpressionEvaluator,
        context: &EvaluationContext,
    ) -> crate::core::Result<Collection> {
        // Evaluate condition first
        let condition_result = evaluator
            .evaluate_expression(condition_expr, context)
            .await?;

        // Check if condition is truthy
        let is_condition_true = is_truthy(&condition_result);

        if is_condition_true {
            // Short-circuit: only evaluate true branch
            evaluator.evaluate_expression(true_expr, context).await
        } else {
            // Short-circuit: only evaluate false branch if provided
            if let Some(false_expr) = false_expr {
                evaluator.evaluate_expression(false_expr, context).await
            } else {
                // Return empty collection if no else branch
                Ok(Collection::empty())
            }
        }
    }

    /// Get current scope depth for debugging
    pub fn scope_depth(&self) -> usize {
        self.scope_manager.scope_depth()
    }

    /// Get scope information for debugging
    pub fn get_scope_info(&self) -> Vec<crate::evaluator::scoping::ScopeInfo> {
        self.scope_manager.get_scope_info()
    }
}

/// Trait for evaluating expressions within lambda contexts
///
/// This trait abstracts the expression evaluation to allow the LambdaEvaluator
/// to work with different evaluation engines while maintaining proper scoping.
#[async_trait::async_trait(?Send)]
pub trait LambdaExpressionEvaluator {
    /// Evaluate an expression in the given context
    ///
    /// # Arguments
    /// * `expr` - Expression to evaluate
    /// * `context` - Evaluation context with variables and built-ins
    ///
    /// # Returns
    /// Result of expression evaluation
    async fn evaluate_expression(
        &self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> crate::core::Result<Collection>;
}

/// Check if a collection result is truthy for boolean evaluation
///
/// FHIRPath truthiness rules:
/// - Empty collection is falsy
/// - Single boolean false is falsy
/// - Single integer 0 is falsy
/// - Single decimal 0.0 is falsy
/// - Single empty string is falsy
/// - Everything else is truthy
///
/// # Arguments
/// * `result` - Collection to check for truthiness
///
/// # Returns
/// True if the collection is truthy, false otherwise
fn is_truthy(result: &Collection) -> bool {
    match result.len() {
        0 => false, // Empty collection is falsy
        1 => {
            // Single item - check its boolean value
            match result.first().unwrap() {
                FhirPathValue::Boolean(b) => *b,
                FhirPathValue::Integer(i) => *i != 0,
                FhirPathValue::Decimal(d) => *d != rust_decimal::Decimal::ZERO,
                FhirPathValue::String(s) => !s.is_empty(),
                _ => true, // Non-empty non-boolean values are truthy
            }
        }
        _ => true, // Multiple items are truthy
    }
}

/// Create a simple lambda evaluator adapter
///
/// This is a convenience function for creating a lambda evaluator that works
/// with a closure for expression evaluation.
pub fn create_lambda_evaluator_adapter<F>(
    global_context: Arc<EvaluationContext>,
    _evaluator_fn: F,
) -> LambdaEvaluator
where
    F: Fn(&ExpressionNode, &EvaluationContext) -> crate::core::Result<Collection>
        + Send
        + Sync
        + 'static,
{
    LambdaEvaluator::new(global_context)
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use crate::ast::{ExpressionNode, LiteralNode, LiteralValue};
    use crate::core::{Collection, FhirPathValue};

    // Mock evaluator for testing
    struct MockExpressionEvaluator {
        result: Collection,
    }

    #[async_trait::async_trait]
    impl LambdaExpressionEvaluator for MockExpressionEvaluator {
        async fn evaluate_expression(
            &self,
            _expr: &ExpressionNode,
            _context: &EvaluationContext,
        ) -> crate::core::Result<Collection> {
            Ok(self.result.clone())
        }
    }

    #[tokio::test]
    async fn test_lambda_where_evaluation() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test collection: [1, 2, 3, 4, 5]
        let collection = Collection::from_values(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
            FhirPathValue::Integer(5),
        ]);

        // Mock lambda expression that returns true for items > 3
        let lambda_expr = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Boolean(true),
            location: None,
        });

        // Mock evaluator that returns true for even indices (simulating > 3)
        let mock_evaluator = MockExpressionEvaluator {
            result: Collection::from_values(vec![FhirPathValue::Boolean(true)]),
        };

        let result = lambda_evaluator
            .evaluate_where(&collection, &lambda_expr, &mock_evaluator)
            .await
            .unwrap();

        // All items should pass since mock always returns true
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_lambda_all_evaluation() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test collection: [1, 2, 3]
        let collection = Collection::from_values(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);

        let lambda_expr = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Boolean(true),
            location: None,
        });

        // Mock evaluator that always returns true
        let mock_evaluator = MockExpressionEvaluator {
            result: Collection::from_values(vec![FhirPathValue::Boolean(true)]),
        };

        let result = lambda_evaluator
            .evaluate_all(&collection, &lambda_expr, &mock_evaluator)
            .await
            .unwrap();
        assert_eq!(result, true);
    }

    #[tokio::test]
    async fn test_lambda_sort_natural() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test natural sort: (3 | 2 | 1).sort() = (1 | 2 | 3)
        let collection = Collection::from_values(vec![
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1),
        ]);

        let result = lambda_evaluator
            .evaluate_sort(
                &collection,
                vec![],
                &MockExpressionEvaluator {
                    result: Collection::empty(),
                },
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.first().unwrap(), &FhirPathValue::Integer(1));
        assert_eq!(result.get(1).unwrap(), &FhirPathValue::Integer(2));
        assert_eq!(result.get(2).unwrap(), &FhirPathValue::Integer(3));
    }

    #[tokio::test]
    async fn test_lambda_sort_with_criteria() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test sort with ascending criteria: (3 | 2 | 1).sort($this) = (1 | 2 | 3)
        let collection = Collection::from_values(vec![
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1),
        ]);

        let sort_criteria = vec![SortCriterion {
            expression: ExpressionNode::Literal(crate::ast::LiteralNode {
                value: crate::ast::LiteralValue::Integer(1), // Mock - would be $this in real usage
                location: None,
            }),
            descending: false,
        }];

        let mock_evaluator = MockExpressionEvaluatorForSort::new();
        let result = lambda_evaluator
            .evaluate_sort(&collection, sort_criteria, &mock_evaluator)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        // Results depend on the mock evaluator behavior
    }

    #[tokio::test]
    async fn test_lambda_sort_descending() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test natural sort strings: ('c' | 'b' | 'a').sort() = ('a' | 'b' | 'c')
        let collection = Collection::from_values(vec![
            FhirPathValue::String("c".to_string()),
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("a".to_string()),
        ]);

        let result = lambda_evaluator
            .evaluate_sort(
                &collection,
                vec![],
                &MockExpressionEvaluator {
                    result: Collection::empty(),
                },
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(
            result.first().unwrap(),
            &FhirPathValue::String("a".to_string())
        );
        assert_eq!(
            result.get(1).unwrap(),
            &FhirPathValue::String("b".to_string())
        );
        assert_eq!(
            result.get(2).unwrap(),
            &FhirPathValue::String("c".to_string())
        );
    }

    #[tokio::test]
    async fn test_empty_collection_sort() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        let empty_collection = Collection::empty();
        let result = lambda_evaluator
            .evaluate_sort(
                &empty_collection,
                vec![],
                &MockExpressionEvaluator {
                    result: Collection::empty(),
                },
            )
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_repeat_simple_projection() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test simple repeat that finds no additional items
        let collection = Collection::from_values(vec![
            FhirPathValue::String("item1".to_string()),
            FhirPathValue::String("item2".to_string()),
        ]);

        let lambda_expr = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String("no-match".to_string()),
            location: None,
        });

        // Mock evaluator that returns empty for all evaluations (no projection results)
        let mock_evaluator = MockExpressionEvaluator {
            result: Collection::empty(),
        };

        let result = lambda_evaluator
            .evaluate_repeat(
                &collection,
                &lambda_expr,
                &mock_evaluator,
                Some(100), // Small max iterations for testing
                Some(50),  // Small max unique items for testing
            )
            .await
            .unwrap();

        // Should return original items since no projection results were found
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_repeat_with_cycle_detection() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test repeat with cyclical projection (A -> B -> A)
        let collection = Collection::from_values(vec![FhirPathValue::String("A".to_string())]);

        let lambda_expr = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String("B".to_string()),
            location: None,
        });

        // Mock evaluator that creates a cycle: A -> B, B -> A
        let mock_evaluator = MockCyclicalEvaluator::new();

        let result = lambda_evaluator
            .evaluate_repeat(
                &collection,
                &lambda_expr,
                &mock_evaluator,
                Some(10), // Low iteration limit to test cycle detection
                Some(10), // Low unique items limit
            )
            .await
            .unwrap();

        // Should detect cycle and stop, returning both A and B
        assert!(result.len() >= 1);
        assert!(result.len() <= 3); // Original A, projected B, and possibly cycle detection
    }

    #[tokio::test]
    async fn test_repeat_iteration_limit() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test repeat with iteration limit hit
        let collection = Collection::from_values(vec![FhirPathValue::Integer(1)]);

        let lambda_expr = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Integer(2),
            location: None,
        });

        // Mock evaluator that always produces new items (would cause infinite loop)
        let mock_evaluator = MockInfiniteEvaluator::new();

        let result = lambda_evaluator
            .evaluate_repeat(
                &collection,
                &lambda_expr,
                &mock_evaluator,
                Some(5),   // Very low iteration limit
                Some(100), // Higher unique items limit
            )
            .await
            .unwrap();

        // Should hit iteration limit and stop
        assert!(result.len() <= 10); // Should be limited by max iterations
    }

    #[tokio::test]
    async fn test_repeat_unique_items_limit() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test repeat with unique items limit hit
        let collection = Collection::from_values(vec![FhirPathValue::Integer(1)]);

        let lambda_expr = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Integer(2),
            location: None,
        });

        // Mock evaluator that produces many unique items
        let mock_evaluator = MockUniqueItemsEvaluator::new();

        let result = lambda_evaluator
            .evaluate_repeat(
                &collection,
                &lambda_expr,
                &mock_evaluator,
                Some(1000), // High iteration limit
                Some(3),    // Very low unique items limit
            )
            .await
            .unwrap();

        // Should hit unique items limit and stop
        assert!(result.len() <= 3); // Should be limited by max unique items
    }

    #[tokio::test]
    async fn test_repeat_empty_collection() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test repeat with empty input collection
        let empty_collection = Collection::empty();

        let lambda_expr = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String("test".to_string()),
            location: None,
        });

        let mock_evaluator = MockExpressionEvaluator {
            result: Collection::empty(),
        };

        let result = lambda_evaluator
            .evaluate_repeat(
                &empty_collection,
                &lambda_expr,
                &mock_evaluator,
                None, // Use defaults
                None, // Use defaults
            )
            .await
            .unwrap();

        // Empty input should return empty result
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_generate_unique_node_key() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let lambda_evaluator = LambdaEvaluator::new(global_context);

        // Test unique key generation for different value types
        let string_val = FhirPathValue::String("test".to_string());
        let int_val = FhirPathValue::Integer(42);
        let bool_val = FhirPathValue::Boolean(true);

        let string_key = lambda_evaluator.generate_unique_node_key(&string_val);
        let int_key = lambda_evaluator.generate_unique_node_key(&int_val);
        let bool_key = lambda_evaluator.generate_unique_node_key(&bool_val);

        // Keys should be different for different values
        assert_ne!(string_key, int_key);
        assert_ne!(string_key, bool_key);
        assert_ne!(int_key, bool_key);

        // Same values should produce same keys
        let string_val2 = FhirPathValue::String("test".to_string());
        let string_key2 = lambda_evaluator.generate_unique_node_key(&string_val2);
        assert_eq!(string_key, string_key2);
    }

    // Mock evaluator for sort testing
    #[derive(Debug)]
    struct MockExpressionEvaluatorForSort {
        call_count: std::sync::Mutex<usize>,
    }

    impl MockExpressionEvaluatorForSort {
        fn new() -> Self {
            Self {
                call_count: std::sync::Mutex::new(0),
            }
        }
    }

    // Safe Send + Sync since Mutex provides synchronization
    unsafe impl Send for MockExpressionEvaluatorForSort {}
    unsafe impl Sync for MockExpressionEvaluatorForSort {}

    #[async_trait::async_trait]
    impl LambdaExpressionEvaluator for MockExpressionEvaluatorForSort {
        async fn evaluate_expression(
            &self,
            _expr: &ExpressionNode,
            context: &EvaluationContext,
        ) -> crate::core::Result<Collection> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            // Return the current $this value from context for sorting
            if let Some(this_value) = context.get_variable("this") {
                Ok(Collection::single(this_value.clone()))
            } else {
                Ok(Collection::empty())
            }
        }
    }

    // Mock evaluator that creates cyclical projections for testing cycle detection
    #[derive(Debug)]
    struct MockCyclicalEvaluator {
        call_count: std::sync::Mutex<usize>,
    }

    impl MockCyclicalEvaluator {
        fn new() -> Self {
            Self {
                call_count: std::sync::Mutex::new(0),
            }
        }
    }

    // Safe to send across threads
    unsafe impl Send for MockCyclicalEvaluator {}
    unsafe impl Sync for MockCyclicalEvaluator {}

    #[async_trait::async_trait]
    impl LambdaExpressionEvaluator for MockCyclicalEvaluator {
        async fn evaluate_expression(
            &self,
            _expr: &ExpressionNode,
            context: &EvaluationContext,
        ) -> crate::core::Result<Collection> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            // Create a cycle: A -> B, B -> A, anything else -> empty
            if let Some(this_value) = context.get_variable("this") {
                match this_value {
                    FhirPathValue::String(s) => {
                        if s == "A" {
                            Ok(Collection::single(FhirPathValue::String("B".to_string())))
                        } else if s == "B" {
                            Ok(Collection::single(FhirPathValue::String("A".to_string())))
                        } else {
                            Ok(Collection::empty())
                        }
                    }
                    _ => Ok(Collection::empty()),
                }
            } else {
                Ok(Collection::empty())
            }
        }
    }

    // Mock evaluator that always produces new items (for testing infinite loop prevention)
    #[derive(Debug)]
    struct MockInfiniteEvaluator {
        call_count: std::sync::Mutex<usize>,
    }

    impl MockInfiniteEvaluator {
        fn new() -> Self {
            Self {
                call_count: std::sync::Mutex::new(0),
            }
        }
    }

    unsafe impl Send for MockInfiniteEvaluator {}
    unsafe impl Sync for MockInfiniteEvaluator {}

    #[async_trait::async_trait]
    impl LambdaExpressionEvaluator for MockInfiniteEvaluator {
        async fn evaluate_expression(
            &self,
            _expr: &ExpressionNode,
            _context: &EvaluationContext,
        ) -> crate::core::Result<Collection> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            // Always produce a new unique item based on call count
            Ok(Collection::single(FhirPathValue::Integer(*count as i64)))
        }
    }

    // Mock evaluator that produces many unique items (for testing unique items limit)
    #[derive(Debug)]
    struct MockUniqueItemsEvaluator {
        call_count: std::sync::Mutex<usize>,
    }

    impl MockUniqueItemsEvaluator {
        fn new() -> Self {
            Self {
                call_count: std::sync::Mutex::new(0),
            }
        }
    }

    unsafe impl Send for MockUniqueItemsEvaluator {}
    unsafe impl Sync for MockUniqueItemsEvaluator {}

    #[async_trait::async_trait]
    impl LambdaExpressionEvaluator for MockUniqueItemsEvaluator {
        async fn evaluate_expression(
            &self,
            _expr: &ExpressionNode,
            _context: &EvaluationContext,
        ) -> crate::core::Result<Collection> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            // Produce two new unique items per call
            Ok(Collection::from_values(vec![
                FhirPathValue::String(format!("item_{}", *count)),
                FhirPathValue::String(format!("extra_{}", *count)),
            ]))
        }
    }

    #[test]
    fn test_is_truthy() {
        // Empty collection is falsy
        assert!(!is_truthy(&Collection::empty()));

        // Boolean values
        assert!(is_truthy(&Collection::from_values(vec![
            FhirPathValue::Boolean(true)
        ])));
        assert!(!is_truthy(&Collection::from_values(vec![
            FhirPathValue::Boolean(false)
        ])));

        // Integer values
        assert!(is_truthy(&Collection::from_values(vec![
            FhirPathValue::Integer(1)
        ])));
        assert!(!is_truthy(&Collection::from_values(vec![
            FhirPathValue::Integer(0)
        ])));

        // String values
        assert!(is_truthy(&Collection::from_values(vec![
            FhirPathValue::String("hello".to_string())
        ])));
        assert!(!is_truthy(&Collection::from_values(vec![
            FhirPathValue::String("".to_string())
        ])));

        // Multiple items are truthy
        assert!(is_truthy(&Collection::from_values(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(false),
        ])));
    }
}
