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

//! Shared Expression Context for Arc-based memory optimization
//!
//! This module provides enhanced context sharing capabilities for FHIRPath expressions
//! using Arc-backed data structures to minimize memory allocations and enable efficient
//! context reuse across related expressions.

use super::context::EvaluationContext;
use fhirpath_model::FhirPathValue;
use fhirpath_registry::{FunctionRegistry, OperatorRegistry};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Type alias for Arc-backed variable map
pub type VarMap = FxHashMap<String, FhirPathValue>;

/// Shared evaluation context optimized for Arc-based memory sharing
///
/// This structure allows multiple expressions to share the same base context,
/// function registry, and variable scopes efficiently using Arc reference counting.
#[derive(Clone)]
pub struct SharedEvaluationContext {
    /// Base evaluation context shared across expressions
    pub base: Arc<EvaluationContext>,

    /// Variables shared through Arc for efficient copying
    pub variables: Arc<RwLock<VarMap>>,

    /// Function registry shared through Arc
    pub functions: Arc<FunctionRegistry>,

    /// Operator registry shared through Arc  
    pub operators: Arc<OperatorRegistry>,

    /// Current input value for this context
    pub input: FhirPathValue,

    /// Root input value (for %context and $resource variables)
    pub root: FhirPathValue,

    /// Context generation for cache invalidation
    pub generation: u64,
}

impl SharedEvaluationContext {
    /// Create a new shared evaluation context
    pub fn new(
        input: FhirPathValue,
        functions: Arc<FunctionRegistry>,
        operators: Arc<OperatorRegistry>,
    ) -> Self {
        // For internal shared contexts, using MockModelProvider is acceptable
        let provider = Arc::new(fhirpath_model::MockModelProvider::empty());
        let base = Arc::new(EvaluationContext::new(
            input.clone(),
            functions.clone(),
            operators.clone(),
            provider,
        ));

        Self {
            base,
            variables: Arc::new(RwLock::new(FxHashMap::default())),
            functions,
            operators,
            root: input.clone(),
            input,
            generation: 0,
        }
    }

    /// Create a shared context from an existing evaluation context
    pub fn from_context(context: EvaluationContext) -> Self {
        let functions = context.functions.clone();
        let operators = context.operators.clone();
        let input = context.input.clone();
        let root = context.root.clone();

        // Extract variables from the context's variable scope
        let variables = {
            let all_vars = context.variable_scope.collect_all_variables();
            Arc::new(RwLock::new(all_vars))
        };

        Self {
            base: Arc::new(context),
            variables,
            functions,
            operators,
            input,
            root,
            generation: 0,
        }
    }

    /// Create a child context with new input value (efficient Arc sharing)
    pub fn with_input(&self, input: FhirPathValue) -> Self {
        Self {
            base: self.base.clone(),
            variables: self.variables.clone(),
            functions: self.functions.clone(),
            operators: self.operators.clone(),
            root: self.root.clone(),
            input,
            generation: self.generation,
        }
    }

    /// Create a child context with fresh variable scope
    pub fn with_fresh_variables(&self) -> Self {
        Self {
            base: self.base.clone(),
            variables: Arc::new(RwLock::new(FxHashMap::default())),
            functions: self.functions.clone(),
            operators: self.operators.clone(),
            input: self.input.clone(),
            root: self.root.clone(),
            generation: self.generation + 1,
        }
    }

    /// Create a child context inheriting variables (Copy-on-Write semantics)
    pub fn with_inherited_variables(&self, input: FhirPathValue) -> Self {
        // Clone the current variables for CoW semantics
        let inherited_vars = {
            let vars_guard = self.variables.read().unwrap();
            Arc::new(RwLock::new(vars_guard.clone()))
        };

        Self {
            base: self.base.clone(),
            variables: inherited_vars,
            functions: self.functions.clone(),
            operators: self.operators.clone(),
            input,
            root: self.root.clone(),
            generation: self.generation + 1,
        }
    }

    /// Create a child context with shared variable references (zero-copy until modification)
    pub fn with_shared_variables(&self, input: FhirPathValue) -> Self {
        Self {
            base: self.base.clone(),
            variables: self.variables.clone(), // Direct Arc sharing - zero copy
            functions: self.functions.clone(),
            operators: self.operators.clone(),
            input,
            root: self.root.clone(),
            generation: self.generation,
        }
    }

    /// Create a scoped context that can shadow parent variables
    pub fn with_scoped_variables(
        &self,
        input: FhirPathValue,
        scope_vars: FxHashMap<String, FhirPathValue>,
    ) -> Self {
        let context = self.with_inherited_variables(input);

        // Add scope-specific variables
        {
            let mut vars = context.variables.write().unwrap();
            vars.extend(scope_vars);
        }

        context
    }

    /// Merge variables from another context (parent variables are overridden)
    pub fn merge_variables(&self, other: &SharedEvaluationContext) -> Self {
        let merged_vars = {
            let self_vars = self.variables.read().unwrap();
            let other_vars = other.variables.read().unwrap();

            let mut merged = self_vars.clone();
            merged.extend(other_vars.iter().map(|(k, v)| (k.clone(), v.clone())));

            Arc::new(RwLock::new(merged))
        };

        Self {
            base: self.base.clone(),
            variables: merged_vars,
            functions: self.functions.clone(),
            operators: self.operators.clone(),
            input: self.input.clone(),
            root: self.root.clone(),
            generation: self.generation + 1,
        }
    }

    /// Set a variable in the shared context
    pub fn set_variable(&self, name: String, value: FhirPathValue) {
        let mut vars = self.variables.write().unwrap();
        vars.insert(name, value);
    }

    /// Get a variable from the shared context
    pub fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        let vars = self.variables.read().unwrap();
        vars.get(name).cloned()
    }

    /// Check if a variable exists in the context
    pub fn has_variable(&self, name: &str) -> bool {
        let vars = self.variables.read().unwrap();
        vars.contains_key(name)
    }

    /// Get the number of variables in this context
    pub fn variable_count(&self) -> usize {
        let vars = self.variables.read().unwrap();
        vars.len()
    }

    /// Batch set variables for efficient initialization
    pub fn set_variables_batch(&self, variables: FxHashMap<String, FhirPathValue>) {
        let mut vars = self.variables.write().unwrap();
        vars.extend(variables);
    }

    /// Create a variable snapshot (Arc-optimized cloning)
    pub fn snapshot_variables(&self) -> Arc<RwLock<VarMap>> {
        let vars = self.variables.read().unwrap();
        Arc::new(RwLock::new(vars.clone()))
    }

    /// Restore variables from a snapshot
    pub fn restore_variables(&self, snapshot: Arc<RwLock<VarMap>>) {
        let snapshot_vars = snapshot.read().unwrap();
        let mut vars = self.variables.write().unwrap();
        vars.clear();
        vars.extend(snapshot_vars.iter().map(|(k, v)| (k.clone(), v.clone())));
    }

    /// Check if variables are shared with another context (same Arc instance)
    pub fn shares_variables_with(&self, other: &SharedEvaluationContext) -> bool {
        Arc::ptr_eq(&self.variables, &other.variables)
    }

    /// Get variable sharing statistics
    pub fn variable_sharing_info(&self) -> VariableSharingInfo {
        let vars = self.variables.read().unwrap();
        let strong_refs = Arc::strong_count(&self.variables);
        let weak_refs = Arc::weak_count(&self.variables);

        VariableSharingInfo {
            variable_count: vars.len(),
            strong_references: strong_refs,
            weak_references: weak_refs,
            is_shared: strong_refs > 1,
            generation: self.generation,
        }
    }

    /// Convert to a standard EvaluationContext when needed
    pub fn to_evaluation_context(&self) -> EvaluationContext {
        // For internal shared contexts, using MockModelProvider is acceptable
        let provider = Arc::new(fhirpath_model::MockModelProvider::empty());
        let mut context = EvaluationContext::new(
            self.input.clone(),
            self.functions.clone(),
            self.operators.clone(),
            provider,
        );
        context.root = self.root.clone();

        // Copy variables to the evaluation context
        let vars = self.variables.read().unwrap();
        for (name, value) in vars.iter() {
            context.set_variable(name.clone(), value.clone());
        }

        context
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> SharedContextMemoryStats {
        let vars = self.variables.read().unwrap();
        SharedContextMemoryStats {
            variable_count: vars.len(),
            base_context_refs: Arc::strong_count(&self.base),
            variables_refs: Arc::strong_count(&self.variables),
            functions_refs: Arc::strong_count(&self.functions),
            operators_refs: Arc::strong_count(&self.operators),
            generation: self.generation,
        }
    }
}

/// Memory usage statistics for shared contexts
#[derive(Debug, Clone)]
pub struct SharedContextMemoryStats {
    /// Number of variables in this context
    pub variable_count: usize,
    /// Number of references to the base context
    pub base_context_refs: usize,
    /// Number of references to the variables map
    pub variables_refs: usize,
    /// Number of references to the function registry
    pub functions_refs: usize,
    /// Number of references to the operator registry
    pub operators_refs: usize,
    /// Context generation number
    pub generation: u64,
}

/// Variable sharing information for Arc-based optimization analysis
#[derive(Debug, Clone)]
pub struct VariableSharingInfo {
    /// Number of variables in the shared map
    pub variable_count: usize,
    /// Number of strong references to the variables Arc
    pub strong_references: usize,
    /// Number of weak references to the variables Arc
    pub weak_references: usize,
    /// Whether variables are currently being shared
    pub is_shared: bool,
    /// Context generation for tracking
    pub generation: u64,
}

/// Context inheritance manager for optimized context composition
#[derive(Clone)]
pub struct ContextInheritance {
    /// Parent context chain for efficient variable lookup
    parents: VecDeque<Arc<SharedEvaluationContext>>,
    /// Maximum depth of inheritance chain
    max_depth: usize,
}

impl ContextInheritance {
    /// Create a new context inheritance manager
    pub fn new(max_depth: usize) -> Self {
        Self {
            parents: VecDeque::with_capacity(max_depth),
            max_depth,
        }
    }

    /// Add a parent context to the inheritance chain
    pub fn add_parent(&mut self, parent: Arc<SharedEvaluationContext>) {
        if self.parents.len() >= self.max_depth {
            self.parents.pop_front(); // Remove oldest parent
        }
        self.parents.push_back(parent);
    }

    /// Look up a variable in the inheritance chain
    pub fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        // Search from most recent to oldest parent
        for parent in self.parents.iter().rev() {
            if let Some(value) = parent.get_variable(name) {
                return Some(value);
            }
        }
        None
    }

    /// Look up a variable with shadowing resolution strategy
    pub fn get_variable_with_shadowing(&self, name: &str) -> Option<(FhirPathValue, usize)> {
        // Returns the value and the depth at which it was found
        for (depth, parent) in self.parents.iter().rev().enumerate() {
            if let Some(value) = parent.get_variable(name) {
                return Some((value, depth));
            }
        }
        None
    }

    /// Collect all variables from all inheritance levels into a flat map
    /// More recent contexts override older ones
    pub fn collect_all_variables(&self) -> FxHashMap<String, FhirPathValue> {
        let mut all_vars = FxHashMap::default();

        // Start from oldest to newest so newer values override older ones
        for parent in self.parents.iter() {
            let vars = parent.variables.read().unwrap();
            for (name, value) in vars.iter() {
                all_vars.insert(name.clone(), value.clone());
            }
        }

        all_vars
    }

    /// Check if a variable exists anywhere in the inheritance chain
    pub fn has_variable(&self, name: &str) -> bool {
        self.parents.iter().any(|parent| parent.has_variable(name))
    }

    /// Get the depth of the inheritance chain
    pub fn depth(&self) -> usize {
        self.parents.len()
    }

    /// Clear the inheritance chain
    pub fn clear(&mut self) {
        self.parents.clear();
    }

    /// Merge another inheritance chain into this one
    pub fn merge(&mut self, other: &ContextInheritance) {
        for parent in &other.parents {
            self.add_parent(parent.clone());
        }
    }

    /// Create a new inheritance chain by composing multiple contexts
    pub fn compose(contexts: Vec<Arc<SharedEvaluationContext>>, max_depth: usize) -> Self {
        let mut inheritance = Self::new(max_depth);
        for context in contexts {
            inheritance.add_parent(context);
        }
        inheritance
    }

    /// Get all contexts in the inheritance chain
    pub fn get_context_chain(&self) -> Vec<Arc<SharedEvaluationContext>> {
        self.parents.iter().cloned().collect()
    }

    /// Remove a specific context from the inheritance chain
    pub fn remove_context(&mut self, generation: u64) -> bool {
        if let Some(pos) = self
            .parents
            .iter()
            .position(|parent| parent.generation == generation)
        {
            self.parents.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Function closure optimization for frequently used function combinations
#[derive(Clone)]
pub struct FunctionClosureOptimizer {
    /// Cached function closures for common patterns
    closures:
        Arc<RwLock<FxHashMap<String, Arc<dyn Fn(&FhirPathValue) -> FhirPathValue + Send + Sync>>>>,
    /// Hit count for each closure
    hit_counts: Arc<RwLock<FxHashMap<String, usize>>>,
    /// Maximum number of cached closures
    max_closures: usize,
}

impl FunctionClosureOptimizer {
    /// Create a new function closure optimizer
    pub fn new(max_closures: usize) -> Self {
        Self {
            closures: Arc::new(RwLock::new(FxHashMap::default())),
            hit_counts: Arc::new(RwLock::new(FxHashMap::default())),
            max_closures,
        }
    }

    /// Create a new optimizer with pre-built common closures
    pub fn with_common_patterns(max_closures: usize) -> Self {
        let optimizer = Self::new(max_closures);
        optimizer.add_common_patterns();
        optimizer
    }

    /// Add commonly used FHIRPath pattern closures
    pub fn add_common_patterns(&self) {
        // Common FHIR property access patterns
        self.cache_closure("not_empty".to_string(), |input| match input {
            FhirPathValue::Empty => FhirPathValue::Boolean(false),
            FhirPathValue::Collection(items) if items.is_empty() => FhirPathValue::Boolean(false),
            _ => FhirPathValue::Boolean(true),
        });

        self.cache_closure("is_empty".to_string(), |input| match input {
            FhirPathValue::Empty => FhirPathValue::Boolean(true),
            FhirPathValue::Collection(items) if items.is_empty() => FhirPathValue::Boolean(true),
            _ => FhirPathValue::Boolean(false),
        });

        self.cache_closure("count".to_string(), |input| match input {
            FhirPathValue::Empty => FhirPathValue::Integer(0),
            FhirPathValue::Collection(items) => FhirPathValue::Integer(items.len() as i64),
            _ => FhirPathValue::Integer(1),
        });

        self.cache_closure("single".to_string(), |input| match input {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    input.first().cloned().unwrap_or(FhirPathValue::Empty)
                } else {
                    FhirPathValue::Empty
                }
            }
            val if !matches!(val, FhirPathValue::Empty | FhirPathValue::Collection(_)) => {
                input.clone()
            }
            _ => FhirPathValue::Empty,
        });

        self.cache_closure("first".to_string(), |input| match input {
            FhirPathValue::Collection(_) => input.first().cloned().unwrap_or(FhirPathValue::Empty),
            val if !matches!(val, FhirPathValue::Empty | FhirPathValue::Collection(_)) => {
                input.clone()
            }
            _ => FhirPathValue::Empty,
        });

        self.cache_closure("last".to_string(), |input| {
            match input {
                FhirPathValue::Collection(items) => {
                    if !items.is_empty() {
                        // Access last element via iteration since we can't index directly
                        items
                            .clone()
                            .into_iter()
                            .last()
                            .unwrap_or(FhirPathValue::Empty)
                    } else {
                        FhirPathValue::Empty
                    }
                }
                val if !matches!(val, FhirPathValue::Empty | FhirPathValue::Collection(_)) => {
                    input.clone()
                }
                _ => FhirPathValue::Empty,
            }
        });
    }

    /// Create and cache a function closure for a given pattern
    pub fn cache_closure<F>(&self, pattern: String, closure: F)
    where
        F: Fn(&FhirPathValue) -> FhirPathValue + Send + Sync + 'static,
    {
        let mut closures = self.closures.write().unwrap();
        let mut hit_counts = self.hit_counts.write().unwrap();

        // If we're at capacity, remove least frequently used closure
        if closures.len() >= self.max_closures {
            if let Some((least_used, _)) = hit_counts.iter().min_by_key(|&(_, &count)| count) {
                let least_used = least_used.clone();
                closures.remove(&least_used);
                hit_counts.remove(&least_used);
            }
        }

        closures.insert(pattern.clone(), Arc::new(closure));
        hit_counts.insert(pattern, 0);
    }

    /// Execute a cached closure if available
    pub fn execute_closure(&self, pattern: &str, input: &FhirPathValue) -> Option<FhirPathValue> {
        let closures = self.closures.read().unwrap();
        if let Some(closure) = closures.get(pattern) {
            // Update hit count
            let mut hit_counts = self.hit_counts.write().unwrap();
            *hit_counts.entry(pattern.to_string()).or_insert(0) += 1;

            Some(closure(input))
        } else {
            None
        }
    }

    /// Batch execute multiple closures with the same input
    pub fn execute_closures_batch(
        &self,
        patterns: &[&str],
        input: &FhirPathValue,
    ) -> Vec<Option<FhirPathValue>> {
        let closures = self.closures.read().unwrap();
        let mut hit_counts = self.hit_counts.write().unwrap();

        patterns
            .iter()
            .map(|pattern| {
                if let Some(closure) = closures.get(*pattern) {
                    *hit_counts.entry(pattern.to_string()).or_insert(0) += 1;
                    Some(closure(input))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a pattern is cached without executing
    pub fn has_closure(&self, pattern: &str) -> bool {
        let closures = self.closures.read().unwrap();
        closures.contains_key(pattern)
    }

    /// Remove a cached closure by pattern
    pub fn remove_closure(&self, pattern: &str) -> bool {
        let mut closures = self.closures.write().unwrap();
        let mut hit_counts = self.hit_counts.write().unwrap();

        let removed = closures.remove(pattern).is_some();
        hit_counts.remove(pattern);
        removed
    }

    /// Clear all cached closures
    pub fn clear_closures(&self) {
        let mut closures = self.closures.write().unwrap();
        let mut hit_counts = self.hit_counts.write().unwrap();

        closures.clear();
        hit_counts.clear();
    }

    /// Get patterns sorted by hit count (most used first)
    pub fn get_patterns_by_usage(&self) -> Vec<(String, usize)> {
        let hit_counts = self.hit_counts.read().unwrap();
        let mut patterns: Vec<_> = hit_counts
            .iter()
            .map(|(pattern, &count)| (pattern.clone(), count))
            .collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        patterns
    }

    /// Get closure cache statistics
    pub fn stats(&self) -> ClosureOptimizerStats {
        let closures = self.closures.read().unwrap();
        let hit_counts = self.hit_counts.read().unwrap();

        let total_hits = hit_counts.values().sum();
        let average_hits = if closures.is_empty() {
            0.0
        } else {
            total_hits as f64 / closures.len() as f64
        };

        ClosureOptimizerStats {
            cached_closures: closures.len(),
            total_hits,
            average_hits,
            max_capacity: self.max_closures,
        }
    }
}

/// Statistics for the function closure optimizer
#[derive(Debug, Clone)]
pub struct ClosureOptimizerStats {
    /// Number of cached closures
    pub cached_closures: usize,
    /// Total number of cache hits
    pub total_hits: usize,
    /// Average hits per closure
    pub average_hits: f64,
    /// Maximum cache capacity
    pub max_capacity: usize,
}

/// Context composition builder for creating optimized shared contexts
#[derive(Default)]
pub struct SharedContextBuilder {
    input: Option<FhirPathValue>,
    root: Option<FhirPathValue>,
    functions: Option<Arc<FunctionRegistry>>,
    operators: Option<Arc<OperatorRegistry>>,
    variables: FxHashMap<String, FhirPathValue>,
    inheritance: Option<ContextInheritance>,
}

impl SharedContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the input value
    pub fn with_input(mut self, input: FhirPathValue) -> Self {
        self.input = Some(input);
        self
    }

    /// Set the root value
    pub fn with_root(mut self, root: FhirPathValue) -> Self {
        self.root = Some(root);
        self
    }

    /// Set the function registry
    pub fn with_functions(mut self, functions: Arc<FunctionRegistry>) -> Self {
        self.functions = Some(functions);
        self
    }

    /// Set the operator registry
    pub fn with_operators(mut self, operators: Arc<OperatorRegistry>) -> Self {
        self.operators = Some(operators);
        self
    }

    /// Add a variable
    pub fn with_variable(mut self, name: String, value: FhirPathValue) -> Self {
        self.variables.insert(name, value);
        self
    }

    /// Add multiple variables
    pub fn with_variables(mut self, variables: FxHashMap<String, FhirPathValue>) -> Self {
        self.variables.extend(variables);
        self
    }

    /// Set context inheritance
    pub fn with_inheritance(mut self, inheritance: ContextInheritance) -> Self {
        self.inheritance = Some(inheritance);
        self
    }

    /// Build the shared context
    pub fn build(self) -> Result<SharedEvaluationContext, SharedContextError> {
        let input = self.input.ok_or(SharedContextError::MissingInput)?;
        let functions = self.functions.ok_or(SharedContextError::MissingFunctions)?;
        let operators = self.operators.ok_or(SharedContextError::MissingOperators)?;

        let root = self.root.unwrap_or_else(|| input.clone());

        let mut context = SharedEvaluationContext::new(input, functions, operators);
        context.root = root;

        // Add variables
        for (name, value) in self.variables {
            context.set_variable(name, value);
        }

        Ok(context)
    }
}

/// Errors that can occur when building shared contexts
#[derive(Debug, thiserror::Error)]
pub enum SharedContextError {
    #[error("Missing input value")]
    MissingInput,
    #[error("Missing function registry")]
    MissingFunctions,
    #[error("Missing operator registry")]
    MissingOperators,
}

#[cfg(test)]
mod tests {
    use super::*;
    use fhirpath_registry::{FunctionRegistry, OperatorRegistry};

    #[test]
    fn test_shared_context_creation() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let input = FhirPathValue::Integer(42);

        let context = SharedEvaluationContext::new(input.clone(), functions, operators);

        assert_eq!(context.input, input);
        assert_eq!(context.root, input);
        assert_eq!(context.variable_count(), 0);
        assert_eq!(context.generation, 0);
    }

    #[test]
    fn test_shared_context_variables() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let input = FhirPathValue::Integer(42);

        let context = SharedEvaluationContext::new(input, functions, operators);

        // Set and get variables
        context.set_variable("test_var".to_string(), FhirPathValue::Boolean(true));
        assert_eq!(
            context.get_variable("test_var"),
            Some(FhirPathValue::Boolean(true))
        );
        assert!(context.has_variable("test_var"));
        assert!(!context.has_variable("nonexistent"));
        assert_eq!(context.variable_count(), 1);
    }

    #[test]
    fn test_shared_context_child_creation() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let input = FhirPathValue::Integer(42);

        let parent_context = SharedEvaluationContext::new(input.clone(), functions, operators);
        parent_context.set_variable(
            "parent_var".to_string(),
            FhirPathValue::String("parent".to_string().into()),
        );

        // Create child with new input
        let new_input = FhirPathValue::Boolean(true);
        let child_context = parent_context.with_input(new_input.clone());

        assert_eq!(child_context.input, new_input);
        assert_eq!(child_context.root, input); // Root preserved
        assert_eq!(
            child_context.get_variable("parent_var"),
            Some(FhirPathValue::String("parent".to_string().into()))
        );

        // Create child with fresh variables
        let fresh_context = parent_context.with_fresh_variables();
        assert_eq!(fresh_context.variable_count(), 0);
        assert!(fresh_context.generation > parent_context.generation);

        // Create child with inherited variables
        let inherited_context = parent_context
            .with_inherited_variables(FhirPathValue::String("child".to_string().into()));
        assert_eq!(
            inherited_context.get_variable("parent_var"),
            Some(FhirPathValue::String("parent".to_string().into()))
        );
        assert!(inherited_context.generation > parent_context.generation);
    }

    #[test]
    fn test_context_inheritance() {
        let mut inheritance = ContextInheritance::new(3);

        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());

        // Create parent contexts
        let parent1 = Arc::new(SharedEvaluationContext::new(
            FhirPathValue::Integer(1),
            functions.clone(),
            operators.clone(),
        ));
        parent1.set_variable(
            "var1".to_string(),
            FhirPathValue::String("parent1".to_string().into()),
        );

        let parent2 = Arc::new(SharedEvaluationContext::new(
            FhirPathValue::Integer(2),
            functions,
            operators,
        ));
        parent2.set_variable(
            "var2".to_string(),
            FhirPathValue::String("parent2".to_string().into()),
        );
        parent2.set_variable(
            "var1".to_string(),
            FhirPathValue::String("parent2_override".to_string().into()),
        ); // Override var1

        inheritance.add_parent(parent1);
        inheritance.add_parent(parent2);

        // Test variable lookup
        assert_eq!(
            inheritance.get_variable("var1"),
            Some(FhirPathValue::String("parent2_override".to_string().into()))
        ); // Most recent parent wins
        assert_eq!(
            inheritance.get_variable("var2"),
            Some(FhirPathValue::String("parent2".to_string().into()))
        );
        assert_eq!(inheritance.get_variable("nonexistent"), None);

        assert!(inheritance.has_variable("var1"));
        assert!(inheritance.has_variable("var2"));
        assert!(!inheritance.has_variable("nonexistent"));

        assert_eq!(inheritance.depth(), 2);
    }

    #[test]
    fn test_function_closure_optimizer() {
        let optimizer = FunctionClosureOptimizer::new(2);

        // Cache a simple closure
        optimizer.cache_closure("double".to_string(), |input| match input {
            FhirPathValue::Integer(n) => FhirPathValue::Integer(n * 2),
            _ => FhirPathValue::Empty,
        });

        // Execute cached closure
        let input = FhirPathValue::Integer(21);
        let result = optimizer.execute_closure("double", &input);
        assert_eq!(result, Some(FhirPathValue::Integer(42)));

        // Test cache miss
        let result = optimizer.execute_closure("nonexistent", &input);
        assert_eq!(result, None);

        // Check stats
        let stats = optimizer.stats();
        assert_eq!(stats.cached_closures, 1);
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.max_capacity, 2);
    }

    #[test]
    fn test_shared_context_builder() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let input = FhirPathValue::Integer(42);

        let context = SharedContextBuilder::new()
            .with_input(input.clone())
            .with_functions(functions)
            .with_operators(operators)
            .with_variable("test_var".to_string(), FhirPathValue::Boolean(true))
            .build()
            .unwrap();

        assert_eq!(context.input, input);
        assert_eq!(
            context.get_variable("test_var"),
            Some(FhirPathValue::Boolean(true))
        );
    }

    #[test]
    fn test_shared_context_builder_errors() {
        let result = SharedContextBuilder::new().build();
        assert!(matches!(result, Err(SharedContextError::MissingInput)));

        let result = SharedContextBuilder::new()
            .with_input(FhirPathValue::Integer(42))
            .build();
        assert!(matches!(result, Err(SharedContextError::MissingFunctions)));
    }

    #[test]
    fn test_shared_context_memory_stats() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let input = FhirPathValue::Integer(42);

        let context1 =
            SharedEvaluationContext::new(input.clone(), functions.clone(), operators.clone());
        let context2 = context1.with_input(FhirPathValue::Boolean(true));

        let stats1 = context1.memory_stats();
        let stats2 = context2.memory_stats();

        // Both contexts should share the same function and operator registries
        assert!(stats1.functions_refs >= 2); // At least shared between context1 and context2
        assert!(stats1.operators_refs >= 2);
        assert_eq!(stats1.generation, 0);
        assert_eq!(stats2.generation, 0); // Child with same input doesn't increment generation
    }

    #[test]
    fn test_shared_context_conversion() {
        let functions = Arc::new(FunctionRegistry::new());
        let operators = Arc::new(OperatorRegistry::new());
        let input = FhirPathValue::Integer(42);

        let shared_context = SharedEvaluationContext::new(input.clone(), functions, operators);
        shared_context.set_variable("test_var".to_string(), FhirPathValue::Boolean(true));

        let eval_context = shared_context.to_evaluation_context();
        assert_eq!(eval_context.input, input);
        assert_eq!(
            eval_context.get_variable("test_var"),
            Some(&FhirPathValue::Boolean(true))
        );
    }
}
