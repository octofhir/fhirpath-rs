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

//! Variable scoping with Copy-on-Write semantics for FHIRPath expressions
//!
//! This module provides efficient variable scoping using Copy-on-Write (COW) semantics
//! for optimal memory usage and performance. Variable inheritance from parent scopes
//! is achieved through shared references until modification is required, at which point
//! the variables are cloned (COW trigger).
//!
//! # COW Performance Benefits
//!
//! - **Zero-copy inheritance**: Child scopes initially borrow parent variables
//! - **Lazy cloning**: Variables are only copied when first modification occurs
//! - **Memory efficiency**: Deep scope chains share data until mutation
//! - **Arc-based parent sharing**: Parent scopes are shared via Arc for minimal overhead
//!
//! # Variable Resolution Order
//!
//! 1. **Local variables**: Variables defined directly in current scope
//! 2. **Lambda metadata**: Implicit variables like `$this`, `$index`, `$total`
//! 3. **Parent scopes**: Recursively search up the scope chain
//! 4. **Environment variables**: System variables like `%context`, `%resource`

use octofhir_fhirpath_model::FhirPathValue;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::sync::Arc;

use super::lambda_metadata::LambdaMetadata;

/// Variable scope for defineVariable isolation with Copy-on-Write semantics
///
/// This structure provides efficient variable scoping by using Copy-on-Write (COW)
/// semantics. Child scopes initially borrow variables from their parent, only
/// creating owned copies when modifications are made.
///
/// # Performance Characteristics
///
/// - **Creation**: O(1) for child scopes (no copying until modification)
/// - **Variable lookup**: O(1) local, O(depth) for parent chain traversal
/// - **Variable setting**: O(1) if already owned, O(n) for COW trigger (first modification)
/// - **Memory usage**: Shared until first modification, then independent
///
/// # COW State Management
///
/// The `owned` field tracks whether this scope owns its variables:
/// - `owned = false`: Variables are borrowed from parent (COW not triggered)
/// - `owned = true`: Variables are owned and can be modified directly
///
/// # Lambda Support
///
/// Lambda scopes automatically inherit critical environment variables from
/// their parent scope to ensure proper context access within lambda expressions.
#[derive(Clone, Debug)]
pub struct VariableScope {
    /// Variables defined in this scope (Copy-on-Write for efficient inheritance)
    /// 
    /// Uses `Cow<'static, FxHashMap<...>>` for zero-copy inheritance from parent
    /// scopes until first modification. The 'static lifetime constraint comes from
    /// COW requirements but is managed safely through the owned flag.
    pub variables: Cow<'static, FxHashMap<String, FhirPathValue>>,

    /// Parent scope (for nested scoping)
    /// 
    /// Uses Arc for efficient sharing of parent scopes across multiple children.
    /// Variable resolution walks up this chain when variables are not found locally.
    pub parent: Option<Arc<VariableScope>>,

    /// Lambda-specific metadata for implicit variables
    /// 
    /// When `Some`, this scope is a lambda context and provides access to implicit
    /// variables like `$this`, `$index`, and `$total` within lambda expressions.
    pub lambda_metadata: Option<LambdaMetadata>,

    /// Whether this scope owns its variables (true if variables were modified)
    /// 
    /// - `false`: Variables are borrowed from parent (no modifications yet)
    /// - `true`: Variables are owned and can be modified directly
    /// 
    /// This flag is critical for COW optimization and determines whether
    /// variable modifications will trigger cloning.
    owned: bool,
}

impl Default for VariableScope {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableScope {
    /// Create a new root scope
    /// 
    /// Root scopes always own their variables and have no parent scope.
    /// This is the starting point for variable scope chains.
    pub fn new() -> Self {
        Self {
            variables: Cow::Owned(FxHashMap::default()),
            parent: None,
            lambda_metadata: None,
            owned: true,
        }
    }

    /// Create a child scope inheriting from parent (zero-copy initially)
    /// 
    /// This method implements the core COW optimization. Child scopes initially
    /// borrow variables from their parent, avoiding expensive copying until
    /// the first modification occurs.
    /// 
    /// # COW Optimization Details
    /// 
    /// - If parent variables are borrowed (`Cow::Borrowed`), child borrows the same reference
    /// - If parent variables are owned (`Cow::Owned`), child creates empty owned map with parent reference
    /// - No variable copying occurs until `set_variable` is called
    /// 
    /// # Arguments
    /// * `parent` - Parent scope to inherit variables from
    /// 
    /// # Performance
    /// O(1) operation - no variable copying occurs during construction
    pub fn child(parent: VariableScope) -> Self {
        Self {
            variables: Cow::Borrowed(match &parent.variables {
                Cow::Borrowed(map) => map,
                Cow::Owned(_map) => {
                    // If parent owns its variables, we need to create a static reference
                    // This is a limitation - we'll create an empty scope and rely on parent traversal
                    return Self {
                        variables: Cow::Owned(FxHashMap::default()),
                        parent: Some(Arc::new(parent)),
                        lambda_metadata: None,
                        owned: false,
                    };
                }
            }),
            parent: Some(Arc::new(parent)),
            lambda_metadata: None,
            owned: false,
        }
    }

    /// Create a child scope from a shared parent (more efficient)
    /// 
    /// This method is more efficient than `child()` when the parent is already
    /// wrapped in an Arc, avoiding the need to create a new Arc.
    /// 
    /// # Arguments
    /// * `parent` - Arc-wrapped parent scope
    /// 
    /// # Performance
    /// O(1) operation with minimal Arc reference counting overhead
    pub fn child_from_shared(parent: Arc<VariableScope>) -> Self {
        Self {
            variables: Cow::Owned(FxHashMap::default()),
            parent: Some(parent),
            lambda_metadata: None,
            owned: false,
        }
    }

    /// Create a lambda scope with implicit variables
    /// 
    /// Lambda scopes are special contexts created for lambda expressions (like `where`, `select`).
    /// They automatically provide implicit variables (`$this`, `$index`, `$total`) and inherit
    /// critical environment variables from their parent scope.
    /// 
    /// # Environment Variable Inheritance
    /// 
    /// Lambda scopes copy these critical environment variables from parent:
    /// - `context` - Original context node (%context)
    /// - `resource` - Containing resource (%resource) 
    /// - `rootResource` - Root container resource (%rootResource)
    /// - `sct` - SNOMED CT URL (%sct)
    /// - `loinc` - LOINC URL (%loinc)
    /// - `ucum` - UCUM URL (%ucum)
    /// 
    /// This ensures lambda expressions maintain access to the broader evaluation context.
    /// 
    /// # Arguments
    /// * `parent` - Optional parent scope to inherit environment variables from
    /// * `current_item` - Current item being processed (becomes `$this`)
    /// * `index` - Zero-based iteration index (becomes `$index`)
    /// * `total` - Total count or accumulator (becomes `$total`)
    pub fn lambda_scope(
        parent: Option<Arc<VariableScope>>,
        current_item: FhirPathValue,
        index: usize,
        total: FhirPathValue,
    ) -> Self {
        let lambda_metadata = LambdaMetadata::new(current_item, index, total);

        // Initialize with inherited environment variables from parent
        let mut variables = FxHashMap::default();

        // Copy critical environment variables from parent scope to ensure they're accessible
        // This fixes the issue where lambda contexts lose access to Bundle context
        if let Some(ref parent_scope) = parent {
            let env_vars = [
                "context",
                "resource",
                "rootResource",
                "sct",
                "loinc",
                "ucum",
            ];
            for var_name in &env_vars {
                if let Some(var_value) = parent_scope.get_variable(var_name) {
                    variables.insert(var_name.to_string(), var_value.clone());
                }
            }
        }

        Self {
            variables: Cow::Owned(variables),
            parent,
            lambda_metadata: Some(lambda_metadata),
            owned: true,
        }
    }

    /// Create a lambda scope with custom parameter mappings
    /// 
    /// This method creates a lambda scope with explicit parameter mappings in addition
    /// to the standard implicit variables. Useful for complex lambda expressions that
    /// need custom variable bindings.
    /// 
    /// # Parameter Override Behavior
    /// 
    /// Parameter mappings can override inherited environment variables if they use
    /// the same name. The resolution order is:
    /// 1. Explicit parameter mappings (highest priority)
    /// 2. Inherited environment variables
    /// 3. Parent scope variables (through normal resolution)
    /// 
    /// # Arguments
    /// * `parent` - Optional parent scope for environment variable inheritance
    /// * `param_mappings` - Vector of (name, value) pairs for explicit parameters
    /// * `current_item` - Current item being processed (becomes `$this`)
    /// * `index` - Zero-based iteration index (becomes `$index`)
    /// * `total` - Total count or accumulator (becomes `$total`)
    pub fn lambda_scope_with_params(
        parent: Option<Arc<VariableScope>>,
        param_mappings: Vec<(String, FhirPathValue)>,
        current_item: FhirPathValue,
        index: usize,
        total: FhirPathValue,
    ) -> Self {
        let mut variables = FxHashMap::default();

        // Copy critical environment variables from parent scope first
        if let Some(ref parent_scope) = parent {
            let env_vars = [
                "context",
                "resource",
                "rootResource",
                "sct",
                "loinc",
                "ucum",
            ];
            for var_name in &env_vars {
                if let Some(var_value) = parent_scope.get_variable(var_name) {
                    variables.insert(var_name.to_string(), var_value.clone());
                }
            }
        }

        // Set explicit parameter mappings (these can override environment variables)
        for (param_name, param_value) in param_mappings {
            variables.insert(param_name, param_value);
        }

        let lambda_metadata = LambdaMetadata::new(current_item, index, total);

        Self {
            variables: Cow::Owned(variables),
            parent,
            lambda_metadata: Some(lambda_metadata),
            owned: true,
        }
    }

    /// Set a variable in the current scope (triggers copy-on-write)
    /// 
    /// This method implements COW semantics and system variable protection.
    /// System variables (those managed by the FHIRPath runtime) cannot be
    /// overridden to maintain specification compliance.
    /// 
    /// # COW Behavior
    /// 
    /// If this scope does not own its variables (`owned = false`):
    /// 1. All current variables are copied to a new owned HashMap
    /// 2. The scope is marked as `owned = true`
    /// 3. The new variable is inserted into the owned HashMap
    /// 
    /// If this scope already owns its variables (`owned = true`):
    /// 1. The variable is inserted directly (O(1) operation)
    /// 
    /// # System Variable Protection
    /// 
    /// Variables starting with certain prefixes or having special names are
    /// protected from user modification:
    /// - Environment variables: `context`, `resource`, `rootResource`, `sct`, `loinc`, `ucum`
    /// - Lambda variables: `$this`, `$index`, `$total`
    /// - Value set variables: `vs-*` patterns
    /// 
    /// Attempts to override system variables are silently ignored per FHIRPath spec.
    /// 
    /// # Arguments
    /// * `name` - Variable name (without $ or % prefixes for system variables)
    /// * `value` - Value to assign to the variable
    /// 
    /// # Performance
    /// - O(1) if scope already owns variables
    /// - O(n) for COW trigger (n = number of variables being inherited)
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        // Prevent overriding system variables according to FHIRPath spec
        if Self::is_system_variable(&name) {
            // Silently ignore attempts to override system variables
            // This matches FHIRPath spec behavior where system variables cannot be overridden
            return;
        }

        self.set_variable_internal(name, value);
    }

    /// Internal method to set variables without system variable protection
    /// 
    /// Used during context initialization to set standard environment variables.
    /// This bypasses the system variable protection to allow the runtime to
    /// establish the initial variable environment.
    /// 
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Variable value
    /// 
    /// # Safety
    /// This method should only be used during context initialization. Using it
    /// inappropriately could allow user code to override system variables.
    pub(crate) fn set_system_variable(&mut self, name: String, value: FhirPathValue) {
        self.set_variable_internal(name, value);
    }

    /// Internal method that actually sets the variable
    /// 
    /// This method implements the core COW logic without any protection checks.
    /// It handles the transition from borrowed to owned variables when needed.
    /// 
    /// # COW Implementation Details
    /// 
    /// 1. Check if variables are currently owned
    /// 2. If not owned, create new owned HashMap and copy existing variables
    /// 3. Mark scope as owned
    /// 4. Insert the new variable into owned HashMap
    /// 
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Variable value
    fn set_variable_internal(&mut self, name: String, value: FhirPathValue) {
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

    /// Check if a variable name is a system variable that cannot be overridden
    /// 
    /// This method identifies variables that are managed by the FHIRPath runtime
    /// and should not be overridden by user code. This maintains specification
    /// compliance and prevents users from breaking the evaluation context.
    /// 
    /// # System Variable Categories
    /// 
    /// - **Environment variables**: `context`, `resource`, `rootResource`, `sct`, `loinc`, `ucum`
    /// - **Lambda variables**: `$this`, `$index`, `$total` (managed by lambda metadata)
    /// - **Value set variables**: Variables matching `vs-*` patterns
    /// - **User $ variables**: Non-lambda $ variables are allowed
    /// 
    /// # Arguments
    /// * `name` - Variable name to check (without prefixes)
    /// 
    /// # Returns
    /// * `true` - If the variable is a protected system variable
    /// * `false` - If the variable can be set by user code
    fn is_system_variable(name: &str) -> bool {
        match name {
            // Standard environment variables (% prefix stripped during parsing)
            "context" | "resource" | "rootResource" | "sct" | "loinc" | "ucum" => true,
            // Lambda variables ($ prefix kept) - these are managed by lambda metadata
            "$this" | "$index" | "$total" => true,
            // Value set variables (without % prefix)
            name if name.starts_with("\"vs-") && name.ends_with('"') => true,
            name if name.starts_with("vs-") => true,
            // System reserved prefixes - but allow user-defined $ variables that aren't lambda-specific
            name if name.starts_with("$") && !matches!(name, "$this" | "$index" | "$total") => {
                false
            }
            _ => false,
        }
    }

    /// Get a variable from this scope or parent scopes
    /// 
    /// This method implements the complete variable resolution algorithm for FHIRPath.
    /// Variables are resolved in a specific order to ensure correct scoping behavior.
    /// 
    /// # Resolution Order
    /// 
    /// 1. **Local variables**: Check variables defined directly in this scope
    /// 2. **Lambda metadata**: Check implicit lambda variables (`$this`, `$index`, `$total`)
    /// 3. **Parent scopes**: Recursively search up the parent scope chain
    /// 4. **Environment variables**: System variables are typically in parent scopes
    /// 
    /// # Arguments
    /// * `name` - Variable name to resolve
    /// 
    /// # Returns
    /// * `Some(&FhirPathValue)` - If variable is found in resolution order
    /// * `None` - If variable is not found in any scope
    /// 
    /// # Performance
    /// - O(1) for local variables and lambda metadata
    /// - O(depth) for parent chain traversal, where depth is the scope nesting level
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        // First check local variables
        if let Some(value) = self.variables.get(name) {
            return Some(value);
        }

        // Then check lambda metadata for implicit variables
        if let Some(ref lambda_meta) = self.lambda_metadata {
            match name {
                "$this" => return Some(&lambda_meta.current_item),
                "$index" => return Some(&lambda_meta.current_index),
                "$total" => return Some(&lambda_meta.total_value),
                _ => {}
            }
        }

        // Then check parent scopes
        self.parent
            .as_ref()
            .and_then(|parent| parent.get_variable(name))
    }

    /// Check if this scope contains a variable locally (not in parent)
    /// 
    /// This method only checks the current scope's variables and lambda metadata,
    /// it does not traverse parent scopes. Useful for debugging and introspection.
    /// 
    /// # Arguments
    /// * `name` - Variable name to check
    /// 
    /// # Returns
    /// * `true` - If variable exists locally in this scope
    /// * `false` - If variable is not in this scope (may exist in parents)
    pub fn contains_local(&self, name: &str) -> bool {
        // Check local variables
        if self.variables.contains_key(name) {
            return true;
        }

        // Check lambda metadata
        if let Some(ref _lambda_meta) = self.lambda_metadata {
            return matches!(name, "$this" | "$index" | "$total");
        }

        false
    }

    /// Get the number of local variables in this scope
    /// 
    /// Returns the count of variables stored directly in this scope.
    /// Does not include lambda metadata variables or parent scope variables.
    /// 
    /// # Returns
    /// Number of variables stored in this scope's HashMap
    pub fn local_count(&self) -> usize {
        self.variables.len()
    }

    /// Check if this scope is efficiently borrowing (COW not triggered)
    /// 
    /// Returns true if this scope is still in the efficient COW state where
    /// variables are borrowed from parent without copying. Once variables
    /// are modified, this returns false.
    /// 
    /// # Returns  
    /// * `true` - If scope is borrowing variables (COW not triggered)
    /// * `false` - If scope owns its variables (COW triggered or root scope)
    pub fn is_efficiently_borrowing(&self) -> bool {
        !self.owned && matches!(self.variables, Cow::Borrowed(_))
    }

    /// Create an optimized scope for simple expressions (pre-allocated capacity)
    /// 
    /// This method creates a root scope with pre-allocated HashMap capacity
    /// for better performance when the expected number of variables is known.
    /// 
    /// # Arguments
    /// * `capacity` - Expected number of variables to pre-allocate
    /// 
    /// # Returns
    /// New root scope with pre-allocated variable storage
    /// 
    /// # Performance
    /// Reduces HashMap reallocations when setting multiple variables
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            variables: Cow::Owned(FxHashMap::with_capacity_and_hasher(
                capacity,
                Default::default(),
            )),
            parent: None,
            lambda_metadata: None,
            owned: true,
        }
    }

    /// Create a scope from an existing variables map
    /// 
    /// This method creates a new root scope containing the provided variables.
    /// Useful for creating independent scopes from collected variable data.
    /// 
    /// # Arguments
    /// * `variables` - HashMap of variables to include in the scope
    /// 
    /// # Returns
    /// New root scope containing the provided variables
    pub fn from_variables_map(variables: FxHashMap<String, FhirPathValue>) -> Self {
        Self {
            variables: Cow::Owned(variables),
            parent: None,
            lambda_metadata: None,
            owned: true,
        }
    }

    /// Get debug information about this scope's COW state
    /// 
    /// Returns a string describing the current state for debugging purposes.
    /// Useful for understanding COW behavior and performance characteristics.
    /// 
    /// # Returns
    /// Debug string describing scope state
    pub fn debug_cow_state(&self) -> String {
        match (&self.variables, self.owned) {
            (Cow::Borrowed(_), false) => "Efficiently borrowing (COW not triggered)".to_string(),
            (Cow::Owned(_), true) => format!("Owns {} variables", self.variables.len()),
            (Cow::Borrowed(_), true) => "Inconsistent state: borrowed but marked owned".to_string(),
            (Cow::Owned(_), false) => "Inconsistent state: owned but marked not owned".to_string(),
        }
    }

    /// Create a debug representation of the variable resolution chain
    /// 
    /// Returns a string showing the scope chain for debugging variable resolution issues.
    /// Shows local variables, lambda metadata, and parent chain structure.
    /// 
    /// # Returns
    /// Multi-line debug string showing scope chain
    pub fn debug_variable_chain(&self) -> String {
        let mut result = String::new();
        self.debug_variable_chain_recursive(&mut result, 0);
        result
    }

    /// Recursive helper for debug_variable_chain
    fn debug_variable_chain_recursive(&self, result: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        
        result.push_str(&format!("{}Scope[{}]:\n", indent, depth));
        result.push_str(&format!("{}  COW State: {}\n", indent, self.debug_cow_state()));
        result.push_str(&format!("{}  Local vars: {:?}\n", indent, self.variables.keys().collect::<Vec<_>>()));
        
        if let Some(ref lambda_meta) = self.lambda_metadata {
            result.push_str(&format!("{}  Lambda vars: $this, $index, $total\n", indent));
            result.push_str(&format!("{}    $index = {}\n", indent, lambda_meta.current_index_as_i64()));
        }
        
        if let Some(ref parent) = self.parent {
            result.push_str(&format!("{}  Parent:\n", indent));
            parent.debug_variable_chain_recursive(result, depth + 1);
        } else {
            result.push_str(&format!("{}  No parent (root scope)\n", indent));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scope_is_owned() {
        let scope = VariableScope::new();
        assert!(scope.owned);
        assert_eq!(scope.local_count(), 0);
        assert!(scope.parent.is_none());
        assert!(scope.lambda_metadata.is_none());
    }

    #[test]
    fn test_child_scope_cow_optimization() {
        let mut parent = VariableScope::new();
        parent.set_variable("test".to_string(), FhirPathValue::String("value".into()));
        
        let child = VariableScope::child(parent);
        assert!(!child.owned); // Child should not own variables initially
        
        // Child should be able to access parent variable
        assert_eq!(
            child.get_variable("test"),
            Some(&FhirPathValue::String("value".into()))
        );
    }

    #[test]
    fn test_cow_trigger_on_set_variable() {
        let mut parent = VariableScope::new();
        parent.set_variable("parent_var".to_string(), FhirPathValue::String("parent_value".into()));
        
        let mut child = VariableScope::child(parent);
        assert!(!child.owned); // Initially not owned
        
        // Setting a variable should trigger COW
        child.set_variable("child_var".to_string(), FhirPathValue::String("child_value".into()));
        assert!(child.owned); // Should now be owned
        
        // Both variables should be accessible
        assert_eq!(
            child.get_variable("parent_var"),
            Some(&FhirPathValue::String("parent_value".into()))
        );
        assert_eq!(
            child.get_variable("child_var"),
            Some(&FhirPathValue::String("child_value".into()))
        );
    }

    #[test]
    fn test_lambda_scope_creation() {
        let current_item = FhirPathValue::String("test_item".into());
        let total = FhirPathValue::Integer(10);
        
        let lambda_scope = VariableScope::lambda_scope(None, current_item.clone(), 5, total.clone());
        
        assert!(lambda_scope.owned);
        assert!(lambda_scope.lambda_metadata.is_some());
        
        // Check lambda variables
        assert_eq!(lambda_scope.get_variable("$this"), Some(&current_item));
        assert_eq!(lambda_scope.get_variable("$index"), Some(&FhirPathValue::Integer(5)));
        assert_eq!(lambda_scope.get_variable("$total"), Some(&total));
    }

    #[test]
    fn test_system_variable_protection() {
        let mut scope = VariableScope::new();
        
        // System variables should be ignored
        scope.set_variable("context".to_string(), FhirPathValue::String("should_be_ignored".into()));
        scope.set_variable("$this".to_string(), FhirPathValue::String("should_be_ignored".into()));
        
        // These should not be set
        assert!(scope.get_variable("context").is_none());
        assert!(scope.get_variable("$this").is_none());
        
        // Non-system variables should work
        scope.set_variable("my_var".to_string(), FhirPathValue::String("allowed".into()));
        assert_eq!(
            scope.get_variable("my_var"),
            Some(&FhirPathValue::String("allowed".into()))
        );
    }

    #[test]
    fn test_environment_variable_inheritance_in_lambda() {
        let mut parent = VariableScope::new();
        parent.set_system_variable("context".to_string(), FhirPathValue::String("parent_context".into()));
        parent.set_system_variable("resource".to_string(), FhirPathValue::String("parent_resource".into()));
        
        let parent_arc = Arc::new(parent);
        let lambda_scope = VariableScope::lambda_scope(
            Some(parent_arc),
            FhirPathValue::String("lambda_item".into()),
            0,
            FhirPathValue::Integer(1),
        );
        
        // Lambda scope should inherit environment variables
        assert_eq!(
            lambda_scope.get_variable("context"),
            Some(&FhirPathValue::String("parent_context".into()))
        );
        assert_eq!(
            lambda_scope.get_variable("resource"),
            Some(&FhirPathValue::String("parent_resource".into()))
        );
    }

    #[test]
    fn test_variable_resolution_order() {
        let mut parent = VariableScope::new();
        parent.set_system_variable("test_var".to_string(), FhirPathValue::String("parent_value".into()));
        
        let parent_arc = Arc::new(parent);
        let mut lambda_scope = VariableScope::lambda_scope(
            Some(parent_arc),
            FhirPathValue::String("lambda_item".into()),
            0,
            FhirPathValue::Integer(1),
        );
        
        // Set local variable with same name
        lambda_scope.set_variable("test_var".to_string(), FhirPathValue::String("local_value".into()));
        
        // Local variable should take precedence
        assert_eq!(
            lambda_scope.get_variable("test_var"),
            Some(&FhirPathValue::String("local_value".into()))
        );
        
        // Lambda variables should work
        assert_eq!(
            lambda_scope.get_variable("$this"),
            Some(&FhirPathValue::String("lambda_item".into()))
        );
    }

    #[test]
    fn test_debug_functionality() {
        let scope = VariableScope::new();
        let debug_state = scope.debug_cow_state();
        assert!(debug_state.contains("Owns"));
        
        let debug_chain = scope.debug_variable_chain();
        assert!(debug_chain.contains("Scope[0]"));
        assert!(debug_chain.contains("No parent"));
    }
}