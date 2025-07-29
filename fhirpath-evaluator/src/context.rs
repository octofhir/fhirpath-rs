// Evaluation context for FHIRPath expressions

use fhirpath_model::FhirPathValue;
use fhirpath_registry::{FunctionRegistry, OperatorRegistry};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Variable scope for defineVariable isolation
#[derive(Clone, Debug)]
pub struct VariableScope {
    /// Variables defined in this scope
    pub variables: FxHashMap<String, FhirPathValue>,
    /// Parent scope (for nested scoping)
    pub parent: Option<Box<VariableScope>>,
}

impl VariableScope {
    /// Create a new root scope
    pub fn new() -> Self {
        Self {
            variables: FxHashMap::default(),
            parent: None,
        }
    }

    /// Create a child scope
    pub fn child(parent: VariableScope) -> Self {
        Self {
            variables: FxHashMap::default(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Set a variable in the current scope
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        self.variables.insert(name, value);
    }

    /// Get a variable from this scope or parent scopes
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        self.variables.get(name).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.get_variable(name))
        })
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
        all_variables.extend(self.variables.clone());

        all_variables
    }
}
