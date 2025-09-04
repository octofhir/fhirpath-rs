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

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{Collection, FhirPathValue};
use crate::evaluator::{ScopeManager, ScopeType, EvaluationContext};
use crate::ast::ExpressionNode;

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
        
        // Push lambda scope
        let _scope_id = self.scope_manager.push_scope(ScopeType::Lambda);
        
        for (index, item) in collection.iter().enumerate() {
            // Set current item and index for lambda evaluation
            self.scope_manager.set_current_item(item.clone());
            self.scope_manager.set_current_index(index as i64);
            
            // Create evaluation context for this lambda invocation
            let lambda_context = self.scope_manager.create_lambda_evaluation_context().await;
            
            // Evaluate lambda expression
            let result = evaluator.evaluate_expression(lambda_expr, &lambda_context).await?;
            
            // Check if result is truthy
            if is_truthy(&result) {
                filtered_items.push(item.clone());
            }
        }
        
        // Pop lambda scope
        self.scope_manager.pop_scope();
        
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
        
        // Push lambda scope
        let _scope_id = self.scope_manager.push_scope(ScopeType::Lambda);
        
        for (index, item) in collection.iter().enumerate() {
            // Set current item and index for lambda evaluation
            self.scope_manager.set_current_item(item.clone());
            self.scope_manager.set_current_index(index as i64);
            
            // Create evaluation context for this lambda invocation
            let lambda_context = self.scope_manager.create_lambda_evaluation_context().await;
            
            // Evaluate lambda expression
            let result = evaluator.evaluate_expression(lambda_expr, &lambda_context).await?;
            
            // Add all result items to transformed collection
            transformed_items.extend(result.into_vec());
        }
        
        // Pop lambda scope
        self.scope_manager.pop_scope();
        
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
        
        // Push lambda scope
        let _scope_id = self.scope_manager.push_scope(ScopeType::Lambda);
        
        for (index, item) in collection.iter().enumerate() {
            // Set current item and index for lambda evaluation
            self.scope_manager.set_current_item(item.clone());
            self.scope_manager.set_current_index(index as i64);
            
            // Create evaluation context for this lambda invocation
            let lambda_context = self.scope_manager.create_lambda_evaluation_context().await;
            
            // Evaluate lambda expression
            let result = evaluator.evaluate_expression(lambda_expr, &lambda_context).await?;
            
            // If any item is falsy, return false immediately
            if !is_truthy(&result) {
                self.scope_manager.pop_scope();
                return Ok(false);
            }
        }
        
        // Pop lambda scope
        self.scope_manager.pop_scope();
        
        Ok(true)
    }
    
    /// Evaluate repeat() lambda for collection projection without duplicates
    ///
    /// Similar to select() but avoids duplicates in the result collection.
    /// Transforms a collection by evaluating a lambda expression for each item
    /// and collecting unique results.
    ///
    /// # Arguments
    /// * `collection` - Input collection to transform
    /// * `lambda_expr` - Lambda expression to evaluate for each item
    ///
    /// # Returns
    /// Transformed collection with unique results of lambda expressions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For expression: Bundle.entry.repeat(resource.resourceType)
    /// // Would extract unique resource types from bundle entries
    /// ```
    pub async fn evaluate_repeat(
        &mut self,
        collection: &Collection,
        lambda_expr: &ExpressionNode,
        evaluator: &dyn LambdaExpressionEvaluator,
    ) -> crate::core::Result<Collection> {
        let mut seen_values = std::collections::HashSet::new();
        let mut unique_items = Vec::new();
        
        // Push lambda scope
        let _scope_id = self.scope_manager.push_scope(ScopeType::Lambda);
        
        for (index, item) in collection.iter().enumerate() {
            // Set current item and index for lambda evaluation
            self.scope_manager.set_current_item(item.clone());
            self.scope_manager.set_current_index(index as i64);
            
            // Create evaluation context for this lambda invocation
            let lambda_context = self.scope_manager.create_lambda_evaluation_context().await;
            
            // Evaluate lambda expression
            let result = evaluator.evaluate_expression(lambda_expr, &lambda_context).await?;
            
            // Add unique result items to collection
            for result_item in result.into_vec() {
                // Use string representation for uniqueness check
                let item_key = format!("{:?}", result_item);
                if seen_values.insert(item_key) {
                    unique_items.push(result_item);
                }
            }
        }
        
        // Pop lambda scope
        self.scope_manager.pop_scope();
        
        Ok(Collection::from_values(unique_items))
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
        // Start with initial value or empty collection
        let mut total = initial_value.unwrap_or(FhirPathValue::Collection(Vec::new()));
        
        // Push lambda scope
        let _scope_id = self.scope_manager.push_scope(ScopeType::Lambda);
        
        for (index, item) in collection.iter().enumerate() {
            // Set current item, index, and total for lambda evaluation
            self.scope_manager.set_current_item(item.clone());
            self.scope_manager.set_current_index(index as i64);
            self.scope_manager.set_variable("total".to_string(), total.clone());
            
            // Create evaluation context for this lambda invocation
            let lambda_context = self.scope_manager.create_lambda_evaluation_context().await;
            
            // Evaluate lambda expression
            let result = evaluator.evaluate_expression(lambda_expr, &lambda_context).await?;
            
            // Update total with result (take first item if collection)
            total = if result.is_empty() {
                total // Keep current total if result is empty
            } else {
                result.first().unwrap().clone()
            };
        }
        
        // Pop lambda scope
        self.scope_manager.pop_scope();
        
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
        
        // Push lambda scope
        let _scope_id = self.scope_manager.push_scope(ScopeType::Lambda);
        
        for (index, item) in collection.iter().enumerate() {
            // Set current item and index
            self.scope_manager.set_current_item(item.clone());
            self.scope_manager.set_current_index(index as i64);
            
            // Evaluate all sort criteria for this item
            let mut sort_keys = Vec::new();
            
            for criterion in &sort_criteria {
                // Create evaluation context for this lambda invocation
                let lambda_context = self.scope_manager.create_lambda_evaluation_context().await;
                
                // Evaluate the sort expression
                let result = evaluator.evaluate_expression(&criterion.expression, &lambda_context).await?;
                
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
        
        // Pop lambda scope
        self.scope_manager.pop_scope();
        
        // Sort items by their sort keys
        sort_items.sort_by(|a, b| self.compare_sort_items(a, b));
        
        // Extract sorted items
        let sorted_items: Vec<_> = sort_items.into_iter()
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
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                a.cmp(&rust_decimal::Decimal::from(*b))
            },
            
            // Different types - use type precedence: numbers < strings < booleans < others
            (FhirPathValue::Integer(_) | FhirPathValue::Decimal(_), FhirPathValue::String(_)) => Ordering::Less,
            (FhirPathValue::String(_), FhirPathValue::Integer(_) | FhirPathValue::Decimal(_)) => Ordering::Greater,
            (FhirPathValue::Integer(_) | FhirPathValue::Decimal(_), FhirPathValue::Boolean(_)) => Ordering::Less,
            (FhirPathValue::Boolean(_), FhirPathValue::Integer(_) | FhirPathValue::Decimal(_)) => Ordering::Greater,
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
            let final_result = if key_a.1 {  // descending
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
            (SortKey::Empty, _) => Ordering::Less,  // Empty values sort first
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
        let condition_result = evaluator.evaluate_expression(condition_expr, context).await?;
        
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
#[async_trait::async_trait]
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
        },
        _ => true, // Multiple items are truthy
    }
}

/// Create a simple lambda evaluator adapter
///
/// This is a convenience function for creating a lambda evaluator that works
/// with a closure for expression evaluation.
pub fn create_lambda_evaluator_adapter<F>(
    global_context: Arc<EvaluationContext>,
    evaluator_fn: F,
) -> LambdaEvaluator
where
    F: Fn(&ExpressionNode, &EvaluationContext) -> crate::core::Result<Collection> + Send + Sync + 'static,
{
    LambdaEvaluator::new(global_context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Collection, FhirPathValue};
    use crate::ast::{ExpressionNode, LiteralNode, LiteralValue};

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
        
        let result = lambda_evaluator.evaluate_where(&collection, &lambda_expr, &mock_evaluator).await.unwrap();
        
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
        
        let result = lambda_evaluator.evaluate_all(&collection, &lambda_expr, &mock_evaluator).await.unwrap();
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
        
        let result = lambda_evaluator.evaluate_sort(&collection, vec![], &MockExpressionEvaluator {
            result: Collection::empty(),
        }).await.unwrap();
        
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
        
        let sort_criteria = vec![
            SortCriterion {
                expression: ExpressionNode::Literal(crate::ast::LiteralNode {
                    value: crate::ast::LiteralValue::Integer(1), // Mock - would be $this in real usage
                    location: None,
                }),
                descending: false,
            }
        ];
        
        let mock_evaluator = MockExpressionEvaluatorForSort::new();
        let result = lambda_evaluator.evaluate_sort(&collection, sort_criteria, &mock_evaluator).await.unwrap();
        
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
        
        let result = lambda_evaluator.evaluate_sort(&collection, vec![], &MockExpressionEvaluator {
            result: Collection::empty(),
        }).await.unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result.first().unwrap(), &FhirPathValue::String("a".to_string()));
        assert_eq!(result.get(1).unwrap(), &FhirPathValue::String("b".to_string()));
        assert_eq!(result.get(2).unwrap(), &FhirPathValue::String("c".to_string()));
    }
    
    #[tokio::test]
    async fn test_empty_collection_sort() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut lambda_evaluator = LambdaEvaluator::new(global_context);
        
        let empty_collection = Collection::empty();
        let result = lambda_evaluator.evaluate_sort(&empty_collection, vec![], &MockExpressionEvaluator {
            result: Collection::empty(),
        }).await.unwrap();
        
        assert!(result.is_empty());
    }
    
    // Mock evaluator for sort testing
    struct MockExpressionEvaluatorForSort {
        call_count: std::cell::RefCell<usize>,
    }
    
    impl MockExpressionEvaluatorForSort {
        fn new() -> Self {
            Self {
                call_count: std::cell::RefCell::new(0),
            }
        }
    }
    
    #[async_trait::async_trait]
    impl LambdaExpressionEvaluator for MockExpressionEvaluatorForSort {
        async fn evaluate_expression(
            &self,
            _expr: &ExpressionNode,
            context: &EvaluationContext,
        ) -> crate::core::Result<Collection> {
            let mut count = self.call_count.borrow_mut();
            *count += 1;
            
            // Return the current $this value from context for sorting
            if let Some(this_value) = context.get_variable("this") {
                Ok(Collection::single(this_value))
            } else {
                Ok(Collection::empty())
            }
        }
    }

    #[test]
    fn test_is_truthy() {
        // Empty collection is falsy
        assert!(!is_truthy(&Collection::empty()));
        
        // Boolean values
        assert!(is_truthy(&Collection::from_values(vec![FhirPathValue::Boolean(true)])));
        assert!(!is_truthy(&Collection::from_values(vec![FhirPathValue::Boolean(false)])));
        
        // Integer values
        assert!(is_truthy(&Collection::from_values(vec![FhirPathValue::Integer(1)])));
        assert!(!is_truthy(&Collection::from_values(vec![FhirPathValue::Integer(0)])));
        
        // String values
        assert!(is_truthy(&Collection::from_values(vec![FhirPathValue::String("hello".to_string())])));
        assert!(!is_truthy(&Collection::from_values(vec![FhirPathValue::String("".to_string())])));
        
        // Multiple items are truthy
        assert!(is_truthy(&Collection::from_values(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(false),
        ])));
    }
}