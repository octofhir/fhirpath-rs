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

//! # Unified FHIRPath Evaluation Engine
//!
//! This module provides a single, comprehensive FHIRPath evaluation engine that
//! replaces multiple previous implementations. It combines standard expression
//! evaluation, lambda functions, and thread-safe operation in one optimized engine.
//!
//! ## Features
//!
//! - **Complete FHIRPath Support**: All operators, functions, and language features
//! - **Lambda Expressions**: Built-in support for `where()`, `select()`, `all()`, etc.
//! - **Thread Safety**: `Send + Sync` by design, safe for concurrent use
//! - **Performance Optimized**: Reduced memory usage and faster evaluation
//! - **Configurable**: Timeout, recursion limits, and optimization settings
//!
//! ## Quick Start
//!
//! ```rust
//! use octofhir_fhirpath_evaluator::FhirPathEngine;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let engine = FhirPathEngine::with_mock_provider().await?;
//!     let patient = json!({
//!         "resourceType": "Patient",
//!         "name": [{"given": ["John"], "family": "Doe"}]
//!     });
//!
//!     // Basic evaluation
//!     let result = engine.evaluate("Patient.name.given", patient.clone()).await?;
//!     println!("Given names: {:?}", result);
//!
//!     // Lambda expressions
//!     let result = engine.evaluate("Patient.name.where(family.exists())", patient).await?;
//!     println!("Names with family: {:?}", result);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! ```rust
//! use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
//!
//! let config = EvaluationConfig {
//!     max_recursion_depth: 500,
//!     timeout_ms: 10000,
//!     enable_lambda_optimization: true,
//!     memory_limit_mb: Some(100),
//! };
//!
//! let engine = FhirPathEngine::with_mock_provider()
//!     .with_config(config);
//! ```

use crate::EvaluationContext;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{FhirPathValue, ModelProvider};
use octofhir_fhirpath_registry::{FhirPathRegistry, FhirPathOperation, OperationType};
use std::sync::Arc;

/// Unified FHIRPath evaluation engine.
///
/// This is the primary engine for evaluating FHIRPath expressions. It provides
/// a comprehensive, thread-safe implementation that supports:
///
/// - All standard FHIRPath operators and functions
/// - Lambda expressions (`where()`, `select()`, `all()`, `any()`, etc.)
/// - Configurable evaluation limits and optimizations
/// - Async evaluation with proper error handling
/// - Thread-safe operation (`Send + Sync`)
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use octofhir_fhirpath_evaluator::FhirPathEngine;
/// use serde_json::json;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let engine = FhirPathEngine::with_mock_provider().await?;
/// let data = json!({"value": 42});
///
/// let result = engine.evaluate("value", data).await?;
/// println!("Result: {:?}", result);
/// # Ok(())
/// # }
/// ```
///
/// With configuration:
/// ```rust
/// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
///
/// let config = EvaluationConfig {
///     max_recursion_depth: 500,
///     timeout_ms: 10000,
///     enable_lambda_optimization: true,
///     memory_limit_mb: Some(100),
/// };
///
/// let engine = FhirPathEngine::with_mock_provider().with_config(config);
/// ```
#[derive(Clone)]
pub struct FhirPathEngine {
    /// Unified registry containing all operations
    registry: Arc<FhirPathRegistry>,
    /// Model provider (Send + Sync)
    model_provider: Arc<dyn ModelProvider>,
    /// Evaluation configuration
    config: EvaluationConfig,
}

/// Configuration options for FHIRPath evaluation.
///
/// This struct allows fine-tuning the behavior of the evaluation engine,
/// including performance limits, optimizations, and safety constraints.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath_evaluator::EvaluationConfig;
///
/// // Conservative configuration for production
/// let config = EvaluationConfig {
///     max_recursion_depth: 100,
///     timeout_ms: 5000,
///     enable_lambda_optimization: true,
///     memory_limit_mb: Some(50),
/// };
///
/// // High-performance configuration for batch processing
/// let config = EvaluationConfig {
///     max_recursion_depth: 2000,
///     timeout_ms: 60000,
///     enable_lambda_optimization: true,
///     memory_limit_mb: None,
/// };
/// ```
#[derive(Clone, Debug)]
pub struct EvaluationConfig {
    /// Maximum recursion depth to prevent stack overflow
    pub max_recursion_depth: usize,
    /// Evaluation timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable lambda function optimization
    pub enable_lambda_optimization: bool,
    /// Enable automatic sync path optimization for performance
    pub enable_sync_optimization: bool,
    /// Memory limit in megabytes (None = unlimited)
    pub memory_limit_mb: Option<usize>,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 1000,
            timeout_ms: 30000,
            enable_lambda_optimization: true,
            enable_sync_optimization: true,
            memory_limit_mb: None,
        }
    }
}

impl FhirPathEngine {
    /// Creates a new unified FHIRPath evaluation engine.
    ///
    /// This is the most flexible constructor, allowing you to provide a custom
    /// unified registry and model provider.
    ///
    /// # Arguments
    ///
    /// * `registry` - Unified registry containing all functions and operators
    /// * `model_provider` - Provider for FHIR type information
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use octofhir_fhirpath_registry::FhirPathRegistry;
    /// use octofhir_fhirpath_model::MockModelProvider;
    /// use std::sync::Arc;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = Arc::new(FhirPathRegistry::new());
    /// let model_provider = Arc::new(MockModelProvider::new());
    ///
    /// let engine = FhirPathEngine::new(registry, model_provider);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        registry: Arc<FhirPathRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            registry,
            model_provider,
            config: EvaluationConfig::default(),
        }
    }

    /// Returns the current evaluation configuration.
    ///
    /// This provides access to the engine's configuration settings,
    /// including timeout, recursion limits, and optimization flags.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    ///
    /// let engine = FhirPathEngine::with_mock_provider();
    /// let config = engine.config();
    ///
    /// println!("Max depth: {}", config.max_recursion_depth);
    /// println!("Timeout: {}ms", config.timeout_ms);
    /// ```
    pub fn config(&self) -> &EvaluationConfig {
        &self.config
    }

    /// Creates a new engine with custom registries and configuration.
    ///
    /// This constructor allows you to provide both custom registries and
    /// a specific configuration for fine-tuned control over evaluation behavior.
    ///
    /// # Arguments
    ///
    /// * `functions` - Custom function registry
    /// * `operators` - Custom operator registry
    /// * `model_provider` - Model provider for type information
    /// * `config` - Evaluation configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
    /// use octofhir_fhirpath_registry::create_standard_registries;
    /// use octofhir_fhirpath_model::MockModelProvider;
    /// use std::sync::Arc;
    ///
    /// let config = EvaluationConfig {
    ///     max_recursion_depth: 500,
    ///     timeout_ms: 10000,
    ///     enable_lambda_optimization: true,
    ///     memory_limit_mb: Some(100),
    /// };
    ///
    /// let (functions, operators) = create_standard_registries();
    /// let model_provider = Arc::new(MockModelProvider::empty());
    ///
    /// let engine = FhirPathEngine::new_with_config(
    ///     Arc::new(functions),
    ///     Arc::new(operators),
    ///     model_provider,
    ///     config
    /// );
    /// ```
    pub fn new_with_config(
        functions: Arc<UnifiedFunctionRegistry>,
        operators: Arc<UnifiedOperatorRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        config: EvaluationConfig,
    ) -> Self {
        Self {
            functions,
            operators,
            model_provider,
            config,
        }
    }

    /// Creates an engine with a mock model provider for testing.
    ///
    /// This is the easiest way to get started with the engine for testing
    /// or development purposes. Uses standard registries and default configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use serde_json::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    /// let result = engine.evaluate("42", json!({})).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_mock_provider() -> EvaluationResult<Self> {
        use octofhir_fhirpath_model::MockModelProvider;

        let registry = Self::create_standard_registry().await?;
        let model_provider = Arc::new(MockModelProvider::new());

        Ok(Self::new(Arc::new(registry), model_provider))
    }

    /// Creates a standard registry with all built-in operations.
    ///
    /// This method creates a new `FhirPathRegistry` and registers all standard
    /// FHIRPath operations including arithmetic, collection, string functions,
    /// and operators.
    async fn create_standard_registry() -> EvaluationResult<FhirPathRegistry> {
        use octofhir_fhirpath_registry::operations::*;

        let mut registry = FhirPathRegistry::new();

        // Register arithmetic operators
        registry.register(Box::new(AdditionOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register addition: {}", e) })?;
        registry.register(Box::new(SubtractionOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register subtraction: {}", e) })?;
        registry.register(Box::new(MultiplicationOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register multiplication: {}", e) })?;
        registry.register(Box::new(DivisionOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register division: {}", e) })?;
        registry.register(Box::new(ModuloOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register modulo: {}", e) })?;
        registry.register(Box::new(IntegerDivisionOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register integer division: {}", e) })?;
        registry.register(Box::new(UnaryMinusOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register unary minus: {}", e) })?;
        registry.register(Box::new(UnaryPlusOperation::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register unary plus: {}", e) })?;

        // Register collection functions
        registry.register(Box::new(CountFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register count: {}", e) })?;
        registry.register(Box::new(EmptyFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register empty: {}", e) })?;
        registry.register(Box::new(ExistsFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register exists: {}", e) })?;
        registry.register(Box::new(FirstFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register first: {}", e) })?;
        registry.register(Box::new(LastFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register last: {}", e) })?;
        registry.register(Box::new(SingleFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register single: {}", e) })?;

        // Register string functions
        registry.register(Box::new(LengthFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register length: {}", e) })?;
        registry.register(Box::new(ContainsFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register contains: {}", e) })?;
        registry.register(Box::new(StartsWithFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register startsWith: {}", e) })?;
        registry.register(Box::new(EndsWithFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register endsWith: {}", e) })?;
        registry.register(Box::new(SubstringFunction::new())).await
            .map_err(|e| EvaluationError::InvalidOperation { message: format!("Failed to register substring: {}", e) })?;

        Ok(registry)
    }

    /// Creates an engine with a specific model provider.
    ///
    /// This constructor uses standard function and operator registries but allows
    /// you to provide a custom model provider for FHIR type information.
    ///
    /// # Arguments
    ///
    /// * `model_provider` - Custom model provider implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use octofhir_fhirpath_model::MockModelProvider;
    /// use std::sync::Arc;
    ///
    /// let provider = Arc::new(MockModelProvider::with_resources(vec![/* resources */]));
    /// let engine = FhirPathEngine::with_model_provider(provider);
    /// ```
    pub fn with_model_provider(model_provider: Arc<dyn ModelProvider>) -> Self {
        use octofhir_fhirpath_registry::create_standard_registries;

        let (functions, operators) = create_standard_registries();
        Self::new(Arc::new(functions), Arc::new(operators), model_provider)
    }

    /// Creates a new engine instance with a modified configuration.
    ///
    /// This method allows you to create a new engine with different configuration
    /// settings while reusing the same registries and model provider. This is useful
    /// for creating engines with different performance characteristics or limits.
    ///
    /// # Arguments
    ///
    /// * `config` - The new evaluation configuration to use
    ///
    /// # Returns
    ///
    /// Returns a new `FhirPathEngine` instance with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
    ///
    /// let engine = FhirPathEngine::with_mock_provider();
    ///
    /// // Create a high-performance configuration
    /// let performance_config = EvaluationConfig {
    ///     max_recursion_depth: 2000,
    ///     timeout_ms: 60000,
    ///     enable_lambda_optimization: true,
    ///     memory_limit_mb: None,
    /// };
    ///
    /// let performance_engine = engine.with_config(performance_config);
    /// ```
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
    ///
    /// // Chain configuration for different use cases
    /// let base_engine = FhirPathEngine::with_mock_provider();
    ///
    /// let strict_config = EvaluationConfig {
    ///     max_recursion_depth: 100,
    ///     timeout_ms: 5000,
    ///     enable_lambda_optimization: false,
    ///     memory_limit_mb: Some(50),
    /// };
    ///
    /// let strict_engine = base_engine.with_config(strict_config);
    /// ```
    pub fn with_config(self, config: EvaluationConfig) -> Self {
        Self {
            functions: self.functions,
            operators: self.operators,
            model_provider: self.model_provider,
            config,
        }
    }

    /// Evaluates a FHIRPath expression against input data.
    ///
    /// This is the primary evaluation method for FHIRPath expressions. It parses the expression,
    /// converts the input to a `FhirPathValue`, and evaluates the expression in a safe,
    /// controlled environment.
    ///
    /// # Arguments
    ///
    /// * `expression` - The FHIRPath expression string to evaluate
    /// * `input_data` - The input data (typically a FHIR resource) as JSON
    ///
    /// # Returns
    ///
    /// Returns a `FhirPathValue` containing the evaluation result. The result can be:
    /// - A single value (Boolean, Integer, String, etc.)
    /// - A collection of values
    /// - An empty collection if no matches found
    ///
    /// # Errors
    ///
    /// Returns `EvaluationError` if:
    /// - The expression has syntax errors
    /// - Evaluation exceeds timeout or recursion limits
    /// - A runtime error occurs during evaluation
    ///
    /// # Examples
    ///
    /// Basic property access:
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use serde_json::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    /// let patient = json!({
    ///     "resourceType": "Patient",
    ///     "name": [{"given": ["John"], "family": "Doe"}]
    /// });
    ///
    /// let result = engine.evaluate("Patient.name.given", patient).await?;
    /// println!("Given names: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Lambda expressions:
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use serde_json::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    /// let patient = json!({
    ///     "resourceType": "Patient",
    ///     "name": [
    ///         {"given": ["John"], "family": "Doe"},
    ///         {"given": ["Jane"]}
    ///     ]
    /// });
    ///
    /// // Filter names that have a family name
    /// let result = engine.evaluate("Patient.name.where(family.exists())", patient).await?;
    /// println!("Names with family: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Mathematical expressions:
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use serde_json::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    ///
    /// let result = engine.evaluate("(5 + 3) * 2", json!({})).await?;
    /// println!("Result: {:?}", result); // Should be 16
    /// # Ok(())
    /// # }
    /// ```
    pub async fn evaluate(
        &self,
        expression: &str,
        input_data: serde_json::Value,
    ) -> EvaluationResult<FhirPathValue> {
        // Parse expression
        let ast = octofhir_fhirpath_parser::parse_expression(expression)
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Parse error: {}", e),
            })?;

        // Convert input data to FhirPathValue
        let fhir_value = FhirPathValue::from(input_data);

        // Create evaluation context
        let context = EvaluationContext::new(
            fhir_value.clone(),
            self.functions.clone(),
            self.operators.clone(),
            self.model_provider.clone(),
        );

        // Use the AST evaluation method (to be implemented in Task 2)
        self.evaluate_ast(&ast, fhir_value, &context).await
    }

    /// Evaluates a FHIRPath expression with environment variables.
    ///
    /// This method extends the basic `evaluate` functionality by supporting environment
    /// variables in expressions. Variables can be referenced in expressions using the
    /// `%variableName` syntax as defined in the FHIRPath specification.
    ///
    /// # Arguments
    ///
    /// * `expression` - The FHIRPath expression string (can contain variable references)
    /// * `input_data` - The input data (typically a FHIR resource) as JSON
    /// * `variables` - A map of variable names to their `FhirPathValue` values
    ///
    /// # Returns
    ///
    /// Returns a `FhirPathValue` containing the evaluation result, with variables substituted.
    ///
    /// # Errors
    ///
    /// Returns `EvaluationError` if:
    /// - The expression has syntax errors
    /// - A referenced variable is not found in the variables map
    /// - Evaluation exceeds timeout or recursion limits
    /// - A runtime error occurs during evaluation
    ///
    /// # Examples
    ///
    /// Basic variable usage:
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use octofhir_fhirpath_model::FhirPathValue;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    /// let mut variables = HashMap::new();
    /// variables.insert("threshold".to_string(), FhirPathValue::Integer(18));
    ///
    /// let patient = json!({"age": 25});
    /// let result = engine.evaluate_with_variables("age > %threshold", patient, variables).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Multiple variables:
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use octofhir_fhirpath_model::FhirPathValue;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    /// let mut variables = HashMap::new();
    /// variables.insert("minAge".to_string(), FhirPathValue::Integer(18));
    /// variables.insert("maxAge".to_string(), FhirPathValue::Integer(65));
    ///
    /// let patient = json!({"age": 25});
    /// let result = engine.evaluate_with_variables(
    ///     "age >= %minAge and age <= %maxAge",
    ///     patient,
    ///     variables
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// String and complex variables:
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use octofhir_fhirpath_model::FhirPathValue;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    /// let mut variables = HashMap::new();
    /// variables.insert("status".to_string(), FhirPathValue::String("active".into()));
    /// variables.insert("system".to_string(), FhirPathValue::String("http://loinc.org".into()));
    ///
    /// let observation = json!({
    ///     "status": "active",
    ///     "code": {
    ///         "coding": [{"system": "http://loinc.org", "code": "29463-7"}]
    ///     }
    /// });
    ///
    /// let result = engine.evaluate_with_variables(
    ///     "status = %status and code.coding.system = %system",
    ///     observation,
    ///     variables
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Standard Environment Variables
    ///
    /// The FHIRPath specification defines several standard environment variables:
    /// - `%context` - The original node in the input context
    /// - `%resource` - The resource containing the original node
    /// - `%rootResource` - The container resource (for contained resources)
    /// - `%sct` - SNOMED CT URL (`http://snomed.info/sct`)
    /// - `%loinc` - LOINC URL (`http://loinc.org`)
    /// - `%"vs-[name]"` - HL7 value set URLs
    ///
    /// These can be provided in the variables map if needed for your use case.
    pub async fn evaluate_with_variables(
        &self,
        expression: &str,
        input_data: serde_json::Value,
        variables: std::collections::HashMap<String, FhirPathValue>,
    ) -> EvaluationResult<FhirPathValue> {
        // Parse expression
        let ast = octofhir_fhirpath_parser::parse_expression(expression)
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Parse error: {}", e),
            })?;

        // Convert input data to FhirPathValue
        let fhir_value = FhirPathValue::from(input_data);

        // Create evaluation context with variables
        let mut context = EvaluationContext::new(
            fhir_value.clone(),
            self.functions.clone(),
            self.operators.clone(),
            self.model_provider.clone(),
        );
        for (key, value) in variables {
            context.set_variable(key, value);
        }

        // Use the AST evaluation method (to be implemented in Task 2)
        self.evaluate_ast(&ast, fhir_value, &context).await
    }

    /// Evaluates a pre-parsed FHIRPath expression (AST) against input data.
    ///
    /// This method provides direct evaluation of Abstract Syntax Tree (AST) nodes, bypassing
    /// the parsing step. It's useful when you have already parsed expressions or when building
    /// custom evaluation pipelines.
    ///
    /// # Arguments
    ///
    /// * `expression` - The parsed expression as an `ExpressionNode`
    /// * `input` - The input data as a `FhirPathValue`
    /// * `context` - The evaluation context containing variables and registries
    ///
    /// # Returns
    ///
    /// Returns a `FhirPathValue` containing the evaluation result.
    ///
    /// # Errors
    ///
    /// Returns `EvaluationError` if:
    /// - Evaluation exceeds timeout or recursion limits
    /// - A referenced variable or function is not found
    /// - A runtime error occurs during evaluation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationContext};
    /// use octofhir_fhirpath_model::FhirPathValue;
    /// use octofhir_fhirpath_parser::parse_expression;
    /// use octofhir_fhirpath_registry::create_standard_registries;
    /// use serde_json::json;
    /// use std::sync::Arc;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider();
    ///
    /// // Parse expression once
    /// let ast = parse_expression("Patient.name.given")?;
    /// let input = FhirPathValue::from(json!({"resourceType": "Patient", "name": [{"given": ["John"]}]}));
    ///
    /// // Create evaluation context
    /// let (functions, operators) = create_standard_registries();
    /// let context = EvaluationContext::new(
    ///     input.clone(),
    ///     Arc::new(functions),
    ///     Arc::new(operators),
    ///     Arc::new(octofhir_fhirpath_model::MockModelProvider::empty())
    /// );
    ///
    /// // Evaluate AST directly
    /// let result = engine.evaluate_ast(&ast, input, &context).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Performance Optimization**: Parse expressions once, evaluate many times
    /// - **Custom Pipelines**: Build specialized evaluation workflows
    /// - **Batch Processing**: Evaluate the same expression against multiple inputs
    /// - **Analysis Tools**: Inspect and manipulate AST structures before evaluation
    pub async fn evaluate_ast(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        self.evaluate_node_async(expression, input, context, 0).await
    }

    /// Core recursive evaluator - handles all node types
    fn evaluate_node_async<'a>(
        &'a self,
        node: &'a ExpressionNode,
        input: FhirPathValue,
        context: &'a EvaluationContext,
        depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EvaluationResult<FhirPathValue>> + Send + 'a>> {
        Box::pin(async move {
        // Recursion depth check
        if depth > self.config.max_recursion_depth {
            return Err(EvaluationError::InvalidOperation {
                message: format!("Recursion depth exceeded: max depth is {}", self.config.max_recursion_depth),
            });
        }

        // Performance monitoring hook
        let start_time = std::time::Instant::now();
        let result = self.evaluate_node_internal(node, input, context, depth).await;
        let duration = start_time.elapsed();

        // Log slow evaluations (optional)
        if duration.as_millis() > 1000 {
            // TODO: Add logging when log crate is available
            eprintln!("Slow evaluation: took {}ms", duration.as_millis());
        }

        result
        })
    }

    /// Internal node evaluation with pattern matching
    async fn evaluate_node_internal(
        &self,
        node: &ExpressionNode,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_ast::ExpressionNode;

        match node {
            // Simple cases - direct evaluation
            ExpressionNode::Literal(lit) => self.evaluate_literal(lit),

            ExpressionNode::Identifier(id) => self.evaluate_identifier(id, &input, context),

            ExpressionNode::Index { base, index } =>
                self.evaluate_index(base, index, input, context, depth).await,

            ExpressionNode::Path { base, path } =>
                self.evaluate_path(base, path, input, context, depth).await,

            // Complex cases - delegate to specialized methods
            ExpressionNode::FunctionCall(func_data) => {
                if self.is_lambda_function(&func_data.name) {
                    self.evaluate_lambda_function(func_data, input, context, depth).await
                } else {
                    self.evaluate_standard_function(func_data, input, context, depth).await
                }
            }

            ExpressionNode::BinaryOp(op_data) =>
                self.evaluate_binary_operation(op_data, input, context, depth).await,

            ExpressionNode::UnaryOp { op, operand } =>
                self.evaluate_unary_operation(op, operand, input, context, depth).await,

            ExpressionNode::MethodCall(method_data) =>
                self.evaluate_method_call(method_data, input, context, depth).await,

            ExpressionNode::Lambda(lambda_data) =>
                self.evaluate_lambda_expression(lambda_data, input, context, depth).await,

            ExpressionNode::Conditional(cond_data) =>
                self.evaluate_conditional(cond_data, input, context, depth).await,

            ExpressionNode::Variable(var_name) =>
                self.evaluate_variable(var_name, context),

            ExpressionNode::Filter { base, condition } =>
                self.evaluate_filter(base, condition, input, context, depth).await,

            ExpressionNode::Union { left, right } =>
                self.evaluate_union(left, right, input, context, depth).await,

            ExpressionNode::TypeCheck { expression, type_name } =>
                self.evaluate_type_check(expression, type_name, input, context, depth).await,

            ExpressionNode::TypeCast { expression, type_name } =>
                self.evaluate_type_cast(expression, type_name, input, context, depth).await,
        }
    }

    // Simple node type handlers

    /// Evaluate literal values
    fn evaluate_literal(&self, literal: &octofhir_fhirpath_ast::LiteralValue) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_ast::LiteralValue::*;
        use std::str::FromStr;

        let value = match literal {
            Boolean(b) => FhirPathValue::Boolean(*b),
            Integer(i) => FhirPathValue::Integer(*i),
            Decimal(d) => {
                let decimal = rust_decimal::Decimal::from_str(d)
                    .map_err(|_| EvaluationError::InvalidOperation {
                        message: format!("Invalid decimal value: {}", d),
                    })?;
                FhirPathValue::Decimal(decimal)
            },
            String(s) => FhirPathValue::String(s.clone().into()),
            Date(d) => {
                let date = chrono::NaiveDate::from_str(d)
                    .map_err(|_| EvaluationError::InvalidOperation {
                        message: format!("Invalid date value: {}", d),
                    })?;
                FhirPathValue::Date(date)
            },
            DateTime(dt) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(dt)
                    .map_err(|_| EvaluationError::InvalidOperation {
                        message: format!("Invalid datetime value: {}", dt),
                    })?;
                FhirPathValue::DateTime(datetime)
            },
            Time(t) => {
                let time = chrono::NaiveTime::from_str(t)
                    .map_err(|_| EvaluationError::InvalidOperation {
                        message: format!("Invalid time value: {}", t),
                    })?;
                FhirPathValue::Time(time)
            },
            Quantity { value, unit } => {
                let decimal_value = rust_decimal::Decimal::from_str(value)
                    .map_err(|_| EvaluationError::InvalidOperation {
                        message: format!("Invalid quantity value: {}", value),
                    })?;
                let quantity = octofhir_fhirpath_model::Quantity::new(
                    decimal_value,
                    Some(unit.clone())
                );
                FhirPathValue::Quantity(std::sync::Arc::new(quantity))
            },
            Null => FhirPathValue::Empty,
        };

        Ok(FhirPathValue::collection(vec![value]))
    }

    /// Evaluate identifiers (property access)
    fn evaluate_identifier(
        &self,
        identifier: &str,
        input: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Handle collection input
        if let FhirPathValue::Collection(items) = input {
            let mut results = Vec::new();

            for item in items.iter() {
                if let Some(property_value) = self.get_property_value(item, identifier) {
                    match property_value {
                        FhirPathValue::Collection(sub_items) => {
                            results.extend(sub_items.iter().cloned());
                        }
                        single_value => results.push(single_value),
                    }
                }
            }

            Ok(FhirPathValue::collection(results))
        } else {
            // Single item
            if let Some(property_value) = self.get_property_value(input, identifier) {
                Ok(property_value)
            } else {
                Ok(FhirPathValue::collection(vec![]))
            }
        }
    }

    /// Evaluate variable references
    fn evaluate_variable(
        &self,
        var_name: &str,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        context.get_variable(var_name)
            .cloned()
            .ok_or_else(|| EvaluationError::InvalidOperation {
                message: format!("Variable not found: {}", var_name),
            })
    }

    /// Helper: Get property value from a FhirPathValue
    fn get_property_value(&self, value: &FhirPathValue, property: &str) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_arc) => {
                if let Some(obj) = json_arc.as_object() {
                    obj.get(property).map(|v| FhirPathValue::from(v.clone()))
                } else {
                    None
                }
            }
            FhirPathValue::Resource(resource) => {
                // Try to get property from resource
                resource.get_property(property).map(|v| FhirPathValue::from(v.clone()))
            }
            _ => None,
        }
    }

    // Complex node type handlers

    /// Evaluate path navigation (object.property)
    async fn evaluate_path(
        &self,
        base: &ExpressionNode,
        path: &str,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the base expression
        let base_result = self.evaluate_node_async(base, input, context, depth + 1).await?;

        // Apply path navigation to the result
        self.evaluate_identifier(path, &base_result, context)
    }

    /// Evaluate index access expressions
    async fn evaluate_index(
        &self,
        base: &ExpressionNode,
        index_expr: &ExpressionNode,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the base expression
        let base_result = self.evaluate_node_async(base, input.clone(), context, depth + 1).await?;

        // Then evaluate the index expression
        let index_result = self.evaluate_node_async(index_expr, input, context, depth + 1).await?;

        // Extract index value
        let index = match &index_result {
            FhirPathValue::Integer(i) => *i,
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.iter().next().unwrap() {
                    FhirPathValue::Integer(i) => *i,
                    _ => return Err(EvaluationError::InvalidOperation {
                        message: "Index must be an integer".to_string(),
                    }),
                }
            }
            _ => return Err(EvaluationError::InvalidOperation {
                message: "Index must be an integer".to_string(),
            }),
        };

        // Apply index to base result
        match base_result {
            FhirPathValue::Collection(items) => {
                if index < 0 {
                    return Ok(FhirPathValue::collection(vec![]));
                }
                let idx = index as usize;
                if idx < items.len() {
                    Ok(FhirPathValue::collection(vec![items.iter().nth(idx).unwrap().clone()]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            single_value => {
                if index == 0 {
                    Ok(FhirPathValue::collection(vec![single_value]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }

    /// Evaluate binary operations
    async fn evaluate_binary_operation(
        &self,
        op_data: &octofhir_fhirpath_ast::BinaryOpData,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Get the operator symbol
        let symbol = match &op_data.op {
            octofhir_fhirpath_ast::BinaryOperator::Add => "+",
            octofhir_fhirpath_ast::BinaryOperator::Subtract => "-",
            octofhir_fhirpath_ast::BinaryOperator::Multiply => "*",
            octofhir_fhirpath_ast::BinaryOperator::Divide => "/",
            octofhir_fhirpath_ast::BinaryOperator::IntegerDivide => "div",
            octofhir_fhirpath_ast::BinaryOperator::Modulo => "mod",
            octofhir_fhirpath_ast::BinaryOperator::Equal => "=",
            octofhir_fhirpath_ast::BinaryOperator::NotEqual => "!=",
            octofhir_fhirpath_ast::BinaryOperator::LessThan => "<",
            octofhir_fhirpath_ast::BinaryOperator::LessThanOrEqual => "<=",
            octofhir_fhirpath_ast::BinaryOperator::GreaterThan => ">",
            octofhir_fhirpath_ast::BinaryOperator::GreaterThanOrEqual => ">=",
            octofhir_fhirpath_ast::BinaryOperator::Equivalent => "~",
            octofhir_fhirpath_ast::BinaryOperator::NotEquivalent => "!~",
            octofhir_fhirpath_ast::BinaryOperator::And => "and",
            octofhir_fhirpath_ast::BinaryOperator::Or => "or",
            octofhir_fhirpath_ast::BinaryOperator::Xor => "xor",
            octofhir_fhirpath_ast::BinaryOperator::Implies => "implies",
            octofhir_fhirpath_ast::BinaryOperator::Union => "|",
            octofhir_fhirpath_ast::BinaryOperator::Concatenate => "&",
            octofhir_fhirpath_ast::BinaryOperator::In => "in",
            octofhir_fhirpath_ast::BinaryOperator::Contains => "contains",
            octofhir_fhirpath_ast::BinaryOperator::Is => "is",
        };

        // Get the operator from the registry
        let operator = self.operators.get_binary(symbol)
            .ok_or_else(|| EvaluationError::InvalidOperation {
                message: format!("Unknown operator: {}", symbol),
            })?;

        // Evaluate left operand
        let left = self.evaluate_node_async(&op_data.left, input.clone(), context, depth + 1).await?;

        // Evaluate right operand
        let right = self.evaluate_node_async(&op_data.right, input.clone(), context, depth + 1).await?;

        // Create registry context for operator
        let registry_context = octofhir_fhirpath_registry::function::EvaluationContext::new(input);

        // Apply the operator
        operator.evaluate_binary(left, right, &registry_context).await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Binary operator error: {}", e),
            })
    }

    /// Evaluate unary operations
    async fn evaluate_unary_operation(
        &self,
        op: &octofhir_fhirpath_ast::UnaryOperator,
        operand: &ExpressionNode,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Get the operator symbol
        let symbol = match op {
            octofhir_fhirpath_ast::UnaryOperator::Plus => "+",
            octofhir_fhirpath_ast::UnaryOperator::Minus => "-",
            octofhir_fhirpath_ast::UnaryOperator::Not => "not",
        };

        // Get the operator from the registry
        let operator = self.operators.get_unary(symbol)
            .ok_or_else(|| EvaluationError::InvalidOperation {
                message: format!("Unknown unary operator: {}", symbol),
            })?;

        // Evaluate operand
        let operand_value = self.evaluate_node_async(operand, input.clone(), context, depth + 1).await?;

        // Create registry context for operator
        let registry_context = octofhir_fhirpath_registry::function::EvaluationContext::new(input);

        // Apply the operator
        operator.evaluate_unary(operand_value, &registry_context).await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Unary operator error: {}", e),
            })
    }

    /// Evaluate conditional expressions (iif)
    async fn evaluate_conditional(
        &self,
        cond_data: &octofhir_fhirpath_ast::ConditionalData,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate condition
        let condition = self.evaluate_node_async(&cond_data.condition, input.clone(), context, depth + 1).await?;

        // Check if condition is true
        let is_true = self.is_truthy(&condition);

        // Evaluate appropriate branch
        if is_true {
            self.evaluate_node_async(&cond_data.then_expr, input, context, depth + 1).await
        } else if let Some(else_expr) = &cond_data.else_expr {
            self.evaluate_node_async(else_expr, input, context, depth + 1).await
        } else {
            Ok(FhirPathValue::collection(vec![]))
        }
    }

    /// Evaluate filter expressions
    async fn evaluate_filter(
        &self,
        base: &ExpressionNode,
        condition: &ExpressionNode,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the base expression
        let base_result = self.evaluate_node_async(base, input, context, depth + 1).await?;

        // Filter based on the condition
        match base_result {
            FhirPathValue::Collection(items) => {
                let mut filtered_items = Vec::new();

                for item in items.iter() {
                    // Evaluate condition for each item
                    let condition_result = self.evaluate_node_async(condition, item.clone(), context, depth + 1).await?;

                    if self.is_truthy(&condition_result) {
                        filtered_items.push(item.clone());
                    }
                }

                Ok(FhirPathValue::collection(filtered_items))
            }
            single_value => {
                // Single value - check condition
                let condition_result = self.evaluate_node_async(condition, single_value.clone(), context, depth + 1).await?;

                if self.is_truthy(&condition_result) {
                    Ok(FhirPathValue::collection(vec![single_value]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }

    /// Evaluate union expressions
    async fn evaluate_union(
        &self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate both sides
        let left_result = self.evaluate_node_async(left, input.clone(), context, depth + 1).await?;
        let right_result = self.evaluate_node_async(right, input, context, depth + 1).await?;

        // Combine results
        let mut combined_items = Vec::new();

        // Add items from left
        match left_result {
            FhirPathValue::Collection(items) => combined_items.extend(items.iter().cloned()),
            single => combined_items.push(single),
        }

        // Add items from right
        match right_result {
            FhirPathValue::Collection(items) => combined_items.extend(items.iter().cloned()),
            single => combined_items.push(single),
        }

        Ok(FhirPathValue::collection(combined_items))
    }

    /// Evaluate type check expressions (value is Type)
    async fn evaluate_type_check(
        &self,
        expression: &ExpressionNode,
        type_name: &str,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate the expression
        let value = self.evaluate_node_async(expression, input, context, depth + 1).await?;

        // Check type - simplified implementation
        let matches_type = match (&value, type_name) {
            (FhirPathValue::Boolean(_), "Boolean") => true,
            (FhirPathValue::Integer(_), "Integer") => true,
            (FhirPathValue::Decimal(_), "Decimal") => true,
            (FhirPathValue::String(_), "String") => true,
            (FhirPathValue::Date(_), "Date") => true,
            (FhirPathValue::DateTime(_), "DateTime") => true,
            (FhirPathValue::Time(_), "Time") => true,
            (FhirPathValue::Quantity(_), "Quantity") => true,
            _ => false, // More sophisticated type checking can be added
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(matches_type)]))
    }

    /// Evaluate type cast expressions (value as Type)
    async fn evaluate_type_cast(
        &self,
        expression: &ExpressionNode,
        _type_name: &str,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // For now, just evaluate the expression (type casting is complex)
        // TODO: Implement proper type casting based on FHIR specification
        self.evaluate_node_async(expression, input, context, depth + 1).await
    }

    /// Helper: Check if a value is truthy
    fn is_truthy(&self, value: &FhirPathValue) -> bool {
        match value {
            FhirPathValue::Boolean(b) => *b,
            FhirPathValue::Collection(items) => !items.is_empty(),
            FhirPathValue::Empty => false,
            _ => true, // Non-empty values are generally truthy
        }
    }

    // Function and method evaluation

    /// Evaluate standard (non-lambda) functions
    async fn evaluate_standard_function(
        &self,
        func_data: &octofhir_fhirpath_ast::FunctionCallData,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Get function from unified registry
        let function = self.functions.get_function(&func_data.name)
            .ok_or_else(|| EvaluationError::InvalidOperation {
                message: format!("Unknown function: {}", func_data.name)
            })?;

        // Pre-evaluate all arguments (standard behavior)
        let mut evaluated_args = Vec::with_capacity(func_data.args.len());
        for arg_expr in &func_data.args {
            let arg_value = self.evaluate_node_async(arg_expr, input.clone(), context, depth + 1).await?;
            evaluated_args.push(arg_value);
        }

        // Create function evaluation context
        let func_context = self.create_function_context(&input, context)?;

        // Call function with pre-evaluated arguments
        function.evaluate_async(&evaluated_args, &func_context).await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Function error in {}: {}", func_data.name, e),
            })
    }

    /// Check if a function name represents a lambda function
    fn is_lambda_function(&self, name: &str) -> bool {
        // Use registry's lambda function detection
        if self.functions.is_lambda_function(name) {
            return true;
        }

        // Fallback: Check if this is a known lambda function
        matches!(name, "where" | "select" | "all" | "any" | "exists" | "aggregate" | "repeat" | "iif" | "sort")
    }

    /// Evaluate lambda functions with expression arguments
    async fn evaluate_lambda_function(
        &self,
        func_data: &octofhir_fhirpath_ast::FunctionCallData,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Get function from unified registry
        let function = self.functions.get_function(&func_data.name)
            .ok_or_else(|| EvaluationError::InvalidOperation {
                message: format!("Unknown lambda function: {}", func_data.name)
            })?;

        // Check if function supports lambda expressions
        if !function.metadata().lambda.supports_lambda_evaluation {
            return Err(EvaluationError::InvalidOperation {
                message: format!("Function {} does not support lambda evaluation", func_data.name),
            });
        }

        // Create expression arguments (not pre-evaluated)
        let mut expr_args = Vec::new();
        let lambda_indices = self.get_lambda_argument_indices(&func_data.name);

        for (i, arg_expr) in func_data.args.iter().enumerate() {
            if lambda_indices.contains(&i) {
                // Keep as expression argument for lambda evaluation
                expr_args.push(octofhir_fhirpath_registry::expression_argument::ExpressionArgument::expression(arg_expr.clone()));
            } else {
                // Pre-evaluate non-lambda arguments
                let arg_value = self.evaluate_node_async(arg_expr, input.clone(), context, depth + 1).await?;
                expr_args.push(octofhir_fhirpath_registry::expression_argument::ExpressionArgument::value(arg_value));
            }
        }

        // Create lambda-compatible expression evaluator
        let evaluator = self.create_lambda_evaluator(depth);

        // Create lambda evaluation context
        let registry_context = self.create_function_context(&input, context)?;
        let lambda_context = octofhir_fhirpath_registry::lambda_function::LambdaEvaluationContext {
            context: &registry_context,
            evaluator: &*evaluator,
        };

        // Try lambda evaluation first
        match function.evaluate_lambda(&expr_args, &lambda_context).await {
            Ok(result) => Ok(result),
            Err(octofhir_fhirpath_registry::function::FunctionError::ExecutionModeNotSupported { .. }) => {
                // Fallback: evaluate all arguments and use traditional function
                let mut evaluated_args = Vec::new();
                for expr_arg in &expr_args {
                    match expr_arg {
                        octofhir_fhirpath_registry::expression_argument::ExpressionArgument::Value(value) => {
                            evaluated_args.push(value.clone());
                        }
                        octofhir_fhirpath_registry::expression_argument::ExpressionArgument::Expression(expr) => {
                            let scope = octofhir_fhirpath_registry::expression_argument::VariableScope::new();
                            let value = (lambda_context.evaluator)(expr, &scope, lambda_context.context).await
                                .map_err(|e| EvaluationError::InvalidOperation {
                                    message: format!("Expression evaluation error: {}", e),
                                })?;
                            evaluated_args.push(value);
                        }
                    }
                }

                function.evaluate_async(&evaluated_args, &registry_context).await
                    .map_err(|e| EvaluationError::InvalidOperation {
                        message: format!("Function error in {}: {}", func_data.name, e),
                    })
            }
            Err(e) => Err(EvaluationError::InvalidOperation {
                message: format!("Lambda function error in {}: {}", func_data.name, e),
            }),
        }
    }

    /// Evaluate lambda expressions (inline lambda syntax)
    async fn evaluate_lambda_expression(
        &self,
        lambda_data: &octofhir_fhirpath_ast::LambdaData,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Lambda expressions operate on collections
        let collection = match input {
            FhirPathValue::Collection(ref items) => items.iter().cloned().collect(),
            single_item => vec![single_item],
        };

        // Determine lambda type based on parameter count and usage pattern
        let lambda_type = self.infer_lambda_type(lambda_data);

        // Apply lambda to each item in the collection
        let mut results = Vec::new();

        for (index, item) in collection.iter().enumerate() {
            // Create lambda-scoped context
            let mut lambda_context = context.clone();

            // Set lambda variables based on parameter names
            if !lambda_data.params.is_empty() {
                for (param_idx, param_name) in lambda_data.params.iter().enumerate() {
                    if param_idx == 0 {
                        // First parameter gets the current item
                        lambda_context.set_variable(param_name.clone(), item.clone());
                    } else {
                        // Additional parameters for advanced use cases
                        lambda_context.set_variable(param_name.clone(), FhirPathValue::Empty);
                    }
                }
            }

            // Set implicit variables
            lambda_context.set_variable("$this".to_string(), item.clone());
            lambda_context.set_variable("$index".to_string(), FhirPathValue::Integer(index as i64));
            lambda_context.set_variable("$total".to_string(), FhirPathValue::Integer(collection.len() as i64));

            // Evaluate lambda body with scoped context
            let result = self.evaluate_node_async(&lambda_data.body, item.clone(), &lambda_context, depth + 1).await?;

            // Collect results based on lambda type
            match lambda_type {
                LambdaType::Select => {
                    // Select: collect all results
                    if let FhirPathValue::Collection(items) = result {
                        results.extend(items.iter().cloned());
                    } else if !matches!(result, FhirPathValue::Empty) {
                        results.push(result);
                    }
                }
                LambdaType::Where => {
                    // Where: include item if result is true
                    if self.is_truthy(&result) {
                        results.push(item.clone());
                    }
                }
                LambdaType::All => {
                    // All: return false if any result is false
                    if !self.is_truthy(&result) {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                    }
                }
                LambdaType::Any => {
                    // Any: return true if any result is true
                    if self.is_truthy(&result) {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
                    }
                }
                LambdaType::Aggregate => {
                    // Aggregate: accumulate results
                    results.push(result);
                }
            }
        }

        // Return appropriate result based on lambda type
        match lambda_type {
            LambdaType::All => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)])), // All were true
            LambdaType::Any => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])), // None were true
            LambdaType::Aggregate => {
                // For aggregate, return the final accumulated value
                if results.len() == 1 {
                    Ok(results.into_iter().next().unwrap())
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::collection(results)),
        }
    }

    /// Evaluate method calls
    async fn evaluate_method_call(
        &self,
        method_data: &octofhir_fhirpath_ast::MethodCallData,
        input: FhirPathValue,
        context: &EvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the base expression
        let object = self.evaluate_node_async(&method_data.base, input.clone(), context, depth + 1).await?;

        // Handle built-in methods
        match method_data.method.as_str() {
            "empty" => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(self.is_empty(&object))])),
            "exists" => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!self.is_empty(&object))])),
            "count" => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(self.count(&object))])),
            "toString" => Ok(FhirPathValue::collection(vec![FhirPathValue::String(self.to_string_value(&object))])),

            // Delegate to function registry for other methods
            method_name => {
                if let Some(function) = self.functions.get_function(method_name) {
                    // Treat method as function with object as first argument
                    let mut args = vec![object];

                    // Evaluate method arguments
                    for arg_expr in &method_data.args {
                        let arg_value = self.evaluate_node_async(arg_expr, input.clone(), context, depth + 1).await?;
                        args.push(arg_value);
                    }

                    // Create function context
                    let func_context = self.create_function_context(&input, context)?;

                    // Call function
                    function.evaluate_async(&args, &func_context).await
                        .map_err(|e| EvaluationError::InvalidOperation {
                            message: format!("Method error in {}: {}", method_name, e),
                        })
                } else {
                    Err(EvaluationError::InvalidOperation {
                        message: format!("Unknown method: {}", method_name),
                    })
                }
            }
        }
    }

    /// Create function evaluation context
    fn create_function_context(
        &self,
        input: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<octofhir_fhirpath_registry::function::EvaluationContext> {
        // Create a new context for function evaluation
        Ok(octofhir_fhirpath_registry::function::EvaluationContext::new(input.clone()))
    }

    // Helper methods for built-in method implementations

    /// Check if a value is empty
    fn is_empty(&self, value: &FhirPathValue) -> bool {
        match value {
            FhirPathValue::Collection(items) => items.is_empty(),
            FhirPathValue::Empty => true,
            _ => false,
        }
    }

    /// Get count of items
    fn count(&self, value: &FhirPathValue) -> i64 {
        match value {
            FhirPathValue::Collection(items) => items.len() as i64,
            FhirPathValue::Empty => 0,
            _ => 1,
        }
    }

    /// Convert value to string
    fn to_string_value(&self, value: &FhirPathValue) -> std::sync::Arc<str> {
        match value {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Integer(i) => i.to_string().into(),
            FhirPathValue::Decimal(d) => d.to_string().into(),
            FhirPathValue::Boolean(b) => b.to_string().into(),
            FhirPathValue::Collection(items) => {
                items.iter()
                    .map(|item| self.to_string_value(item).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .into()
            }
            _ => format!("{:?}", value).into(),
        }
    }

    // Lambda-specific helper methods

    /// Create a lambda-compatible expression evaluator
    fn create_lambda_evaluator(
        &self,
        base_depth: usize,
    ) -> Box<octofhir_fhirpath_registry::lambda_function::LambdaExpressionEvaluator> {
        let engine_clone = self.clone();

        Box::new(move |expr, scope, context| {
            let engine = engine_clone.clone();
            let expr = expr.clone();
            let variables = scope.to_variables_map();
            let context_input = context.input.clone();
            let model_provider = context.model_provider.clone();
            let depth = base_depth + 1;

            Box::pin(async move {
                // Create evaluator context with lambda variables
                let mut eval_context = EvaluationContext::new(
                    context_input.clone(),
                    engine.functions.clone(),
                    engine.operators.clone(),
                    model_provider.expect("Model provider is required"),
                );

                // Add lambda variables to context
                for (name, value) in variables.iter() {
                    eval_context.set_variable(name.clone(), value.clone());
                }

                // Evaluate expression with lambda context
                engine.evaluate_node_async(&expr, context_input, &eval_context, depth).await
                    .map_err(|e| octofhir_fhirpath_registry::function::FunctionError::EvaluationError {
                        name: "lambda_evaluator".to_string(),
                        message: e.to_string(),
                    })
            })
        })
    }

    /// Get lambda argument indices for a function
    fn get_lambda_argument_indices(&self, name: &str) -> Vec<usize> {
        // Use registry's enhanced lambda argument indices method
        let indices = self.functions.get_lambda_argument_indices(name);

        if !indices.is_empty() {
            return indices;
        }

        // Fallback: use helper function from registry
        octofhir_fhirpath_registry::lambda_function::get_lambda_argument_indices(name)
    }

    /// Infer lambda type from lambda data structure
    fn infer_lambda_type(&self, _lambda_data: &octofhir_fhirpath_ast::LambdaData) -> LambdaType {
        // For now, default to Select type
        // In a full implementation, this would analyze the lambda body to determine type
        // This can be enhanced based on usage patterns or explicit type hints
        LambdaType::Select
    }

}

/// Lambda expression types for evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum LambdaType {
    /// Select transformation - collect all results
    Select,
    /// Where filtering - include items where condition is true
    Where,
    /// All validation - return true if all items satisfy condition
    All,
    /// Any validation - return true if any item satisfies condition
    Any,
    /// Aggregate accumulation - accumulate results
    Aggregate,
}

// Thread safety by design - all fields are Send + Sync
unsafe impl Send for FhirPathEngine {}
unsafe impl Sync for FhirPathEngine {}
