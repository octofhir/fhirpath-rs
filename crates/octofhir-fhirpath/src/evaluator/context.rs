//! Enhanced Evaluation Context System for FHIRPath
//!
//! This module implements the comprehensive evaluation context system required by
//! the FHIRPath specification, including built-in variables (%terminologies, %server, %factory),
//! user-defined variables, and async service integration.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt;

use crate::core::{Collection, FhirPathValue, FhirPathError, Result, ModelProvider};

/// Comprehensive evaluation context with all FHIRPath requirements
///
/// This context provides everything needed for FHIRPath expression evaluation:
/// - Start context (initial resource collection)
/// - User-defined variables from `defineVariable()` 
/// - Built-in variables (%terminologies, %server, %factory, etc.)
/// - Terminology server integration
/// - FHIR server API integration  
/// - Type factory for creating FHIR data types
///
/// # Thread Safety
/// 
/// All context types implement Send + Sync for multi-threaded use.
///
/// # Examples
///
/// ```rust,no_run
/// use octofhir_fhirpath::evaluator::*;
/// use std::collections::HashMap;
/// 
/// # async fn example() -> octofhir_fhirpath::Result<()> {
/// // Create context with initial resource
/// let patient = octofhir_fhirpath::Collection::single(
///     octofhir_fhirpath::FhirPathValue::resource(serde_json::json!({
///         "resourceType": "Patient",
///         "name": [{"family": "Smith", "given": ["John"]}]
///     }))
/// );
/// 
/// let context = EvaluationContext::new(patient);
/// 
/// // Add variables
/// let mut variables = HashMap::new();
/// variables.insert("threshold".to_string(), octofhir_fhirpath::FhirPathValue::Integer(25));
/// let context = context.with_variables(variables);
/// 
/// // Configure terminology server
/// let context = context.with_terminology_server(
///     "https://tx.fhir.org/r4/".to_string(),
///     "r4".to_string()
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Starting context for evaluation (e.g., Patient resource collection)
    pub start_context: Collection,
    /// User-defined variables available during evaluation
    pub variables: HashMap<String, FhirPathValue>,
    /// Built-in variables from FHIRPath specification
    pub builtin_variables: BuiltinVariables,
    /// Server context for FHIR operations
    pub server_context: Option<ServerContext>,
    /// Current evaluation depth (for recursion protection)
    pub depth: usize,
}

impl EvaluationContext {
    /// Create new context with input collection as start context
    ///
    /// In FHIRPath evaluation, the input collection becomes the start context
    /// for expression evaluation. For example, if you pass a Patient resource
    /// collection, expressions like "Patient.name" will start evaluation from
    /// that Patient resource.
    ///
    /// # Arguments
    /// * `start_context` - Input collection that becomes the start context for evaluation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use octofhir_fhirpath::{Collection, FhirPathValue, evaluator::EvaluationContext};
    /// 
    /// let patient = Collection::single(FhirPathValue::resource(serde_json::json!({
    ///     "resourceType": "Patient",
    ///     "id": "example",
    ///     "name": [{"family": "Smith", "given": ["John"]}]
    /// })));
    /// 
    /// // Create context - the patient collection becomes the start context
    /// let context = EvaluationContext::new(patient);
    /// // Now expressions like "Patient.name.family" will evaluate against this Patient
    /// ```
    pub fn new(start_context: Collection) -> Self {
        let mut builtin_variables = BuiltinVariables::default();
        
        // Set %context to the start context
        if !start_context.is_empty() {
            let context_value = if start_context.len() == 1 {
                start_context.first().unwrap().clone()
            } else {
                FhirPathValue::Collection(start_context.clone().into_vec())
            };
            
            builtin_variables.set_context(context_value.clone());
            
            // For now, assume %resource and %rootResource are the same as %context
            // This can be refined later based on contained resources
            builtin_variables.set_resource(context_value.clone());
            builtin_variables.set_root_resource(context_value);
        }
        
        Self {
            start_context,
            variables: HashMap::new(),
            builtin_variables,
            server_context: None,
            depth: 0,
        }
    }
    
    /// Create context with custom variables
    ///
    /// # Arguments
    /// * `start_context` - Initial resource collection
    /// * `variables` - User-defined variables map
    pub fn with_variables(
        start_context: Collection,
        variables: HashMap<String, FhirPathValue>
    ) -> Self {
        Self {
            start_context,
            variables,
            builtin_variables: BuiltinVariables::default(),
            server_context: None,
            depth: 0,
        }
    }
    
    /// Create context with terminology server configuration
    ///
    /// # Arguments
    /// * `start_context` - Initial resource collection
    /// * `terminology_server` - Terminology server URL 
    /// * `fhir_version` - FHIR version (r4, r4b, r5)
    pub fn with_terminology_server(
        start_context: Collection,
        terminology_server: String,
        fhir_version: String
    ) -> Self {
        let mut builtin_variables = BuiltinVariables::new(fhir_version);
        builtin_variables.terminology_server = terminology_server;
        
        Self {
            start_context,
            variables: HashMap::new(),
            builtin_variables,
            server_context: None,
            depth: 0,
        }
    }
    
    /// Get variable value by name
    /// 
    /// Searches user-defined variables first, then built-in environment variables.
    ///
    /// # Arguments
    /// * `name` - Variable name (with or without % prefix)
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        // Check user variables first
        if let Some(value) = self.variables.get(name) {
            return Some(value);
        }
        
        // Check built-in environment variables
        self.builtin_variables.get_environment_variable(name)
    }
    
    /// Set variable value
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Variable value
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        self.variables.insert(name, value);
    }
    
    /// Create child context for nested evaluation
    /// 
    /// Child contexts inherit all variables and built-ins from parent context
    /// but can have different start context and increased depth.
    ///
    /// # Arguments
    /// * `new_context` - New start context for child
    pub fn create_child_context(&self, new_context: Collection) -> Self {
        Self {
            start_context: new_context,
            variables: self.variables.clone(),
            builtin_variables: self.builtin_variables.clone(),
            server_context: self.server_context.clone(),
            depth: self.depth + 1,
        }
    }

    /// Get the terminology service if configured
    pub fn get_terminology_service(&self) -> Option<&Arc<dyn TerminologyService>> {
        self.builtin_variables.terminologies.as_ref()
    }

    /// Get the server API if configured  
    pub fn get_server_api(&self) -> Option<&Arc<dyn ServerApi>> {
        self.builtin_variables.server.as_ref()
    }

    /// Get the type factory if configured
    pub fn get_type_factory(&self) -> Option<&Arc<dyn TypeFactory>> {
        self.builtin_variables.factory.as_ref()
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new(Collection::empty())
    }
}

/// Built-in variables from FHIRPath specification
///
/// Provides access to standard FHIRPath built-in variables including:
/// - %terminologies - Terminology service integration
/// - %server - FHIR server API integration
/// - %factory - Type factory for creating FHIR data types
/// - %resource - The resource that contains the original node that is in %context
/// - %rootResource - The container resource for the resource identified by %resource
/// - %context - Current focus node in FHIRPath evaluation
/// - %sct, %loinc - Standard coding system URLs
/// - %vs-[name], %ext-[name] - Value set and extension URLs
#[derive(Clone)]
pub struct BuiltinVariables {
    /// Terminology server URL with FHIR version prefix (default: https://tx.fhir.org/r4/)
    pub terminology_server: String,
    /// Terminology service implementation (%terminologies variable)
    pub terminologies: Option<Arc<dyn TerminologyService>>,
    /// Server API implementation (%server variable)
    pub server: Option<Arc<dyn ServerApi>>,
    /// Factory object for creating FHIR data types (%factory variable)
    pub factory: Option<Arc<dyn TypeFactory>>,
    /// Current resource context (%resource variable)
    pub resource: Option<FhirPathValue>,
    /// Root resource context (%rootResource variable)
    pub root_resource: Option<FhirPathValue>,
    /// Current evaluation context (%context variable)
    pub context: Option<FhirPathValue>,
    /// Environment variables (%sct, %loinc, %vs-[name], %ext-[name])
    pub environment_variables: HashMap<String, FhirPathValue>,
    /// FHIR version (r4, r4b, r5) - affects terminology server URL
    pub fhir_version: String,
}

impl BuiltinVariables {
    /// Create new built-in variables with defaults for specified FHIR version
    ///
    /// # Arguments
    /// * `fhir_version` - FHIR version string (r4, r4b, r5)
    pub fn new(fhir_version: String) -> Self {
        let terminology_server = format!("https://tx.fhir.org/{}/", fhir_version);
        let mut env_vars = HashMap::new();
        
        // Add common SNOMED CT environment variable
        env_vars.insert("%sct".to_string(), FhirPathValue::String("http://snomed.info/sct".to_string()));
        
        // Add common LOINC environment variable
        env_vars.insert("%loinc".to_string(), FhirPathValue::String("http://loinc.org".to_string()));
        
        Self {
            terminology_server,
            terminologies: None,
            server: None,
            factory: None,
            resource: None,
            root_resource: None,
            context: None,
            environment_variables: env_vars,
            fhir_version,
        }
    }
    
    /// Get built-in variable value
    ///
    /// # Arguments  
    /// * `name` - Variable name (with or without % prefix)
    pub fn get_environment_variable(&self, name: &str) -> Option<&FhirPathValue> {
        // Remove % prefix for consistent handling
        let clean_name = if name.starts_with('%') {
            &name[1..]
        } else {
            name
        };
        
        // Check context-specific variables first
        match clean_name {
            "resource" => self.resource.as_ref(),
            "rootResource" => self.root_resource.as_ref(),
            "context" => self.context.as_ref(),
            _ => {
                // Check environment variables
                let var_name = if name.starts_with('%') {
                    name.to_string()
                } else {
                    format!("%{}", name)
                };
                self.environment_variables.get(&var_name)
            }
        }
    }
    
    /// Set environment variable
    ///
    /// # Arguments
    /// * `name` - Variable name (% prefix will be added if not present)
    /// * `value` - Variable value
    pub fn set_environment_variable(&mut self, name: String, value: FhirPathValue) {
        let var_name = if name.starts_with('%') {
            name
        } else {
            format!("%{}", name)
        };
        self.environment_variables.insert(var_name, value);
    }
    
    /// Add value set variable (%vs-[name])
    ///
    /// # Arguments
    /// * `name` - Value set name
    /// * `url` - Value set URL
    pub fn add_value_set(&mut self, name: &str, url: &str) {
        let var_name = format!("%vs-{}", name);
        self.environment_variables.insert(var_name, FhirPathValue::String(url.to_string()));
    }
    
    /// Add extension variable (%ext-[name])
    ///
    /// # Arguments
    /// * `name` - Extension name
    /// * `url` - Extension URL
    pub fn add_extension(&mut self, name: &str, url: &str) {
        let var_name = format!("%ext-{}", name);
        self.environment_variables.insert(var_name, FhirPathValue::String(url.to_string()));
    }
    
    /// Set the current resource (%resource variable)
    ///
    /// According to FHIRPath specification, %resource is "the resource that contains 
    /// the original node that is in %context"
    ///
    /// # Arguments
    /// * `resource` - The current resource value
    pub fn set_resource(&mut self, resource: FhirPathValue) {
        self.resource = Some(resource);
    }
    
    /// Set the root resource (%rootResource variable)
    ///
    /// According to FHIRPath specification, %rootResource is "the container resource 
    /// for the resource identified by %resource". In most cases, this is the same as
    /// %resource unless the resource is contained within another resource.
    ///
    /// # Arguments
    /// * `root_resource` - The root resource value
    pub fn set_root_resource(&mut self, root_resource: FhirPathValue) {
        self.root_resource = Some(root_resource);
    }
    
    /// Set the current context (%context variable)
    ///
    /// The %context variable represents the current focus node in FHIRPath evaluation.
    /// Often %resource = %context, but %context can change during navigation.
    ///
    /// # Arguments
    /// * `context` - The current context value
    pub fn set_context(&mut self, context: FhirPathValue) {
        self.context = Some(context);
    }
    
    /// Get the current resource (%resource variable)
    pub fn get_resource(&self) -> Option<&FhirPathValue> {
        self.resource.as_ref()
    }
    
    /// Get the root resource (%rootResource variable)
    pub fn get_root_resource(&self) -> Option<&FhirPathValue> {
        self.root_resource.as_ref()
    }
    
    /// Get the current context (%context variable)
    pub fn get_context(&self) -> Option<&FhirPathValue> {
        self.context.as_ref()
    }
}

impl fmt::Debug for BuiltinVariables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltinVariables")
            .field("terminology_server", &self.terminology_server)
            .field("terminologies", &self.terminologies.as_ref().map(|_| "Some(TerminologyService)"))
            .field("server", &self.server.as_ref().map(|_| "Some(ServerApi)"))
            .field("factory", &self.factory.as_ref().map(|_| "Some(TypeFactory)"))
            .field("resource", &self.resource)
            .field("root_resource", &self.root_resource)
            .field("context", &self.context)
            .field("environment_variables", &self.environment_variables)
            .field("fhir_version", &self.fhir_version)
            .finish()
    }
}

impl Default for BuiltinVariables {
    fn default() -> Self {
        Self::new("r4".to_string())
    }
}

/// Configuration for terminology and server operations
///
/// Provides connection settings and authentication for external services.
#[derive(Debug, Clone)]
pub struct ServerContext {
    /// Base URL for FHIR server operations
    pub base_url: String,
    /// Authentication headers for server requests
    pub headers: HashMap<String, String>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    /// Connection pool settings
    pub connection_pool_size: usize,
}

impl ServerContext {
    /// Create new server context
    ///
    /// # Arguments
    /// * `base_url` - Base URL for FHIR server
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            headers: HashMap::new(),
            timeout_seconds: 30,
            max_concurrent_requests: 10,
            connection_pool_size: 5,
        }
    }
    
    /// Add authentication header
    ///
    /// # Arguments
    /// * `key` - Header key (e.g., "Authorization")
    /// * `value` - Header value (e.g., "Bearer token123")
    pub fn with_auth_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    /// Set timeout
    ///
    /// # Arguments
    /// * `seconds` - Timeout in seconds
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
    
    /// Get authorization header value
    ///
    /// # Arguments
    /// * `key` - Header key to retrieve
    pub fn get_auth_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

/// Factory trait for creating FHIR data types (%factory variable)
///
/// Provides dynamic factory functionality as specified in FHIRPath:
/// %factory.{type}(value, extensions) : {type}
///
/// The factory enables creating FHIR data types dynamically based on type names,
/// supporting both primitive and complex types with proper validation and extensions.
#[async_trait]
pub trait TypeFactory: Send + Sync + std::fmt::Debug {
    /// Create primitive type dynamically
    /// 
    /// Examples:
    /// - %factory.string("test") : string  
    /// - %factory.integer(42) : integer
    /// - %factory.boolean(true) : boolean
    ///
    /// # Arguments
    /// * `type_name` - FHIR primitive type name
    /// * `value` - Value to assign to the primitive
    /// * `extensions` - Optional extensions to add
    async fn create_primitive(
        &self, 
        type_name: &str, 
        value: &FhirPathValue, 
        extensions: Option<Collection>
    ) -> Result<FhirPathValue>;
    
    /// Create complex type dynamically
    /// 
    /// Examples:
    /// - %factory.Identifier({"system": "http://example.com", "value": "123"})
    /// - %factory.HumanName({"family": "Smith", "given": ["John"]})
    ///
    /// # Arguments
    /// * `type_name` - FHIR complex type name
    /// * `properties` - Properties to set on the complex type
    /// * `extensions` - Optional extensions to add
    async fn create_complex(
        &self, 
        type_name: &str, 
        properties: Collection, 
        extensions: Option<Collection>
    ) -> Result<FhirPathValue>;
    
    /// Check if factory can create the specified type
    ///
    /// Used for dynamic type checking before creation attempts.
    ///
    /// # Arguments
    /// * `type_name` - Type name to check
    async fn can_create(&self, type_name: &str) -> bool;
    
    /// Get available type names from FHIR schema
    ///
    /// Returns all type names that this factory can create.
    async fn get_available_types(&self) -> Result<Vec<String>>;
    
    /// Get type definition and constraints from ModelProvider
    ///
    /// # Arguments
    /// * `type_name` - Type name to get definition for
    async fn get_type_definition(&self, type_name: &str) -> Result<TypeDefinition>;
}

/// Type definition information from FHIR schema
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Type name
    pub name: String,
    /// Type classification
    pub kind: TypeKind,
    /// Available properties
    pub properties: Vec<PropertyDefinition>,
    /// Validation constraints
    pub constraints: Vec<String>,
}

/// Type classification
#[derive(Debug, Clone)]
pub enum TypeKind {
    /// Primitive types (string, integer, boolean, etc.)
    Primitive,
    /// Complex types (Identifier, HumanName, etc.)
    Complex,
    /// Resource types (Patient, Observation, etc.)
    Resource,
}

/// Property definition for complex types
#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    /// Property name
    pub name: String,
    /// Property type name
    pub type_name: String,
    /// Minimum cardinality
    pub min: u32,
    /// Maximum cardinality (None = unbounded)
    pub max: Option<u32>,
}

/// Terminology Service trait for FHIRPath %terminologies variable
///
/// Provides access to terminology services as specified in FHIRPath.
/// All operations follow FHIR Terminology Service patterns with async support
/// for external terminology server communication.
#[async_trait]
pub trait TerminologyService: Send + Sync + std::fmt::Debug {
    /// Expand value set and return all codes (expand operation)
    ///
    /// # Arguments
    /// * `value_set` - Value set URL or canonical reference
    /// * `params` - Optional parameters for expansion
    async fn expand(
        &self, 
        value_set: &str, 
        params: Option<HashMap<String, String>>
    ) -> Result<Collection>;
    
    /// Look up properties of a coded value (lookup operation)
    ///
    /// # Arguments
    /// * `coded` - Coded value to look up
    /// * `params` - Optional parameters for lookup
    async fn lookup(
        &self, 
        coded: &FhirPathValue, 
        params: Option<HashMap<String, String>>
    ) -> Result<Collection>;
    
    /// Validate code against value set (validateVS operation)
    ///
    /// # Arguments
    /// * `value_set` - Value set URL to validate against
    /// * `coded` - Coded value to validate
    /// * `params` - Optional parameters for validation
    async fn validate_vs(
        &self, 
        value_set: &str, 
        coded: &FhirPathValue, 
        params: Option<HashMap<String, String>>
    ) -> Result<Collection>;
    
    /// Check subsumption relationship (subsumes operation)
    ///
    /// # Arguments  
    /// * `system` - Code system URL
    /// * `coded1` - First coded value
    /// * `coded2` - Second coded value
    /// * `params` - Optional parameters for subsumption check
    async fn subsumes(
        &self, 
        system: &str, 
        coded1: &FhirPathValue, 
        coded2: &FhirPathValue, 
        params: Option<HashMap<String, String>>
    ) -> Result<Collection>;
    
    /// Translate using concept map (translate operation)
    ///
    /// # Arguments
    /// * `concept_map` - Concept map URL
    /// * `coded` - Coded value to translate
    /// * `params` - Optional parameters for translation
    async fn translate(
        &self, 
        concept_map: &str, 
        coded: &FhirPathValue, 
        params: Option<HashMap<String, String>>
    ) -> Result<Collection>;
    
    /// Get terminology server base URL
    fn get_server_url(&self) -> &str;
    
    /// Set authentication credentials
    ///
    /// # Arguments
    /// * `credentials` - Authentication credentials map
    async fn set_credentials(&mut self, credentials: HashMap<String, String>) -> Result<()>;
}

/// Server API trait for FHIRPath %server variable
///
/// Provides FHIR RESTful API operations for the %server built-in variable.
/// All operations support async execution for external server communication.
#[async_trait]
pub trait ServerApi: Send + Sync + std::fmt::Debug {
    /// Read resource by type and id
    ///
    /// # Arguments
    /// * `resource_type` - FHIR resource type (e.g., "Patient")
    /// * `id` - Resource id
    async fn read(&self, resource_type: &str, id: &str) -> Result<FhirPathValue>;
    
    /// Create new resource
    ///
    /// # Arguments
    /// * `resource` - Resource to create
    async fn create(&self, resource: &FhirPathValue) -> Result<FhirPathValue>;
    
    /// Update existing resource
    ///
    /// # Arguments  
    /// * `resource` - Resource to update
    async fn update(&self, resource: &FhirPathValue) -> Result<FhirPathValue>;
    
    /// Delete resource by type and id
    ///
    /// # Arguments
    /// * `resource_type` - FHIR resource type
    /// * `id` - Resource id
    async fn delete(&self, resource_type: &str, id: &str) -> Result<FhirPathValue>;
    
    /// Search for resources with parameters
    ///
    /// # Arguments
    /// * `resource_type` - FHIR resource type to search
    /// * `params` - Search parameters
    async fn search(
        &self, 
        resource_type: &str, 
        params: HashMap<String, String>
    ) -> Result<Collection>;
    
    /// Validate resource against profile
    ///
    /// # Arguments
    /// * `resource` - Resource to validate
    /// * `profile` - Optional profile URL to validate against
    async fn validate(
        &self, 
        resource: &FhirPathValue, 
        profile: Option<&str>
    ) -> Result<Collection>;
    
    /// Execute custom operation
    ///
    /// # Arguments
    /// * `operation_name` - Operation name
    /// * `resource` - Optional resource for operation
    /// * `params` - Operation parameters
    async fn operation(
        &self, 
        operation_name: &str, 
        resource: Option<&FhirPathValue>, 
        params: HashMap<String, String>
    ) -> Result<FhirPathValue>;
    
    /// Get capability statement
    async fn capabilities(&self) -> Result<FhirPathValue>;
    
    /// Get server metadata
    async fn metadata(&self) -> Result<FhirPathValue>;
    
    /// Get server base URL
    fn get_base_url(&self) -> &str;
    
    /// Set server context (auth, timeout, etc.)
    ///
    /// # Arguments
    /// * `context` - Server context configuration
    async fn set_context(&mut self, context: ServerContext) -> Result<()>;
}

/// Builder for creating evaluation contexts with fluent API
///
/// Provides a convenient builder pattern for constructing complex evaluation contexts
/// with various configuration options.
///
/// # Examples
///
/// ```rust,no_run
/// use octofhir_fhirpath::evaluator::*;
/// use std::collections::HashMap;
/// 
/// # async fn example() -> octofhir_fhirpath::Result<()> {
/// let mut variables = HashMap::new();
/// variables.insert("threshold".to_string(), octofhir_fhirpath::FhirPathValue::Integer(30));
/// 
/// let context = EvaluationContextBuilder::new()
///     .with_start_context(octofhir_fhirpath::Collection::empty())
///     .with_variables(variables)
///     .with_fhir_version("r5".to_string())
///     .with_terminology_server("https://custom.tx.server/".to_string())
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct EvaluationContextBuilder {
    /// Context being built
    context: EvaluationContext,
}

impl EvaluationContextBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            context: EvaluationContext::default(),
        }
    }
    
    /// Set start context
    ///
    /// # Arguments
    /// * `start_context` - Initial resource collection
    pub fn with_start_context(mut self, start_context: Collection) -> Self {
        self.context.start_context = start_context;
        self
    }
    
    /// Add single variable
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Variable value
    pub fn with_variable(mut self, name: String, value: FhirPathValue) -> Self {
        self.context.variables.insert(name, value);
        self
    }
    
    /// Add multiple variables
    ///
    /// # Arguments
    /// * `variables` - Variables map to add
    pub fn with_variables(mut self, variables: HashMap<String, FhirPathValue>) -> Self {
        self.context.variables.extend(variables);
        self
    }
    
    /// Set FHIR version
    ///
    /// # Arguments
    /// * `version` - FHIR version (r4, r4b, r5)
    pub fn with_fhir_version(mut self, version: String) -> Self {
        self.context.builtin_variables.fhir_version = version.clone();
        self.context.builtin_variables.terminology_server = format!("https://tx.fhir.org/{}/", version);
        self
    }
    
    /// Set custom terminology server URL
    ///
    /// # Arguments
    /// * `url` - Terminology server URL
    pub fn with_terminology_server(mut self, url: String) -> Self {
        self.context.builtin_variables.terminology_server = url;
        self
    }
    
    /// Set terminology service implementation
    ///
    /// # Arguments
    /// * `service` - Terminology service implementation
    pub fn with_terminology_service(mut self, service: Arc<dyn TerminologyService>) -> Self {
        self.context.builtin_variables.terminologies = Some(service);
        self
    }
    
    /// Set server API implementation
    ///
    /// # Arguments
    /// * `server` - Server API implementation
    pub fn with_server_api(mut self, server: Arc<dyn ServerApi>) -> Self {
        self.context.builtin_variables.server = Some(server);
        self
    }
    
    /// Set type factory implementation
    ///
    /// # Arguments
    /// * `factory` - Type factory implementation
    pub fn with_type_factory(mut self, factory: Arc<dyn TypeFactory>) -> Self {
        self.context.builtin_variables.factory = Some(factory);
        self
    }
    
    /// Set server context
    ///
    /// # Arguments
    /// * `server_context` - Server context configuration
    pub fn with_server_context(mut self, server_context: ServerContext) -> Self {
        self.context.server_context = Some(server_context);
        self
    }
    
    /// Build final context
    pub fn build(self) -> EvaluationContext {
        self.context
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

    #[test]
    fn test_evaluation_context_creation() {
        let context = EvaluationContext::new(Collection::empty());
        assert_eq!(context.depth, 0);
        assert!(context.variables.is_empty());
        assert_eq!(context.builtin_variables.fhir_version, "r4");
    }

    #[test]
    fn test_context_builder() {
        let mut variables = HashMap::new();
        variables.insert("test".to_string(), FhirPathValue::String("value".to_string()));
        
        let context = EvaluationContextBuilder::new()
            .with_fhir_version("r5".to_string())
            .with_variables(variables)
            .with_terminology_server("https://custom.tx.server/".to_string())
            .build();
        
        assert_eq!(context.builtin_variables.fhir_version, "r5");
        assert_eq!(context.builtin_variables.terminology_server, "https://custom.tx.server/");
        assert_eq!(context.variables.len(), 1);
    }

    #[test]
    fn test_builtin_variables_defaults() {
        let builtin = BuiltinVariables::default();
        assert_eq!(builtin.fhir_version, "r4");
        assert_eq!(builtin.terminology_server, "https://tx.fhir.org/r4/");
        
        // Check default environment variables
        assert_eq!(
            builtin.get_environment_variable("%sct"),
            Some(&FhirPathValue::String("http://snomed.info/sct".to_string()))
        );
        assert_eq!(
            builtin.get_environment_variable("%loinc"),
            Some(&FhirPathValue::String("http://loinc.org".to_string()))
        );
    }

    #[test]
    fn test_child_context_creation() {
        let mut parent = EvaluationContext::new(Collection::empty());
        parent.set_variable("parent_var".to_string(), FhirPathValue::String("test".to_string()));
        
        let child = parent.create_child_context(Collection::single(FhirPathValue::String("child".to_string())));
        
        assert_eq!(child.depth, 1);
        assert_eq!(child.variables.len(), 1);
        assert_eq!(child.get_variable("parent_var"), Some(&FhirPathValue::String("test".to_string())));
    }

    #[test]
    fn test_server_context() {
        let context = ServerContext::new("https://hapi.fhir.org/baseR4".to_string())
            .with_auth_header("Authorization".to_string(), "Bearer token123".to_string())
            .with_timeout(60);
        
        assert_eq!(context.base_url, "https://hapi.fhir.org/baseR4");
        assert_eq!(context.timeout_seconds, 60);
        assert_eq!(context.get_auth_header("Authorization"), Some(&"Bearer token123".to_string()));
    }

    #[test]
    fn test_environment_variables() {
        let mut builtin = BuiltinVariables::default();
        
        // Test value set addition
        builtin.add_value_set("allergies", "http://hl7.org/fhir/ValueSet/allergyintolerance-code");
        assert_eq!(
            builtin.get_environment_variable("%vs-allergies"),
            Some(&FhirPathValue::String("http://hl7.org/fhir/ValueSet/allergyintolerance-code".to_string()))
        );
        
        // Test extension addition  
        builtin.add_extension("birthPlace", "http://hl7.org/fhir/StructureDefinition/birthPlace");
        assert_eq!(
            builtin.get_environment_variable("%ext-birthPlace"),
            Some(&FhirPathValue::String("http://hl7.org/fhir/StructureDefinition/birthPlace".to_string()))
        );
    }

    // Mock implementations for testing
    #[derive(Debug)]
    struct MockTerminologyService;
    
    #[async_trait]
    impl TerminologyService for MockTerminologyService {
        async fn expand(&self, _value_set: &str, _params: Option<HashMap<String, String>>) -> Result<Collection> {
            Ok(Collection::empty())
        }
        
        async fn lookup(&self, _coded: &FhirPathValue, _params: Option<HashMap<String, String>>) -> Result<Collection> {
            Ok(Collection::empty())
        }
        
        async fn validate_vs(&self, _value_set: &str, _coded: &FhirPathValue, _params: Option<HashMap<String, String>>) -> Result<Collection> {
            Ok(Collection::empty())
        }
        
        async fn subsumes(&self, _system: &str, _coded1: &FhirPathValue, _coded2: &FhirPathValue, _params: Option<HashMap<String, String>>) -> Result<Collection> {
            Ok(Collection::empty())
        }
        
        async fn translate(&self, _concept_map: &str, _coded: &FhirPathValue, _params: Option<HashMap<String, String>>) -> Result<Collection> {
            Ok(Collection::empty())
        }
        
        fn get_server_url(&self) -> &str {
            "https://tx.fhir.org/r4/"
        }
        
        async fn set_credentials(&mut self, _credentials: HashMap<String, String>) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_terminology_service_integration() {
        let service = Arc::new(MockTerminologyService);
        let context = EvaluationContextBuilder::new()
            .with_terminology_service(service.clone())
            .build();
        
        assert!(context.get_terminology_service().is_some());
    }
}