// Evaluation context for FHIRPath expressions

use fhirpath_model::FhirPathValue;
use fhirpath_registry::{FunctionRegistry, OperatorRegistry};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Context for evaluating FHIRPath expressions
#[derive(Clone)]
pub struct EvaluationContext {
    /// Current input value being evaluated
    pub input: FhirPathValue,

    /// Root input value (for %context and $resource variables)
    pub root: FhirPathValue,

    /// Variable bindings (for $this, $index, etc.)
    pub variables: FxHashMap<String, FhirPathValue>,

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
            variables: FxHashMap::default(),
            functions,
            operators,
        }
    }

    /// Create a child context with new input value
    pub fn with_input(&self, input: FhirPathValue) -> Self {
        Self {
            input,
            root: self.root.clone(),
            variables: self.variables.clone(),
            functions: self.functions.clone(),
            operators: self.operators.clone(),
        }
    }

    /// Set a variable in the context
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        self.variables.insert(name, value);
    }

    /// Get a variable from the context
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        self.variables.get(name)
    }
}
