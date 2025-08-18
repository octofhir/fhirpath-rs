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
//! use sonic_rs::json;
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
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
//!
//! let config = EvaluationConfig {
//!     max_recursion_depth: 500,
//!     timeout_ms: 10000,
//!     enable_lambda_optimization: true,
//!     enable_sync_optimization: true,
//!     memory_limit_mb: Some(100),
//!     max_expression_nodes: 5000,
//!     max_collection_size: 50000,
//! };
//!
//! let engine = FhirPathEngine::with_mock_provider().await?
//!     .with_config(config);
//! # Ok(())
//! # }
//! ```

use crate::context::EvaluationContext as LocalEvaluationContext;
use sonic_rs::JsonValueTrait;

// Import the new modular components
use async_trait::async_trait;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{FhirPathValue, ModelProvider};
use octofhir_fhirpath_registry::{
    ExpressionEvaluator, FhirPathRegistry,
    operations::EvaluationContext as RegistryEvaluationContext,
};
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
/// use sonic_rs::json;
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
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
///
/// let config = EvaluationConfig {
///     max_recursion_depth: 500,
///     timeout_ms: 10000,
///     enable_lambda_optimization: true,
///     enable_sync_optimization: true,
///     memory_limit_mb: Some(100),
///     max_expression_nodes: 5000,
///     max_collection_size: 50000,
/// };
///
/// let engine = FhirPathEngine::with_mock_provider().await?.with_config(config);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct FhirPathEngine {
    /// Unified registry containing all operations (functions and operators)
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
///     enable_sync_optimization: false,
///     memory_limit_mb: Some(50),
///     max_expression_nodes: 1000,
///     max_collection_size: 10000,
/// };
///
/// // High-performance configuration for batch processing
/// let config = EvaluationConfig {
///     max_recursion_depth: 2000,
///     timeout_ms: 60000,
///     enable_lambda_optimization: true,
///     enable_sync_optimization: true,
///     memory_limit_mb: None,
///     max_expression_nodes: 10000,
///     max_collection_size: 100000,
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
    /// Maximum number of nodes in an expression to prevent complexity attacks
    pub max_expression_nodes: usize,
    /// Maximum collection size to prevent memory exhaustion
    pub max_collection_size: usize,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 1000,
            timeout_ms: 30000,
            enable_lambda_optimization: true,
            enable_sync_optimization: true,
            memory_limit_mb: None,
            max_expression_nodes: 10000, // Prevent parsing extremely complex expressions
            max_collection_size: 100000, // Prevent memory exhaustion from large collections
        }
    }
}

/// Recursive helper for counting expression nodes
fn count_nodes_recursive(node: &ExpressionNode) -> usize {
    use octofhir_fhirpath_ast::ExpressionNode as Node;

    match node {
        Node::Literal(_) => 1,
        Node::Identifier(_) => 1,
        Node::Path { base, path: _ } => 1 + count_nodes_recursive(base),
        Node::BinaryOp(binary_data) => {
            1 + count_nodes_recursive(&binary_data.left) + count_nodes_recursive(&binary_data.right)
        }
        Node::UnaryOp { op: _, operand } => 1 + count_nodes_recursive(operand),
        Node::FunctionCall(func_data) => {
            1 + func_data
                .args
                .iter()
                .map(count_nodes_recursive)
                .sum::<usize>()
        }
        Node::MethodCall(method_data) => {
            1 + count_nodes_recursive(&method_data.base)
                + method_data
                    .args
                    .iter()
                    .map(count_nodes_recursive)
                    .sum::<usize>()
        }
        Node::Index { base, index } => {
            1 + count_nodes_recursive(base) + count_nodes_recursive(index)
        }
        Node::Filter { base, condition } => {
            1 + count_nodes_recursive(base) + count_nodes_recursive(condition)
        }
        Node::Union { left, right } => {
            1 + count_nodes_recursive(left) + count_nodes_recursive(right)
        }
        Node::TypeCheck {
            expression,
            type_name: _,
        } => 1 + count_nodes_recursive(expression),
        Node::TypeCast {
            expression,
            type_name: _,
        } => 1 + count_nodes_recursive(expression),
        Node::Lambda(lambda_data) => {
            1 + lambda_data.params.len() + count_nodes_recursive(&lambda_data.body)
        }
        Node::Conditional(cond_data) => {
            1 + count_nodes_recursive(&cond_data.condition)
                + count_nodes_recursive(&cond_data.then_expr)
                + cond_data
                    .else_expr
                    .as_ref()
                    .map(|e| count_nodes_recursive(e))
                    .unwrap_or(0)
        }
        Node::Variable(_) => 1,
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
    pub fn new(registry: Arc<FhirPathRegistry>, model_provider: Arc<dyn ModelProvider>) -> Self {
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
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    ///
    /// let engine = FhirPathEngine::with_mock_provider().await?;
    /// let config = engine.config();
    ///
    /// println!("Max depth: {}", config.max_recursion_depth);
    /// println!("Timeout: {}ms", config.timeout_ms);
    /// # Ok(())
    /// # }
    /// ```
    pub fn config(&self) -> &EvaluationConfig {
        &self.config
    }

    /// Get the registry reference
    pub fn registry(&self) -> &Arc<FhirPathRegistry> {
        &self.registry
    }

    /// Get the model provider reference  
    pub fn model_provider(&self) -> &Arc<dyn ModelProvider> {
        &self.model_provider
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
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
    ///
    /// let config = EvaluationConfig {
    ///     max_recursion_depth: 500,
    ///     timeout_ms: 10000,
    ///     enable_lambda_optimization: true,
    ///     enable_sync_optimization: true,
    ///     memory_limit_mb: Some(100),
    ///     max_expression_nodes: 5000,
    ///     max_collection_size: 50000,
    /// };
    ///
    /// let engine = FhirPathEngine::with_mock_provider().await?
    ///     .with_config(config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_config(mut self, config: EvaluationConfig) -> Self {
        self.config = config;
        self
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
    /// use sonic_rs::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
    /// let result = engine.evaluate("42", json!({})).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_mock_provider() -> EvaluationResult<Self> {
        use octofhir_fhirpath_model::MockModelProvider;

        let registry = octofhir_fhirpath_registry::create_standard_registry()
            .await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Failed to create registry: {e}"),
            })?;

        let model_provider = Arc::new(MockModelProvider::new());
        Ok(Self::new(Arc::new(registry), model_provider))
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
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use octofhir_fhirpath_model::MockModelProvider;
    /// use std::sync::Arc;
    ///
    /// let provider = Arc::new(MockModelProvider::new());
    /// let engine = FhirPathEngine::with_model_provider(provider).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_model_provider(
        model_provider: Arc<dyn ModelProvider>,
    ) -> EvaluationResult<Self> {
        let registry = octofhir_fhirpath_registry::create_standard_registry()
            .await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Failed to create registry: {e}"),
            })?;
        Ok(Self::new(Arc::new(registry), model_provider))
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
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
    ///
    /// let engine = FhirPathEngine::with_mock_provider().await?;
    ///
    /// // Create a high-performance configuration
    /// let performance_config = EvaluationConfig {
    ///     max_recursion_depth: 2000,
    ///     timeout_ms: 60000,
    ///     enable_lambda_optimization: true,
    ///     enable_sync_optimization: true,
    ///     memory_limit_mb: None,
    ///     max_expression_nodes: 10000,
    ///     max_collection_size: 100000,
    /// };
    ///
    /// let performance_engine = engine.with_config(performance_config);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use octofhir_fhirpath_evaluator::{FhirPathEngine, EvaluationConfig};
    ///
    /// // Chain configuration for different use cases
    /// let base_engine = FhirPathEngine::with_mock_provider().await?;
    ///
    /// let strict_config = EvaluationConfig {
    ///     max_recursion_depth: 100,
    ///     timeout_ms: 5000,
    ///     enable_lambda_optimization: false,
    ///     enable_sync_optimization: false,
    ///     memory_limit_mb: Some(50),
    ///     max_expression_nodes: 1000,
    ///     max_collection_size: 10000,
    /// };
    ///
    /// let strict_engine = base_engine.with_config(strict_config);
    /// # Ok(())
    /// # }
    /// ```

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
    /// use sonic_rs::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
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
    /// use sonic_rs::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
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
    /// use sonic_rs::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
    ///
    /// let result = engine.evaluate("(5 + 3) * 2", json!({})).await?;
    /// println!("Result: {:?}", result); // Should be 16
    /// # Ok(())
    /// # }
    /// ```
    pub async fn evaluate(
        &self,
        expression: &str,
        input_data: sonic_rs::Value,
    ) -> EvaluationResult<FhirPathValue> {
        // Parse expression
        let ast = match octofhir_fhirpath_parser::parse_expression(expression) {
            Ok(ast) => ast,
            Err(e) => {
                // FAIL FAST: All parse errors should be treated as actual errors
                // Error recovery should be configurable when needed, not default behavior
                return Err(EvaluationError::InvalidOperation {
                    message: format!("Parse error: {e}"),
                });
            }
        };

        // SECURITY: Check expression complexity to prevent DoS attacks
        let node_count = self.count_expression_nodes(&ast);
        if node_count > self.config.max_expression_nodes {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "Expression too complex: {} nodes exceeds maximum of {}",
                    node_count, self.config.max_expression_nodes
                ),
            });
        }

        // Convert input data to FhirPathValue
        let fhir_value = FhirPathValue::from(input_data);

        // Create evaluation context with unified registry and proper root preservation
        let mut context = LocalEvaluationContext::new(
            fhir_value.clone(),
            self.registry().clone(),
            self.model_provider().clone(),
        );

        // Set standard FHIRPath environment variables according to the specification
        // These must be set for ALL evaluation contexts to ensure resolve() works properly
        // Use set_system_variable to bypass the system variable protection
        // %context - The original node in the input context (always the starting resource)
        context.set_system_variable("context".to_string(), fhir_value.clone());
        // %resource - The resource containing the original node (same as context for top-level)
        context.set_system_variable("resource".to_string(), fhir_value.clone());
        // %rootResource - The container resource (same as resource unless dealing with contained resources)
        context.set_system_variable("rootResource".to_string(), fhir_value.clone());

        // Use the AST evaluation method and ensure result is always a collection
        let result = self.evaluate_ast(&ast, fhir_value, &context).await?;
        Ok(FhirPathEngine::ensure_collection_result(result))
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
    /// use sonic_rs::json;
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
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
    /// use sonic_rs::json;
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
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
    /// use sonic_rs::json;
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
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
    ///     "status = 'active'",
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
        input_data: sonic_rs::Value,
        variables: std::collections::HashMap<String, FhirPathValue>,
    ) -> EvaluationResult<FhirPathValue> {
        // Parse expression
        let ast = match octofhir_fhirpath_parser::parse_expression(expression) {
            Ok(ast) => ast,
            Err(e) => {
                // FAIL FAST: All parse errors should be treated as actual errors
                // Error recovery should be configurable when needed, not default behavior
                return Err(EvaluationError::InvalidOperation {
                    message: format!("Parse error: {e}"),
                });
            }
        };

        // Convert input data to FhirPathValue
        let fhir_value = FhirPathValue::from(input_data);

        // Create evaluation context with variables
        let context = LocalEvaluationContext::with_variables(
            fhir_value.clone(),
            self.registry().clone(),
            self.model_provider().clone(),
            variables.into_iter().collect(),
        );

        // Use the AST evaluation method and ensure result is always a collection
        let result = self.evaluate_ast(&ast, fhir_value, &context).await?;
        Ok(FhirPathEngine::ensure_collection_result(result))
    }

    /// Ensures that evaluation results are always collections per FHIRPath spec.
    /// Transforms FhirPathValue::Empty to an empty collection.

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
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use sonic_rs::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
    ///
    /// // For direct AST evaluation, it's usually easier to use the evaluate method
    /// let input = json!({"resourceType": "Patient", "name": [{"given": ["John"]}]});
    /// let result = engine.evaluate("Patient.name.given", input).await?;
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
        context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        self.evaluate_node_async(expression, input, context, 0)
            .await
    }

    /// Evaluate a FHIRPath expression with JSON string input using adaptive parsing.
    ///
    /// This method automatically selects the optimal JSON parser based on input size:
    /// - sonic-rs for large JSON (when available) for enhanced performance
    /// - sonic_rs for high-performance JSON processing
    ///
    /// # Arguments
    ///
    /// * `expression` - The FHIRPath expression string
    /// * `json_str` - The input JSON as a string
    ///
    /// # Returns
    ///
    /// Returns a `FhirPathValue` containing the evaluation result.
    ///
    /// # Errors
    ///
    /// Returns `EvaluationError` if:
    /// - JSON parsing fails
    /// - The expression has syntax errors
    /// - Evaluation exceeds timeout or recursion limits
    /// - A runtime error occurs during evaluation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
    /// let json_str = r#"{"resourceType": "Patient", "name": [{"given": ["John"]}]}"#;
    ///
    /// let result = engine.evaluate_json_str("Patient.name.given", json_str).await?;
    /// println!("Result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn evaluate_json_str(
        &self,
        expression: &str,
        json_str: &str,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::JsonValue;

        // Parse JSON using sonic_rs via JsonValue
        let json_value =
            JsonValue::parse(json_str).map_err(|e| EvaluationError::InvalidOperation {
                message: format!("JSON parsing failed: {e}"),
            })?;

        // Use sonic_rs::Value directly
        let sonic_value = json_value.as_sonic_value().clone();

        self.evaluate(expression, sonic_value).await
    }

    /// Evaluate a FHIRPath expression with a FhirPathValue input.
    ///
    /// This method accepts FhirPathValue directly, enabling native evaluation
    /// without any JSON conversions.
    pub async fn evaluate_fhir_value(
        &self,
        expression: &str,
        input_value: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // Parse the expression
        let parsed_expr = octofhir_fhirpath_parser::parse_expression(expression).map_err(|e| {
            EvaluationError::InvalidOperation {
                message: format!("Parse error: {e}"),
            }
        })?;

        // Create evaluation context
        let context = LocalEvaluationContext::new(
            input_value.clone(),
            self.registry().clone(),
            self.model_provider().clone(),
        );

        // Evaluate the parsed expression
        self.evaluate_node_async(&parsed_expr, input_value, &context, 0)
            .await
    }
    /// Core recursive evaluator - handles all node types

    // Simple node type handlers

    /// Parse FHIRPath date literal supporting partial dates
    /// Supports: @YYYY, @YYYY-MM, @YYYY-MM-DD
    fn parse_fhirpath_date(
        date_str: &str,
    ) -> Result<octofhir_fhirpath_model::PrecisionDate, String> {
        // Remove the @ prefix if present
        let date_str = date_str.strip_prefix('@').unwrap_or(date_str);

        // Count the number of parts
        let parts: Vec<&str> = date_str.split('-').collect();

        match parts.len() {
            1 => {
                // Year only: @2015
                let year = parts[0]
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid year: {}", parts[0]))?;
                // Use January 1st for partial year
                let date = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                    .ok_or_else(|| format!("Invalid date: {date_str}"))?;
                Ok(octofhir_fhirpath_model::PrecisionDate::new(
                    date,
                    octofhir_fhirpath_model::TemporalPrecision::Year,
                ))
            }
            2 => {
                // Year and month: @2015-02
                let year = parts[0]
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid year: {}", parts[0]))?;
                let month = parts[1]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid month: {}", parts[1]))?;
                // Use 1st day for partial month
                let date = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| format!("Invalid date: {date_str}"))?;
                Ok(octofhir_fhirpath_model::PrecisionDate::new(
                    date,
                    octofhir_fhirpath_model::TemporalPrecision::Month,
                ))
            }
            3 => {
                // Full date: @2015-02-04
                let year = parts[0]
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid year: {}", parts[0]))?;
                let month = parts[1]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid month: {}", parts[1]))?;
                let day = parts[2]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid day: {}", parts[2]))?;
                let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
                    .ok_or_else(|| format!("Invalid date: {date_str}"))?;
                Ok(octofhir_fhirpath_model::PrecisionDate::new(
                    date,
                    octofhir_fhirpath_model::TemporalPrecision::Day,
                ))
            }
            _ => Err(format!("Invalid date format: {date_str}")),
        }
    }

    /// Parse FHIRPath datetime literal supporting partial datetimes
    /// Supports: @2015T, @2015-02T, @2015-02-04T14:34:28Z, etc.
    fn parse_fhirpath_datetime(
        datetime_str: &str,
    ) -> Result<octofhir_fhirpath_model::PrecisionDateTime, String> {
        use chrono::TimeZone;

        // Remove the @ prefix if present
        let datetime_str = datetime_str.strip_prefix('@').unwrap_or(datetime_str);

        // Split on 'T' to separate date and time parts
        let parts: Vec<&str> = datetime_str.split('T').collect();

        if parts.len() < 2 {
            return Err(format!("Invalid datetime format: {datetime_str}"));
        }

        let date_part = parts[0];
        let time_part = parts[1];

        // Parse the date part
        let (date, date_precision) = if date_part.is_empty() {
            // Handle @T... format (time only, use epoch date)
            let date = chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .ok_or_else(|| "Failed to create epoch date".to_string())?;
            (date, octofhir_fhirpath_model::TemporalPrecision::Day)
        } else {
            let precision_date = Self::parse_fhirpath_date(&format!("@{date_part}"))?;
            (precision_date.date, precision_date.precision)
        };

        // Parse the time part
        let (time, offset, time_precision) = if time_part.is_empty() {
            // Handle partial datetime like @2015T (no time specified)
            (
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                chrono::FixedOffset::east_opt(0).unwrap(),
                octofhir_fhirpath_model::TemporalPrecision::Hour,
            )
        } else {
            let (time, offset) = Self::parse_fhirpath_time_with_tz(time_part)?;
            // Determine precision from time components
            let precision = Self::determine_time_precision(time_part);
            (time, offset, precision)
        };

        // Combine date and time
        let naive_datetime = date.and_time(time);

        // Create datetime with timezone
        let datetime = offset
            .from_local_datetime(&naive_datetime)
            .single()
            .ok_or_else(|| format!("Invalid datetime: {datetime_str}"))?;

        // Use the more specific precision between date and time
        let final_precision = match (date_precision, time_precision) {
            (_, t) if t as u8 > date_precision as u8 => t,
            (d, _) => d,
        };

        Ok(octofhir_fhirpath_model::PrecisionDateTime::new(
            datetime,
            final_precision,
        ))
    }

    /// Parse FHIRPath time literal supporting partial times and timezones
    /// Supports: @T14, @T14:34, @T14:34:28, @T14:34:28.123, @T14:34:28Z, @T14:34:28+10:00
    fn parse_fhirpath_time(
        time_str: &str,
    ) -> Result<octofhir_fhirpath_model::PrecisionTime, String> {
        // Remove the @T prefix if present
        let time_str = time_str
            .strip_prefix('@')
            .and_then(|s| s.strip_prefix('T'))
            .unwrap_or(time_str);

        // Remove timezone info for parsing time only
        let (time_part, _) = Self::split_time_timezone(time_str);

        let time = Self::parse_time_components(time_part)?;
        let precision = Self::determine_time_precision(time_part);

        Ok(octofhir_fhirpath_model::PrecisionTime::new(time, precision))
    }

    /// Parse time with timezone information
    fn parse_fhirpath_time_with_tz(
        time_str: &str,
    ) -> Result<(chrono::NaiveTime, chrono::FixedOffset), String> {
        let (time_part, tz_part) = Self::split_time_timezone(time_str);

        let time = Self::parse_time_components(time_part)?;
        let offset = Self::parse_timezone_offset(tz_part)?;

        Ok((time, offset))
    }

    /// Split time string into time and timezone parts
    fn split_time_timezone(time_str: &str) -> (&str, Option<&str>) {
        if let Some(pos) = time_str.find('Z') {
            (&time_str[..pos], Some("Z"))
        } else if let Some(pos) = time_str.find('+') {
            (&time_str[..pos], Some(&time_str[pos..]))
        } else if let Some(pos) = time_str.rfind('-') {
            // Only treat as timezone if it looks like one (has colon after)
            if time_str[pos..].contains(':') {
                (&time_str[..pos], Some(&time_str[pos..]))
            } else {
                (time_str, None)
            }
        } else {
            (time_str, None)
        }
    }

    /// Parse time components (hour, minute, second, millisecond)
    fn parse_time_components(time_str: &str) -> Result<chrono::NaiveTime, String> {
        let parts: Vec<&str> = time_str.split(':').collect();

        match parts.len() {
            1 => {
                // Hour only: T14
                let hour = parts[0]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
                chrono::NaiveTime::from_hms_opt(hour, 0, 0)
                    .ok_or_else(|| format!("Invalid time: {time_str}"))
            }
            2 => {
                // Hour and minute: T14:34
                let hour = parts[0]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
                let minute = parts[1]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid minute: {}", parts[1]))?;
                chrono::NaiveTime::from_hms_opt(hour, minute, 0)
                    .ok_or_else(|| format!("Invalid time: {time_str}"))
            }
            3 => {
                // Hour, minute, and second: T14:34:28 or T14:34:28.123
                let hour = parts[0]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
                let minute = parts[1]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid minute: {}", parts[1]))?;

                // Handle seconds with optional milliseconds
                let second_part = parts[2];
                if let Some(dot_pos) = second_part.find('.') {
                    let second = second_part[..dot_pos]
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid second: {}", &second_part[..dot_pos]))?;
                    let millis_str = &second_part[dot_pos + 1..];
                    // Pad or truncate to 3 digits for milliseconds
                    let millis_str = if millis_str.len() > 3 {
                        &millis_str[..3]
                    } else {
                        millis_str
                    };
                    let millis = format!("{millis_str:0<3}")
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid milliseconds: {millis_str}"))?;

                    chrono::NaiveTime::from_hms_milli_opt(hour, minute, second, millis)
                        .ok_or_else(|| format!("Invalid time: {time_str}"))
                } else {
                    let second = second_part
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid second: {second_part}"))?;
                    chrono::NaiveTime::from_hms_opt(hour, minute, second)
                        .ok_or_else(|| format!("Invalid time: {time_str}"))
                }
            }
            _ => Err(format!("Invalid time format: {time_str}")),
        }
    }

    /// Parse timezone offset
    fn parse_timezone_offset(tz_str: Option<&str>) -> Result<chrono::FixedOffset, String> {
        match tz_str {
            None => {
                // No timezone specified - use a special offset to distinguish from explicit UTC
                // Use +00:01 to distinguish from explicit Z which uses +00:00
                Ok(chrono::FixedOffset::east_opt(60).unwrap()) // +00:01
            }
            Some("Z") => Ok(chrono::FixedOffset::east_opt(0).unwrap()), // UTC
            Some(tz) if tz.starts_with('+') || tz.starts_with('-') => {
                // Parse +HH:MM or -HH:MM
                let sign = if tz.starts_with('+') { 1 } else { -1 };
                let tz_time = &tz[1..];
                let parts: Vec<&str> = tz_time.split(':').collect();

                if parts.len() != 2 {
                    return Err(format!("Invalid timezone format: {tz}"));
                }

                let hours = parts[0]
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid timezone hours: {}", parts[0]))?;
                let minutes = parts[1]
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid timezone minutes: {}", parts[1]))?;

                let total_seconds = sign * (hours * 3600 + minutes * 60);
                chrono::FixedOffset::east_opt(total_seconds)
                    .ok_or_else(|| format!("Invalid timezone offset: {tz}"))
            }
            Some(tz) => Err(format!("Unsupported timezone format: {tz}")),
        }
    }

    /// Determine time precision from time string format
    fn determine_time_precision(time_str: &str) -> octofhir_fhirpath_model::TemporalPrecision {
        let parts: Vec<&str> = time_str.split(':').collect();
        match parts.len() {
            1 => octofhir_fhirpath_model::TemporalPrecision::Hour,
            2 => octofhir_fhirpath_model::TemporalPrecision::Minute,
            3 => {
                // Check if seconds have fractional part
                if parts[2].contains('.') {
                    octofhir_fhirpath_model::TemporalPrecision::Millisecond
                } else {
                    octofhir_fhirpath_model::TemporalPrecision::Second
                }
            }
            _ => octofhir_fhirpath_model::TemporalPrecision::Second, // fallback
        }
    }

    /// Evaluate literal values

    /// Evaluate variable references

    /// Get the containing resource for %resource variable
    fn get_containing_resource(&self, resource: &FhirPathValue) -> FhirPathValue {
        // For contained resources, walk up to find the containing resource
        // For now, return the resource itself (basic implementation)
        // TODO: Implement proper contained resource navigation
        resource.clone()
    }

    /// Get the root resource for %rootResource variable
    fn get_root_resource(&self, resource: &FhirPathValue) -> FhirPathValue {
        // For contained resources, return the top-level container
        // For bundle entries, return the bundle
        // For now, return the resource itself (basic implementation)
        // TODO: Implement proper root resource navigation
        resource.clone()
    }

    /// Check if a FhirPathValue represents a Bundle resource
    fn is_bundle_resource(&self, value: &FhirPathValue) -> bool {
        match value {
            FhirPathValue::Resource(resource) => resource.resource_type() == Some("Bundle"),
            FhirPathValue::JsonValue(json) => json
                .as_inner()
                .get("resourceType")
                .and_then(|rt| rt.as_str())
                .map(|rt| rt == "Bundle")
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Helper: Get property value from a FhirPathValue
    fn get_property_value(&self, value: &FhirPathValue, property: &str) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_arc) => {
                if json_arc.is_object() {
                    // First try direct property access
                    if let Some(value) = json_arc.get_property(property) {
                        // Convert JsonValue to proper FhirPath type instead of keeping as JsonValue
                        let mut fhir_value = crate::evaluators::navigation::NavigationEvaluator::convert_json_to_fhirpath_value(value);
                        // Special handling for id property - try to convert string numbers to integers
                        if property == "id" {
                            if let FhirPathValue::String(ref s) = fhir_value {
                                if let Ok(i) = s.parse::<i64>() {
                                    fhir_value = FhirPathValue::Integer(i);
                                }
                            }
                        }
                        Some(fhir_value)
                    } else if property == "value" {
                        // For FHIR resources, 'value' property should resolve to value[x] properties
                        // Look for properties starting with "value" (like valueString, valueInteger, etc.)
                        if let Some(iter) = json_arc.object_iter() {
                            for (key, value) in iter {
                                if key.starts_with("value") && key != "value" {
                                    return Some(crate::evaluators::navigation::NavigationEvaluator::convert_json_to_fhirpath_value(value));
                                }
                            }
                        }
                        None
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            FhirPathValue::Resource(resource) => {
                // Try to get property from resource
                resource.get_property(property).map(|v| {
                    // v is already a sonic_rs::Value, create JsonValue directly
                    let json_value = octofhir_fhirpath_model::JsonValue::new(v);
                    let mut fhir_value = crate::evaluators::navigation::NavigationEvaluator::convert_json_to_fhirpath_value(json_value);
                    // Special handling for id property - try to convert string numbers to integers
                    if property == "id" {
                        if let FhirPathValue::String(ref s) = fhir_value {
                            if let Ok(i) = s.parse::<i64>() {
                                fhir_value = FhirPathValue::Integer(i);
                            }
                        }
                    }
                    fhir_value
                })
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // Handle TypeInfoObject property access
                match property {
                    "name" => Some(FhirPathValue::String(name.clone())),
                    "namespace" => Some(FhirPathValue::String(namespace.clone())),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Check if identifier is a FHIR resource type
    fn is_resource_type(&self, identifier: &str) -> bool {
        matches!(
            identifier,
            "Patient"
                | "Observation"
                | "Condition"
                | "Procedure"
                | "MedicationRequest"
                | "DiagnosticReport"
                | "Encounter"
                | "Organization"
                | "Practitioner"
                | "Location"
                | "Device"
                | "Medication"
                | "Substance"
                | "AllergyIntolerance"
                | "CarePlan"
                | "Goal"
                | "ServiceRequest"
                | "Task"
                | "Appointment"
                | "AppointmentResponse"
                | "Schedule"
                | "Slot"
                | "Coverage"
                | "Claim"
                | "ClaimResponse"
                | "ExplanationOfBenefit"
                | "Bundle"
                | "Composition"
                | "DocumentReference"
                | "Binary"
                | "Media"
                | "List"
                | "Library"
                | "Measure"
                | "MeasureReport"
                | "Questionnaire"
                | "QuestionnaireResponse"
                | "StructureDefinition"
                | "ValueSet"
                | "CodeSystem"
                | "ConceptMap"
                | "CapabilityStatement"
                | "OperationDefinition"
                | "SearchParameter"
                | "CompartmentDefinition"
                | "ImplementationGuide"
                | "TestScript"
                | "TestReport"
                | "Provenance"
                | "AuditEvent"
                | "Consent"
                | "Contract"
                | "Person"
                | "RelatedPerson"
                | "Group"
                | "ResearchStudy"
                | "ResearchSubject"
                | "ActivityDefinition"
                | "PlanDefinition"
                | "RequestGroup"
                | "Communication"
                | "CommunicationRequest"
                | "DeviceRequest"
                | "DeviceUseStatement"
                | "Flag"
                | "RiskAssessment"
                | "DetectedIssue"
                | "ClinicalImpression"
                | "FamilyMemberHistory"
                | "ImagingStudy"
                | "Specimen"
                | "BodyStructure"
                | "ImmunizationRecommendation"
                | "Immunization"
                | "NutritionOrder"
                | "VisionPrescription"
                | "SupplyRequest"
                | "SupplyDelivery"
                | "InventoryReport"
                | "BiologicallyDerivedProduct"
                | "NutritionProduct"
                | "SubstanceDefinition"
                | "Ingredient"
                | "ManufacturedItemDefinition"
                | "AdministrableProductDefinition"
                | "PackagedProductDefinition"
                | "ClinicalUseDefinition"
                | "RegulatedAuthorization"
                | "MedicinalProductDefinition"
                | "Citation"
                | "Evidence"
                | "EvidenceReport"
                | "EvidenceVariable"
                | "ResearchElementDefinition"
                | "ChargeItem"
                | "ChargeItemDefinition"
                | "Account"
                | "Invoice"
                | "PaymentNotice"
                | "PaymentReconciliation"
                | "EnrollmentRequest"
                | "EnrollmentResponse"
                | "EligibilityRequest"
                | "EligibilityResponse"
                | "InsurancePlan"
                | "CoverageEligibilityRequest"
                | "CoverageEligibilityResponse"
                | "Endpoint"
                | "HealthcareService"
                | "PractitionerRole"
                | "OrganizationAffiliation"
                | "VerificationResult"
                | "MolecularSequence"
                | "GenomicStudy"
                | "DocumentManifest"
                | "CatalogEntry"
                | "Basic"
                | "Linkage"
                | "MessageDefinition"
                | "MessageHeader"
                | "OperationOutcome"
                | "Parameters"
                | "Subscription"
                | "SubscriptionStatus"
                | "SubscriptionTopic"
                | "Topic"
                | "EventDefinition"
                | "ObservationDefinition"
                | "SpecimenDefinition"
                | "ActorDefinition"
                | "Requirements"
                | "Permission"
                | "CanonicalResource"
                | "MetadataResource"
                | "DomainResource"
                | "Resource"
        )
    }

    /// Filter input by resource type (FHIRPath resource type filtering)
    fn filter_by_resource_type(
        &self,
        input: &FhirPathValue,
        resource_type: &str,
    ) -> EvaluationResult<FhirPathValue> {
        match input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    if self.matches_resource_type(item, resource_type) {
                        results.push(item.clone());
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            single_item => {
                if self.matches_resource_type(single_item, resource_type) {
                    Ok(FhirPathValue::collection(vec![single_item.clone()]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }

    /// Check if a value matches the given resource type
    fn matches_resource_type(&self, value: &FhirPathValue, resource_type: &str) -> bool {
        match value {
            FhirPathValue::JsonValue(json_arc) => {
                if json_arc.is_object() {
                    if let Some(rt_val) = json_arc.get_property("resourceType") {
                        if let Some(rt) = rt_val.as_str() {
                            return rt.eq_ignore_ascii_case(resource_type);
                        }
                    }
                }
                false
            }
            FhirPathValue::Resource(resource) => {
                if let Some(rt) = resource.resource_type() {
                    rt.eq_ignore_ascii_case(resource_type)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    // Complex node type handlers

    /// Evaluate path navigation (object.property)

    /// Evaluate index access expressions

    /// Evaluate filter expressions
    pub async fn evaluate_filter(
        &self,
        base: &ExpressionNode,
        condition: &ExpressionNode,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the base expression
        let base_result = self
            .evaluate_node_async(base, input, context, depth + 1)
            .await?;

        // Filter based on the condition
        match base_result {
            FhirPathValue::Collection(items) => {
                let mut filtered_items = Vec::new();

                for item in items.iter() {
                    // Evaluate condition for each item
                    let condition_result = self
                        .evaluate_node_async(condition, item.clone(), context, depth + 1)
                        .await?;

                    if self.is_truthy(&condition_result) {
                        filtered_items.push(item.clone());
                    }
                }

                Ok(FhirPathValue::collection(filtered_items))
            }
            single_value => {
                // Single value - check condition
                let condition_result = self
                    .evaluate_node_async(condition, single_value.clone(), context, depth + 1)
                    .await?;

                if self.is_truthy(&condition_result) {
                    Ok(FhirPathValue::collection(vec![single_value]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }

    /// Evaluate union expressions
    pub async fn evaluate_union(
        &self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate both sides
        let left_result = self
            .evaluate_node_async(left, input.clone(), context, depth + 1)
            .await?;
        let right_result = self
            .evaluate_node_async(right, input, context, depth + 1)
            .await?;

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
    pub async fn evaluate_type_check(
        &self,
        expression: &ExpressionNode,
        type_name: &str,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate the expression
        let value = self
            .evaluate_node_async(expression, input, context, depth + 1)
            .await?;

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

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            matches_type,
        )]))
    }

    /// Evaluate type cast expressions (value as Type)
    pub async fn evaluate_type_cast(
        &self,
        expression: &ExpressionNode,
        type_name: &str,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the expression to get the value to cast
        let expr_result = self
            .evaluate_node_async(expression, input, context, depth + 1)
            .await?;

        // Get the as operator from the registry
        let as_operation = self
            .registry
            .get_operation("as")
            .await
            .ok_or_else(|| EvaluationError::from_function_error("Unknown operation: as"))?;

        // Create evaluation context for the operation
        let eval_context = RegistryEvaluationContext {
            input: expr_result,
            root: context.root.as_ref().clone(),
            variables: context.variable_scope.variables.as_ref().clone(),
            model_provider: self.model_provider().clone(),
            registry: self.registry().clone(),
        };

        // Call the as operator with the type name as argument
        // Check if this is a known type identifier, otherwise treat as string
        let type_arg = if self.is_type_identifier(type_name) {
            // Create a TypeInfoObject for known type identifiers
            let (namespace, name) = if type_name.contains('.') {
                let parts: Vec<&str> = type_name.split('.').collect();
                (parts[0], parts[1])
            } else {
                // Handle common FHIRPath types
                match type_name.to_lowercase().as_str() {
                    "boolean" | "integer" | "decimal" | "string" | "date" | "datetime" | "time"
                    | "quantity" => ("System", type_name),
                    "code" | "uri" | "url" | "canonical" | "oid" | "uuid" | "id" | "markdown"
                    | "base64binary" | "instant" | "positiveint" | "unsignedint" => {
                        ("FHIR", type_name)
                    }
                    _ => ("System", type_name),
                }
            };
            FhirPathValue::TypeInfoObject {
                namespace: Arc::from(namespace),
                name: Arc::from(name),
            }
        } else {
            // Treat as string literal for backward compatibility
            FhirPathValue::String(type_name.into())
        };
        let args = vec![type_arg];

        as_operation
            .evaluate(&args, &eval_context)
            .await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Method error in as: {e}"),
            })
    }

    /// Helper: Check if a value is truthy

    // Function and method evaluation

    /// Evaluate standard (non-lambda) functions
    ///
    /// For non-lambda functions, arguments are evaluated in the current context.
    /// The key fix: arguments should be evaluated against the current input (the focus
    /// of the function call), not against individual items in a collection.
    ///
    /// Example: In `Patient.name.select(given.combine(family))`:
    /// - `select()` is lambda, creates context for each name element
    /// - `combine()` is non-lambda, its arguments (`given`, `family`) should be
    ///   evaluated against the current name element (the input to combine)

    /// Check if a variable name is a system variable that cannot be overridden
    fn is_system_variable(name: &str) -> bool {
        match name {
            // Standard environment variables
            "context" | "resource" | "rootResource" | "sct" | "loinc" | "ucum" => true,
            // Lambda variables
            "this" | "$this" | "index" | "$index" | "total" | "$total" => true,
            // Value set variables (with or without quotes)
            name if name.starts_with("\"vs-") && name.ends_with('"') => true,
            name if name.starts_with("vs-") => true,
            // Extension variables (with or without quotes)
            name if name.starts_with("\"ext-") && name.ends_with('"') => true,
            name if name.starts_with("ext-") => true,
            _ => false,
        }
    }

    /// Evaluate lambda functions with expression arguments
    ///
    /// Lambda functions receive raw expressions instead of pre-evaluated values,
    /// allowing them to control evaluation context and implement proper variable
    /// scoping for $this, $index, $total, etc.

    /// Evaluate lambda expressions (inline lambda syntax)
    pub async fn evaluate_lambda_expression(
        &self,
        lambda_data: &octofhir_fhirpath_ast::LambdaData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
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
            lambda_context.set_variable(
                "$total".to_string(),
                FhirPathValue::Integer(collection.len() as i64),
            );

            // Evaluate lambda body with scoped context
            let result = self
                .evaluate_node_async(&lambda_data.body, item.clone(), &lambda_context, depth + 1)
                .await?;

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
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                            false,
                        )]));
                    }
                }
                LambdaType::Aggregate => {
                    // Aggregate: accumulate results
                    results.push(result);
                }
                LambdaType::Sort | LambdaType::Repeat => {
                    // These are handled by dedicated functions, not generic lambda evaluation
                    return Err(EvaluationError::InvalidOperation {
                        message: format!(
                            "Lambda type {lambda_type:?} should be handled by dedicated function"
                        ),
                    });
                }
            }
        }

        // Return appropriate result based on lambda type
        match lambda_type {
            LambdaType::All => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                true,
            )])), // All were true
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
    pub async fn evaluate_method_call(
        &self,
        method_data: &octofhir_fhirpath_ast::MethodCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Special handling for defineVariable function calls in the base
        // If the base is a defineVariable call, we need to propagate the variable context
        if let ExpressionNode::FunctionCall(func_data) = &method_data.base {
            if func_data.name == "defineVariable" {
                // First evaluate the defineVariable function to set up the variable
                let object = self
                    .evaluate_define_variable_function(func_data, input.clone(), context, depth + 1)
                    .await?;

                // Extract the variable from the defineVariable call for context propagation
                if !func_data.args.is_empty() {
                    if let Ok(name_value) = self
                        .evaluate_node_async(&func_data.args[0], input.clone(), context, depth + 1)
                        .await
                    {
                        if let FhirPathValue::String(var_name) = name_value {
                            let var_value = if func_data.args.len() == 2 {
                                self.evaluate_node_async(
                                    &func_data.args[1],
                                    input.clone(),
                                    context,
                                    depth + 1,
                                )
                                .await?
                            } else {
                                input.clone()
                            };

                            // Create a new context with the variable defined
                            let mut new_context = context.clone();
                            new_context
                                .variable_scope
                                .set_variable(var_name.as_ref().to_string(), var_value);

                            // Continue evaluation with the updated context using the method call logic
                            return self
                                .evaluate_method_call_with_object(
                                    &method_data.method,
                                    &method_data.args,
                                    object,
                                    &new_context,
                                    depth,
                                    input.clone(),
                                )
                                .await;
                        }
                    }
                }
            }
        }

        // First evaluate the base expression
        let object = self
            .evaluate_node_async(&method_data.base, input.clone(), context, depth + 1)
            .await?;

        self.evaluate_method_call_with_object(
            &method_data.method,
            &method_data.args,
            object,
            context,
            depth,
            input,
        )
        .await
    }

    async fn evaluate_method_call_with_object(
        &self,
        method_name: &str,
        args: &[ExpressionNode],
        object: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
        input: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // Handle built-in methods
        match method_name {
            "empty" => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                self.is_empty(&object),
            )])),
            "exists" => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                !self.is_empty(&object),
            )])),
            "count" => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(
                self.count(&object),
            )])),
            "toString" => Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                self.to_string_value(&object),
            )])),

            // Delegate to function registry for other methods
            method_name => {
                // Check if this is a lambda function first
                if self.is_lambda_function(method_name).await {
                    // Handle lambda method call using dedicated engine methods
                    // Create FunctionCallData structure to reuse existing lambda methods
                    let func_data = octofhir_fhirpath_ast::FunctionCallData {
                        name: method_name.to_string(),
                        args: args.iter().cloned().collect(),
                    };

                    match method_name {
                        "where" => {
                            self.evaluate_where_lambda(&func_data, object, context, depth)
                                .await
                        }
                        "select" => {
                            self.evaluate_select_lambda(&func_data, object, context, depth)
                                .await
                        }
                        "sort" => {
                            self.evaluate_sort_lambda(&func_data, object, context, depth)
                                .await
                        }
                        "repeat" => {
                            self.evaluate_repeat_lambda(&func_data, object, context, depth)
                                .await
                        }
                        "aggregate" => {
                            self.evaluate_aggregate_lambda(&func_data, object, context, depth)
                                .await
                        }
                        "all" => {
                            self.evaluate_all_lambda(&func_data, object, context, depth)
                                .await
                        }
                        _ => {
                            // Fallback to registry for other lambda functions not yet moved to engine
                            if let Some(operation) =
                                self.registry().get_operation(method_name).await
                            {
                                // Create registry context with the object as input for the lambda and variables from engine context
                                let all_variables = context.variable_scope.collect_all_variables();
                                let registry_context = octofhir_fhirpath_registry::operations::EvaluationContext::with_variables(
                                    object,
                                    self.registry().clone(),
                                    self.model_provider().clone(),
                                    all_variables,
                                );

                                // Use generic lambda function evaluation for remaining functions
                                // Note: We can't downcast to dyn LambdaFunction due to size constraints
                                // For now, use the operation directly if it implements lambda evaluation
                                operation
                                    .evaluate(&[], &registry_context)
                                    .await
                                    .map_err(|e| EvaluationError::InvalidOperation {
                                        message: format!(
                                            "Lambda method error in {method_name}: {e}"
                                        ),
                                    })
                            } else {
                                Err(EvaluationError::InvalidOperation {
                                    message: format!("Unknown lambda method: {method_name}"),
                                })
                            }
                        }
                    }
                } else {
                    // Standard method call - pre-evaluate arguments
                    let mut evaluated_args = vec![];

                    // Evaluate method arguments
                    for arg_expr in args.iter() {
                        // For type checking methods, evaluate type identifiers in original context
                        // but other arguments with object context
                        let arg_context = if matches!(method_name, "is" | "as" | "ofType")
                            && Self::is_type_identifier_expression(arg_expr)
                        {
                            // Type identifiers should be evaluated in original context
                            input.clone()
                        } else {
                            // Most arguments need object as context
                            object.clone()
                        };

                        let arg_value = self
                            .evaluate_node_async(arg_expr, arg_context, context, depth + 1)
                            .await?;
                        evaluated_args.push(arg_value);
                    }

                    // Get method from registry and evaluate
                    if let Some(operation) = self.registry().get_operation(method_name).await {
                        // Create registry context with the object as input (context) for the method - PRESERVE ORIGINAL ROOT
                        let registry_context =
                            octofhir_fhirpath_registry::operations::EvaluationContext::with_preserved_root(
                                object,
                                context.root.as_ref().clone(), // ✅ PRESERVE ORIGINAL ROOT
                                self.registry().clone(),
                                self.model_provider().clone(),
                            );

                        operation
                            .evaluate(&evaluated_args, &registry_context)
                            .await
                            .map_err(|e| EvaluationError::InvalidOperation {
                                message: format!("Method error in {method_name}: {e}"),
                            })
                    } else {
                        Err(EvaluationError::InvalidOperation {
                            message: format!("Unknown method: {method_name}"),
                        })
                    }
                }
            }
        }
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
            FhirPathValue::Collection(items) => items
                .iter()
                .map(|item| self.to_string_value(item).to_string())
                .collect::<Vec<_>>()
                .join(", ")
                .into(),
            _ => format!("{value:?}").into(),
        }
    }

    // Lambda-specific helper methods

    /// Infer lambda type from lambda data structure
    fn infer_lambda_type(&self, _lambda_data: &octofhir_fhirpath_ast::LambdaData) -> LambdaType {
        // For now, default to Select type
        // In a full implementation, this would analyze the lambda body to determine type
        // This can be enhanced based on usage patterns or explicit type hints
        LambdaType::Select
    }

    /// Count the total number of nodes in an expression tree
    /// This is used to prevent DoS attacks with extremely complex expressions
    fn count_expression_nodes(&self, node: &ExpressionNode) -> usize {
        count_nodes_recursive(node)
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
    /// Aggregate accumulation - accumulate results
    Aggregate,
    /// Sort ordering - sort collection by expression
    Sort,
    /// Repeat projection - repeat expression until no new items
    Repeat,
}

// Thread safety by design - all fields are Send + Sync
unsafe impl Send for FhirPathEngine {}
unsafe impl Sync for FhirPathEngine {}

/// Implementation of ExpressionEvaluator for lambda functions
#[async_trait]
impl ExpressionEvaluator for FhirPathEngine {
    async fn evaluate_expression(
        &self,
        expression: &ExpressionNode,
        context: &RegistryEvaluationContext,
    ) -> octofhir_fhirpath_core::Result<FhirPathValue> {
        // Convert registry context to local context for evaluation
        let mut local_context = LocalEvaluationContext::new(
            context.input.clone(),
            self.registry().clone(),
            self.model_provider().clone(),
        );

        // Copy variables from registry context to local context
        // Extract lambda variables and regular variables
        let mut lambda_this = None;
        let mut lambda_index = None;
        let mut lambda_total = None;

        for (name, value) in &context.variables {
            match name.as_str() {
                "$this" => lambda_this = Some(value.clone()),
                "$index" => lambda_index = Some(value.clone()),
                "$total" => lambda_total = Some(value.clone()),
                _ => {
                    // Regular variables
                    local_context.set_variable(name.clone(), value.clone());
                }
            }
        }

        // Create lambda context if any lambda variables were found
        let final_context =
            if lambda_this.is_some() || lambda_index.is_some() || lambda_total.is_some() {
                let this_value = lambda_this.unwrap_or(context.input.clone());
                let index_value = if let Some(FhirPathValue::Integer(idx)) = lambda_index {
                    idx as usize
                } else {
                    0
                };
                let total_value = lambda_total.unwrap_or(FhirPathValue::Empty);

                local_context.with_lambda_implicits(this_value, index_value, total_value)
            } else {
                local_context
            };

        self.evaluate_node_async(expression, context.input.clone(), &final_context, 0)
            .await
            .map_err(|e| match e {
                EvaluationError::InvalidOperation { message } => {
                    octofhir_fhirpath_core::FhirPathError::EvaluationError {
                        message,
                        expression: None,
                        location: None,
                    }
                }
                EvaluationError::TypeError { expected, actual } => {
                    octofhir_fhirpath_core::FhirPathError::TypeError {
                        message: format!("Type mismatch: expected {expected}, got {actual}"),
                    }
                }
                EvaluationError::RuntimeError { message } => {
                    octofhir_fhirpath_core::FhirPathError::EvaluationError {
                        message,
                        expression: None,
                        location: None,
                    }
                }
                EvaluationError::Function(message) => {
                    octofhir_fhirpath_core::FhirPathError::FunctionError {
                        function_name: "unknown".to_string(),
                        message,
                        arguments: None,
                    }
                }
                EvaluationError::Operator(message) => {
                    octofhir_fhirpath_core::FhirPathError::EvaluationError {
                        message,
                        expression: None,
                        location: None,
                    }
                }
                _ => octofhir_fhirpath_core::FhirPathError::EvaluationError {
                    message: e.to_string(),
                    expression: None,
                    location: None,
                },
            })
    }

    fn try_evaluate_expression_sync(
        &self,
        _expression: &ExpressionNode,
        _context: &RegistryEvaluationContext,
    ) -> Option<octofhir_fhirpath_core::Result<FhirPathValue>> {
        // For now, we don't support synchronous expression evaluation
        // This could be implemented for specific expression types that don't need async
        None
    }
}
