//! # Symbol Table
//!
//! Manages symbol resolution, scope management, and identifier lookup for FHIRPath analysis.
//! Provides hierarchical symbol storage with proper scoping rules for variables and functions.

use std::collections::HashMap;
use octofhir_fhirpath_diagnostics::{SourceLocation, Span, Position};

use super::type_system::{FhirType, TypeInformation};
use crate::providers::function_provider::FunctionSignature;

/// Symbol table for managing identifiers and their scopes
#[derive(Debug)]
pub struct SymbolTable {
    /// Hierarchical scopes (current scope is last)
    scopes: Vec<Scope>,
    
    /// Built-in function signatures
    builtin_functions: HashMap<String, FunctionSignature>,
    
    /// Built-in variable types
    builtin_variables: HashMap<String, TypeInformation>,
}

/// A single scope containing symbol bindings
#[derive(Debug, Clone)]
pub struct Scope {
    /// Scope identifier
    id: ScopeId,
    
    /// Type of scope
    scope_type: ScopeType,
    
    /// Variable bindings in this scope
    variables: HashMap<String, VariableBinding>,
    
    /// Function bindings in this scope (for lambda functions)
    functions: HashMap<String, FunctionBinding>,
    
    /// Source location where scope was created
    location: SourceLocation,
}

/// Scope identifier
pub type ScopeId = usize;

/// Types of scopes
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    /// Global scope (built-ins, constants)
    Global,
    
    /// Function parameter scope
    Function,
    
    /// Lambda expression scope
    Lambda,
    
    /// Block scope (for complex expressions)
    Block,
    
    /// Resource context scope
    Resource,
}

/// Variable binding information
#[derive(Debug, Clone)]
pub struct VariableBinding {
    /// Variable name
    pub name: String,
    
    /// Type information
    pub type_info: TypeInformation,
    
    /// Whether variable is mutable
    pub is_mutable: bool,
    
    /// Source location where declared
    pub location: SourceLocation,
    
    /// Scope where defined
    pub scope_id: ScopeId,
}

/// Function binding information  
#[derive(Debug, Clone)]
pub struct FunctionBinding {
    /// Function name
    pub name: String,
    
    /// Function signature
    pub signature: FunctionSignature,
    
    /// Source location where declared
    pub location: SourceLocation,
    
    /// Scope where defined
    pub scope_id: ScopeId,
}

/// Symbol table errors
#[derive(Debug, thiserror::Error)]
pub enum SymbolTableError {
    #[error("Variable '{name}' not found in current scope")]
    VariableNotFound { name: String },
    
    #[error("Function '{name}' not found")]
    FunctionNotFound { name: String },
    
    #[error("Variable '{name}' already exists in current scope")]
    VariableAlreadyExists { name: String },
    
    #[error("Cannot shadow immutable variable '{name}'")]
    CannotShadowImmutable { name: String },
    
    #[error("Invalid scope operation: {message}")]
    InvalidScopeOperation { message: String },
}

impl SymbolTable {
    /// Create a new symbol table with global scope
    pub fn new() -> Self {
        let global_scope = Scope {
            id: 0,
            scope_type: ScopeType::Global,
            variables: HashMap::new(),
            functions: HashMap::new(),
            location: SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0))),
        };
        
        let mut symbol_table = Self {
            scopes: vec![global_scope],
            builtin_functions: HashMap::new(),
            builtin_variables: HashMap::new(),
        };
        
        // Initialize built-in functions and variables
        symbol_table.initialize_builtins();
        
        symbol_table
    }
    
    /// Initialize built-in functions and variables
    fn initialize_builtins(&mut self) {
        // Built-in variables
        self.builtin_variables.insert(
            "%context".to_string(),
            TypeInformation::new(FhirType::Unknown, SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0))))
        );
        
        self.builtin_variables.insert(
            "%resource".to_string(),
            TypeInformation::new(FhirType::Unknown, SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0))))
        );
        
        self.builtin_variables.insert(
            "$index".to_string(),
            TypeInformation::new(
                FhirType::Primitive(super::type_system::PrimitiveType::Integer), 
                SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0)))
            )
        );
        
        self.builtin_variables.insert(
            "$total".to_string(),
            TypeInformation::new(
                FhirType::Primitive(super::type_system::PrimitiveType::Integer), 
                SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0)))
            )
        );
        
        // Built-in functions will be populated from the function registry
        // This is a placeholder for common functions
        self.builtin_functions.insert(
            "empty".to_string(),
            FunctionSignature {
                name: "empty".to_string(),
                parameter_count: 0,
                return_type: "boolean".to_string(),
            }
        );
        
        self.builtin_functions.insert(
            "exists".to_string(),
            FunctionSignature {
                name: "exists".to_string(),
                parameter_count: 0,
                return_type: "boolean".to_string(),
            }
        );
    }
    
    /// Push a new scope onto the scope stack
    pub fn push_scope(&mut self, scope_type: ScopeType, location: SourceLocation) -> ScopeId {
        let scope_id = self.scopes.len();
        let scope = Scope {
            id: scope_id,
            scope_type,
            variables: HashMap::new(),
            functions: HashMap::new(),
            location,
        };
        
        self.scopes.push(scope);
        scope_id
    }
    
    /// Pop the current scope from the stack
    pub fn pop_scope(&mut self) -> Result<Option<Scope>, SymbolTableError> {
        if self.scopes.len() <= 1 {
            return Err(SymbolTableError::InvalidScopeOperation {
                message: "Cannot pop global scope".to_string(),
            });
        }
        
        Ok(self.scopes.pop())
    }
    
    /// Define a variable in the current scope
    pub fn define_variable(
        &mut self, 
        name: String, 
        type_info: TypeInformation,
        is_mutable: bool,
        location: SourceLocation,
    ) -> Result<(), SymbolTableError> {
        let current_scope = self.current_scope_mut()?;
        let scope_id = current_scope.id;
        
        // Check if variable already exists in current scope
        if current_scope.variables.contains_key(&name) {
            return Err(SymbolTableError::VariableAlreadyExists { name });
        }
        
        let binding = VariableBinding {
            name: name.clone(),
            type_info,
            is_mutable,
            location,
            scope_id,
        };
        
        current_scope.variables.insert(name, binding);
        Ok(())
    }
    
    /// Resolve a variable by name (searches up scope chain)
    pub fn resolve_variable(&self, name: &str) -> Option<&VariableBinding> {
        // Search current scopes from most recent to oldest
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.variables.get(name) {
                return Some(binding);
            }
        }
        
        // Check built-in variables
        if self.builtin_variables.contains_key(name) {
            // For built-ins, we'd need to create a temporary binding
            // In practice, this should be handled differently
            return None; // TODO: Handle built-in variables properly
        }
        
        None
    }
    
    /// Resolve a function by name
    pub fn resolve_function(&self, name: &str) -> Option<&FunctionSignature> {
        // Search scopes for local function definitions
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.functions.get(name) {
                return Some(&binding.signature);
            }
        }
        
        // Check built-in functions
        self.builtin_functions.get(name)
    }
    
    /// Get current scope
    fn current_scope(&self) -> Result<&Scope, SymbolTableError> {
        self.scopes.last().ok_or_else(|| SymbolTableError::InvalidScopeOperation {
            message: "No active scope".to_string(),
        })
    }
    
    /// Get current scope (mutable)
    fn current_scope_mut(&mut self) -> Result<&mut Scope, SymbolTableError> {
        self.scopes.last_mut().ok_or_else(|| SymbolTableError::InvalidScopeOperation {
            message: "No active scope".to_string(),
        })
    }
    
    /// Get all visible variables in current context
    pub fn visible_variables(&self) -> Vec<&VariableBinding> {
        let mut variables = Vec::new();
        
        // Collect from all scopes (inner to outer)
        for scope in self.scopes.iter().rev() {
            for binding in scope.variables.values() {
                // Only add if not already shadowed
                if !variables.iter().any(|var: &&VariableBinding| var.name == binding.name) {
                    variables.push(binding);
                }
            }
        }
        
        variables
    }
    
    /// Get all visible functions in current context
    pub fn visible_functions(&self) -> Vec<&FunctionSignature> {
        let mut functions = Vec::new();
        
        // Collect from scopes
        for scope in self.scopes.iter().rev() {
            for binding in scope.functions.values() {
                if !functions.iter().any(|func: &&FunctionSignature| func.name == binding.signature.name) {
                    functions.push(&binding.signature);
                }
            }
        }
        
        // Add built-in functions
        for signature in self.builtin_functions.values() {
            if !functions.iter().any(|func| func.name == signature.name) {
                functions.push(signature);
            }
        }
        
        functions
    }
    
    /// Get current scope depth
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }
    
    /// Check if we're in a lambda scope
    pub fn in_lambda_scope(&self) -> bool {
        self.scopes.iter().any(|scope| scope.scope_type == ScopeType::Lambda)
    }
    
    /// Get the current scope type
    pub fn current_scope_type(&self) -> Result<ScopeType, SymbolTableError> {
        Ok(self.current_scope()?.scope_type.clone())
    }
    
    /// Add a built-in function signature
    pub fn add_builtin_function(&mut self, signature: FunctionSignature) {
        self.builtin_functions.insert(signature.name.clone(), signature);
    }
    
    /// Add multiple built-in function signatures
    pub fn add_builtin_functions(&mut self, signatures: Vec<FunctionSignature>) {
        for signature in signatures {
            self.add_builtin_function(signature);
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableBinding {
    /// Create a new variable binding
    pub fn new(
        name: String, 
        type_info: TypeInformation, 
        is_mutable: bool, 
        location: SourceLocation,
        scope_id: ScopeId,
    ) -> Self {
        Self {
            name,
            type_info,
            is_mutable,
            location,
            scope_id,
        }
    }
}

impl FunctionBinding {
    /// Create a new function binding
    pub fn new(
        name: String, 
        signature: FunctionSignature, 
        location: SourceLocation,
        scope_id: ScopeId,
    ) -> Self {
        Self {
            name,
            signature,
            location,
            scope_id,
        }
    }
}