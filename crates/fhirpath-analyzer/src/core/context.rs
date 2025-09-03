//! # Analysis Context
//!
//! Manages the execution context for FHIRPath analysis, including variable scopes,
//! resource context, configuration, and environment state.

use std::collections::HashMap;
use octofhir_fhirpath_diagnostics::{SourceLocation, Span, Position};

use super::type_system::{FhirType, TypeInformation, Cardinality};

/// Analysis context containing all state and configuration for analysis
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Root resource type being analyzed
    pub root_resource_type: Option<String>,
    
    /// Current scope information
    pub current_scope: ScopeInfo,
    
    /// Variable bindings and their types
    pub variable_bindings: HashMap<String, TypeInformation>,
    
    /// Lambda expression context stack
    pub lambda_stack: Vec<LambdaContext>,
    
    /// Path navigation stack
    pub path_stack: Vec<PathSegment>,
    
    /// Current analysis phase
    pub analysis_phase: AnalysisPhase,
}

/// Information about the current scope
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    /// Current context type (the implicit 'this' type)
    pub context_type: Option<FhirType>,
    
    /// Cardinality of current context
    pub cardinality: Cardinality,
    
    /// Whether current context is a collection
    pub is_collection: bool,
    
    /// Source location of this scope
    pub source_location: SourceLocation,
}

/// Lambda expression context
#[derive(Debug, Clone)]
pub struct LambdaContext {
    /// Type of 'this' within the lambda
    pub this_type: FhirType,
    
    /// Whether index variable ('$index') is available
    pub index_available: bool,
    
    /// Type of iteration
    pub iteration_type: IterationType,
    
    /// Variables captured from outer scope
    pub capture_scope: HashMap<String, TypeInformation>,
}

/// Types of iteration in lambda contexts
#[derive(Debug, Clone, PartialEq)]
pub enum IterationType {
    /// Simple iteration over collection elements
    Simple,
    
    /// Indexed iteration with $index available
    Indexed,
    
    /// Aggregation operation (e.g., reduce)
    Aggregation,
}

/// Path navigation segment
#[derive(Debug, Clone)]
pub struct PathSegment {
    /// Property or function name
    pub name: String,
    
    /// Source type (input)
    pub source_type: FhirType,
    
    /// Target type (output)  
    pub target_type: FhirType,
    
    /// Source location
    pub location: SourceLocation,
}

/// Current phase of analysis
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisPhase {
    /// Initial lexical analysis
    Lexical,
    
    /// Property resolution phase
    PropertyResolution,
    
    /// Function validation phase
    FunctionValidation,
    
    /// Type checking phase
    TypeChecking,
    
    /// Lambda validation phase
    LambdaValidation,
    
    /// Optimization analysis phase
    OptimizationAnalysis,
    
    /// Final result generation
    ResultGeneration,
}

impl AnalysisContext {
    /// Create a new analysis context
    pub fn new() -> Self {
        Self {
            root_resource_type: None,
            current_scope: ScopeInfo {
                context_type: None,
                cardinality: Cardinality::default(),
                is_collection: false,
                source_location: SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0))),
            },
            variable_bindings: HashMap::new(),
            lambda_stack: Vec::new(),
            path_stack: Vec::new(),
            analysis_phase: AnalysisPhase::Lexical,
        }
    }
    
    /// Create context with a root resource type
    pub fn with_root_type(root_type: String) -> Self {
        let mut context = Self::new();
        context.root_resource_type = Some(root_type.clone());
        
        // Set initial scope to the root resource type
        context.current_scope.context_type = Some(FhirType::Resource(
            super::type_system::ResourceType {
                name: root_type,
                base_type: None,
            }
        ));
        
        context
    }
    
    /// Enter a new scope with the given context type
    pub fn enter_scope(&mut self, context_type: FhirType, location: SourceLocation) {
        self.current_scope = ScopeInfo {
            context_type: Some(context_type),
            cardinality: Cardinality::default(),
            is_collection: false,
            source_location: location,
        };
    }
    
    /// Set the cardinality for the current scope
    pub fn set_scope_cardinality(&mut self, cardinality: Cardinality) {
        self.current_scope.is_collection = cardinality.max.map_or(true, |max| max > 1);
        self.current_scope.cardinality = cardinality;
    }
    
    /// Push a lambda context onto the stack
    pub fn push_lambda(&mut self, lambda_context: LambdaContext) {
        self.lambda_stack.push(lambda_context);
    }
    
    /// Pop the most recent lambda context
    pub fn pop_lambda(&mut self) -> Option<LambdaContext> {
        self.lambda_stack.pop()
    }
    
    /// Get current lambda context (if any)
    pub fn current_lambda(&self) -> Option<&LambdaContext> {
        self.lambda_stack.last()
    }
    
    /// Bind a variable to a type
    pub fn bind_variable(&mut self, name: String, type_info: TypeInformation) {
        self.variable_bindings.insert(name, type_info);
    }
    
    /// Resolve a variable binding
    pub fn resolve_variable(&self, name: &str) -> Option<&TypeInformation> {
        // Check current variable bindings first
        if let Some(binding) = self.variable_bindings.get(name) {
            return Some(binding);
        }
        
        // Check lambda capture scopes
        for lambda in self.lambda_stack.iter().rev() {
            if let Some(captured) = lambda.capture_scope.get(name) {
                return Some(captured);
            }
        }
        
        // Check for built-in variables
        match name {
            "%context" | "%resource" => {
                if let Some(ref _root_type) = self.root_resource_type {
                    // Create a temporary TypeInformation for the root resource
                    // In practice, this would be cached somewhere
                    return None; // TODO: Implement proper handling
                }
            }
            "$index" => {
                if let Some(lambda) = self.current_lambda() {
                    if lambda.index_available {
                        // Return integer type for $index
                        return None; // TODO: Implement proper handling
                    }
                }
            }
            "$total" => {
                // Available in aggregation contexts
                if let Some(lambda) = self.current_lambda() {
                    if lambda.iteration_type == IterationType::Aggregation {
                        return None; // TODO: Implement proper handling
                    }
                }
            }
            _ => {}
        }
        
        None
    }
    
    /// Push a path segment onto the navigation stack
    pub fn push_path(&mut self, segment: PathSegment) {
        self.path_stack.push(segment);
    }
    
    /// Pop the most recent path segment
    pub fn pop_path(&mut self) -> Option<PathSegment> {
        self.path_stack.pop()
    }
    
    /// Get the current navigation path as a string
    pub fn current_path_string(&self) -> String {
        self.path_stack.iter()
            .map(|segment| segment.name.clone())
            .collect::<Vec<_>>()
            .join(".")
    }
    
    /// Set the current analysis phase
    pub fn set_phase(&mut self, phase: AnalysisPhase) {
        self.analysis_phase = phase;
    }
    
    /// Check if we're currently in a lambda context
    pub fn in_lambda(&self) -> bool {
        !self.lambda_stack.is_empty()
    }
    
    /// Get the depth of lambda nesting
    pub fn lambda_depth(&self) -> usize {
        self.lambda_stack.len()
    }
    
    /// Clone the context for use in nested analysis
    pub fn clone_for_nested(&self) -> Self {
        let mut cloned = self.clone();
        
        // Clear path stack for nested analysis
        cloned.path_stack.clear();
        
        cloned
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LambdaContext {
    /// Create a simple lambda context
    pub fn simple(this_type: FhirType) -> Self {
        Self {
            this_type,
            index_available: false,
            iteration_type: IterationType::Simple,
            capture_scope: HashMap::new(),
        }
    }
    
    /// Create an indexed lambda context
    pub fn indexed(this_type: FhirType) -> Self {
        Self {
            this_type,
            index_available: true,
            iteration_type: IterationType::Indexed,
            capture_scope: HashMap::new(),
        }
    }
    
    /// Create an aggregation lambda context
    pub fn aggregation(this_type: FhirType) -> Self {
        Self {
            this_type,
            index_available: false,
            iteration_type: IterationType::Aggregation,
            capture_scope: HashMap::new(),
        }
    }
    
    /// Add a captured variable
    pub fn capture_variable(&mut self, name: String, type_info: TypeInformation) {
        self.capture_scope.insert(name, type_info);
    }
}

impl PathSegment {
    /// Create a new path segment
    pub fn new(
        name: String, 
        source_type: FhirType, 
        target_type: FhirType, 
        location: SourceLocation
    ) -> Self {
        Self {
            name,
            source_type,
            target_type,
            location,
        }
    }
}