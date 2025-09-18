//! Evaluation context system with variable management and scoping
//!
//! This module provides an enhanced evaluation context with proper variable scoping and system variables.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::trace::SharedTraceProvider;
use crate::core::{Collection, FhirPathValue, ModelProvider, Result};
use crate::evaluator::terminologies_variable::TerminologiesVariable;
use crate::evaluator::environment_variables::EnvironmentVariables;
use octofhir_fhir_model::TerminologyProvider;

/// Enhanced evaluation context with variable management and scoping
pub struct EvaluationContext {
    /// Input collection being evaluated
    input: Collection,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
    /// Optional terminology provider
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    /// Optional trace provider
    trace_provider: Option<SharedTraceProvider>,
    /// Variable stack for scoped variable management
    variables: VariableStack,
    /// System variables ($this, $index, $total)
    system_variables: SystemVariables,
    /// FHIR environment variables (%sct, %loinc, %vs-[name], %ext-[name], etc.)
    environment_variables: Arc<EnvironmentVariables>,
    /// Root context (outermost input)
    root_context: Option<Collection>,
    /// Parent context for nested evaluations
    parent_context: Option<Box<EvaluationContext>>,
}

/// Variable stack for managing scoped variables
#[derive(Debug, Clone)]
pub struct VariableStack {
    /// Variable scopes (innermost scope is last)
    scopes: Vec<HashMap<String, FhirPathValue>>,
}

/// System variables available in FHIRPath expressions
#[derive(Debug, Clone)]
pub struct SystemVariables {
    /// Current item being evaluated ($this)
    this: Option<FhirPathValue>,
    /// Current index in iteration ($index)
    index: Option<i64>,
    /// Total count in iteration ($total)
    total: Option<i64>,
    /// The resource that contains the original node (%resource)
    resource: Option<FhirPathValue>,
    /// The container resource for the resource (%rootResource)
    root_resource: Option<FhirPathValue>,
}

impl EvaluationContext {
    /// Create new evaluation context
    pub async fn new(
        input: Collection,
        model_provider: Arc<dyn ModelProvider>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
        trace_provider: Option<SharedTraceProvider>,
    ) -> Self {
        Self::new_with_environment(
            input,
            model_provider,
            terminology_provider,
            trace_provider,
            Arc::new(EnvironmentVariables::default()),
        ).await
    }

    /// Create new evaluation context with custom environment variables
    pub async fn new_with_environment(
        input: Collection,
        model_provider: Arc<dyn ModelProvider>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
        trace_provider: Option<SharedTraceProvider>,
        environment_variables: Arc<EnvironmentVariables>,
    ) -> Self {
        // Extract the first resource from input for %resource and %rootResource
        let resource_value = input.iter().next().cloned();
        let system_variables = if let Some(resource) = resource_value {
            SystemVariables::with_resource(resource, None)
        } else {
            SystemVariables::new()
        };

        Self {
            input: input.clone(),
            model_provider,
            terminology_provider,
            trace_provider,
            variables: VariableStack::new(),
            system_variables,
            environment_variables,
            root_context: Some(input),
            parent_context: None,
        }
    }

    /// Create a child context for nested evaluations
    pub fn create_child_context(&self, new_input: Collection) -> Self {
        // Preserve the original %resource and %rootResource variables in child contexts
        let mut system_variables = self.system_variables.clone();
        // Clear iteration variables but keep resource variables
        system_variables.this = None;
        system_variables.index = None;
        system_variables.total = None;

        Self {
            input: new_input,
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            trace_provider: self.trace_provider.clone(),
            variables: self.variables.clone(),
            system_variables,
            environment_variables: self.environment_variables.clone(),
            root_context: self.root_context.clone(),
            parent_context: Some(Box::new(self.clone())),
        }
    }

    /// Create a context with iteration variables for functions like where(), select()
    pub fn create_iteration_context(
        &self,
        current_item: FhirPathValue,
        index: i64,
        total: i64,
    ) -> Self {
        let mut context = self.create_child_context(Collection::single(current_item.clone()));
        context.system_variables.this = Some(current_item);
        context.system_variables.index = Some(index);
        context.system_variables.total = Some(total);
        context
    }

    /// Create a context for evaluating expression parameters while preserving system variables
    pub fn create_expression_context(&self) -> Self {
        let current_this = self.get_variable("$this");

        let mut expression_context = if let Some(ref this_value) = current_this {
            self.create_child_context(Collection::single(this_value.clone()))
        } else {
            self.create_child_context(self.get_root_context().clone())
        };

        if let Some(this_value) = current_this {
            expression_context.set_system_this(this_value);
        }
        if let Some(index) = self.get_system_index() {
            expression_context.set_system_index(index);
        }
        if let Some(total) = self.get_system_total() {
            expression_context.set_system_total(total);
        }

        expression_context
    }

    pub fn get_root_evaluation_context(&self) -> &Collection {
        if let Some(ref root_ctx) = self.root_context {
            root_ctx
        } else {
            &self.input
        }
    }

    pub fn get_model_provider(&self) -> Arc<dyn ModelProvider> {
        self.model_provider.clone()
    }

    pub fn get_terminology_provider(&self) -> Option<Arc<dyn TerminologyProvider>> {
        self.terminology_provider.clone()
    }

    pub fn get_trace_provider(&self) -> Option<SharedTraceProvider> {
        self.trace_provider.clone()
    }


    /// Get root input collection for function argument evaluation
    /// Returns reference to avoid unnecessary cloning
    pub fn get_argument_evaluation_input(&self) -> &Collection {
        self.root_context.as_ref().unwrap_or(&self.input)
    }

    /// Get input collection
    pub fn input_collection(&self) -> &Collection {
        &self.input
    }

    /// Get model provider
    pub fn model_provider(&self) -> Arc<dyn ModelProvider> {
        self.model_provider.clone()
    }

    /// Get terminology provider
    pub fn terminology_provider(&self) -> Option<Arc<dyn TerminologyProvider>> {
        self.terminology_provider.clone()
    }

    /// Check if terminology provider is available
    pub fn has_terminology_provider(&self) -> bool {
        self.terminology_provider.is_some()
    }

    /// Get trace provider
    pub fn trace_provider(&self) -> Option<SharedTraceProvider> {
        self.trace_provider.clone()
    }

    /// Check if trace provider is available
    pub fn has_trace_provider(&self) -> bool {
        self.trace_provider.is_some()
    }


    /// Get root context (outermost input)
    pub fn get_root_context(&self) -> &Collection {
        self.root_context.as_ref().unwrap_or(&self.input)
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Push a new variable scope
    pub fn push_scope(&mut self) {
        self.variables.push_scope();
    }

    /// Pop the current variable scope
    pub fn pop_scope(&mut self) {
        self.variables.pop_scope();
    }

    /// Set a user variable in the current scope
    pub fn set_user_variable(&mut self, name: String, value: FhirPathValue) -> Result<()> {
        self.variables.set_variable(name, value);
        Ok(())
    }

    /// Get a variable value (searches through scopes)
    pub fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        // Check system variables first
        match name {
            "$this" | "%this" => self.system_variables.this.clone(),
            "$index" | "%index" => {
                // Convert index to FhirPathValue
                self.system_variables
                    .index
                    .map(|i| FhirPathValue::integer(i))
            }
            "$total" | "%total" => {
                // Convert total to FhirPathValue
                self.system_variables
                    .total
                    .map(|t| FhirPathValue::integer(t))
            }
            "%resource" => {
                // FHIR-defined variable: the resource that contains the original node
                self.system_variables.resource.clone()
            }
            "%rootResource" => {
                // FHIR-defined variable: the container resource for the resource
                self.system_variables.root_resource.clone()
            }
            "%terminologies" => {
                // Return the terminologies variable if terminology provider is available
                self.terminology_provider.as_ref().map(|provider| {
                    let terminologies_var = TerminologiesVariable::new(provider.clone());
                    terminologies_var.to_fhir_path_value()
                })
            }
            _ => {
                // Check environment variables first, then user variables
                self.environment_variables.get_variable(name)
                    .or_else(|| self.variables.get_variable(name).cloned())
            }
        }
    }

    /// Get system variable values
    pub fn get_system_this(&self) -> Option<&FhirPathValue> {
        self.system_variables.this.as_ref()
    }

    /// Get current iteration index
    pub fn get_system_index(&self) -> Option<i64> {
        self.system_variables.index
    }

    /// Get current iteration total
    pub fn get_system_total(&self) -> Option<i64> {
        self.system_variables.total
    }

    /// Set system $this variable
    pub fn set_system_this(&mut self, value: FhirPathValue) {
        self.system_variables.this = Some(value);
    }

    /// Set system $index variable
    pub fn set_system_index(&mut self, index: i64) {
        self.system_variables.index = Some(index);
    }

    /// Set system $total variable
    pub fn set_system_total(&mut self, total: i64) {
        self.system_variables.total = Some(total);
    }

    /// Create a new context with lambda variables set
    pub fn with_lambda_context(&self, item: &FhirPathValue, index: usize) -> Self {
        let mut new_context = self.clone();

        // Set system variables for lambda context
        new_context.set_system_this(item.clone());
        new_context.set_system_index(index as i64);

        new_context
    }

    /// Get parent context
    pub fn parent_context(&self) -> Option<&EvaluationContext> {
        self.parent_context.as_ref().map(|boxed| boxed.as_ref())
    }

    /// Check if this is a root context (no parent)
    pub fn is_root_context(&self) -> bool {
        self.parent_context.is_none()
    }

    /// Get all variable names in current scope
    pub fn list_variables(&self) -> Vec<String> {
        let mut vars = self.variables.list_variables();

        // Add system variables that are currently set
        if self.system_variables.this.is_some() {
            vars.push("$this".to_string());
            vars.push("%this".to_string());
        }
        if self.system_variables.index.is_some() {
            vars.push("$index".to_string());
            vars.push("%index".to_string());
        }
        if self.system_variables.total.is_some() {
            vars.push("$total".to_string());
            vars.push("%total".to_string());
        }

        // Add FHIR resource variables if available
        if self.system_variables.resource.is_some() {
            vars.push("%resource".to_string());
        }
        if self.system_variables.root_resource.is_some() {
            vars.push("%rootResource".to_string());
        }

        // Add terminologies variable if available
        if self.terminology_provider.is_some() {
            vars.push("%terminologies".to_string());
        }

        // Add environment variables
        vars.extend(self.environment_variables.list_variables());

        vars.sort();
        vars.dedup();
        vars
    }

    /// Create a context for function evaluation with proper variable scoping
    pub fn for_function_evaluation(&self, input: Collection) -> Self {
        let mut context = self.create_child_context(input);
        context.push_scope(); // New scope for function variables
        context
    }
}

impl VariableStack {
    /// Create a new variable stack with global scope
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Start with global scope
        }
    }

    /// Push a new variable scope
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current variable scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
        // Always keep at least the global scope
    }

    /// Set a variable in the current scope
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, value);
        }
    }

    /// Get a variable value (searches from innermost to outermost scope)
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        // Search from innermost scope to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Check if a variable exists in any scope
    pub fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }

    /// Get all variable names from all scopes
    pub fn list_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        for scope in &self.scopes {
            for name in scope.keys() {
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
            }
        }
        vars
    }

    /// Get current scope depth
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }

    /// Clear all variables in current scope
    pub fn clear_current_scope(&mut self) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.clear();
        }
    }
}

impl SystemVariables {
    /// Create new system variables (all unset)
    pub fn new() -> Self {
        Self {
            this: None,
            index: None,
            total: None,
            resource: None,
            root_resource: None,
        }
    }

    /// Create system variables with resource information
    pub fn with_resource(resource: FhirPathValue, root_resource: Option<FhirPathValue>) -> Self {
        Self {
            this: None,
            index: None,
            total: None,
            resource: Some(resource.clone()),
            root_resource: root_resource.or(Some(resource)),
        }
    }

    /// Clear all system variables
    pub fn clear(&mut self) {
        self.this = None;
        self.index = None;
        self.total = None;
        self.resource = None;
        self.root_resource = None;
    }

    /// Check if any system variables are set
    pub fn has_any(&self) -> bool {
        self.this.is_some() || self.index.is_some() || self.total.is_some() ||
        self.resource.is_some() || self.root_resource.is_some()
    }
}

impl Default for VariableStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SystemVariables {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for working with evaluation contexts
pub trait EvaluationContextExt {
    /// Execute a closure with a new variable scope
    fn with_scope<F, R>(&mut self, f: F) -> impl std::future::Future<Output = R> + Send
    where
        F: FnOnce(&mut Self) -> R + Send;

    /// Execute a closure with iteration variables set
    async fn with_iteration<F, R>(
        &mut self,
        item: FhirPathValue,
        index: i64,
        total: i64,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

impl EvaluationContextExt for EvaluationContext {
    async fn with_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R + Send,
    {
        self.push_scope();
        let result = f(self);
        self.pop_scope();
        result
    }

    async fn with_iteration<F, R>(&mut self, item: FhirPathValue, index: i64, total: i64, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let old_this = self.system_variables.this.clone();
        let old_index = self.system_variables.index;
        let old_total = self.system_variables.total;

        self.set_system_this(item);
        self.set_system_index(index);
        self.set_system_total(total);

        let result = f(self);

        // Restore previous values
        self.system_variables.this = old_this;
        self.system_variables.index = old_index;
        self.system_variables.total = old_total;

        result
    }
}

impl Clone for EvaluationContext {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            trace_provider: self.trace_provider.clone(),
            variables: self.variables.clone(),
            system_variables: self.system_variables.clone(),
            environment_variables: self.environment_variables.clone(),
            root_context: self.root_context.clone(),
            parent_context: self.parent_context.clone(),
        }
    }
}

impl std::fmt::Debug for EvaluationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvaluationContext")
            .field("input", &self.input)
            .field("model_provider", &"<ModelProvider>")
            .field("terminology_provider", &self.terminology_provider.is_some())
            .field("trace_provider", &self.trace_provider.is_some())
            .field("variables", &self.variables)
            .field("system_variables", &self.system_variables)
            .field("environment_variables", &self.environment_variables)
            .field("root_context", &self.root_context)
            .field("parent_context", &self.parent_context.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::FhirPathValue;

    #[test]
    fn test_variable_stack() {
        let mut stack = VariableStack::new();

        // Set variable in global scope
        stack.set_variable("x".to_string(), FhirPathValue::integer(1));
        assert!(stack.has_variable("x"));

        // Push new scope
        stack.push_scope();
        stack.set_variable("y".to_string(), FhirPathValue::integer(2));
        assert!(stack.has_variable("x")); // Should find in parent scope
        assert!(stack.has_variable("y")); // Should find in current scope

        // Pop scope
        stack.pop_scope();
        assert!(stack.has_variable("x")); // Should still exist
        assert!(!stack.has_variable("y")); // Should be gone
    }

    #[test]
    fn test_system_variables() {
        let mut vars = SystemVariables::new();
        assert!(!vars.has_any());

        vars.this = Some(FhirPathValue::integer(42));
        assert!(vars.has_any());

        vars.clear();
        assert!(!vars.has_any());
    }
}
