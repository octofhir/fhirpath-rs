//! Variable Scoping and Lambda Support for FHIRPath
//!
//! This module implements comprehensive variable scoping and lambda expression support
//! for FHIRPath evaluation, including proper variable capture, scope management, and
//! integration with collection functions like `where()`, `select()`, and `all()`.
//!
//! # Key Features
//!
//! - **Proper Variable Scoping**: Nested scopes with proper variable resolution order
//! - **Lambda Expression Support**: Full support for lambda expressions in collection functions
//! - **Built-in Variables**: Support for all FHIRPath built-in variables ($this, $index, %resource, etc.)
//! - **Performance Optimized**: Minimal copying through reference-based design
//! - **Thread-Safe**: Full Send + Sync support for concurrent evaluation

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{Collection, FhirPathValue};
use crate::evaluator::context::EvaluationContext;

/// Variable scope manager for lambda expressions and nested contexts
///
/// The ScopeManager manages a stack of variable scopes, supporting proper variable
/// resolution with nested scopes. Each scope can contain user-defined variables,
/// built-in variables ($this, $index), and references to parent scopes.
///
/// # Thread Safety
///
/// ScopeManager is designed to be used in single-threaded evaluation contexts.
/// For multi-threaded use, create separate ScopeManager instances per thread.
///
/// # Examples
///
/// ```rust,no_run
/// use octofhir_fhirpath::evaluator::scoping::*;
/// use octofhir_fhirpath::evaluator::EvaluationContext;
/// use octofhir_fhirpath::{Collection, FhirPathValue};
/// use std::sync::Arc;
///
/// # async fn example() -> octofhir_fhirpath::Result<()> {
/// let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
/// let mut scope_manager = ScopeManager::new(global_context);
///
/// // Push lambda scope
/// let scope_id = scope_manager.push_scope(ScopeType::Lambda);
/// 
/// // Set current item ($this)
/// scope_manager.set_current_item(FhirPathValue::Integer(42));
/// 
/// // Get $this variable
/// let this_value = scope_manager.get_variable("this").await;
/// assert_eq!(this_value, Some(FhirPathValue::Integer(42)));
///
/// // Pop scope
/// scope_manager.pop_scope();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ScopeManager {
    /// Stack of variable scopes (innermost scope at top)
    scope_stack: Vec<VariableScope>,
    /// Global evaluation context
    global_context: Arc<EvaluationContext>,
}

/// Individual variable scope
///
/// Each scope contains variables defined at that scope level, along with
/// special variables like $this and $index for lambda evaluation.
#[derive(Debug, Clone)]
pub struct VariableScope {
    /// Variables defined at this scope level
    variables: HashMap<String, FhirPathValue>,
    /// Current context item ($this)
    current_item: Option<FhirPathValue>,
    /// Current index ($index) - optional feature
    current_index: Option<i64>,
    /// Parent scope reference for capturing
    parent_scope_id: Option<usize>,
    /// Scope type (lambda, function, global)
    scope_type: ScopeType,
}

/// Type of variable scope
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    /// Global scope with user variables
    Global,
    /// Lambda expression scope
    Lambda,
    /// Function call scope
    Function,
    /// Nested expression scope
    Nested,
}

/// Lambda expression definition
///
/// Represents a lambda expression that can be evaluated for each item
/// in a collection, with proper variable capture and scoping.
#[derive(Debug, Clone)]
pub struct LambdaExpression {
    /// Expression to evaluate for each item
    expression: Box<crate::ast::ExpressionNode>,
    /// Variables captured from outer scope
    captured_variables: HashMap<String, FhirPathValue>,
    /// Whether to capture $this from outer scope
    capture_this: bool,
}

/// Lambda evaluation context
///
/// Context used when evaluating lambda expressions, containing the current
/// item, index, captured variables, and parent evaluation context.
#[derive(Debug)]
pub struct LambdaContext {
    /// Current item being processed ($this)
    current_item: FhirPathValue,
    /// Current index in collection ($index)
    current_index: Option<i64>,
    /// Captured variables from outer scope
    captured_variables: HashMap<String, FhirPathValue>,
    /// Parent evaluation context
    parent_context: Arc<EvaluationContext>,
}

/// Scope identifier for tracking scope relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeId(pub usize);

/// Scope information for debugging and introspection
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub id: usize,
    pub scope_type: ScopeType,
    pub variable_count: usize,
    pub has_current_item: bool,
    pub has_current_index: bool,
}

impl ScopeManager {
    /// Create new scope manager with global context
    ///
    /// # Arguments
    /// * `global_context` - Global evaluation context containing user variables and built-ins
    pub fn new(global_context: Arc<EvaluationContext>) -> Self {
        Self {
            scope_stack: Vec::new(),
            global_context,
        }
    }

    /// Create a reusable base evaluation context for lambda execution.
    /// Inherits built-ins and server context and merges captured variables once.
    pub fn create_lambda_base_context(&self) -> EvaluationContext {
        let mut ctx = EvaluationContext::new(Collection::empty());
        // Inherit environment
        ctx.builtin_variables = self.global_context.builtin_variables.clone();
        ctx.server_context = self.global_context.server_context.clone();
        // Copy variables from global context (like $this set in parent contexts)
        for (name, value) in &self.global_context.variables {
            ctx.set_variable(name.clone(), value.clone());
        }
        // Merge variables from all scopes (outermost to innermost)
        for scope in &self.scope_stack {
            for (name, value) in &scope.variables {
                ctx.set_variable(name.clone(), value.clone());
            }
        }
        ctx
    }

    /// Update the reusable lambda context with the current item and index.
    /// Sets start_context, $this and $index.
    pub fn update_lambda_item(&self, ctx: &mut EvaluationContext, item: &FhirPathValue, index: usize) {
        // Replace start context with current item
        ctx.start_context = Collection::single(item.clone());
        // Update special variables
        ctx.set_variable("$this".to_string(), item.clone());
        ctx.set_variable("$index".to_string(), FhirPathValue::Integer(index as i64));
    }
    
    /// Push new variable scope
    ///
    /// Creates a new scope on top of the scope stack. The new scope can access
    /// variables from parent scopes but variables defined in this scope will
    /// shadow parent scope variables with the same name.
    ///
    /// # Arguments
    /// * `scope_type` - Type of scope to create
    ///
    /// # Returns
    /// Scope identifier for the new scope
    pub fn push_scope(&mut self, scope_type: ScopeType) -> ScopeId {
        let scope_id = self.scope_stack.len();
        let parent_scope_id = if scope_id > 0 { Some(scope_id - 1) } else { None };
        
        let scope = VariableScope {
            variables: HashMap::new(),
            current_item: None,
            current_index: None,
            parent_scope_id,
            scope_type,
        };
        
        self.scope_stack.push(scope);
        ScopeId(scope_id)
    }
    
    /// Pop variable scope
    ///
    /// Removes the top scope from the scope stack. Returns the popped scope
    /// for inspection or cleanup.
    ///
    /// # Returns
    /// The popped scope, or None if the stack was empty
    pub fn pop_scope(&mut self) -> Option<VariableScope> {
        self.scope_stack.pop()
    }
    
    /// Set current item in top scope ($this)
    ///
    /// Sets the $this variable for the current scope. This is used in lambda
    /// expressions where $this refers to the current item being processed.
    ///
    /// # Arguments
    /// * `item` - Current item value
    pub fn set_current_item(&mut self, item: FhirPathValue) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.current_item = Some(item);
        }
    }
    
    /// Set current index in top scope ($index)
    ///
    /// Sets the $index variable for the current scope. This is used in lambda
    /// expressions where $index refers to the current position in the collection.
    ///
    /// # Arguments
    /// * `index` - Current index (0-based)
    pub fn set_current_index(&mut self, index: i64) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.current_index = Some(index);
        }
    }
    
    /// Set variable in current scope
    ///
    /// Adds or updates a variable in the current (top) scope. This variable
    /// will shadow any variables with the same name in parent scopes.
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Variable value
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.variables.insert(name, value);
        }
    }
    
    /// Get variable value with proper scoping rules
    ///
    /// Resolves variables according to FHIRPath scoping rules:
    /// 1. Special variables ($this, $index) from innermost scope that has them
    /// 2. User-defined variables from scopes (innermost to outermost)
    /// 3. Built-in environment variables from global context
    /// 4. User variables from global context
    ///
    /// # Arguments
    /// * `name` - Variable name (with or without $ prefix)
    ///
    /// # Returns
    /// Variable value, or None if not found
    pub async fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        // Handle variable names with and without $ prefix
        let clean_name = if name.starts_with('$') {
            &name[1..]
        } else {
            name
        };
        
        // Special variables - search from innermost scope
        match clean_name {
            "this" => {
                // Find $this from innermost scope that has it
                for scope in self.scope_stack.iter().rev() {
                    if let Some(item) = &scope.current_item {
                        return Some(item.clone());
                    }
                }
                // Fall back to global context start context
                if !self.global_context.start_context.is_empty() {
                    return Some(FhirPathValue::Collection(self.global_context.start_context.clone().into_vec()));
                }
                return None;
            },
            "index" => {
                // Find $index from innermost scope that has it
                for scope in self.scope_stack.iter().rev() {
                    if let Some(index) = scope.current_index {
                        return Some(FhirPathValue::Integer(index));
                    }
                }
                return None;
            },
            _ => {}
        }
        
        // Search through scope stack (innermost to outermost)
        for scope in self.scope_stack.iter().rev() {
            if let Some(value) = scope.variables.get(clean_name) {
                return Some(value.clone());
            }
        }
        
        // Check global context variables (both user and built-in)
        self.global_context.get_variable(clean_name).cloned()
    }
    
    /// Create lambda context for collection iteration
    ///
    /// Creates a new lambda evaluation context for evaluating lambda expressions
    /// within collection functions. This context includes the current item,
    /// index, and captured variables from outer scopes.
    ///
    /// # Arguments
    /// * `current_item` - Current item being processed
    /// * `current_index` - Current index in collection (optional)
    /// * `captured_variables` - Variables captured from outer scope
    ///
    /// # Returns
    /// Lambda evaluation context
    pub fn create_lambda_context(
        &self,
        current_item: FhirPathValue,
        current_index: Option<i64>,
        captured_variables: HashMap<String, FhirPathValue>
    ) -> LambdaContext {
        LambdaContext {
            current_item,
            current_index,
            captured_variables,
            parent_context: self.global_context.clone(),
        }
    }
    
    /// Get current scope depth
    ///
    /// Returns the number of scopes currently on the stack.
    /// A depth of 0 means no local scopes (only global context).
    pub fn scope_depth(&self) -> usize {
        self.scope_stack.len()
    }
    
    /// Get scope information for debugging
    ///
    /// Returns information about all scopes on the stack for debugging
    /// and introspection purposes.
    pub fn get_scope_info(&self) -> Vec<ScopeInfo> {
        self.scope_stack.iter().enumerate().map(|(id, scope)| {
            ScopeInfo {
                id,
                scope_type: scope.scope_type.clone(),
                variable_count: scope.variables.len(),
                has_current_item: scope.current_item.is_some(),
                has_current_index: scope.current_index.is_some(),
            }
        }).collect()
    }
    
    /// Create child context for lambda evaluation
    ///
    /// Creates a new EvaluationContext with variables from all current scopes
    /// merged together. This is used for lambda expression evaluation.
    ///
    /// # Returns
    /// New evaluation context with merged variables
    pub async fn create_lambda_evaluation_context(&self) -> EvaluationContext {
        // Determine the current item from the innermost scope that has it
        let current_item_opt = self
            .scope_stack
            .iter()
            .rev()
            .find_map(|s| s.current_item.as_ref());

        // Build a fresh context using the current item as the start context
        let start_collection = if let Some(current_item) = current_item_opt {
            match current_item {
                FhirPathValue::Collection(items) => Collection::from_values(items.clone()),
                single => Collection::single(single.clone()),
            }
        } else {
            Collection::empty()
        };

        // Important: avoid cloning the entire global start_context (can be a large Bundle)
        // Instead, create a lightweight context and inherit only environment/built-ins.
        let mut lambda_context = EvaluationContext::new(start_collection);

        // Inherit built-in environment and server context from the global context
        lambda_context.builtin_variables = self.global_context.builtin_variables.clone();
        lambda_context.server_context = self.global_context.server_context.clone();

        // Merge variables from all scopes (outermost to innermost for proper shadowing)
        for scope in &self.scope_stack {
            for (name, value) in &scope.variables {
                lambda_context.set_variable(name.clone(), value.clone());
            }

            // Set $this and $index if available
            if let Some(current_item) = &scope.current_item {
                lambda_context.set_variable("this".to_string(), current_item.clone());
            }

            if let Some(current_index) = scope.current_index {
                lambda_context.set_variable("index".to_string(), FhirPathValue::Integer(current_index));
            }
        }

        lambda_context
    }
}

impl LambdaContext {
    /// Create evaluation context from lambda context
    ///
    /// Creates a new EvaluationContext suitable for evaluating lambda expressions.
    /// The context includes the current item as context, captured variables,
    /// and access to the parent evaluation context.
    pub fn to_evaluation_context(&self) -> EvaluationContext {
        // Create context with current item as start context
        let current_collection = match &self.current_item {
            FhirPathValue::Collection(items) => Collection::from_values(items.clone()),
            single_item => Collection::single(single_item.clone()),
        };
        
        let mut context = EvaluationContext::new(current_collection);
        
        // Add captured variables
        for (name, value) in &self.captured_variables {
            context.set_variable(name.clone(), value.clone());
        }
        
        // Add current item as $this
        context.set_variable("this".to_string(), self.current_item.clone());
        
        // Add current index as $index if available
        if let Some(index) = self.current_index {
            context.set_variable("index".to_string(), FhirPathValue::Integer(index));
        }
        
        // Inherit built-in variables from parent
        context.builtin_variables = self.parent_context.builtin_variables.clone();
        context.server_context = self.parent_context.server_context.clone();
        
        context
    }
}

impl Default for ScopeType {
    fn default() -> Self {
        ScopeType::Global
    }
}

impl std::fmt::Display for ScopeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeType::Global => write!(f, "global"),
            ScopeType::Lambda => write!(f, "lambda"),
            ScopeType::Function => write!(f, "function"),
            ScopeType::Nested => write!(f, "nested"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_scope_management() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut scope_manager = ScopeManager::new(global_context);
        
        // Test scope creation and destruction
        assert_eq!(scope_manager.scope_depth(), 0);
        
        let scope_id = scope_manager.push_scope(ScopeType::Lambda);
        assert_eq!(scope_manager.scope_depth(), 1);
        
        scope_manager.set_variable("test".to_string(), FhirPathValue::Integer(42));
        
        let value = scope_manager.get_variable("test").await;
        assert_eq!(value, Some(FhirPathValue::Integer(42)));
        
        scope_manager.pop_scope();
        assert_eq!(scope_manager.scope_depth(), 0);
        
        // Variable should no longer be accessible
        let value = scope_manager.get_variable("test").await;
        assert_eq!(value, None);
    }
    
    #[tokio::test]
    async fn test_this_variable_scoping() {
        let global_context = Arc::new(EvaluationContext::new(
            Collection::single(FhirPathValue::String("global".to_string()))
        ));
        let mut scope_manager = ScopeManager::new(global_context);
        
        // $this should resolve to global context initially
        let this_value = scope_manager.get_variable("this").await;
        assert!(this_value.is_some());
        
        // Push lambda scope with different $this
        scope_manager.push_scope(ScopeType::Lambda);
        scope_manager.set_current_item(FhirPathValue::String("local".to_string()));
        
        let this_value = scope_manager.get_variable("this").await;
        assert_eq!(this_value, Some(FhirPathValue::String("local".to_string())));
        
        // Pop scope - should revert to global
        scope_manager.pop_scope();
        let this_value = scope_manager.get_variable("this").await;
        assert!(this_value.is_some()); // Should be global context
    }
    
    #[tokio::test]
    async fn test_nested_scope_variable_resolution() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut scope_manager = ScopeManager::new(global_context.clone());
        
        // Set global variable
        {
            let mut context = Arc::try_unwrap(global_context.clone()).unwrap_or_else(|arc| (*arc).clone());
            context.set_variable("global_var".to_string(), FhirPathValue::String("global".to_string()));
        }
        
        // Push outer lambda scope
        scope_manager.push_scope(ScopeType::Lambda);
        scope_manager.set_variable("outer_var".to_string(), FhirPathValue::String("outer".to_string()));
        
        // Push inner lambda scope
        scope_manager.push_scope(ScopeType::Lambda);
        scope_manager.set_variable("inner_var".to_string(), FhirPathValue::String("inner".to_string()));
        
        // Test variable resolution from inner scope
        assert_eq!(scope_manager.get_variable("inner_var").await, Some(FhirPathValue::String("inner".to_string())));
        assert_eq!(scope_manager.get_variable("outer_var").await, Some(FhirPathValue::String("outer".to_string())));
        
        // Pop inner scope
        scope_manager.pop_scope();
        
        // inner_var should no longer be accessible
        assert_eq!(scope_manager.get_variable("inner_var").await, None);
        assert_eq!(scope_manager.get_variable("outer_var").await, Some(FhirPathValue::String("outer".to_string())));
        
        // Pop outer scope
        scope_manager.pop_scope();
        
        // Only global should be accessible
        assert_eq!(scope_manager.get_variable("inner_var").await, None);
        assert_eq!(scope_manager.get_variable("outer_var").await, None);
    }
    
    #[tokio::test]
    async fn test_variable_shadowing() {
        // Create context with a variable already set
        let mut global_context = EvaluationContext::new(Collection::empty());
        global_context.set_variable("var".to_string(), FhirPathValue::String("global".to_string()));
        let global_context = Arc::new(global_context);
        let mut scope_manager = ScopeManager::new(global_context);
        
        // Push scope and shadow the variable
        scope_manager.push_scope(ScopeType::Lambda);
        scope_manager.set_variable("var".to_string(), FhirPathValue::String("local".to_string()));
        
        // Should get the local (shadowed) value
        assert_eq!(scope_manager.get_variable("var").await, Some(FhirPathValue::String("local".to_string())));
        
        // Pop scope - should revert to global value
        scope_manager.pop_scope();
        assert_eq!(scope_manager.get_variable("var").await, Some(FhirPathValue::String("global".to_string())));
    }
    
    #[tokio::test]
    async fn test_index_variable() {
        let global_context = Arc::new(EvaluationContext::new(Collection::empty()));
        let mut scope_manager = ScopeManager::new(global_context);
        
        // $index should be None initially
        assert_eq!(scope_manager.get_variable("index").await, None);
        
        // Push scope and set index
        scope_manager.push_scope(ScopeType::Lambda);
        scope_manager.set_current_index(5);
        
        // Should get the index value
        assert_eq!(scope_manager.get_variable("index").await, Some(FhirPathValue::Integer(5)));
        assert_eq!(scope_manager.get_variable("$index").await, Some(FhirPathValue::Integer(5)));
        
        // Pop scope - should revert to None
        scope_manager.pop_scope();
        assert_eq!(scope_manager.get_variable("index").await, None);
    }
}
