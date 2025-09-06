//! Analysis context for tracking scope, variables, and analysis state
//!
//! This module provides context management for static analysis, including
//! scope tracking, variable management, and type information storage.

use crate::analyzer::type_checker::TypeInfo;
use std::collections::HashMap;

/// Analysis context that tracks the current state during static analysis
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Current resource type being analyzed (e.g., "Patient", "Bundle")
    pub resource_type: Option<String>,
    /// Global variables available throughout the expression
    pub variables: HashMap<String, TypeInfo>,
    /// Stack of scopes for nested expressions
    pub scopes: Vec<ScopeInfo>,
    /// Current navigation path (e.g., ["Patient", "name", "given"])
    pub current_path: Vec<String>,
    /// Maximum allowed nesting depth
    pub max_depth: usize,
    /// Current nesting depth
    pub current_depth: usize,
}

/// Information about a scope in the analysis context
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    /// The type of scope (root, function, lambda, etc.)
    pub scope_type: ScopeType,
    /// Variables defined in this scope
    pub variables: HashMap<String, TypeInfo>,
    /// Return type expected from this scope
    pub return_type: Option<TypeInfo>,
    /// Whether this scope allows breaking/continuing
    pub allows_flow_control: bool,
}

/// Different types of scopes in FHIRPath expressions
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    /// Root scope of the expression
    Root,
    /// Function call scope
    Function {
        /// Name of the function
        name: String,
        /// Whether it's an aggregate function
        is_aggregate: bool,
    },
    /// Lambda expression scope
    Lambda {
        /// Parameter name (e.g., "$this", "$item")
        parameter: Option<String>,
    },
    /// Filter expression scope (where clause)
    Filter,
    /// Where clause scope
    Where,
    /// Select clause scope
    Select,
    /// Collection iteration scope
    Collection,
}

impl AnalysisContext {
    /// Create a new analysis context
    pub fn new() -> Self {
        Self {
            resource_type: None,
            variables: HashMap::new(),
            scopes: vec![ScopeInfo {
                scope_type: ScopeType::Root,
                variables: HashMap::new(),
                return_type: None,
                allows_flow_control: false,
            }],
            current_path: Vec::new(),
            max_depth: 50, // Reasonable default
            current_depth: 0,
        }
    }

    /// Create a new analysis context for a specific resource type
    pub fn with_resource_type(mut self, resource_type: String) -> Self {
        self.resource_type = Some(resource_type);
        self
    }

    /// Set the maximum allowed nesting depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self, scope_type: ScopeType) {
        self.scopes.push(ScopeInfo {
            scope_type,
            variables: HashMap::new(),
            return_type: None,
            allows_flow_control: false,
        });
    }

    /// Pop the current scope from the stack
    pub fn pop_scope(&mut self) -> Option<ScopeInfo> {
        if self.scopes.len() > 1 {
            self.scopes.pop()
        } else {
            None // Don't pop the root scope
        }
    }

    /// Get a mutable reference to the current scope
    pub fn current_scope_mut(&mut self) -> &mut ScopeInfo {
        self.scopes.last_mut().unwrap()
    }

    /// Get a reference to the current scope
    pub fn current_scope(&self) -> &ScopeInfo {
        self.scopes.last().unwrap()
    }

    /// Define a variable in the current scope
    pub fn define_variable(&mut self, name: String, type_info: TypeInfo) {
        self.current_scope_mut().variables.insert(name, type_info);
    }

    /// Define a global variable
    pub fn define_global_variable(&mut self, name: String, type_info: TypeInfo) {
        self.variables.insert(name, type_info);
    }

    /// Look up a variable by name, searching from current scope backwards
    pub fn lookup_variable(&self, name: &str) -> Option<&TypeInfo> {
        // Search from current scope backwards
        for scope in self.scopes.iter().rev() {
            if let Some(type_info) = scope.variables.get(name) {
                return Some(type_info);
            }
        }
        // Check global variables
        self.variables.get(name)
    }

    /// Check if a variable exists in any scope
    pub fn has_variable(&self, name: &str) -> bool {
        self.lookup_variable(name).is_some()
    }

    /// Push a path segment for navigation tracking
    pub fn push_path(&mut self, segment: String) {
        self.current_path.push(segment);
        self.current_depth += 1;
    }

    /// Pop the last path segment
    pub fn pop_path(&mut self) -> Option<String> {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
        self.current_path.pop()
    }

    /// Get the current path as a dot-separated string
    pub fn current_path_string(&self) -> String {
        if self.current_path.is_empty() {
            self.resource_type.clone().unwrap_or_default()
        } else {
            match &self.resource_type {
                Some(resource_type) => {
                    format!("{}.{}", resource_type, self.current_path.join("."))
                }
                None => self.current_path.join("."),
            }
        }
    }

    /// Check if we're at maximum nesting depth
    pub fn is_at_max_depth(&self) -> bool {
        self.current_depth >= self.max_depth
    }

    /// Get the current nesting depth
    pub fn depth(&self) -> usize {
        self.current_depth
    }

    /// Check if we're in a lambda scope
    pub fn is_in_lambda(&self) -> bool {
        self.scopes
            .iter()
            .any(|scope| matches!(scope.scope_type, ScopeType::Lambda { .. }))
    }

    /// Check if we're in a function scope
    pub fn is_in_function(&self) -> bool {
        self.scopes
            .iter()
            .any(|scope| matches!(scope.scope_type, ScopeType::Function { .. }))
    }

    /// Check if we're in an aggregate function scope
    pub fn is_in_aggregate_function(&self) -> bool {
        self.scopes.iter().any(|scope| {
            matches!(
                scope.scope_type,
                ScopeType::Function {
                    is_aggregate: true,
                    ..
                }
            )
        })
    }

    /// Get all variables in the current context (including all scopes)
    pub fn all_variables(&self) -> HashMap<String, &TypeInfo> {
        let mut all_vars = HashMap::new();

        // Add global variables
        for (name, type_info) in &self.variables {
            all_vars.insert(name.clone(), type_info);
        }

        // Add variables from all scopes (current scope overrides outer scopes)
        for scope in &self.scopes {
            for (name, type_info) in &scope.variables {
                all_vars.insert(name.clone(), type_info);
            }
        }

        all_vars
    }

    /// Set the return type for the current scope
    pub fn set_current_return_type(&mut self, return_type: TypeInfo) {
        self.current_scope_mut().return_type = Some(return_type);
    }

    /// Get the expected return type for the current scope
    pub fn current_return_type(&self) -> Option<&TypeInfo> {
        self.current_scope().return_type.as_ref()
    }

    /// Create a child context for nested analysis
    pub fn create_child(&self) -> Self {
        let mut child = self.clone();
        child.current_depth = 0;
        child.current_path.clear();
        child
    }

    /// Get scope depth (number of scopes on the stack)
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }

    /// Check if flow control operations (like break/continue) are allowed
    pub fn allows_flow_control(&self) -> bool {
        self.scopes.iter().any(|scope| scope.allows_flow_control)
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeInfo {
    /// Create a new function scope
    pub fn function(name: String, is_aggregate: bool) -> Self {
        Self {
            scope_type: ScopeType::Function { name, is_aggregate },
            variables: HashMap::new(),
            return_type: None,
            allows_flow_control: false,
        }
    }

    /// Create a new lambda scope
    pub fn lambda(parameter: Option<String>) -> Self {
        Self {
            scope_type: ScopeType::Lambda { parameter },
            variables: HashMap::new(),
            return_type: None,
            allows_flow_control: false,
        }
    }

    /// Create a new filter scope
    pub fn filter() -> Self {
        Self {
            scope_type: ScopeType::Filter,
            variables: HashMap::new(),
            return_type: None,
            allows_flow_control: false,
        }
    }

    /// Create a new where scope
    pub fn where_scope() -> Self {
        Self {
            scope_type: ScopeType::Where,
            variables: HashMap::new(),
            return_type: None,
            allows_flow_control: false,
        }
    }

    /// Create a new select scope
    pub fn select() -> Self {
        Self {
            scope_type: ScopeType::Select,
            variables: HashMap::new(),
            return_type: None,
            allows_flow_control: false,
        }
    }

    /// Create a new collection scope
    pub fn collection() -> Self {
        Self {
            scope_type: ScopeType::Collection,
            variables: HashMap::new(),
            return_type: None,
            allows_flow_control: true, // Collections allow flow control
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::type_checker::TypeInfo;

    #[test]
    fn test_context_creation() {
        let ctx = AnalysisContext::new();
        assert!(ctx.resource_type.is_none());
        assert_eq!(ctx.scopes.len(), 1);
        assert!(matches!(ctx.scopes[0].scope_type, ScopeType::Root));
        assert_eq!(ctx.current_depth, 0);
    }

    #[test]
    fn test_resource_type_context() {
        let ctx = AnalysisContext::new().with_resource_type("Patient".to_string());
        assert_eq!(ctx.resource_type.as_ref(), Some(&"Patient".to_string()));
        assert_eq!(ctx.current_path_string(), "Patient");
    }

    #[test]
    fn test_scope_management() {
        let mut ctx = AnalysisContext::new();

        ctx.push_scope(ScopeType::Function {
            name: "where".to_string(),
            is_aggregate: false,
        });

        assert_eq!(ctx.scopes.len(), 2);
        assert!(matches!(
            ctx.current_scope().scope_type,
            ScopeType::Function { ref name, .. } if name == "where"
        ));

        let popped = ctx.pop_scope();
        assert!(popped.is_some());
        assert_eq!(ctx.scopes.len(), 1);
        assert!(matches!(ctx.current_scope().scope_type, ScopeType::Root));

        // Can't pop the root scope
        let root_pop = ctx.pop_scope();
        assert!(root_pop.is_none());
        assert_eq!(ctx.scopes.len(), 1);
    }

    #[test]
    fn test_variable_management() {
        let mut ctx = AnalysisContext::new();

        // Define global variable
        ctx.define_global_variable("global".to_string(), TypeInfo::String);

        // Define variable in root scope
        ctx.define_variable("local".to_string(), TypeInfo::Integer);

        // Create function scope and define variable
        ctx.push_scope(ScopeType::Function {
            name: "select".to_string(),
            is_aggregate: false,
        });
        ctx.define_variable("function_var".to_string(), TypeInfo::Boolean);

        // Test lookups
        assert!(ctx.has_variable("global"));
        assert!(ctx.has_variable("local"));
        assert!(ctx.has_variable("function_var"));
        assert!(!ctx.has_variable("nonexistent"));

        // Function scope variable should shadow global
        ctx.define_variable("global".to_string(), TypeInfo::Date);
        if let Some(var_type) = ctx.lookup_variable("global") {
            assert!(matches!(var_type, TypeInfo::Date));
        }

        // Pop scope and check shadowing is gone
        ctx.pop_scope();
        if let Some(var_type) = ctx.lookup_variable("global") {
            assert!(matches!(var_type, TypeInfo::String));
        }
        assert!(!ctx.has_variable("function_var"));
    }

    #[test]
    fn test_path_management() {
        let mut ctx = AnalysisContext::new().with_resource_type("Patient".to_string());

        assert_eq!(ctx.current_path_string(), "Patient");
        assert_eq!(ctx.depth(), 0);

        ctx.push_path("name".to_string());
        assert_eq!(ctx.current_path_string(), "Patient.name");
        assert_eq!(ctx.depth(), 1);

        ctx.push_path("given".to_string());
        assert_eq!(ctx.current_path_string(), "Patient.name.given");
        assert_eq!(ctx.depth(), 2);

        let popped = ctx.pop_path();
        assert_eq!(popped, Some("given".to_string()));
        assert_eq!(ctx.current_path_string(), "Patient.name");
        assert_eq!(ctx.depth(), 1);
    }

    #[test]
    fn test_depth_limits() {
        let mut ctx = AnalysisContext::new().with_max_depth(3);

        ctx.push_path("level1".to_string());
        ctx.push_path("level2".to_string());
        ctx.push_path("level3".to_string());

        assert!(ctx.is_at_max_depth());

        ctx.push_path("level4".to_string()); // Should still work but exceed limit
        assert_eq!(ctx.depth(), 4);
        assert!(ctx.is_at_max_depth());
    }

    #[test]
    fn test_scope_type_checks() {
        let mut ctx = AnalysisContext::new();

        assert!(!ctx.is_in_lambda());
        assert!(!ctx.is_in_function());
        assert!(!ctx.is_in_aggregate_function());

        ctx.push_scope(ScopeType::Lambda {
            parameter: Some("$this".to_string()),
        });
        assert!(ctx.is_in_lambda());
        assert!(!ctx.is_in_function());

        ctx.push_scope(ScopeType::Function {
            name: "count".to_string(),
            is_aggregate: true,
        });
        assert!(ctx.is_in_lambda());
        assert!(ctx.is_in_function());
        assert!(ctx.is_in_aggregate_function());
    }
}
