// Evaluation context for FHIRPath expressions

use crate::model::FhirPathValue;
use crate::registry::{FunctionRegistry, OperatorRegistry};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Variable scope for defineVariable isolation with Copy-on-Write semantics
#[derive(Clone, Debug)]
pub struct VariableScope {
    /// Variables defined in this scope (Copy-on-Write for efficient inheritance)
    pub variables: Cow<'static, FxHashMap<String, FhirPathValue>>,
    /// Parent scope (for nested scoping)
    pub parent: Option<Arc<VariableScope>>,
    /// Whether this scope owns its variables (true if variables were modified)
    owned: bool,
}

impl Default for VariableScope {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableScope {
    /// Create a new root scope
    pub fn new() -> Self {
        Self {
            variables: Cow::Owned(FxHashMap::default()),
            parent: None,
            owned: true,
        }
    }

    /// Create a child scope inheriting from parent (zero-copy initially)
    pub fn child(parent: VariableScope) -> Self {
        Self {
            variables: Cow::Borrowed(match &parent.variables {
                Cow::Borrowed(map) => map,
                Cow::Owned(_map) => {
                    // If parent owns its variables, we need to create a static reference
                    // This is a limitation - we'll clone for now but optimize common cases
                    return Self {
                        variables: Cow::Owned(FxHashMap::default()),
                        parent: Some(Arc::new(parent)),
                        owned: false,
                    };
                }
            }),
            parent: Some(Arc::new(parent)),
            owned: false,
        }
    }

    /// Create a child scope from a shared parent (more efficient)
    pub fn child_from_shared(parent: Arc<VariableScope>) -> Self {
        Self {
            variables: Cow::Owned(FxHashMap::default()),
            parent: Some(parent),
            owned: false,
        }
    }

    /// Set a variable in the current scope (triggers copy-on-write)
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        // Trigger copy-on-write if we're borrowing
        if !self.owned {
            let mut new_vars = FxHashMap::default();
            // Copy existing variables if any
            for (k, v) in self.variables.iter() {
                new_vars.insert(k.clone(), v.clone());
            }
            self.variables = Cow::Owned(new_vars);
            self.owned = true;
        }

        // Now we can safely insert into owned variables
        if let Cow::Owned(ref mut vars) = self.variables {
            vars.insert(name, value);
        }
    }

    /// Get a variable from this scope or parent scopes
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        // First check local variables
        if let Some(value) = self.variables.get(name) {
            return Some(value);
        }

        // Then check parent scopes
        self.parent
            .as_ref()
            .and_then(|parent| parent.get_variable(name))
    }

    /// Check if this scope contains a variable locally (not in parent)
    pub fn contains_local(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Get the number of local variables in this scope
    pub fn local_count(&self) -> usize {
        self.variables.len()
    }

    /// Create an optimized scope for simple expressions (pre-allocated capacity)
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            variables: Cow::Owned(FxHashMap::with_capacity_and_hasher(
                capacity,
                Default::default(),
            )),
            parent: None,
            owned: true,
        }
    }
}

/// Context for evaluating FHIRPath expressions
#[derive(Clone)]
pub struct EvaluationContext {
    /// Current input value being evaluated
    pub input: FhirPathValue,

    /// Root input value (for %context and $resource variables)
    pub root: FhirPathValue,

    /// Variable scope stack for proper scoping
    pub variable_scope: VariableScope,

    /// Function registry for evaluating function calls
    pub functions: Arc<FunctionRegistry>,

    /// Operator registry for evaluating operations
    pub operators: Arc<OperatorRegistry>,
}

impl EvaluationContext {
    /// Create a new evaluation context
    pub fn new(
        input: FhirPathValue,
        functions: Arc<FunctionRegistry>,
        operators: Arc<OperatorRegistry>,
    ) -> Self {
        Self {
            root: input.clone(),
            input,
            variable_scope: VariableScope::new(),
            functions,
            operators,
        }
    }

    /// Create a child context with new input value
    pub fn with_input(&self, input: FhirPathValue) -> Self {
        Self {
            input,
            root: self.root.clone(),
            variable_scope: self.variable_scope.clone(),
            functions: self.functions.clone(),
            operators: self.operators.clone(),
        }
    }

    /// Create a child context with fresh variable scope (for union isolation)
    pub fn with_fresh_variable_scope(&self) -> Self {
        Self {
            input: self.input.clone(),
            root: self.root.clone(),
            variable_scope: VariableScope::new(),
            functions: self.functions.clone(),
            operators: self.operators.clone(),
        }
    }

    /// Create a child context with inherited variable scope (Copy-on-Write)
    pub fn with_inherited_scope(&self, input: FhirPathValue) -> Self {
        Self {
            input,
            root: self.root.clone(),
            variable_scope: VariableScope::child_from_shared(Arc::new(self.variable_scope.clone())),
            functions: self.functions.clone(),
            operators: self.operators.clone(),
        }
    }

    /// Set a variable in the context
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        self.variable_scope.set_variable(name, value);
    }

    /// Get a variable from the context
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        self.variable_scope.get_variable(name)
    }
}

impl VariableScope {
    /// Collect all variables from this scope and parent scopes into a flat map
    pub fn collect_all_variables(&self) -> FxHashMap<String, FhirPathValue> {
        let mut all_variables = FxHashMap::default();

        // First collect from parent scopes (so child scope variables override parent)
        if let Some(parent) = &self.parent {
            all_variables.extend(parent.collect_all_variables());
        }

        // Then add variables from this scope (overriding any parent variables)
        // Use efficient cloning based on Cow state
        match &self.variables {
            Cow::Borrowed(vars) => {
                all_variables.extend(vars.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
            Cow::Owned(vars) => {
                all_variables.extend(vars.clone());
            }
        }

        all_variables
    }

    /// Create a flattened scope (useful for serialization or debugging)
    pub fn flatten(&self) -> Self {
        let all_vars = self.collect_all_variables();
        Self {
            variables: Cow::Owned(all_vars),
            parent: None,
            owned: true,
        }
    }

    /// Check if this scope is efficiently borrowing from parent
    pub fn is_efficient(&self) -> bool {
        !self.owned && matches!(self.variables, Cow::Borrowed(_))
    }

    /// Get memory usage information for debugging
    pub fn memory_info(&self) -> VariableScopeMemoryInfo {
        let local_vars = self.variables.len();
        let mut total_vars = local_vars;
        let mut depth = 1;
        let mut efficient_scopes = if self.is_efficient() { 1 } else { 0 };

        // Count parent scope info
        let mut current_parent = &self.parent;
        while let Some(parent) = current_parent {
            total_vars += parent.variables.len();
            depth += 1;
            if parent.is_efficient() {
                efficient_scopes += 1;
            }
            current_parent = &parent.parent;
        }

        VariableScopeMemoryInfo {
            local_variables: local_vars,
            total_variables: total_vars,
            scope_depth: depth,
            efficient_scopes,
            is_cow_optimized: self.is_efficient(),
        }
    }
}

/// Memory usage information for variable scopes
#[derive(Debug, Clone)]
pub struct VariableScopeMemoryInfo {
    /// Number of variables in this scope
    pub local_variables: usize,
    /// Total variables including all parent scopes
    pub total_variables: usize,
    /// Depth of scope nesting
    pub scope_depth: usize,
    /// Number of scopes using efficient CoW
    pub efficient_scopes: usize,
    /// Whether this scope is using Copy-on-Write optimization
    pub is_cow_optimized: bool,
}

/// A pool of EvaluationContext instances to reduce allocation overhead
///
/// This pool maintains a collection of pre-allocated contexts that can be reused
/// across multiple evaluations, significantly reducing memory allocation pressure
/// in high-throughput scenarios.
#[derive(Clone)]
#[allow(dead_code)]
pub struct ContextPool {
    /// The pool of available contexts
    pool: Arc<Mutex<VecDeque<EvaluationContext>>>,
    /// Maximum number of contexts to keep in the pool
    max_size: usize,
    /// Template context used for creating new contexts
    template: EvaluationContext,
}

#[allow(dead_code)]
impl ContextPool {
    /// Create a new context pool with the given maximum size
    pub fn new(
        max_size: usize,
        functions: Arc<FunctionRegistry>,
        operators: Arc<OperatorRegistry>,
    ) -> Self {
        let template = EvaluationContext::new(FhirPathValue::Empty, functions, operators);

        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
            template,
        }
    }

    /// Create a new context pool with default size (32 contexts)
    pub fn with_defaults(
        functions: Arc<FunctionRegistry>,
        operators: Arc<OperatorRegistry>,
    ) -> Self {
        Self::new(32, functions, operators)
    }

    /// Acquire a context from the pool or create a new one
    pub fn acquire(&self, input: FhirPathValue) -> PooledContext {
        let context = {
            let mut pool = self.pool.lock().unwrap();
            if let Some(mut context) = pool.pop_front() {
                // Reset the context for reuse
                context.input = input.clone();
                context.root = input;
                context.variable_scope = VariableScope::new();
                context
            } else {
                // Create new context if pool is empty
                EvaluationContext::new(
                    input,
                    self.template.functions.clone(),
                    self.template.operators.clone(),
                )
            }
        };

        PooledContext {
            context,
            pool: self.pool.clone(),
            max_size: self.max_size,
        }
    }

    /// Get the current number of contexts in the pool
    pub fn size(&self) -> usize {
        self.pool.lock().unwrap().len()
    }
}

/// A context that automatically returns to the pool when dropped
pub struct PooledContext {
    context: EvaluationContext,
    pool: Arc<Mutex<VecDeque<EvaluationContext>>>,
    max_size: usize,
}

#[allow(dead_code)]
impl PooledContext {
    /// Get a reference to the underlying context
    pub fn as_ref(&self) -> &EvaluationContext {
        &self.context
    }

    /// Get a mutable reference to the underlying context
    pub fn as_mut(&mut self) -> &mut EvaluationContext {
        &mut self.context
    }

    /// Create a child context with new input value
    pub fn with_input(&self, input: FhirPathValue) -> EvaluationContext {
        self.context.with_input(input)
    }

    /// Create a child context with fresh variable scope
    pub fn with_fresh_variable_scope(&self) -> EvaluationContext {
        self.context.with_fresh_variable_scope()
    }
}

impl std::ops::Deref for PooledContext {
    type Target = EvaluationContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl std::ops::DerefMut for PooledContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl Drop for PooledContext {
    fn drop(&mut self) {
        // Return the context to the pool if there's room
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            // Clear sensitive data before returning to pool
            self.context.variable_scope = VariableScope::new();
            self.context.input = FhirPathValue::Empty;
            self.context.root = FhirPathValue::Empty;

            // Clone the registries before the replace operation
            let functions = self.context.functions.clone();
            let operators = self.context.operators.clone();

            pool.push_back(std::mem::replace(
                &mut self.context,
                EvaluationContext::new(FhirPathValue::Empty, functions, operators),
            ));
        }
    }
}

/// A lightweight, stack-allocated context for simple expression evaluation
///
/// This struct avoids heap allocations for simple expressions that don't require
/// complex variable scoping or function registries. It provides a significant
/// performance improvement for basic property access and simple operations.
#[derive(Clone)]
#[allow(dead_code)]
pub struct StackContext<'a> {
    /// Current input value being evaluated
    pub input: &'a FhirPathValue,
    /// Root input value (for %context and $resource variables)
    pub root: &'a FhirPathValue,
    /// Simple variable storage for basic variables (limited capacity)
    pub variables: FxHashMap<&'static str, &'a FhirPathValue>,
    /// Function registry reference (shared)
    pub functions: &'a FunctionRegistry,
    /// Operator registry reference (shared)
    pub operators: &'a OperatorRegistry,
}

#[allow(dead_code)]
impl<'a> StackContext<'a> {
    /// Create a new stack-allocated context
    pub fn new(
        input: &'a FhirPathValue,
        functions: &'a FunctionRegistry,
        operators: &'a OperatorRegistry,
    ) -> Self {
        Self {
            root: input,
            input,
            variables: FxHashMap::default(),
            functions,
            operators,
        }
    }

    /// Create a child context with new input value
    pub fn with_input(&self, input: &'a FhirPathValue) -> Self {
        Self {
            input,
            root: self.root,
            variables: self.variables.clone(),
            functions: self.functions,
            operators: self.operators,
        }
    }

    /// Set a simple variable (limited to static string keys for performance)
    pub fn set_variable(&mut self, name: &'static str, value: &'a FhirPathValue) {
        self.variables.insert(name, value);
    }

    /// Get a variable from the context
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        self.variables.get(name).copied()
    }

    /// Convert to a heap-allocated EvaluationContext when needed
    pub fn to_heap_context(&self) -> EvaluationContext {
        let mut context = EvaluationContext::new(
            self.input.clone(),
            Arc::new(self.functions.clone()),
            Arc::new(self.operators.clone()),
        );
        context.root = self.root.clone();

        // Convert variables to owned form
        for (name, value) in &self.variables {
            context.set_variable(name.to_string(), (*value).clone());
        }

        context
    }
}

/// Enum to choose between stack and heap allocation for contexts
#[derive(Clone)]
#[allow(dead_code)]
pub enum ContextStorage<'a> {
    /// Stack-allocated context for simple expressions
    Stack(StackContext<'a>),
    /// Heap-allocated context for complex expressions
    Heap(EvaluationContext),
}

#[allow(dead_code)]
impl<'a> ContextStorage<'a> {
    /// Create a stack context if input is borrowable, otherwise heap
    pub fn new_optimal(
        input: &'a FhirPathValue,
        functions: &'a FunctionRegistry,
        operators: &'a OperatorRegistry,
        prefer_stack: bool,
    ) -> Self {
        if prefer_stack {
            Self::Stack(StackContext::new(input, functions, operators))
        } else {
            Self::Heap(EvaluationContext::new(
                input.clone(),
                Arc::new(functions.clone()),
                Arc::new(operators.clone()),
            ))
        }
    }

    /// Get the input value
    pub fn input(&self) -> &FhirPathValue {
        match self {
            Self::Stack(ctx) => ctx.input,
            Self::Heap(ctx) => &ctx.input,
        }
    }

    /// Get the root value
    pub fn root(&self) -> &FhirPathValue {
        match self {
            Self::Stack(ctx) => ctx.root,
            Self::Heap(ctx) => &ctx.root,
        }
    }

    /// Create a child context with new input
    pub fn with_input(&self, input: &'a FhirPathValue) -> Self
    where
        Self: 'a,
    {
        match self {
            Self::Stack(ctx) => Self::Stack(ctx.with_input(input)),
            Self::Heap(ctx) => Self::Heap(ctx.with_input(input.clone())),
        }
    }

    /// Convert to heap context if not already
    pub fn to_heap(&self) -> EvaluationContext {
        match self {
            Self::Stack(ctx) => ctx.to_heap_context(),
            Self::Heap(ctx) => ctx.clone(),
        }
    }

    /// Check if this is a stack context
    pub fn is_stack(&self) -> bool {
        matches!(self, Self::Stack(_))
    }

    /// Check if this is a heap context
    pub fn is_heap(&self) -> bool {
        matches!(self, Self::Heap(_))
    }
}

/// Context factory for choosing optimal allocation strategy
#[allow(dead_code)]
pub struct ContextFactory;

#[allow(dead_code)]
impl ContextFactory {
    /// Create a context using the optimal allocation strategy based on expression complexity
    pub fn create_for_expression<'a>(
        input: &'a FhirPathValue,
        functions: &'a FunctionRegistry,
        operators: &'a OperatorRegistry,
        is_simple: bool,
    ) -> ContextStorage<'a> {
        ContextStorage::new_optimal(input, functions, operators, is_simple)
    }

    /// Determine if an expression is simple enough for stack allocation
    pub fn is_simple_expression(expr_str: &str) -> bool {
        // Heuristics for determining if expression is simple:
        // - Short length
        // - No complex operations
        // - Basic property access patterns

        if expr_str.len() > 50 {
            return false;
        }

        // Check for complex patterns that require heap allocation
        let complex_patterns = [
            "where(",
            "select(",
            "all(",
            "any(",
            "aggregate(",
            "defineVariable(",
            "repeat(",
            "extension(",
        ];

        for pattern in &complex_patterns {
            if expr_str.contains(pattern) {
                return false;
            }
        }

        // Count parentheses depth - complex nesting suggests heap allocation
        let mut depth: i32 = 0;
        let mut max_depth: i32 = 0;
        for ch in expr_str.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                ')' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }

        max_depth <= 2 // Allow simple function calls but not deep nesting
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{FunctionRegistry, OperatorRegistry};

    #[test]
    fn test_context_pool_acquire_and_return() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let pool = ContextPool::new(2, functions, operators);

        // Pool should start empty
        assert_eq!(pool.size(), 0);

        // Acquire a context
        let input = FhirPathValue::Integer(42);
        {
            let mut ctx = pool.acquire(input.clone());
            assert_eq!(ctx.input, input);
            assert_eq!(pool.size(), 0); // Still empty while context is in use

            // Modify the context
            ctx.set_variable("test".to_string(), FhirPathValue::Boolean(true));
        } // Context should be returned to pool here

        // Pool should now have one context
        assert_eq!(pool.size(), 1);

        // Acquire again - should reuse the pooled context
        {
            let ctx = pool.acquire(FhirPathValue::String("hello".to_string()));
            // Variables should be cleared
            assert!(ctx.get_variable("test").is_none());
            assert_eq!(ctx.input, FhirPathValue::String("hello".to_string()));
        }

        // Pool should still have one context
        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_context_pool_max_size() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let pool = ContextPool::new(1, functions, operators); // Max size 1

        // Create multiple contexts
        let input = FhirPathValue::Integer(1);
        {
            let _ctx1 = pool.acquire(input.clone());
            let ctx2 = pool.acquire(input.clone());
            // Both contexts exist, pool is empty
            assert_eq!(pool.size(), 0);
            drop(ctx2);
        } // ctx1 and ctx2 drop here

        // Only one should be returned to pool due to max size
        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_pooled_context_deref() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let pool = ContextPool::with_defaults(functions, operators);

        let input = FhirPathValue::Integer(42);
        let mut ctx = pool.acquire(input.clone());

        // Test deref functionality
        assert_eq!(ctx.input, input);
        ctx.set_variable("test".to_string(), FhirPathValue::Boolean(true));
        assert_eq!(
            ctx.get_variable("test"),
            Some(&FhirPathValue::Boolean(true))
        );
    }

    #[test]
    fn test_context_pool_child_contexts() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let pool = ContextPool::with_defaults(functions, operators);

        let input = FhirPathValue::Integer(42);
        let ctx = pool.acquire(input.clone());

        // Create child contexts
        let child = ctx.with_input(FhirPathValue::String("child".to_string()));
        assert_eq!(child.input, FhirPathValue::String("child".to_string()));
        assert_eq!(child.root, input); // Root should be preserved

        let fresh = ctx.with_fresh_variable_scope();
        assert_eq!(fresh.input, input);
        assert_eq!(fresh.root, input);
    }

    #[test]
    fn test_stack_context() {
        let functions = FunctionRegistry::new();
        let operators = OperatorRegistry::new();

        let input = FhirPathValue::Integer(42);
        let mut stack_ctx = StackContext::new(&input, &functions, &operators);

        // Test basic functionality
        assert_eq!(stack_ctx.input, &input);
        assert_eq!(stack_ctx.root, &input);

        // Test variable operations
        let var_value = FhirPathValue::String("test".to_string());
        stack_ctx.set_variable("test_var", &var_value);
        assert_eq!(stack_ctx.get_variable("test_var"), Some(&var_value));
        assert_eq!(stack_ctx.get_variable("nonexistent"), None);

        // Test child context creation
        let new_input = FhirPathValue::Boolean(true);
        let child_ctx = stack_ctx.with_input(&new_input);
        assert_eq!(child_ctx.input, &new_input);
        assert_eq!(child_ctx.root, &input); // Root should be preserved
        assert_eq!(child_ctx.get_variable("test_var"), Some(&var_value)); // Variables inherited
    }

    #[test]
    fn test_stack_to_heap_conversion() {
        let functions = FunctionRegistry::new();
        let operators = OperatorRegistry::new();

        let input = FhirPathValue::Integer(42);
        let var_value = FhirPathValue::String("test".to_string());

        let mut stack_ctx = StackContext::new(&input, &functions, &operators);
        stack_ctx.set_variable("test_var", &var_value);

        // Convert to heap context
        let heap_ctx = stack_ctx.to_heap_context();
        assert_eq!(heap_ctx.input, input);
        assert_eq!(heap_ctx.root, input);
        assert_eq!(heap_ctx.get_variable("test_var"), Some(&var_value));
    }

    #[test]
    fn test_context_storage() {
        let functions = FunctionRegistry::new();
        let operators = OperatorRegistry::new();
        let input = FhirPathValue::Integer(42);

        // Test stack storage creation
        let stack_storage = ContextStorage::new_optimal(&input, &functions, &operators, true);
        assert!(stack_storage.is_stack());
        assert!(!stack_storage.is_heap());
        assert_eq!(stack_storage.input(), &input);

        // Test heap storage creation
        let heap_storage = ContextStorage::new_optimal(&input, &functions, &operators, false);
        assert!(heap_storage.is_heap());
        assert!(!heap_storage.is_stack());
        assert_eq!(heap_storage.input(), &input);

        // Test conversion to heap
        let heap_from_stack = stack_storage.to_heap();
        assert_eq!(heap_from_stack.input, input);
    }

    #[test]
    fn test_context_factory_expression_analysis() {
        // Simple expressions should use stack allocation
        assert!(ContextFactory::is_simple_expression("Patient.name"));
        assert!(ContextFactory::is_simple_expression("active"));
        assert!(ContextFactory::is_simple_expression("name.given.first()"));
        assert!(ContextFactory::is_simple_expression("value > 100"));

        // Complex expressions should use heap allocation
        assert!(!ContextFactory::is_simple_expression(
            "Patient.name.where(use = 'official')"
        ));
        assert!(!ContextFactory::is_simple_expression(
            "entry.select(resource.name)"
        ));
        assert!(!ContextFactory::is_simple_expression(
            "extension('http://example.com/url')"
        ));
        assert!(!ContextFactory::is_simple_expression(
            "defineVariable('x', 42)"
        ));

        // Very long expressions should use heap allocation
        let long_expr = "Patient.name.given.first().value.substring(0, 10).length()";
        assert!(!ContextFactory::is_simple_expression(long_expr));

        // Deeply nested expressions should use heap allocation
        assert!(!ContextFactory::is_simple_expression("a.b(c.d(e.f(g)))"));
    }

    #[test]
    fn test_context_factory_creation() {
        let functions = FunctionRegistry::new();
        let operators = OperatorRegistry::new();
        let input = FhirPathValue::Integer(42);

        // Simple expression should create stack context
        let simple_ctx = ContextFactory::create_for_expression(
            &input,
            &functions,
            &operators,
            ContextFactory::is_simple_expression("Patient.name"),
        );
        assert!(simple_ctx.is_stack());

        // Complex expression should create heap context
        let complex_ctx = ContextFactory::create_for_expression(
            &input,
            &functions,
            &operators,
            ContextFactory::is_simple_expression("Patient.name.where(use = 'official')"),
        );
        assert!(complex_ctx.is_heap());
    }

    #[test]
    fn test_variable_scope_cow_semantics() {
        // Create a parent scope with variables
        let mut parent_scope = VariableScope::new();
        parent_scope.set_variable("parent_var".to_string(), FhirPathValue::Integer(42));
        parent_scope.set_variable(
            "shared_var".to_string(),
            FhirPathValue::String("parent".to_string()),
        );

        // Create child scope - should not copy variables immediately
        let mut child_scope = VariableScope::child_from_shared(Arc::new(parent_scope.clone()));

        // Child should be able to read parent variables
        assert_eq!(
            child_scope.get_variable("parent_var"),
            Some(&FhirPathValue::Integer(42))
        );
        assert_eq!(
            child_scope.get_variable("shared_var"),
            Some(&FhirPathValue::String("parent".to_string()))
        );

        // Child starts with zero local variables
        assert_eq!(child_scope.local_count(), 0);

        // Setting a variable should trigger copy-on-write
        child_scope.set_variable("child_var".to_string(), FhirPathValue::Boolean(true));
        assert_eq!(child_scope.local_count(), 1);

        // Child should still see parent variables
        assert_eq!(
            child_scope.get_variable("parent_var"),
            Some(&FhirPathValue::Integer(42))
        );
        assert_eq!(
            child_scope.get_variable("child_var"),
            Some(&FhirPathValue::Boolean(true))
        );

        // Overriding a parent variable should work
        child_scope.set_variable(
            "shared_var".to_string(),
            FhirPathValue::String("child".to_string()),
        );
        assert_eq!(
            child_scope.get_variable("shared_var"),
            Some(&FhirPathValue::String("child".to_string()))
        );

        // Parent should still have original value
        assert_eq!(
            parent_scope.get_variable("shared_var"),
            Some(&FhirPathValue::String("parent".to_string()))
        );
    }

    #[test]
    fn test_variable_scope_memory_efficiency() {
        let mut parent = VariableScope::new();
        parent.set_variable("var1".to_string(), FhirPathValue::Integer(1));
        parent.set_variable("var2".to_string(), FhirPathValue::Integer(2));

        // Create child that doesn't modify variables
        let child = VariableScope::child_from_shared(Arc::new(parent));

        // Check memory info
        let child_info = child.memory_info();
        assert_eq!(child_info.local_variables, 0);
        assert_eq!(child_info.total_variables, 2);
        assert_eq!(child_info.scope_depth, 2);

        // Child that modifies variables
        let mut modifying_child = VariableScope::child_from_shared(Arc::new(
            child.parent.as_ref().unwrap().as_ref().clone(),
        ));
        modifying_child.set_variable("child_var".to_string(), FhirPathValue::Boolean(true));

        let modifying_info = modifying_child.memory_info();
        assert_eq!(modifying_info.local_variables, 1);
        assert_eq!(modifying_info.total_variables, 3);
    }

    #[test]
    fn test_variable_scope_collect_all_variables() {
        let mut parent = VariableScope::new();
        parent.set_variable("parent_var".to_string(), FhirPathValue::Integer(42));
        parent.set_variable(
            "shared_var".to_string(),
            FhirPathValue::String("parent".to_string()),
        );

        let mut child = VariableScope::child_from_shared(Arc::new(parent));
        child.set_variable("child_var".to_string(), FhirPathValue::Boolean(true));
        child.set_variable(
            "shared_var".to_string(),
            FhirPathValue::String("child".to_string()),
        );

        let all_vars = child.collect_all_variables();
        assert_eq!(all_vars.len(), 3);
        assert_eq!(
            all_vars.get("parent_var"),
            Some(&FhirPathValue::Integer(42))
        );
        assert_eq!(
            all_vars.get("child_var"),
            Some(&FhirPathValue::Boolean(true))
        );
        assert_eq!(
            all_vars.get("shared_var"),
            Some(&FhirPathValue::String("child".to_string()))
        ); // Child overrides parent
    }

    #[test]
    fn test_variable_scope_flatten() {
        let mut parent = VariableScope::new();
        parent.set_variable("parent_var".to_string(), FhirPathValue::Integer(42));

        let mut child = VariableScope::child_from_shared(Arc::new(parent));
        child.set_variable("child_var".to_string(), FhirPathValue::Boolean(true));

        let flattened = child.flatten();
        assert!(flattened.parent.is_none());
        assert_eq!(flattened.local_count(), 2);
        assert_eq!(
            flattened.get_variable("parent_var"),
            Some(&FhirPathValue::Integer(42))
        );
        assert_eq!(
            flattened.get_variable("child_var"),
            Some(&FhirPathValue::Boolean(true))
        );
    }

    #[test]
    fn test_evaluation_context_inherited_scope() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());

        let mut parent_ctx = EvaluationContext::new(
            FhirPathValue::Integer(42),
            functions.clone(),
            operators.clone(),
        );

        parent_ctx.set_variable(
            "parent_var".to_string(),
            FhirPathValue::String("parent".to_string()),
        );

        // Create child context with inherited scope
        let child_ctx = parent_ctx.with_inherited_scope(FhirPathValue::Boolean(true));

        // Child should see parent variables
        assert_eq!(
            child_ctx.get_variable("parent_var"),
            Some(&FhirPathValue::String("parent".to_string()))
        );
        assert_eq!(child_ctx.input, FhirPathValue::Boolean(true));

        // Child scope should initially be efficient (no local variables)
        assert_eq!(child_ctx.variable_scope.local_count(), 0);
    }

    #[test]
    fn test_variable_scope_capacity_optimization() {
        // Test pre-allocated capacity optimization
        let mut scope = VariableScope::with_capacity(10);

        // Add variables up to capacity
        for i in 0..5 {
            scope.set_variable(format!("var{}", i), FhirPathValue::Integer(i));
        }

        assert_eq!(scope.local_count(), 5);
        assert_eq!(scope.get_variable("var3"), Some(&FhirPathValue::Integer(3)));
    }
}
