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

//! Context creation helpers and utilities
//!
//! This module provides helper structures and utilities to simplify the creation
//! of evaluation contexts, particularly for lambda expressions and complex scoping
//! scenarios. These helpers maintain the underlying COW semantics and performance
//! optimizations while providing more user-friendly APIs.

use octofhir_fhirpath_model::{FhirPathValue, provider::ModelProvider};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

use super::{EvaluationContext, VariableScope};

/// Helper for consistent lambda context creation
///
/// This builder provides a fluent API for creating lambda contexts with proper
/// variable scoping and parameter mappings. It ensures that all lambda contexts
/// are created consistently with the correct implicit variables and environment
/// variable inheritance.
///
/// # Lambda Context Requirements
///
/// Lambda contexts must provide:
/// - Implicit variables: `$this`, `$index`, `$total`
/// - Environment variable inheritance from parent scope
/// - Optional explicit parameter mappings
/// - Proper Arc-based resource sharing for memory efficiency
///
/// # Usage Examples
///
/// ```rust,no_run
/// use octofhir_fhirpath_evaluator::{LambdaContextBuilder, EvaluationContext};
/// use octofhir_fhirpath_model::FhirPathValue;
///
/// // Assume we have a base context from somewhere (e.g., from FhirPathEngine)
/// # let base_context: EvaluationContext = unimplemented!();
/// let item = FhirPathValue::String("test".into());
///
/// // Simple lambda context for iteration
/// let lambda_ctx = LambdaContextBuilder::new(&base_context)
///     .with_current_item(item.clone())
///     .with_index(5)
///     .with_total(FhirPathValue::Integer(10))
///     .build();
///
/// // Lambda context with explicit parameters
/// let lambda_ctx = LambdaContextBuilder::new(&base_context)
///     .with_current_item(item.clone())
///     .with_parameter("customVar".to_string(), FhirPathValue::String("value".into()))
///     .build();
/// ```
pub struct LambdaContextBuilder {
    /// Base context to inherit from (provides registry, model provider, etc.)
    base_context: EvaluationContext,
    /// Current item being processed (becomes `$this`)
    current_item: Option<FhirPathValue>,
    /// Current index in iteration (becomes `$index`)  
    index: Option<usize>,
    /// Total count or accumulator (becomes `$total`)
    total: Option<FhirPathValue>,
    /// Explicit parameter mappings for lambda expression
    param_mappings: Vec<(String, FhirPathValue)>,
}

impl LambdaContextBuilder {
    /// Create a new lambda context builder from a base context
    ///
    /// The base context provides the registry, model provider, root resource,
    /// and variable scope that the lambda context will inherit from.
    ///
    /// # Arguments
    /// * `base_context` - The parent context to inherit from
    ///
    /// # Performance
    /// The base context is cloned, but Arc-shared resources (registry, model provider,
    /// root resource) are shared efficiently without deep copying.
    pub fn new(base_context: &EvaluationContext) -> Self {
        Self {
            base_context: base_context.clone(),
            current_item: None,
            index: None,
            total: None,
            param_mappings: Vec::new(),
        }
    }

    /// Create a lambda context builder for a specific item and index
    ///
    /// This is a convenience constructor for the common case where you have
    /// an item and its iteration index. The total value can be set later.
    ///
    /// # Arguments
    /// * `base_context` - The parent context to inherit from
    /// * `item` - Current item being processed (becomes `$this`)
    /// * `index` - Zero-based iteration index (becomes `$index`)
    pub fn for_item(base_context: &EvaluationContext, item: FhirPathValue, index: usize) -> Self {
        Self {
            base_context: base_context.clone(),
            current_item: Some(item),
            index: Some(index),
            total: None,
            param_mappings: Vec::new(),
        }
    }

    /// Set the current item ($this variable)
    ///
    /// # Arguments
    /// * `item` - The value that will be available as `$this` in the lambda
    pub fn with_current_item(mut self, item: FhirPathValue) -> Self {
        self.current_item = Some(item);
        self
    }

    /// Set the current index ($index variable)
    ///
    /// # Arguments
    /// * `index` - Zero-based iteration index (will be converted to Integer)
    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Set the total value ($total variable)
    ///
    /// The meaning of the total value depends on the lambda function context:
    /// - For `where` and `select`: usually the total collection size
    /// - For aggregates: may be an accumulator value
    /// - For other functions: context-specific meaning
    ///
    /// # Arguments
    /// * `total` - The value that will be available as `$total` in the lambda
    pub fn with_total(mut self, total: FhirPathValue) -> Self {
        self.total = Some(total);
        self
    }

    /// Add a parameter mapping for lambda parameters
    ///
    /// This allows lambda expressions to access custom variables in addition
    /// to the standard implicit variables. Parameter names should not conflict
    /// with system variables or implicit lambda variables.
    ///
    /// # Arguments
    /// * `name` - Parameter name (will be available as variable in lambda)
    /// * `value` - Parameter value
    pub fn with_parameter(mut self, name: String, value: FhirPathValue) -> Self {
        self.param_mappings.push((name, value));
        self
    }

    /// Add multiple parameter mappings
    ///
    /// Convenience method for adding multiple parameters at once.
    ///
    /// # Arguments
    /// * `params` - Vector of (name, value) pairs for lambda parameters
    pub fn with_parameters(mut self, params: Vec<(String, FhirPathValue)>) -> Self {
        self.param_mappings.extend(params);
        self
    }

    /// Build the lambda context
    ///
    /// Creates the final `EvaluationContext` with the configured lambda scope.
    /// This method chooses between simple lambda scope creation and parameter-
    /// enhanced creation based on whether explicit parameters were provided.
    ///
    /// # Default Values
    /// - `current_item`: Defaults to base context input if not specified
    /// - `index`: Defaults to 0 if not specified  
    /// - `total`: Defaults to Integer(0) if not specified
    ///
    /// # Returns
    /// Fully configured `EvaluationContext` ready for lambda expression evaluation
    pub fn build(self) -> EvaluationContext {
        let current_item = self
            .current_item
            .unwrap_or_else(|| self.base_context.input.clone());
        let index = self.index.unwrap_or(0);
        let total = self.total.unwrap_or(FhirPathValue::Integer(0));

        if self.param_mappings.is_empty() {
            // Use simple lambda scope for better performance
            let lambda_scope = VariableScope::lambda_scope(
                Some(Arc::new(self.base_context.variable_scope.clone())),
                current_item.clone(),
                index,
                total,
            );

            EvaluationContext {
                input: current_item,
                root: self.base_context.root.clone(), // Arc-shared
                variable_scope: lambda_scope,
                registry: self.base_context.registry.clone(), // Arc-shared
                model_provider: self.base_context.model_provider.clone(), // Arc-shared
                type_annotations: self.base_context.type_annotations.clone(), // Arc-shared
            }
        } else {
            // Use lambda scope with parameters for complex cases
            let lambda_scope = VariableScope::lambda_scope_with_params(
                Some(Arc::new(self.base_context.variable_scope.clone())),
                self.param_mappings,
                current_item.clone(),
                index,
                total,
            );

            EvaluationContext {
                input: current_item,
                root: self.base_context.root.clone(), // Arc-shared
                variable_scope: lambda_scope,
                registry: self.base_context.registry.clone(), // Arc-shared
                model_provider: self.base_context.model_provider.clone(), // Arc-shared
                type_annotations: self.base_context.type_annotations.clone(), // Arc-shared
            }
        }
    }
}

/// Memory usage information for variable scopes
///
/// This structure provides debugging and monitoring information about variable
/// scope memory usage and COW optimization effectiveness. Useful for performance
/// analysis and optimization.
#[derive(Debug, Clone)]
pub struct VariableScopeMemoryInfo {
    /// Number of variables in this scope only
    pub local_variables: usize,
    /// Total variables including all parent scopes
    pub total_variables: usize,
    /// Depth of scope nesting (1 for root scope)
    pub scope_depth: usize,
    /// Number of scopes using efficient Copy-on-Write
    pub efficient_scopes: usize,
    /// Whether this scope is using Copy-on-Write optimization
    pub is_cow_optimized: bool,
}

impl VariableScopeMemoryInfo {
    /// Calculate COW efficiency as a percentage
    ///
    /// Returns the percentage of scopes in the chain that are using
    /// efficient COW optimization.
    ///
    /// # Returns
    /// Efficiency percentage (0.0 to 100.0)
    pub fn cow_efficiency_percent(&self) -> f64 {
        if self.scope_depth == 0 {
            0.0
        } else {
            (self.efficient_scopes as f64 / self.scope_depth as f64) * 100.0
        }
    }

    /// Get a human-readable memory usage summary
    ///
    /// # Returns
    /// String describing memory usage characteristics
    pub fn summary(&self) -> String {
        format!(
            "Scope depth: {}, Local vars: {}, Total vars: {}, COW efficiency: {:.1}%",
            self.scope_depth,
            self.local_variables,
            self.total_variables,
            self.cow_efficiency_percent()
        )
    }
}

/// Context creation utilities
///
/// This struct provides static utility methods for creating contexts in
/// common scenarios. These methods encapsulate best practices and ensure
/// consistent context creation across the codebase.
pub struct ContextFactory;

impl ContextFactory {
    /// Create a context with pre-allocated variable capacity
    ///
    /// This method creates an optimized context when you know approximately
    /// how many variables will be used. Pre-allocation reduces HashMap
    /// reallocations during variable assignment.
    ///
    /// # Arguments
    /// * `input` - Input value for the context
    /// * `registry` - Function registry for operations
    /// * `model_provider` - Model provider for type information
    /// * `capacity` - Expected number of variables to pre-allocate
    ///
    /// # Returns
    /// EvaluationContext with pre-allocated variable storage
    pub fn with_capacity(
        input: FhirPathValue,
        registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        capacity: usize,
    ) -> EvaluationContext {
        EvaluationContext {
            root: Arc::new(input.clone()),
            input,
            variable_scope: VariableScope::with_capacity(capacity),
            registry,
            model_provider,
            type_annotations: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    /// Create a context with initial environment variables
    ///
    /// This method creates a context and immediately sets up the standard
    /// FHIRPath environment variables. This is useful for creating root
    /// contexts that will be used for expression evaluation.
    ///
    /// # Standard Environment Variables
    /// - `context` - The original context node
    /// - `resource` - The resource containing the context
    /// - `rootResource` - The root resource (may be same as resource)
    ///
    /// # Arguments
    /// * `input` - Input value for the context
    /// * `registry` - Function registry for operations
    /// * `model_provider` - Model provider for type information
    /// * `root_resource` - Optional root resource (defaults to input)
    /// * `containing_resource` - Optional containing resource (defaults to input)
    ///
    /// # Returns
    /// EvaluationContext with standard environment variables configured
    pub fn with_environment(
        input: FhirPathValue,
        registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        root_resource: Option<FhirPathValue>,
        containing_resource: Option<FhirPathValue>,
    ) -> EvaluationContext {
        let root_resource = root_resource.unwrap_or_else(|| input.clone());
        let containing_resource = containing_resource.unwrap_or_else(|| input.clone());

        let mut context = EvaluationContext {
            root: Arc::new(root_resource.clone()),
            input,
            variable_scope: VariableScope::new(),
            registry,
            model_provider,
            type_annotations: Arc::new(Mutex::new(FxHashMap::default())),
        };

        // Set up standard environment variables
        context
            .variable_scope
            .set_system_variable("context".to_string(), context.input.clone());
        context
            .variable_scope
            .set_system_variable("resource".to_string(), containing_resource);
        context
            .variable_scope
            .set_system_variable("rootResource".to_string(), root_resource);

        // Set up standard URL constants
        context.variable_scope.set_system_variable(
            "sct".to_string(),
            FhirPathValue::String("http://snomed.info/sct".into()),
        );
        context.variable_scope.set_system_variable(
            "loinc".to_string(),
            FhirPathValue::String("http://loinc.org".into()),
        );
        context.variable_scope.set_system_variable(
            "ucum".to_string(),
            FhirPathValue::String("http://unitsofmeasure.org".into()),
        );

        context
    }

    /// Create a child context from a parent with shared resources
    ///
    /// This method creates a child context that shares Arc-based resources
    /// (registry, model provider, root resource) with the parent while
    /// creating a new variable scope. This is efficient for creating
    /// contexts for sub-expressions.
    ///
    /// # Arguments
    /// * `parent` - Parent context to inherit shared resources from
    /// * `new_input` - New input value for the child context
    ///
    /// # Returns
    /// Child EvaluationContext with shared resources and new variable scope
    pub fn child_context(
        parent: &EvaluationContext,
        new_input: FhirPathValue,
    ) -> EvaluationContext {
        EvaluationContext {
            input: new_input,
            root: parent.root.clone(), // Shared Arc
            variable_scope: VariableScope::child_from_shared(Arc::new(
                parent.variable_scope.clone(),
            )),
            registry: parent.registry.clone(), // Shared Arc
            model_provider: parent.model_provider.clone(), // Shared Arc
            type_annotations: parent.type_annotations.clone(), // Shared Arc
        }
    }
}

/// Additional utilities for variable scope analysis
impl VariableScope {
    /// Collect all variables from this scope and parent scopes into a flat map
    ///
    /// This method creates a flattened view of all variables accessible from
    /// this scope, with child scope variables overriding parent scope variables
    /// of the same name. Useful for debugging and serialization.
    ///
    /// # Performance
    /// O(n) where n is the total number of variables in the scope chain.
    /// Variables are cloned, so this should not be used in hot paths.
    ///
    /// # Returns
    /// HashMap containing all accessible variables with child overriding parent
    pub fn collect_all_variables(&self) -> FxHashMap<String, FhirPathValue> {
        let mut all_variables = FxHashMap::default();

        // First collect from parent scopes (so child scope variables override parent)
        if let Some(parent) = &self.parent {
            all_variables.extend(parent.collect_all_variables());
        }

        // Then add variables from this scope (overriding any parent variables)
        // Use efficient cloning based on Cow state
        match &self.variables {
            std::borrow::Cow::Borrowed(vars) => {
                all_variables.extend(
                    vars.iter()
                        .map(|(k, v): (&String, &FhirPathValue)| (k.clone(), v.clone())),
                );
            }
            std::borrow::Cow::Owned(vars) => {
                all_variables.extend(vars.clone());
            }
        }

        all_variables
    }

    /// Create a flattened scope (useful for serialization or debugging)
    ///
    /// This method creates a new scope containing all variables from this scope
    /// and its parents, with no parent chain. The resulting scope is independent
    /// and does not share any data with the original scope chain.
    ///
    /// # Use Cases
    /// - Debugging variable resolution
    /// - Serializing scope state
    /// - Creating independent context snapshots
    ///
    /// # Returns
    /// New VariableScope with all variables flattened and no parent chain
    pub fn flatten(&self) -> Self {
        let all_vars = self.collect_all_variables();
        VariableScope::from_variables_map(all_vars)
    }

    /// Get memory usage information for debugging
    ///
    /// This method analyzes the scope chain to provide detailed information
    /// about memory usage and COW optimization effectiveness. Useful for
    /// performance analysis and debugging memory issues.
    ///
    /// # Returns
    /// VariableScopeMemoryInfo containing detailed usage statistics
    pub fn memory_info(&self) -> VariableScopeMemoryInfo {
        let local_vars = self.variables.len();
        let mut total_vars = local_vars;
        let mut depth = 1;
        let mut efficient_scopes = if self.is_efficiently_borrowing() {
            1
        } else {
            0
        };

        // Count parent scope info
        let mut current_parent = &self.parent;
        while let Some(parent) = current_parent {
            total_vars += parent.variables.len();
            depth += 1;
            if parent.is_efficiently_borrowing() {
                efficient_scopes += 1;
            }
            current_parent = &parent.parent;
        }

        VariableScopeMemoryInfo {
            local_variables: local_vars,
            total_variables: total_vars,
            scope_depth: depth,
            efficient_scopes,
            is_cow_optimized: self.is_efficiently_borrowing(),
        }
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
    fn test_lambda_context_builder_simple() {
        let base_context = create_test_context();
        let item = FhirPathValue::String("lambda_item".into());

        let lambda_context = LambdaContextBuilder::new(&base_context)
            .with_current_item(item.clone())
            .with_index(5)
            .with_total(FhirPathValue::Integer(10))
            .build();

        assert_eq!(lambda_context.input, item);
        assert_eq!(
            lambda_context.variable_scope.get_variable("$this"),
            Some(&item)
        );
        assert_eq!(
            lambda_context.variable_scope.get_variable("$index"),
            Some(&FhirPathValue::Integer(5))
        );
        assert_eq!(
            lambda_context.variable_scope.get_variable("$total"),
            Some(&FhirPathValue::Integer(10))
        );
    }

    #[test]
    fn test_lambda_context_builder_with_parameters() {
        let base_context = create_test_context();

        let lambda_context = LambdaContextBuilder::new(&base_context)
            .with_current_item(FhirPathValue::String("item".into()))
            .with_parameter(
                "customVar".to_string(),
                FhirPathValue::String("custom_value".into()),
            )
            .build();

        assert_eq!(
            lambda_context.variable_scope.get_variable("customVar"),
            Some(&FhirPathValue::String("custom_value".into()))
        );
    }

    #[test]
    fn test_context_factory_with_capacity() {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(octofhir_fhirpath_registry::FunctionRegistry::new());

        let context = ContextFactory::with_capacity(
            FhirPathValue::String("test".into()),
            registry,
            model_provider,
            10,
        );

        // Context should be created successfully
        assert_eq!(context.input, FhirPathValue::String("test".into()));
    }

    #[test]
    fn test_context_factory_with_environment() {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(octofhir_fhirpath_registry::FunctionRegistry::new());

        let context = ContextFactory::with_environment(
            FhirPathValue::String("test".into()),
            registry,
            model_provider,
            None,
            None,
        );

        // Check environment variables are set
        assert_eq!(
            context.variable_scope.get_variable("context"),
            Some(&FhirPathValue::String("test".into()))
        );
        assert_eq!(
            context.variable_scope.get_variable("sct"),
            Some(&FhirPathValue::String("http://snomed.info/sct".into()))
        );
    }

    #[test]
    fn test_memory_info_efficiency_calculation() {
        let info = VariableScopeMemoryInfo {
            local_variables: 5,
            total_variables: 15,
            scope_depth: 3,
            efficient_scopes: 2,
            is_cow_optimized: true,
        };

        assert!((info.cow_efficiency_percent() - 66.66666666666667).abs() < 0.0001);
        assert!(info.summary().contains("COW efficiency: 66.7%"));
    }

    #[test]
    fn test_variable_scope_flattening() {
        let mut parent = VariableScope::new();
        parent.set_variable(
            "parent_var".to_string(),
            FhirPathValue::String("parent_value".into()),
        );

        let mut child = VariableScope::child(parent);
        child.set_variable(
            "child_var".to_string(),
            FhirPathValue::String("child_value".into()),
        );

        let flattened = child.flatten();

        // Flattened scope should have all variables but no parent
        assert!(flattened.parent.is_none());
        assert_eq!(
            flattened.get_variable("parent_var"),
            Some(&FhirPathValue::String("parent_value".into()))
        );
        assert_eq!(
            flattened.get_variable("child_var"),
            Some(&FhirPathValue::String("child_value".into()))
        );
    }

    #[test]
    fn test_collect_all_variables() {
        let mut parent = VariableScope::new();
        parent.set_variable(
            "shared_var".to_string(),
            FhirPathValue::String("parent_value".into()),
        );
        parent.set_variable(
            "parent_only".to_string(),
            FhirPathValue::String("parent".into()),
        );

        let mut child = VariableScope::child(parent);
        child.set_variable(
            "shared_var".to_string(),
            FhirPathValue::String("child_value".into()),
        );
        child.set_variable(
            "child_only".to_string(),
            FhirPathValue::String("child".into()),
        );

        let all_vars = child.collect_all_variables();

        // Child should override parent for shared_var
        assert_eq!(
            all_vars.get("shared_var"),
            Some(&FhirPathValue::String("child_value".into()))
        );
        assert_eq!(
            all_vars.get("parent_only"),
            Some(&FhirPathValue::String("parent".into()))
        );
        assert_eq!(
            all_vars.get("child_only"),
            Some(&FhirPathValue::String("child".into()))
        );
        assert_eq!(all_vars.len(), 3);
    }
}
