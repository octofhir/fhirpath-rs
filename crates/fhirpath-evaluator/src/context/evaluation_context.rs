// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Core evaluation context for FHIRPath expressions
//!
//! This module provides the main `EvaluationContext` structure that holds all
//! the state needed for FHIRPath expression evaluation, including the current
//! input value, variable scopes, registry, and model provider.
//!
//! # Architecture
//!
//! The evaluation context uses Arc-based resource sharing for memory efficiency:
//! - **Root resource**: Shared via Arc to prevent cloning large FHIR resources
//! - **Registry**: Shared function registry for all operations
//! - **Model provider**: Shared type information provider
//! - **Type annotations**: Shared cache for async type resolution results
//!
//! # Context Hierarchies
//!
//! Contexts can form hierarchies through variable scope inheritance:
//! - **Root context**: Initial context with base variables
//! - **Child contexts**: Inherit variables with COW semantics
//! - **Lambda contexts**: Special contexts for lambda expressions with implicit variables

use octofhir_fhirpath_model::{
    FhirPathValue, 
    provider::ModelProvider, 
    provider::TypeReflectionInfo
};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

use super::{VariableScope, LambdaMetadata};

/// Context for evaluating FHIRPath expressions
///
/// This is the main context structure that holds all state needed for FHIRPath
/// expression evaluation. It provides the current input value, variable scoping,
/// function registry, and type information provider.
///
/// # Memory Efficiency
///
/// The context is designed for efficient cloning and resource sharing:
/// - Arc-based sharing for large resources (root, registry, model provider)
/// - COW semantics for variable scopes to minimize copying
/// - Shared type annotation cache across context hierarchies
///
/// # Thread Safety
///
/// The context can be safely shared across threads:
/// - All Arc-shared resources are thread-safe
/// - Type annotation cache uses Mutex for thread-safe access
/// - Variable scopes are immutable after creation (COW handles modifications)
#[derive(Clone)]
pub struct EvaluationContext {
    /// Current input value being evaluated
    /// 
    /// This is the value that expressions like `Patient.name` operate on.
    /// It changes as navigation proceeds through the FHIR resource structure.
    pub input: FhirPathValue,

    /// Root input value (for %context and $resource variables) - shared for memory efficiency
    /// 
    /// This is the original input value that started the evaluation, wrapped in Arc
    /// for efficient sharing across context hierarchies. Used for environment variables
    /// like `%context` and `%resource`.
    pub root: Arc<FhirPathValue>,

    /// Variable scope stack for proper scoping
    /// 
    /// Provides access to variables defined by `defineVariable()` expressions and
    /// implicit lambda variables like `$this`, `$index`, `$total`. Uses COW semantics
    /// for efficient inheritance in context hierarchies.
    pub variable_scope: VariableScope,

    /// Unified registry for evaluating all operations (functions and operators)
    /// 
    /// Contains implementations for all FHIRPath functions and operators.
    /// Shared via Arc across all contexts for memory efficiency and consistency.
    pub registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,

    /// Async ModelProvider for type checking and validation (required)
    /// 
    /// Provides FHIR type information needed for operations like `is`, `as`, and `ofType`.
    /// Required for full FHIRPath compliance and shared via Arc for efficiency.
    pub model_provider: Arc<dyn ModelProvider>,

    /// Cached type annotations from previous async operations
    /// 
    /// Cache for expensive type resolution operations to avoid repeated async calls.
    /// Shared across context hierarchies and protected by Mutex for thread safety.
    pub type_annotations: Arc<Mutex<FxHashMap<String, TypeReflectionInfo>>>,
}

impl EvaluationContext {
    /// Create a new evaluation context (ModelProvider required)
    /// 
    /// Creates a root evaluation context with the provided input value, registry,
    /// and model provider. This is the typical entry point for FHIRPath evaluation.
    /// 
    /// # Arguments
    /// * `input` - Initial input value for evaluation
    /// * `registry` - Function registry for operations
    /// * `model_provider` - Type information provider
    /// 
    /// # Returns
    /// New EvaluationContext ready for expression evaluation
    pub fn new(
        input: FhirPathValue,
        registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            root: Arc::new(input.clone()),
            input,
            variable_scope: VariableScope::new(),
            registry,
            model_provider,
            type_annotations: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    /// Create a new evaluation context with initial variables
    /// 
    /// Creates a root context and immediately sets the provided variables in its scope.
    /// Useful when you need to start evaluation with predefined variables.
    /// 
    /// # Arguments
    /// * `input` - Initial input value for evaluation
    /// * `registry` - Function registry for operations
    /// * `model_provider` - Type information provider
    /// * `initial_variables` - Variables to set in the root scope
    /// 
    /// # Returns
    /// New EvaluationContext with initial variables configured
    pub fn with_variables(
        input: FhirPathValue,
        registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        initial_variables: FxHashMap<String, FhirPathValue>,
    ) -> Self {
        let mut variable_scope = VariableScope::new();

        // Set initial variables in the scope
        for (name, value) in initial_variables {
            variable_scope.set_variable(name, value);
        }

        Self {
            root: Arc::new(input.clone()),
            input,
            variable_scope,
            registry,
            model_provider,
            type_annotations: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    /// Create a child context with new input value
    /// 
    /// Creates a new context with the same shared resources but different input.
    /// The variable scope is cloned (potentially triggering COW), making this
    /// suitable for navigation operations that need to preserve variables.
    /// 
    /// # Use Cases
    /// - Navigation operations (`.name`, `.telecom[0]`, etc.)
    /// - Function calls that need to preserve current variable scope
    /// - Sub-expression evaluation within the same variable context
    /// 
    /// # Arguments
    /// * `input` - New input value for the child context
    /// 
    /// # Returns
    /// Child context with new input and preserved variable scope
    /// 
    /// # Performance
    /// Arc-shared resources (root, registry, model provider, type annotations)
    /// are shared efficiently. Variable scope may trigger COW if modified.
    pub fn with_input(&self, input: FhirPathValue) -> Self {
        Self {
            input,
            root: self.root.clone(), // Arc::clone is cheap
            variable_scope: self.variable_scope.clone(),
            registry: self.registry.clone(),
            model_provider: self.model_provider.clone(),
            type_annotations: self.type_annotations.clone(),
        }
    }

    /// Create a child context with fresh variable scope (for union isolation)
    /// 
    /// Creates a child context with a completely new variable scope, isolating
    /// it from the parent's variables. This is useful for operations that need
    /// variable isolation, such as union expressions.
    /// 
    /// # Use Cases
    /// - Union expressions where branches should not share variables
    /// - Isolated sub-expression evaluation
    /// - Operations that need clean variable state
    /// 
    /// # Returns
    /// Child context with fresh variable scope and same input
    pub fn with_fresh_variable_scope(&self) -> Self {
        Self {
            input: self.input.clone(),
            root: self.root.clone(), // Arc::clone is cheap
            variable_scope: VariableScope::new(),
            registry: self.registry.clone(),
            model_provider: self.model_provider.clone(),
            type_annotations: self.type_annotations.clone(),
        }
    }

    /// Create a child context with inherited variable scope (Copy-on-Write)
    /// 
    /// Creates a child context that efficiently inherits variables from the parent
    /// using COW semantics. Variables are shared until modification, at which point
    /// they are copied. This is the most efficient way to create child contexts.
    /// 
    /// # COW Benefits
    /// - Zero-copy inheritance until first variable modification
    /// - Memory efficient for deep context hierarchies
    /// - Performance optimized for read-heavy variable usage
    /// 
    /// # Use Cases
    /// - Nested expression evaluation
    /// - Function calls that may define new variables
    /// - Context creation for sub-expressions
    /// 
    /// # Arguments
    /// * `input` - New input value for the child context
    /// 
    /// # Returns
    /// Child context with COW-inherited variable scope
    pub fn with_inherited_scope(&self, input: FhirPathValue) -> Self {
        Self {
            input,
            root: self.root.clone(), // Arc::clone is cheap
            variable_scope: VariableScope::child_from_shared(Arc::new(self.variable_scope.clone())),
            registry: self.registry.clone(),
            model_provider: self.model_provider.clone(),
            type_annotations: self.type_annotations.clone(),
        }
    }

    /// Create a lambda context with implicit variables ($this, $index, $total)
    /// 
    /// Creates a specialized context for lambda expression evaluation. Lambda contexts
    /// provide the implicit variables that are available within lambda expressions
    /// like `where`, `select`, `all`, `any`, etc.
    /// 
    /// # Implicit Variables
    /// - `$this` - Current item being processed in the lambda
    /// - `$index` - Zero-based index of current item in iteration
    /// - `$total` - Total count or accumulator value (context-dependent)
    /// 
    /// # Environment Variable Inheritance
    /// Lambda contexts automatically inherit critical environment variables
    /// from the parent scope to maintain proper evaluation context.
    /// 
    /// # Arguments
    /// * `current_item` - Current item being processed (becomes `$this`)
    /// * `index` - Zero-based iteration index (becomes `$index`)
    /// * `total` - Total count or accumulator (becomes `$total`)
    /// 
    /// # Returns
    /// Lambda context with implicit variables configured
    pub fn with_lambda_context(
        &self,
        current_item: FhirPathValue,
        index: usize,
        total: FhirPathValue,
    ) -> Self {
        let lambda_scope = VariableScope::lambda_scope(
            Some(Arc::new(self.variable_scope.clone())),
            current_item.clone(),
            index,
            total,
        );

        Self {
            input: current_item,
            root: self.root.clone(), // Arc::clone is cheap
            variable_scope: lambda_scope,
            registry: self.registry.clone(),
            model_provider: self.model_provider.clone(),
            type_annotations: self.type_annotations.clone(),
        }
    }

    /// Create lambda context preserving existing lambda variables (especially $index)
    /// 
    /// This method creates a lambda context that preserves existing lambda metadata
    /// from the current context, particularly the `$index` variable. This is useful
    /// for functions like `iif()` that need to maintain lambda context when used
    /// inside other lambda expressions like `select()`.
    /// 
    /// # Lambda Context Preservation
    /// - If current context has lambda metadata, preserves `$index` and `$total`
    /// - Updates `$this` to the new current item
    /// - If no lambda context exists, creates a simple lambda context
    /// 
    /// # Use Cases
    /// - `iif()` function within `select()` expressions
    /// - Nested lambda expressions that need to preserve outer context
    /// - Functions that operate within lambda contexts but need their own `$this`
    /// 
    /// # Arguments
    /// * `current_item` - New current item (becomes `$this`)
    /// 
    /// # Returns
    /// Lambda context preserving existing lambda variables where possible
    pub fn with_lambda_context_preserving_index(&self, current_item: FhirPathValue) -> Self {
        // Get current lambda metadata if it exists
        if let Some(current_lambda) = &self.variable_scope.lambda_metadata {
            // Preserve existing lambda metadata but update $this to current_item
            let lambda_scope = VariableScope::lambda_scope(
                Some(Arc::new(self.variable_scope.clone())),
                current_item.clone(),
                // Preserve existing index by extracting from current_index
                match &current_lambda.current_index {
                    FhirPathValue::Integer(idx) => *idx as usize,
                    _ => 0,
                },
                current_lambda.total_value.clone(),
            );

            Self {
                input: current_item,
                root: self.root.clone(),
                variable_scope: lambda_scope,
                registry: self.registry.clone(),
                model_provider: self.model_provider.clone(),
                type_annotations: self.type_annotations.clone(),
            }
        } else {
            // No existing lambda context, create a simple one with index 0
            self.with_lambda_context(current_item, 0, FhirPathValue::Empty)
        }
    }

    /// Set a variable in the context
    /// 
    /// Sets a user-defined variable in the current variable scope. System variables
    /// (those starting with % or $ that are managed by the runtime) cannot be
    /// overridden and attempts to set them are silently ignored per FHIRPath spec.
    /// 
    /// # Variable Protection
    /// - Environment variables (`%context`, `%resource`, etc.) - Protected
    /// - Lambda variables (`$this`, `$index`, `$total`) - Protected  
    /// - Value set variables (`%vs-*`) - Protected
    /// - User variables - Allowed
    /// 
    /// # COW Behavior
    /// Setting a variable may trigger COW if the variable scope is currently
    /// sharing data with a parent scope.
    /// 
    /// # Arguments
    /// * `name` - Variable name (without % or $ prefixes)
    /// * `value` - Variable value
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        self.variable_scope.set_variable(name, value);
    }

    /// Set a system variable during context initialization
    /// 
    /// This method bypasses the system variable protection to allow the runtime
    /// to establish the initial variable environment. It should only be used
    /// during context initialization.
    /// 
    /// # Safety
    /// This method should only be used by the FHIRPath runtime for setting up
    /// environment variables. Inappropriate use could allow user code to override
    /// system variables, breaking the evaluation environment.
    /// 
    /// # Arguments
    /// * `name` - System variable name
    /// * `value` - System variable value
    pub(crate) fn set_system_variable(&mut self, name: String, value: FhirPathValue) {
        self.variable_scope.set_system_variable(name, value);
    }

    /// Get a variable from the context
    /// 
    /// Resolves a variable using the complete variable resolution chain:
    /// 1. Local variables in current scope
    /// 2. Lambda metadata (implicit variables)
    /// 3. Parent scopes (recursive search)
    /// 4. Environment variables
    /// 
    /// # Arguments
    /// * `name` - Variable name to resolve
    /// 
    /// # Returns
    /// * `Some(&FhirPathValue)` - If variable found in resolution chain
    /// * `None` - If variable not found
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        self.variable_scope.get_variable(name)
    }

    /// Set a type annotation in the cache
    /// 
    /// Caches type information from expensive async type resolution operations
    /// to avoid repeated ModelProvider calls for the same types.
    /// 
    /// # Thread Safety
    /// Uses Mutex to ensure thread-safe access to the shared type cache.
    /// If the mutex is poisoned, the operation is silently ignored.
    /// 
    /// # Arguments
    /// * `key` - Cache key (typically type name or expression signature)
    /// * `type_info` - Type information to cache
    pub fn set_type_annotation(&self, key: String, type_info: TypeReflectionInfo) {
        if let Ok(mut annotations) = self.type_annotations.lock() {
            annotations.insert(key, type_info);
        }
    }

    /// Get a type annotation from the cache
    /// 
    /// Retrieves cached type information to avoid expensive async type resolution.
    /// 
    /// # Thread Safety
    /// Uses Mutex to ensure thread-safe access to the shared type cache.
    /// If the mutex is poisoned, returns None.
    /// 
    /// # Arguments
    /// * `key` - Cache key to lookup
    /// 
    /// # Returns
    /// * `Some(TypeReflectionInfo)` - If type information is cached
    /// * `None` - If not cached or mutex is poisoned
    pub fn get_type_annotation(&self, key: &str) -> Option<TypeReflectionInfo> {
        if let Ok(annotations) = self.type_annotations.lock() {
            annotations.get(key).cloned()
        } else {
            None
        }
    }

    /// Get the ModelProvider (always available now)
    /// 
    /// Returns a reference to the Arc-wrapped model provider. The model provider
    /// is always available in modern contexts and provides type information
    /// necessary for full FHIRPath compliance.
    /// 
    /// # Returns
    /// Reference to the Arc-wrapped ModelProvider
    pub fn model_provider(&self) -> &Arc<dyn ModelProvider> {
        &self.model_provider
    }

    /// Clear type annotation cache
    /// 
    /// Clears all cached type information. Useful for testing or when type
    /// information becomes stale.
    /// 
    /// # Thread Safety
    /// Uses Mutex to ensure thread-safe cache clearing. If the mutex is
    /// poisoned, the operation is silently ignored.
    pub fn clear_type_annotations(&self) {
        if let Ok(mut annotations) = self.type_annotations.lock() {
            annotations.clear();
        }
    }

    /// Get debug information about the current context state
    /// 
    /// Returns a string containing detailed information about the context's
    /// current state, including input value, variable scope information,
    /// and memory usage characteristics.
    /// 
    /// # Returns
    /// Multi-line debug string describing context state
    pub fn debug_info(&self) -> String {
        let mut info = String::new();
        
        info.push_str(&format!("EvaluationContext Debug Info:\n"));
        info.push_str(&format!("  Input: {:?}\n", self.input));
        info.push_str(&format!("  Root resource type: {}\n", 
            match &*self.root {
                FhirPathValue::Resource(_) => "Resource",
                FhirPathValue::String(_) => "String", 
                FhirPathValue::Integer(_) => "Integer",
                FhirPathValue::Boolean(_) => "Boolean",
                FhirPathValue::Collection(_) => "Collection",
                _ => "Other",
            }
        ));
        
        // Variable scope information
        info.push_str("  Variable Scope:\n");
        let memory_info = self.variable_scope.memory_info();
        info.push_str(&format!("    {}\n", memory_info.summary()));
        
        // Lambda context information
        if let Some(ref lambda_meta) = self.variable_scope.lambda_metadata {
            info.push_str("  Lambda Context:\n");
            info.push_str(&format!("    $index: {}\n", lambda_meta.current_index_as_i64()));
            info.push_str(&format!("    $this: {:?}\n", lambda_meta.current_item()));
        } else {
            info.push_str("  No lambda context\n");
        }
        
        // Type annotation cache info
        if let Ok(annotations) = self.type_annotations.lock() {
            info.push_str(&format!("  Type annotations cached: {}\n", annotations.len()));
        } else {
            info.push_str("  Type annotation cache: unavailable (poisoned mutex)\n");
        }
        
        info
    }

    /// Create a debug representation of the variable resolution chain
    /// 
    /// Delegates to the variable scope's debug chain functionality to show
    /// the complete variable resolution hierarchy.
    /// 
    /// # Returns
    /// Multi-line debug string showing variable resolution chain
    pub fn debug_variable_chain(&self) -> String {
        self.variable_scope.debug_variable_chain()
    }

    /// Check if this context is in a lambda expression
    /// 
    /// Returns true if this context has lambda metadata, indicating it was
    /// created for lambda expression evaluation.
    /// 
    /// # Returns
    /// * `true` - If context has lambda metadata
    /// * `false` - If context is not a lambda context
    pub fn is_lambda_context(&self) -> bool {
        self.variable_scope.lambda_metadata.is_some()
    }

    /// Get the current lambda metadata if available
    /// 
    /// Returns the lambda metadata for this context, if it's a lambda context.
    /// 
    /// # Returns
    /// * `Some(&LambdaMetadata)` - If this is a lambda context
    /// * `None` - If this is not a lambda context
    pub fn lambda_metadata(&self) -> Option<&LambdaMetadata> {
        self.variable_scope.lambda_metadata.as_ref()
    }

    /// Get memory usage information for this context
    /// 
    /// Returns detailed memory usage information about the variable scope
    /// and COW optimization effectiveness.
    /// 
    /// # Returns
    /// VariableScopeMemoryInfo with usage statistics
    pub fn memory_info(&self) -> super::helpers::VariableScopeMemoryInfo {
        self.variable_scope.memory_info()
    }
}

// Debug implementation for EvaluationContext
impl std::fmt::Debug for EvaluationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvaluationContext")
            .field("input", &self.input)
            .field("variable_scope", &self.variable_scope)
            .field("is_lambda_context", &self.is_lambda_context())
            .field("registry", &"<FunctionRegistry>")
            .field("model_provider", &"<ModelProvider>")
            .field("type_annotations", &"<Arc<Mutex<...>>>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::MockModelProvider;

    fn create_test_context() -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(octofhir_fhirpath_registry::FunctionRegistry::new());
        
        EvaluationContext::new(
            FhirPathValue::String("test_input".into()),
            registry,
            model_provider,
        )
    }

    #[test]
    fn test_context_creation() {
        let context = create_test_context();
        assert_eq!(context.input, FhirPathValue::String("test_input".into()));
        assert!(!context.is_lambda_context());
        assert_eq!(context.variable_scope.local_count(), 0);
    }

    #[test]
    fn test_with_variables() {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(octofhir_fhirpath_registry::FunctionRegistry::new());
        
        let mut variables = FxHashMap::default();
        variables.insert("test_var".to_string(), FhirPathValue::String("test_value".into()));
        
        let context = EvaluationContext::with_variables(
            FhirPathValue::String("input".into()),
            registry,
            model_provider,
            variables,
        );
        
        assert_eq!(
            context.get_variable("test_var"),
            Some(&FhirPathValue::String("test_value".into()))
        );
    }

    #[test]
    fn test_with_input() {
        let context = create_test_context();
        let new_input = FhirPathValue::String("new_input".into());
        
        let child_context = context.with_input(new_input.clone());
        
        assert_eq!(child_context.input, new_input);
        // Root should be shared
        assert!(Arc::ptr_eq(&context.root, &child_context.root));
    }

    #[test]
    fn test_lambda_context() {
        let context = create_test_context();
        let current_item = FhirPathValue::String("lambda_item".into());
        
        let lambda_context = context.with_lambda_context(
            current_item.clone(),
            5,
            FhirPathValue::Integer(10),
        );
        
        assert!(lambda_context.is_lambda_context());
        assert_eq!(lambda_context.input, current_item);
        assert_eq!(lambda_context.get_variable("$this"), Some(&current_item));
        assert_eq!(lambda_context.get_variable("$index"), Some(&FhirPathValue::Integer(5)));
        assert_eq!(lambda_context.get_variable("$total"), Some(&FhirPathValue::Integer(10)));
    }

    #[test]
    fn test_lambda_context_preserving_index() {
        let context = create_test_context();
        
        // Create a lambda context first
        let lambda_context = context.with_lambda_context(
            FhirPathValue::String("original_item".into()),
            7,
            FhirPathValue::Integer(20),
        );
        
        // Create another lambda context preserving the index
        let new_item = FhirPathValue::String("new_item".into());
        let preserving_context = lambda_context.with_lambda_context_preserving_index(new_item.clone());
        
        assert_eq!(preserving_context.input, new_item);
        assert_eq!(preserving_context.get_variable("$this"), Some(&new_item));
        assert_eq!(preserving_context.get_variable("$index"), Some(&FhirPathValue::Integer(7))); // Preserved
        assert_eq!(preserving_context.get_variable("$total"), Some(&FhirPathValue::Integer(20))); // Preserved
    }

    #[test]
    fn test_variable_operations() {
        let mut context = create_test_context();
        
        // Set a regular variable
        context.set_variable("user_var".to_string(), FhirPathValue::String("user_value".into()));
        assert_eq!(
            context.get_variable("user_var"),
            Some(&FhirPathValue::String("user_value".into()))
        );
        
        // Try to set a system variable (should be ignored)
        context.set_variable("context".to_string(), FhirPathValue::String("should_be_ignored".into()));
        assert!(context.get_variable("context").is_none());
    }

    #[test]
    fn test_type_annotation_cache() {
        let context = create_test_context();
        let type_info = TypeReflectionInfo::simple_type("Test", "TestType");
        
        // Set and get type annotation
        context.set_type_annotation("test_key".to_string(), type_info.clone());
        let retrieved = context.get_type_annotation("test_key");
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "TestType");
        
        // Clear cache
        context.clear_type_annotations();
        assert!(context.get_type_annotation("test_key").is_none());
    }

    #[test]
    fn test_with_fresh_variable_scope() {
        let mut context = create_test_context();
        context.set_variable("parent_var".to_string(), FhirPathValue::String("parent_value".into()));
        
        let fresh_context = context.with_fresh_variable_scope();
        
        // Fresh context should not have access to parent variables
        assert!(fresh_context.get_variable("parent_var").is_none());
        assert_eq!(fresh_context.variable_scope.local_count(), 0);
    }

    #[test]
    fn test_with_inherited_scope() {
        let mut context = create_test_context();
        context.set_variable("parent_var".to_string(), FhirPathValue::String("parent_value".into()));
        
        let inherited_context = context.with_inherited_scope(FhirPathValue::String("new_input".into()));
        
        // Inherited context should have access to parent variables
        assert_eq!(
            inherited_context.get_variable("parent_var"),
            Some(&FhirPathValue::String("parent_value".into()))
        );
        assert_eq!(inherited_context.input, FhirPathValue::String("new_input".into()));
    }

    #[test]
    fn test_debug_functionality() {
        let context = create_test_context();
        
        let debug_info = context.debug_info();
        assert!(debug_info.contains("EvaluationContext Debug Info"));
        assert!(debug_info.contains("Input:"));
        assert!(debug_info.contains("No lambda context"));
        
        let variable_chain = context.debug_variable_chain();
        assert!(variable_chain.contains("Scope[0]"));
    }

    #[test]
    fn test_memory_info() {
        let context = create_test_context();
        let memory_info = context.memory_info();
        
        assert_eq!(memory_info.local_variables, 0);
        assert_eq!(memory_info.scope_depth, 1);
        assert_eq!(memory_info.total_variables, 0);
    }
}