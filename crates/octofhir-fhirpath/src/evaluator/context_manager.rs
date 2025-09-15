//! Prototype-based context management for O(1) context operations
//!
//! This module implements a prototype-based inheritance system for evaluation contexts,
//! eliminating the O(n) HashMap cloning overhead and providing proper variable scoping
//! for lambda functions with system variables ($this, $index, $total).

use crate::core::{
    error::{FhirPathError, Result},
    error_code::*,
    model_provider::ModelProvider,
    Collection, FhirPathValue,
};
use octofhir_fhir_model::terminology::TerminologyProvider;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Prototype-based context manager with O(1) inheritance
#[derive(Debug, Clone)]
pub struct ContextManager {
    /// Parent context for prototype chain (O(1) inheritance)
    parent: Option<Arc<ContextManager>>,
    /// User-defined variables (%, defineVariable)
    variables: HashMap<String, FhirPathValue>,
    /// System variables ($this, $index, $total)
    system_variables: HashMap<String, FhirPathValue>,
    /// Built-in environment variables (%sct, %loinc, etc.)
    environment_variables: HashMap<String, FhirPathValue>,
}

impl ContextManager {
    /// Create new root context with input collection
    pub fn create(input: Collection) -> Self {
        let mut context = Self {
            parent: None,
            variables: HashMap::new(),
            system_variables: HashMap::new(),
            environment_variables: Self::default_environment_variables(),
        };

        // Set %context to input
        let context_value = if input.is_empty() {
            FhirPathValue::Empty
        } else if input.len() == 1 {
            input.first().unwrap().clone()
        } else {
            FhirPathValue::Collection(input)
        };

        context.environment_variables.insert(
            "%context".to_string(),
            context_value.clone(),
        );

        // Set %resource and %rootResource to same as %context initially
        context.environment_variables.insert(
            "%resource".to_string(),
            context_value.clone(),
        );
        context.environment_variables.insert(
            "%rootResource".to_string(),
            context_value,
        );

        context
    }

    /// Create child context with O(1) prototype inheritance
    pub fn create_child(&self) -> Self {
        Self {
            parent: Some(Arc::new(self.clone())),
            variables: HashMap::new(),
            system_variables: HashMap::new(),
            environment_variables: HashMap::new(), // Inherit from parent
        }
    }

    /// Create new context manager with added variable
    pub fn new_with_variable(parent: Arc<ContextManager>, name: String, value: FhirPathValue) -> Self {
        let mut new_manager = Self {
            parent: Some(parent),
            variables: HashMap::new(),
            system_variables: HashMap::new(),
            environment_variables: HashMap::new(),
        };

        // Set the variable (ignoring redefinition for now)
        let var_name = if name.starts_with('%') { name } else { format!("%{}", name) };
        new_manager.variables.insert(var_name, value);

        new_manager
    }

    /// Create iterator context for lambda functions
    pub fn create_iterator_context(&self, item: FhirPathValue, index: usize, total: usize) -> Self {
        let mut child = self.create_child();

        // Set lambda variables with proper scoping
        child.system_variables.insert("$this".to_string(), item);
        child.system_variables.insert("$index".to_string(), FhirPathValue::integer(index as i64));
        child.system_variables.insert("$total".to_string(), FhirPathValue::integer(total as i64));

        child
    }

    /// Set user variable with redefinition protection
    pub fn set_user_variable(&mut self, name: String, value: FhirPathValue) -> Result<()> {
        // Add % prefix if not present
        let var_name = if name.starts_with('%') {
            name
        } else {
            format!("%{}", name)
        };

        // Check for redefinition (FHIRPath spec §1.5.10.3)
        if self.has_user_variable(&var_name) {
            return Err(FhirPathError::evaluation_error(
                FP0152,
                format!("Variable '{}' is already defined", var_name),
            ));
        }

        self.variables.insert(var_name, value);
        Ok(())
    }

    /// Set system variable (internal use for $this, $index, $total)
    pub fn set_system_variable(&mut self, name: String, value: FhirPathValue) {
        self.system_variables.insert(name, value);
    }

    /// Set environment variable (%sct, %loinc, custom environment variables)
    pub fn set_environment_variable(&mut self, name: String, value: FhirPathValue) {
        let var_name = if name.starts_with('%') {
            name
        } else {
            format!("%{}", name)
        };
        self.environment_variables.insert(var_name, value);
    }

    /// Get variable with prototype chain lookup
    pub fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        // Check system variables first ($this, $index, $total)
        if let Some(value) = self.system_variables.get(name) {
            return Some(value.clone());
        }

        // If name doesn't start with $, also check with $ prefix for system variables
        // This handles the case where parser strips $ but system vars are stored with $
        if !name.starts_with('$') {
            let system_name = format!("${}", name);
            if let Some(value) = self.system_variables.get(&system_name) {
                return Some(value.clone());
            }
        }

        // Check user variables (%)
        let user_name = if name.starts_with('%') {
            name.to_string()
        } else {
            format!("%{}", name)
        };

        if let Some(value) = self.variables.get(&user_name) {
            return Some(value.clone());
        }

        // Check environment variables (%context, %sct, %loinc, etc.)
        if let Some(value) = self.environment_variables.get(name) {
            return Some(value.clone());
        }

        let env_name = if name.starts_with('%') {
            name.to_string()
        } else {
            format!("%{}", name)
        };

        if let Some(value) = self.environment_variables.get(&env_name) {
            return Some(value.clone());
        }

        // Try dynamic variable resolution for patterns like %vs-name and %ext-name
        if let Some(dynamic_value) = self.resolve_dynamic_variable(name) {
            return Some(dynamic_value);
        }

        // Check parent context via prototype chain
        if let Some(parent) = &self.parent {
            return parent.get_variable(name);
        }

        None
    }

    /// Check if user variable exists (for redefinition protection)
    pub fn has_user_variable(&self, name: &str) -> bool {
        let var_name = if name.starts_with('%') {
            name.to_string()
        } else {
            format!("%{}", name)
        };

        // Check current context
        if self.variables.contains_key(&var_name) {
            return true;
        }

        // Check prototype chain
        if let Some(parent) = &self.parent {
            return parent.has_user_variable(name);
        }

        false
    }

    /// Check if system variable exists in current context
    pub fn has_system_variable(&self, name: &str) -> bool {
        self.system_variables.contains_key(name)
    }

    /// Get all variable names accessible in current context (for debugging)
    pub fn get_all_variable_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        // Add system variables
        names.extend(self.system_variables.keys().cloned());

        // Add user variables  
        names.extend(self.variables.keys().cloned());

        // Add environment variables
        names.extend(self.environment_variables.keys().cloned());

        // Add parent variables via prototype chain
        if let Some(parent) = &self.parent {
            let parent_names = parent.get_all_variable_names();
            for name in parent_names {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }

        names.sort();
        names
    }

    /// Resolve dynamic variable patterns like %vs-name and %ext-name
    /// 
    /// This provides the core dynamic variable functionality that allows FHIRPath
    /// expressions to use variables like %vs-administrative-gender or %ext-birthPlace
    /// without having to predefine every possible ValueSet or StructureDefinition URL.
    fn resolve_dynamic_variable(&self, var_name: &str) -> Option<FhirPathValue> {
        // Normalize variable name (remove % prefix if present)
        let clean_name = if var_name.starts_with('%') {
            &var_name[1..]
        } else {
            var_name
        };

        if clean_name.starts_with("vs-") {
            // Extract the part after "vs-" and construct ValueSet URL
            let valueset_name = &clean_name[3..]; // Remove "vs-" prefix
            let url = format!("http://hl7.org/fhir/ValueSet/{}", valueset_name);
            Some(FhirPathValue::string(url))
        } else if clean_name.starts_with("ext-") {
            // Extract the part after "ext-" and construct StructureDefinition URL  
            let extension_name = &clean_name[4..]; // Remove "ext-" prefix
            let url = format!("http://hl7.org/fhir/StructureDefinition/{}", extension_name);
            Some(FhirPathValue::string(url))
        } else {
            None
        }
    }

    /// Create default environment variables
    fn default_environment_variables() -> HashMap<String, FhirPathValue> {
        let mut env = HashMap::new();

        // Standard FHIR terminology URLs
        env.insert("%sct".to_string(), FhirPathValue::string("http://snomed.info/sct"));
        env.insert("%loinc".to_string(), FhirPathValue::string("http://loinc.org"));
        env.insert("%ucum".to_string(), FhirPathValue::string("http://unitsofmeasure.org"));
        
        // Note: We don't need to predefine %vs-* and %ext-* variables since they're handled dynamically
        // Examples of what dynamic resolution provides:
        // %vs-administrative-gender → http://hl7.org/fhir/ValueSet/administrative-gender
        // %ext-birthPlace → http://hl7.org/fhir/StructureDefinition/birthPlace
        
        // Provide a lightweight %terminologies handle so functions can be invoked even without a live service
        env.insert(
            "%terminologies".to_string(),
            FhirPathValue::TypeInfoObject { namespace: "FHIR".to_string(), name: "terminologies".to_string() }
        );

        env
    }
}

/// Evaluation context with prototype-based context management
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current focus collection for evaluation
    focus: Collection,
    /// Root context (initial input, never changes)
    root_context: Collection,
    /// Context manager for variable handling
    context_manager: ContextManager,
    /// Shared user variables by scope id. Each scope holds its own variables.
    shared_user_variables: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<u64, std::collections::HashMap<String, FhirPathValue>>>>,
    /// Current scope id for side-effect variable definitions
    scope_id: u64,
    /// Visible scope ids for lookups (from outer-most to inner-most)
    visible_scopes: Vec<u64>,
    /// Shared counter for generating fresh scope ids across clones
    scope_seq: std::sync::Arc<AtomicU64>,
    /// Model provider reference
    model_provider: Arc<dyn ModelProvider>,
    /// Optional terminology provider
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    /// Evaluation depth for recursion protection
    depth: usize,
    /// Track whether current evaluation is within FHIR navigation context
    is_fhir_navigation: bool,
}

impl EvaluationContext {
    /// Create new evaluation context (async-first)
    pub async fn new(
        input: Collection,
        model_provider: Arc<dyn ModelProvider>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    ) -> Self {
        let context_manager = ContextManager::create(input.clone());

        Self {
            focus: input.clone(),
            root_context: input,
            context_manager,
            shared_user_variables: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            scope_id: 0,
            visible_scopes: vec![0],
            scope_seq: std::sync::Arc::new(AtomicU64::new(1)),
            model_provider,
            terminology_provider,
            depth: 0,
            is_fhir_navigation: false,
        }
    }

    /// Create child context for nested evaluation
    pub fn create_child(&self, new_focus: Collection) -> Self {
        Self {
            focus: new_focus,
            root_context: self.root_context.clone(),
            context_manager: self.context_manager.create_child(),
            shared_user_variables: self.shared_user_variables.clone(),
            scope_id: self.scope_id,
            visible_scopes: self.visible_scopes.clone(),
            scope_seq: self.scope_seq.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            depth: self.depth + 1,
            is_fhir_navigation: self.is_fhir_navigation,
        }
    }

    /// Create iterator context for lambda functions
    pub fn create_iterator_context(&self, item: FhirPathValue, index: usize) -> Self {
        let total = self.focus.len();

        Self {
            focus: Collection::single(item.clone()),
            root_context: self.root_context.clone(),
            context_manager: self.context_manager.create_iterator_context(item, index, total),
            shared_user_variables: self.shared_user_variables.clone(),
            scope_id: self.scope_id,
            visible_scopes: self.visible_scopes.clone(),
            scope_seq: self.scope_seq.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            depth: self.depth + 1,
            is_fhir_navigation: self.is_fhir_navigation,
        }
    }

    /// Create context with FHIR navigation enabled
    pub fn with_fhir_navigation(&self) -> Self {
        let mut new_context = self.clone();
        new_context.is_fhir_navigation = true;
        new_context
    }

    /// Set user-defined variable
    pub fn set_user_variable(&mut self, name: String, value: FhirPathValue) -> Result<()> {
        self.context_manager.set_user_variable(name, value)
    }

    /// Get variable by name
    pub fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        // 1) System/user/env via context manager
        if let Some(v) = self.context_manager.get_variable(name) { return Some(v); }
        // 2) Shared variables across visible scopes
        let key = if name.starts_with('%') { name.to_string() } else { format!("%{}", name) };
        if let Ok(all_scopes) = self.shared_user_variables.read() {
            for sid in &self.visible_scopes {
                if let Some(scope_map) = all_scopes.get(sid) {
                    if let Some(v) = scope_map.get(&key) { return Some(v.clone()); }
                }
            }
        }
        None
    }

    /// Define/set variable value - supports multiple variables
    pub fn define_variable(&mut self, name: &str, value: FhirPathValue) -> Result<()> {
        // Use the context manager's built-in variable setting
        self.context_manager.set_user_variable(name.to_string(), value)
    }

    /// Set variable value (convenience method)
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) -> Result<()> {
        self.define_variable(&name, value)
    }

    /// INTERNAL: set a system variable (e.g., "$this", "$index", "$total", "$acc") for lambda evaluation.
    pub fn set_system_variable_internal(&mut self, name: &str, value: FhirPathValue) {
        let var_name = if name.starts_with('$') { name.to_string() } else { format!("${}", name) };
        self.context_manager.set_system_variable(var_name, value);
    }

    /// Add multiple variables at once
    pub fn add_variables(&mut self, variables: std::collections::HashMap<String, FhirPathValue>) -> Result<()> {
        for (name, value) in variables {
            self.define_variable(&name, value)?;
        }
        Ok(())
    }

    /// Get current focus collection
    pub fn get_focus(&self) -> &Collection {
        &self.focus
    }

    /// Get root context collection
    pub fn get_root_context(&self) -> &Collection {
        &self.root_context
    }

    /// Get model provider
    pub fn get_model_provider(&self) -> &Arc<dyn ModelProvider> {
        &self.model_provider
    }

    /// Get terminology provider
    pub fn get_terminology_provider(&self) -> Option<&Arc<dyn TerminologyProvider>> {
        self.terminology_provider.as_ref()
    }

    /// Check if model provider is available
    pub fn has_model_provider(&self) -> bool {
        true // Always available
    }

    /// Check if terminology provider is available
    pub fn has_terminology_provider(&self) -> bool {
        self.terminology_provider.is_some()
    }

    /// Get terminology provider or return error if not available
    pub fn require_terminology_provider(&self) -> Result<&Arc<dyn TerminologyProvider>> {
        self.terminology_provider.as_ref()
            .ok_or_else(|| FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "Function requires terminology provider but none configured".to_string(),
            ))
    }

    /// Check evaluation depth for recursion protection
    pub fn check_depth(&self) -> Result<()> {
        const MAX_DEPTH: usize = 1000;
        if self.depth > MAX_DEPTH {
            Err(FhirPathError::evaluation_error(
                FP0153,
                "Maximum evaluation depth exceeded".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Get current evaluation depth
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    /// Check if current context is within FHIR navigation
    pub fn is_fhir_navigation(&self) -> bool {
        self.is_fhir_navigation
    }

    /// Create isolated context (for union operations where variables should not cross sides)
    pub fn create_isolated_context(&self) -> Self {
        // Create new root context manager (no prototype inheritance)
        let context_manager = ContextManager::create(self.focus.clone());

        Self {
            focus: self.focus.clone(),
            root_context: self.root_context.clone(),
            context_manager,
            // Keep shared structures but reset scope to current to maintain isolation semantics
            shared_user_variables: self.shared_user_variables.clone(),
            scope_id: self.scope_id,
            visible_scopes: self.visible_scopes.clone(),
            scope_seq: self.scope_seq.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            depth: self.depth,
            is_fhir_navigation: self.is_fhir_navigation,
        }
    }

    /// Get all accessible variable names (for debugging/introspection)
    pub fn get_all_variable_names(&self) -> Vec<String> {
        self.context_manager.get_all_variable_names()
    }

    /// Update %context to new value (used during property navigation)
    pub fn update_context(&mut self, new_context: FhirPathValue) {
        self.context_manager.set_environment_variable("%context".to_string(), new_context);
    }

    /// Define a variable as a side-effect in the shared scope for the current evaluation.
    /// This is used by defineVariable() to make variables visible to subsequent sibling nodes
    /// such as the right-hand side of a union expression.
    pub fn define_side_effect_variable(&self, name: &str, value: FhirPathValue) -> Result<()> {
        let key = if name.starts_with('%') { name.to_string() } else { format!("%{}", name) };

        // Disallow overwriting of system/environment variables
        const RESERVED: [&str; 7] = [
            "%context", "%resource", "%rootResource", "%terminologies", "%sct", "%loinc", "%ucum",
        ];
        if RESERVED.contains(&key.as_str()) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0152,
                format!("Variable '{}' is reserved and cannot be redefined", key),
            ));
        }

        // Enforce redefinition protection across both scoped and shared stores
        if self.context_manager.has_user_variable(&key) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0152,
                format!("Variable '{}' is already defined", key),
            ));
        }
        if let Ok(scopes) = self.shared_user_variables.read() {
            // Check redefinition in any visible scope
            for sid in &self.visible_scopes {
                if let Some(scope_map) = scopes.get(sid) {
                    if scope_map.contains_key(&key) {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0152,
                            format!("Variable '{}' is already defined", key),
                        ));
                    }
                }
            }
        }

        if let Ok(mut scopes) = self.shared_user_variables.write() {
            let entry = scopes.entry(self.scope_id).or_insert_with(std::collections::HashMap::new);
            entry.insert(key, value);
        }
        Ok(())
    }

    /// Create a context with a fresh child scope id (used to isolate union branches)
    pub fn with_new_child_scope(&self) -> Self {
        let new_scope_id = self.scope_seq.fetch_add(1, Ordering::SeqCst);
        let mut new_visible = self.visible_scopes.clone();
        new_visible.push(new_scope_id);

        Self {
            focus: self.focus.clone(),
            root_context: self.root_context.clone(),
            context_manager: self.context_manager.clone(),
            shared_user_variables: self.shared_user_variables.clone(),
            scope_id: new_scope_id,
            visible_scopes: new_visible,
            scope_seq: self.scope_seq.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            depth: self.depth,
            is_fhir_navigation: self.is_fhir_navigation,
        }
    }
}

/// Builder for creating evaluation contexts with fluent API
pub struct EvaluationContextBuilder {
    input: Option<Collection>,
    model_provider: Option<Arc<dyn ModelProvider>>,
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    variables: HashMap<String, FhirPathValue>,
    environment_variables: HashMap<String, FhirPathValue>,
}

impl EvaluationContextBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            input: None,
            model_provider: None,
            terminology_provider: None,
            variables: HashMap::new(),
            environment_variables: HashMap::new(),
        }
    }

    /// Set input collection
    pub fn with_input(mut self, input: Collection) -> Self {
        self.input = Some(input);
        self
    }

    /// Set model provider
    pub fn with_model_provider(mut self, provider: Arc<dyn ModelProvider>) -> Self {
        self.model_provider = Some(provider);
        self
    }

    /// Set terminology provider
    pub fn with_terminology_provider(mut self, provider: Arc<dyn TerminologyProvider>) -> Self {
        self.terminology_provider = Some(provider);
        self
    }

    /// Add user variable
    pub fn with_variable(mut self, name: String, value: FhirPathValue) -> Self {
        self.variables.insert(name, value);
        self
    }

    /// Add multiple user variables
    pub fn with_variables(mut self, variables: HashMap<String, FhirPathValue>) -> Self {
        self.variables.extend(variables);
        self
    }

    /// Add environment variable
    pub fn with_environment_variable(mut self, name: String, value: FhirPathValue) -> Self {
        self.environment_variables.insert(name, value);
        self
    }

    /// Build evaluation context (async)
    pub async fn build(self) -> Result<EvaluationContext> {
        let input = self.input.unwrap_or_else(Collection::empty);
        let model_provider = self.model_provider.ok_or_else(|| {
            FhirPathError::evaluation_error(
                FP0154,
                "ModelProvider is required for EvaluationContext".to_string(),
            )
        })?;

        let mut context = EvaluationContext::new(input, model_provider, self.terminology_provider).await;

        // Add user variables
        for (name, value) in self.variables {
            context.set_user_variable(name, value)?;
        }

        // Add environment variables
        for (name, value) in self.environment_variables {
            context.context_manager.set_environment_variable(name, value);
        }

        Ok(context)
    }
}

impl Default for EvaluationContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model_provider::MockModelProvider;

    #[test]
    fn test_prototype_context_inheritance() {
        let mut parent = ContextManager::create(Collection::empty());
        parent.set_user_variable("parentVar".to_string(), FhirPathValue::string("parent")).unwrap();

        let child = parent.create_child();

        // Child should inherit parent variable
        assert_eq!(child.get_variable("parentVar"), Some(FhirPathValue::string("parent")));

        // Child should be able to set its own variables
        let mut child = child;
        child.set_user_variable("childVar".to_string(), FhirPathValue::string("child")).unwrap();

        // Parent should not see child variable
        assert_eq!(parent.get_variable("childVar"), None);
    }

    #[test]
    fn test_iterator_context_isolation() {
        let parent = ContextManager::create(Collection::empty());

        let iter1 = parent.create_iterator_context(FhirPathValue::string("item1"), 0, 2);
        let iter2 = parent.create_iterator_context(FhirPathValue::string("item2"), 1, 2);

        // Each iterator should have isolated system variables
        assert_eq!(iter1.get_variable("$this"), Some(FhirPathValue::string("item1")));
        assert_eq!(iter1.get_variable("$index"), Some(FhirPathValue::integer(0)));
        assert_eq!(iter1.get_variable("$total"), Some(FhirPathValue::integer(2)));

        assert_eq!(iter2.get_variable("$this"), Some(FhirPathValue::string("item2")));
        assert_eq!(iter2.get_variable("$index"), Some(FhirPathValue::integer(1)));
        assert_eq!(iter2.get_variable("$total"), Some(FhirPathValue::integer(2)));
    }

    #[test]
    fn test_variable_redefinition_protection() {
        let mut context = ContextManager::create(Collection::empty());

        // First definition should succeed
        context.set_user_variable("myVar".to_string(), FhirPathValue::string("first")).unwrap();

        // Second definition should fail
        let result = context.set_user_variable("myVar".to_string(), FhirPathValue::string("second"));
        assert!(result.is_err());
    }

    #[test]
    fn test_environment_variables() {
        let context = ContextManager::create(Collection::empty());

        // Should have default environment variables
        assert_eq!(context.get_variable("%sct"), Some(FhirPathValue::string("http://snomed.info/sct")));
        assert_eq!(context.get_variable("%loinc"), Some(FhirPathValue::string("http://loinc.org")));
        assert_eq!(context.get_variable("%ucum"), Some(FhirPathValue::string("http://unitsofmeasure.org")));
    }

    #[test]
    fn test_dynamic_variable_resolution() {
        let context = ContextManager::create(Collection::empty());

        // Test ValueSet dynamic variables (%vs-*)
        assert_eq!(
            context.get_variable("%vs-administrative-gender"),
            Some(FhirPathValue::string("http://hl7.org/fhir/ValueSet/administrative-gender"))
        );
        assert_eq!(
            context.get_variable("%vs-allergyintolerance-code"),
            Some(FhirPathValue::string("http://hl7.org/fhir/ValueSet/allergyintolerance-code"))
        );
        assert_eq!(
            context.get_variable("vs-custom-valueset"),
            Some(FhirPathValue::string("http://hl7.org/fhir/ValueSet/custom-valueset"))
        );

        // Test StructureDefinition dynamic variables (%ext-*)
        assert_eq!(
            context.get_variable("%ext-birthPlace"),
            Some(FhirPathValue::string("http://hl7.org/fhir/StructureDefinition/birthPlace"))
        );
        assert_eq!(
            context.get_variable("%ext-patient-nationality"),
            Some(FhirPathValue::string("http://hl7.org/fhir/StructureDefinition/patient-nationality"))
        );
        assert_eq!(
            context.get_variable("ext-custom-extension"),
            Some(FhirPathValue::string("http://hl7.org/fhir/StructureDefinition/custom-extension"))
        );

        // Test non-dynamic variables return None
        assert_eq!(context.get_variable("%unknown-pattern"), None);
        assert_eq!(context.get_variable("%prefix-unknown"), None);
    }

    #[test]
    fn test_dynamic_variables_in_child_contexts() {
        let parent = ContextManager::create(Collection::empty());
        let child = parent.create_child();

        // Child should inherit dynamic variable resolution capability
        assert_eq!(
            child.get_variable("%vs-inherited"),
            Some(FhirPathValue::string("http://hl7.org/fhir/ValueSet/inherited"))
        );
        assert_eq!(
            child.get_variable("%ext-inherited"),
            Some(FhirPathValue::string("http://hl7.org/fhir/StructureDefinition/inherited"))
        );
    }

    #[tokio::test]
    async fn test_evaluation_context_creation() {
        let model_provider = Arc::new(MockModelProvider);
        let context = EvaluationContext::new(Collection::empty(), model_provider.clone(), None).await;

        assert!(Arc::ptr_eq(&context.get_model_provider(), &model_provider));
        assert!(context.get_terminology_provider().is_none());
        assert_eq!(context.get_depth(), 0);
    }

    #[tokio::test]
    async fn test_context_builder() {
        let model_provider = Arc::new(MockModelProvider);
        let mut variables = HashMap::new();
        variables.insert("testVar".to_string(), FhirPathValue::string("test"));

        let context = EvaluationContextBuilder::new()
            .with_input(Collection::empty())
            .with_model_provider(model_provider)
            .with_variables(variables)
            .with_environment_variable("%custom".to_string(), FhirPathValue::string("custom_value"))
            .build()
            .await
            .unwrap();

        assert_eq!(context.get_variable("testVar"), Some(FhirPathValue::string("test")));
        assert_eq!(context.get_variable("%custom"), Some(FhirPathValue::string("custom_value")));
    }

    #[tokio::test]
    async fn test_child_context_performance() {
        let model_provider = Arc::new(MockModelProvider);
        let context = EvaluationContext::new(Collection::empty(), model_provider, None).await;

        // Creating child contexts should be O(1) - no expensive cloning
        let start = std::time::Instant::now();
        let _child1 = context.create_child(Collection::empty());
        let _child2 = context.create_child(Collection::empty()); 
        let _child3 = context.create_child(Collection::empty());
        let duration = start.elapsed();

        // Should be very fast (< 1ms for multiple child creations)
        assert!(duration.as_millis() < 10);
    }

    #[tokio::test]
    async fn test_iterator_context_variable_isolation() {
        let model_provider = Arc::new(MockModelProvider);
        let context = EvaluationContext::new(
            Collection::from_values(vec![
                FhirPathValue::string("item1"),
                FhirPathValue::string("item2"),
                FhirPathValue::string("item3"),
            ]),
            model_provider,
            None,
        ).await;

        let iter_ctx = context.create_iterator_context(FhirPathValue::string("item2"), 1);

        // Should have proper lambda variables
        assert_eq!(iter_ctx.get_variable("$this"), Some(FhirPathValue::string("item2")));
        assert_eq!(iter_ctx.get_variable("$index"), Some(FhirPathValue::integer(1)));
        assert_eq!(iter_ctx.get_variable("$total"), Some(FhirPathValue::integer(3)));
    }
}
