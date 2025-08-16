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
    /// use serde_json::json;
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
    /// use serde_json::json;
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
    /// use serde_json::json;
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
    /// use serde_json::json;
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
        input_data: serde_json::Value,
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

        // Create evaluation context with unified registry
        let context = LocalEvaluationContext::new(
            fhir_value.clone(),
            self.registry.clone(),
            self.model_provider.clone(),
        );

        // Use the AST evaluation method
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
    /// use serde_json::json;
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
    /// use serde_json::json;
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
        input_data: serde_json::Value,
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
            self.registry.clone(),
            self.model_provider.clone(),
            variables.into_iter().collect(),
        );

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
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use serde_json::json;
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

    /// Core recursive evaluator - handles all node types
    fn evaluate_node_async<'a>(
        &'a self,
        node: &'a ExpressionNode,
        input: FhirPathValue,
        context: &'a LocalEvaluationContext,
        depth: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = EvaluationResult<FhirPathValue>> + Send + 'a>,
    > {
        Box::pin(async move {
            // Recursion depth check
            if depth > self.config.max_recursion_depth {
                return Err(EvaluationError::InvalidOperation {
                    message: format!(
                        "Recursion depth exceeded: max depth is {}",
                        self.config.max_recursion_depth
                    ),
                });
            }

            // Performance monitoring hook
            let start_time = std::time::Instant::now();
            let result = self
                .evaluate_node_internal(node, input, context, depth)
                .await;
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
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_ast::ExpressionNode;

        match node {
            // Simple cases - direct evaluation
            ExpressionNode::Literal(lit) => self.evaluate_literal(lit),

            ExpressionNode::Identifier(id) => self.evaluate_identifier(id, &input, context),

            ExpressionNode::Index { base, index } => {
                self.evaluate_index(base, index, input, context, depth)
                    .await
            }

            ExpressionNode::Path { base, path } => {
                self.evaluate_path(base, path, input, context, depth).await
            }

            // Complex cases - delegate to specialized methods
            ExpressionNode::FunctionCall(func_data) => {
                if self.is_lambda_function(&func_data.name).await {
                    self.evaluate_lambda_function(func_data, input, context, depth)
                        .await
                } else {
                    self.evaluate_standard_function(func_data, input, context, depth)
                        .await
                }
            }

            ExpressionNode::BinaryOp(op_data) => {
                self.evaluate_binary_operation(op_data, input, context, depth)
                    .await
            }

            ExpressionNode::UnaryOp { op, operand } => {
                self.evaluate_unary_operation(op, operand, input, context, depth)
                    .await
            }

            ExpressionNode::MethodCall(method_data) => {
                self.evaluate_method_call(method_data, input, context, depth)
                    .await
            }

            ExpressionNode::Lambda(lambda_data) => {
                self.evaluate_lambda_expression(lambda_data, input, context, depth)
                    .await
            }

            ExpressionNode::Conditional(cond_data) => {
                self.evaluate_conditional(cond_data, input, context, depth)
                    .await
            }

            ExpressionNode::Variable(var_name) => self.evaluate_variable(var_name, context),

            ExpressionNode::Filter { base, condition } => {
                self.evaluate_filter(base, condition, input, context, depth)
                    .await
            }

            ExpressionNode::Union { left, right } => {
                self.evaluate_union(left, right, input, context, depth)
                    .await
            }

            ExpressionNode::TypeCheck {
                expression,
                type_name,
            } => {
                self.evaluate_type_check(expression, type_name, input, context, depth)
                    .await
            }

            ExpressionNode::TypeCast {
                expression,
                type_name,
            } => {
                self.evaluate_type_cast(expression, type_name, input, context, depth)
                    .await
            }
        }
    }

    // Simple node type handlers

    /// Check if an expression represents a type identifier (like Date, String, etc.)
    fn is_type_identifier_expression(expr: &ExpressionNode) -> bool {
        match expr {
            ExpressionNode::Identifier(name) => {
                // Check if this identifier looks like a type name
                // Type names typically start with uppercase letter
                name.chars().next().is_some_and(|c| c.is_uppercase()) ||
                // Or are known primitive type names  
                matches!(name.as_str(), "boolean" | "integer" | "decimal" | "string" | "date" | "datetime" | "time" | "collection" | "empty" | "quantity")
            }
            _ => false,
        }
    }

    /// Parse FHIRPath date literal supporting partial dates
    /// Supports: @YYYY, @YYYY-MM, @YYYY-MM-DD
    fn parse_fhirpath_date(date_str: &str) -> Result<chrono::NaiveDate, String> {
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
                chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                    .ok_or_else(|| format!("Invalid date: {date_str}"))
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
                chrono::NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| format!("Invalid date: {date_str}"))
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
                chrono::NaiveDate::from_ymd_opt(year, month, day)
                    .ok_or_else(|| format!("Invalid date: {date_str}"))
            }
            _ => Err(format!("Invalid date format: {date_str}")),
        }
    }

    /// Parse FHIRPath datetime literal supporting partial datetimes
    /// Supports: @2015T, @2015-02T, @2015-02-04T14:34:28Z, etc.
    fn parse_fhirpath_datetime(
        datetime_str: &str,
    ) -> Result<chrono::DateTime<chrono::FixedOffset>, String> {
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
        let date = if date_part.is_empty() {
            // Handle @T... format (time only, use epoch date)
            chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .ok_or_else(|| "Failed to create epoch date".to_string())?
        } else {
            Self::parse_fhirpath_date(&format!("@{date_part}"))?
        };

        // Parse the time part
        let (time, offset) = if time_part.is_empty() {
            // Handle partial datetime like @2015T (no time specified)
            (
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                chrono::FixedOffset::east_opt(0).unwrap(),
            )
        } else {
            Self::parse_fhirpath_time_with_tz(time_part)?
        };

        // Combine date and time
        let naive_datetime = date.and_time(time);

        // Create datetime with timezone
        offset
            .from_local_datetime(&naive_datetime)
            .single()
            .ok_or_else(|| format!("Invalid datetime: {datetime_str}"))
    }

    /// Parse FHIRPath time literal supporting partial times and timezones
    /// Supports: @T14, @T14:34, @T14:34:28, @T14:34:28.123, @T14:34:28Z, @T14:34:28+10:00
    fn parse_fhirpath_time(time_str: &str) -> Result<chrono::NaiveTime, String> {
        // Remove the @T prefix if present
        let time_str = time_str
            .strip_prefix('@')
            .and_then(|s| s.strip_prefix('T'))
            .unwrap_or(time_str);

        // Remove timezone info for parsing time only
        let (time_part, _) = Self::split_time_timezone(time_str);

        Self::parse_time_components(time_part)
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

    /// Evaluate literal values
    fn evaluate_literal(
        &self,
        literal: &octofhir_fhirpath_ast::LiteralValue,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_ast::LiteralValue::*;
        use std::str::FromStr;

        let value = match literal {
            Boolean(b) => FhirPathValue::Boolean(*b),
            Integer(i) => FhirPathValue::Integer(*i),
            Decimal(d) => {
                let decimal = rust_decimal::Decimal::from_str(d).map_err(|_| {
                    EvaluationError::InvalidOperation {
                        message: format!("Invalid decimal value: {d}"),
                    }
                })?;
                FhirPathValue::Decimal(decimal)
            }
            String(s) => FhirPathValue::String(s.clone().into()),
            Date(d) => {
                let date = Self::parse_fhirpath_date(d).map_err(|err| {
                    EvaluationError::InvalidOperation {
                        message: format!("Invalid date value: {d} - {err}"),
                    }
                })?;
                FhirPathValue::Date(date)
            }
            DateTime(dt) => {
                let datetime = Self::parse_fhirpath_datetime(dt).map_err(|err| {
                    EvaluationError::InvalidOperation {
                        message: format!("Invalid datetime value: {dt} - {err}"),
                    }
                })?;
                FhirPathValue::DateTime(datetime)
            }
            Time(t) => {
                let time = Self::parse_fhirpath_time(t).map_err(|err| {
                    EvaluationError::InvalidOperation {
                        message: format!("Invalid time value: {t} - {err}"),
                    }
                })?;
                FhirPathValue::Time(time)
            }
            Quantity { value, unit } => {
                let decimal_value = rust_decimal::Decimal::from_str(value).map_err(|_| {
                    EvaluationError::InvalidOperation {
                        message: format!("Invalid quantity value: {value}"),
                    }
                })?;
                let quantity =
                    octofhir_fhirpath_model::Quantity::new(decimal_value, Some(unit.clone()));
                FhirPathValue::Quantity(std::sync::Arc::new(quantity))
            }
            Null => FhirPathValue::Empty,
        };

        Ok(FhirPathValue::collection(vec![value]))
    }

    /// Evaluate identifiers (property access or resource type filtering)
    fn evaluate_identifier(
        &self,
        identifier: &str,
        input: &FhirPathValue,
        _context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let clean_identifier = identifier.trim_matches('`');

        if self.is_resource_type(clean_identifier) {
            return self.filter_by_resource_type(input, clean_identifier);
        }

        // First try property access on collections or single values
        if let FhirPathValue::Collection(items) = input {
            let mut results = Vec::new();
            for item in items.iter() {
                if let Some(property_value) = self.get_property_value(item, clean_identifier) {
                    match property_value {
                        FhirPathValue::Collection(sub_items) => {
                            results.extend(sub_items.iter().cloned());
                        }
                        single_value => {
                            results.push(single_value);
                        }
                    }
                }
            }
            Ok(FhirPathValue::collection(results))
        } else if let Some(property_value) = self.get_property_value(input, clean_identifier) {
            Ok(property_value)
        } else {
            // Only check for type identifiers if property access fails
            if self.is_type_identifier(clean_identifier) {
                let (namespace, name) = if clean_identifier.contains('.') {
                    let parts: Vec<&str> = clean_identifier.split('.').collect();
                    (parts[0], parts[1])
                } else {
                    ("System", clean_identifier)
                };

                Ok(FhirPathValue::TypeInfoObject {
                    namespace: Arc::from(namespace),
                    name: Arc::from(name),
                })
            } else {
                Ok(FhirPathValue::collection(vec![]))
            }
        }
    }

    /// Evaluate variable references
    fn evaluate_variable(
        &self,
        var_name: &str,
        context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Handle implicit lambda variables ($this, $index, $total)
        // Note: The parser strips the $ prefix, so we need to check both with and without

        match var_name {
            "this" | "$this" => {
                // In lambda context, $this refers to the current item
                if let Some(var) = context.get_variable("$this") {
                    return Ok(var.clone());
                }
                // If not in lambda context, $this refers to the input context
                Ok(context.input.clone())
            }
            "index" | "$index" => {
                // Return the current index in lambda context
                if let Some(var) = context.get_variable("$index") {
                    return Ok(var.clone());
                }
                // Default to 0 if not in indexed lambda
                Ok(FhirPathValue::singleton(FhirPathValue::Integer(0)))
            }
            "total" | "$total" => {
                // Handle $total for aggregate functions
                if let Some(var) = context.get_variable("$total") {
                    return Ok(var.clone());
                }
                // For aggregate operations, initialize to empty
                Ok(FhirPathValue::Empty)
            }
            _ => {
                // Handle standard environment variables according to FHIRPath spec
                // Note: The % prefix is stripped during parsing, so we match on the name without %
                match var_name {
                    // Standard system environment variables (cannot be overridden)
                    "context" => {
                        // The original node that was passed to the evaluation engine
                        Ok(context.root.clone())
                    }
                    "resource" => {
                        // The resource that contains the original node
                        // For most cases, this is the same as %context
                        Ok(self.get_containing_resource(&context.root))
                    }
                    "rootResource" => {
                        // The container resource (for contained resources)
                        // In most cases, same as %resource unless dealing with contained resources
                        Ok(self.get_root_resource(&context.root))
                    }
                    "sct" => {
                        // SNOMED CT URL
                        Ok(FhirPathValue::singleton(FhirPathValue::String(
                            "http://snomed.info/sct".into(),
                        )))
                    }
                    "loinc" => {
                        // LOINC URL
                        Ok(FhirPathValue::singleton(FhirPathValue::String(
                            "http://loinc.org".into(),
                        )))
                    }
                    "ucum" => {
                        // UCUM URL
                        Ok(FhirPathValue::singleton(FhirPathValue::String(
                            "http://unitsofmeasure.org".into(),
                        )))
                    }
                    // Check for HL7 value set variables ("vs-[name]" without the % prefix)
                    name if name.starts_with("\"vs-") && name.ends_with('"') => {
                        let vs_name = &name[4..name.len() - 1]; // Extract vs name between quotes
                        Ok(FhirPathValue::singleton(FhirPathValue::String(
                            format!("http://hl7.org/fhir/ValueSet/{vs_name}").into(),
                        )))
                    }
                    // Check for standard value set pattern without quotes (vs-name)
                    name if name.starts_with("vs-") => {
                        let vs_name = &name[3..]; // Extract vs name after vs-
                        Ok(FhirPathValue::singleton(FhirPathValue::String(
                            format!("http://hl7.org/fhir/ValueSet/{vs_name}").into(),
                        )))
                    }
                    // Check for extension pattern (ext-name)
                    name if name.starts_with("ext-") => {
                        let ext_name = &name[4..]; // Extract extension name after ext-
                        Ok(FhirPathValue::singleton(FhirPathValue::String(
                            format!("http://hl7.org/fhir/StructureDefinition/{ext_name}").into(),
                        )))
                    }
                    // Check for quoted extension pattern ("ext-name")
                    name if name.starts_with("\"ext-") && name.ends_with('"') => {
                        let ext_name = &name[5..name.len() - 1]; // Extract extension name between quotes
                        Ok(FhirPathValue::singleton(FhirPathValue::String(
                            format!("http://hl7.org/fhir/StructureDefinition/{ext_name}").into(),
                        )))
                    }
                    // User-defined variables (can be overridden by user)
                    _ => {
                        // First check user-defined variables
                        if let Some(var) = context.get_variable(var_name) {
                            Ok(var.clone())
                        } else {
                            // Return empty for undefined variables (per FHIRPath spec)
                            Ok(FhirPathValue::Empty)
                        }
                    }
                }
            }
        }
    }

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

    /// Helper: Get property value from a FhirPathValue
    fn get_property_value(&self, value: &FhirPathValue, property: &str) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_arc) => {
                if let Some(obj) = json_arc.as_object() {
                    // First try direct property access
                    if let Some(value) = obj.get(property) {
                        let mut fhir_value = FhirPathValue::from(value.clone());
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
                        for (key, value) in obj.iter() {
                            if key.starts_with("value") && key != "value" {
                                return Some(FhirPathValue::from(value.clone()));
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
                    let mut fhir_value = FhirPathValue::from(v.clone());
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

    /// Check if an identifier represents a FHIRPath type identifier
    fn is_type_identifier(&self, identifier: &str) -> bool {
        // Handle namespaced types
        if identifier.contains('.') {
            let parts: Vec<&str> = identifier.split('.').collect();
            if parts.len() == 2 {
                let (namespace, type_name) = (parts[0], parts[1]);
                match namespace {
                    "System" => matches!(
                        type_name,
                        "Boolean"
                            | "Integer"
                            | "Decimal"
                            | "String"
                            | "Date"
                            | "DateTime"
                            | "Time"
                            | "Quantity"
                            | "Collection"
                    ),
                    "FHIR" => {
                        // Common FHIR resource types and primitive types
                        matches!(
                            type_name,
                            "Patient"
                                | "Observation"
                                | "Practitioner"
                                | "Organization"
                                | "Encounter"
                                | "Condition"
                                | "Procedure"
                                | "DiagnosticReport"
                                | "Medication"
                                | "MedicationStatement"
                                | "AllergyIntolerance"
                                | "Bundle"
                                | "CapabilityStatement"
                                | "ValueSet"
                                | "CodeSystem"
                                | "StructureDefinition"
                                | "OperationDefinition"
                                | "SearchParameter"
                                | "Resource"
                                | "DomainResource"
                                | "MetadataResource"
                                | "boolean"
                                | "integer"
                                | "decimal"
                                | "string"
                                | "date"
                                | "dateTime"
                                | "time"
                                | "uri"
                                | "url"
                                | "canonical"
                                | "uuid"
                                | "oid"
                                | "id"
                                | "code"
                                | "markdown"
                                | "base64Binary"
                                | "instant"
                                | "positiveInt"
                                | "unsignedInt"
                        )
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else {
            // Handle unqualified types
            matches!(
                identifier,
                "Boolean" | "Integer" | "Decimal" | "String" | "Date" | "DateTime" | "Time" | "Quantity" | "Collection" |
                // FHIR resource types
                "Patient" | "Observation" | "Practitioner" | "Organization" | "Encounter" |
                "Condition" | "Procedure" | "DiagnosticReport" | "Medication" | "MedicationStatement" |
                "AllergyIntolerance" | "Bundle" | "CapabilityStatement" | "ValueSet" | "CodeSystem" |
                "StructureDefinition" | "OperationDefinition" | "SearchParameter" | "Resource" |
                "DomainResource" | "MetadataResource" |
                // FHIR primitive types
                "boolean" | "integer" | "decimal" | "string" | "date" | "dateTime" | "time" |
                "uri" | "url" | "canonical" | "uuid" | "oid" | "id" | "code" | "markdown" |
                "base64Binary" | "instant" | "positiveInt" | "unsignedInt"
            )
        }
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
                if let Some(obj) = json_arc.as_object() {
                    if let Some(rt) = obj.get("resourceType").and_then(|v| v.as_str()) {
                        return rt.eq_ignore_ascii_case(resource_type);
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
    async fn evaluate_path(
        &self,
        base: &ExpressionNode,
        path: &str,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Check if this is a type identifier like FHIR.Patient, System.String, etc.
        if let ExpressionNode::Identifier(base_id) = base {
            let full_type_name = format!("{base_id}.{path}");
            if self.is_type_identifier(&full_type_name) {
                let (namespace, name) = (base_id.as_str(), path);
                return Ok(FhirPathValue::TypeInfoObject {
                    namespace: Arc::from(namespace),
                    name: Arc::from(name),
                });
            }
        }

        // Special handling for defineVariable function calls in the base
        // If the base is a defineVariable call, we need to propagate the variable context
        if let ExpressionNode::FunctionCall(func_data) = base {
            if func_data.name == "defineVariable" {
                // First evaluate the defineVariable function to set up the variable
                let base_result = self
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

                            // Continue evaluation with the updated context
                            return self.evaluate_identifier(path, &base_result, &new_context);
                        }
                    }
                }
            }
        }

        // First evaluate the base expression
        let base_result = self
            .evaluate_node_async(base, input, context, depth + 1)
            .await?;
        // Apply path navigation to the result
        self.evaluate_identifier(path, &base_result, context)
    }

    /// Evaluate index access expressions
    async fn evaluate_index(
        &self,
        base: &ExpressionNode,
        index_expr: &ExpressionNode,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the base expression
        let base_result = self
            .evaluate_node_async(base, input.clone(), context, depth + 1)
            .await?;

        // Then evaluate the index expression
        let index_result = self
            .evaluate_node_async(index_expr, input, context, depth + 1)
            .await?;

        // Extract index value
        let index = match &index_result {
            FhirPathValue::Integer(i) => *i,
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.iter().next().unwrap() {
                    FhirPathValue::Integer(i) => *i,
                    _ => {
                        return Err(EvaluationError::InvalidOperation {
                            message: "Index must be an integer".to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(EvaluationError::InvalidOperation {
                    message: "Index must be an integer".to_string(),
                });
            }
        };

        // Apply index to base result
        match base_result {
            FhirPathValue::Collection(items) => {
                if index < 0 {
                    return Ok(FhirPathValue::collection(vec![]));
                }
                let idx = index as usize;
                if idx < items.len() {
                    Ok(FhirPathValue::collection(vec![
                        items.iter().nth(idx).unwrap().clone(),
                    ]))
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
        context: &LocalEvaluationContext,
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

        // Evaluate left operand
        let left = self
            .evaluate_node_async(&op_data.left, input.clone(), context, depth + 1)
            .await?;

        // Evaluate right operand
        let right = self
            .evaluate_node_async(&op_data.right, input.clone(), context, depth + 1)
            .await?;

        // Get operation from registry and evaluate
        if let Some(operation) = self.registry.get_operation(symbol).await {
            // Create registry context for operation evaluation
            let registry_context = octofhir_fhirpath_registry::operations::EvaluationContext::new(
                input,
                self.registry.clone(),
                self.model_provider.clone(),
            );

            operation
                .evaluate(&[left, right], &registry_context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Binary operator error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Unknown binary operator: {symbol}"),
            })
        }
    }

    /// Evaluate unary operations
    async fn evaluate_unary_operation(
        &self,
        op: &octofhir_fhirpath_ast::UnaryOperator,
        operand: &ExpressionNode,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Get the operator symbol
        let symbol = match op {
            octofhir_fhirpath_ast::UnaryOperator::Plus => "+",
            octofhir_fhirpath_ast::UnaryOperator::Minus => "-",
            octofhir_fhirpath_ast::UnaryOperator::Not => "not",
        };

        // Evaluate operand
        let operand_value = self
            .evaluate_node_async(operand, input.clone(), context, depth + 1)
            .await?;

        // Get operation from registry and evaluate
        if let Some(operation) = self.registry.get_operation(symbol).await {
            // Create registry context for operation evaluation
            let registry_context = octofhir_fhirpath_registry::operations::EvaluationContext::new(
                input,
                self.registry.clone(),
                self.model_provider.clone(),
            );

            operation
                .evaluate(&[operand_value], &registry_context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Unary operator error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Unknown unary operator: {symbol}"),
            })
        }
    }

    /// Evaluate conditional expressions (iif)
    async fn evaluate_conditional(
        &self,
        cond_data: &octofhir_fhirpath_ast::ConditionalData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate condition
        let condition = self
            .evaluate_node_async(&cond_data.condition, input.clone(), context, depth + 1)
            .await?;

        // Check if condition is true
        let is_true = self.is_truthy(&condition);

        // Evaluate appropriate branch
        if is_true {
            self.evaluate_node_async(&cond_data.then_expr, input, context, depth + 1)
                .await
        } else if let Some(else_expr) = &cond_data.else_expr {
            self.evaluate_node_async(else_expr, input, context, depth + 1)
                .await
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
    async fn evaluate_union(
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
    async fn evaluate_type_check(
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
    async fn evaluate_type_cast(
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
            root: context.root.clone(),
            variables: context.variable_scope.variables.as_ref().clone(),
            model_provider: self.model_provider.clone(),
            registry: self.registry.clone(),
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
    ///
    /// For non-lambda functions, arguments are evaluated in the current context.
    /// The key fix: arguments should be evaluated against the current input (the focus
    /// of the function call), not against individual items in a collection.
    ///
    /// Example: In `Patient.name.select(given.combine(family))`:
    /// - `select()` is lambda, creates context for each name element
    /// - `combine()` is non-lambda, its arguments (`given`, `family`) should be
    ///   evaluated against the current name element (the input to combine)
    async fn evaluate_standard_function(
        &self,
        func_data: &octofhir_fhirpath_ast::FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Special handling for defineVariable function - it needs to modify the evaluation context
        if func_data.name == "defineVariable" {
            return self
                .evaluate_define_variable_function(func_data, input, context, depth)
                .await;
        }

        // Pre-evaluate all arguments against the current input with current context
        // This is the correct behavior: arguments are evaluated in the current context
        // but against the input that the function will operate on
        let mut evaluated_args = Vec::with_capacity(func_data.args.len());
        for arg_expr in &func_data.args {
            // Evaluate arguments against the function's input (current focus)
            // but with the current variable scope from context
            let arg_value = self
                .evaluate_node_async(arg_expr, input.clone(), context, depth + 1)
                .await?;
            evaluated_args.push(arg_value);
        }

        // Get function from registry and evaluate
        if let Some(operation) = self.registry.get_operation(&func_data.name).await {
            // Create registry context for operation evaluation
            // Pass variables from the current context to the registry context
            let all_variables = context.variable_scope.collect_all_variables();
            let registry_context =
                octofhir_fhirpath_registry::operations::EvaluationContext::with_variables(
                    input,
                    self.registry.clone(),
                    self.model_provider.clone(),
                    all_variables,
                );

            operation
                .evaluate(&evaluated_args, &registry_context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Function error in {}: {}", func_data.name, e),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Unknown function: {}", func_data.name),
            })
        }
    }

    /// Special evaluation for defineVariable function that needs to modify the evaluation context
    pub async fn evaluate_define_variable_function(
        &self,
        func_data: &octofhir_fhirpath_ast::FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Validate arguments: defineVariable(name) or defineVariable(name, value)
        if func_data.args.is_empty() || func_data.args.len() > 2 {
            return Err(EvaluationError::InvalidOperation {
                message: "defineVariable() requires 1 or 2 arguments (name, [value])".to_string(),
            });
        }

        // Evaluate the variable name argument
        let name_value = self
            .evaluate_node_async(&func_data.args[0], input.clone(), context, depth + 1)
            .await?;

        // Extract variable name
        let var_name = match &name_value {
            FhirPathValue::String(name) => name.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => match items.first().unwrap() {
                FhirPathValue::String(name) => name.as_ref(),
                _ => {
                    return Err(EvaluationError::InvalidOperation {
                        message: "defineVariable() name parameter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(EvaluationError::InvalidOperation {
                    message: "defineVariable() name parameter must be a string".to_string(),
                });
            }
        };

        // Check if the variable name is a system variable (protected)
        if Self::is_system_variable(var_name) {
            return Err(EvaluationError::InvalidOperation {
                message: format!("Cannot override system variable '{var_name}'"),
            });
        }

        // Extract variable value - use current input if not provided
        let var_value = if func_data.args.len() == 2 {
            self.evaluate_node_async(&func_data.args[1], input.clone(), context, depth + 1)
                .await?
        } else {
            input.clone()
        };

        // This is the key part: defineVariable should create a special result that carries
        // the variable context forward. Since we can't modify the context directly here,
        // we need a different approach. The variable needs to be set in a way that
        // subsequent operations in the same expression chain can access it.

        // For now, return the input and note that this implementation is incomplete
        // The proper solution requires architectural changes to how contexts are handled
        // TODO: Implement proper context propagation for defineVariable
        Ok(input)
    }

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

    /// Check if a function name represents a lambda function by delegating to registry
    ///
    /// This method provides a robust way to distinguish between lambda functions
    /// (which need raw expressions) and regular functions (which need pre-evaluated arguments).
    ///
    /// The actual detection logic is implemented in the registry to maintain consistency
    /// across different evaluation contexts.
    async fn is_lambda_function(&self, name: &str) -> bool {
        self.registry.is_lambda_function(name).await
    }

    /// Evaluate lambda functions with expression arguments
    ///
    /// Lambda functions receive raw expressions instead of pre-evaluated values,
    /// allowing them to control evaluation context and implement proper variable
    /// scoping for $this, $index, $total, etc.
    async fn evaluate_lambda_function(
        &self,
        func_data: &octofhir_fhirpath_ast::FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        _depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Get operation from registry
        if let Some(operation) = self.registry.get_operation(&func_data.name).await {
            // Create registry context for lambda evaluation with variables from engine context
            let all_variables = context.variable_scope.collect_all_variables();
            let registry_context =
                octofhir_fhirpath_registry::operations::EvaluationContext::with_variables(
                    input,
                    self.registry.clone(),
                    self.model_provider.clone(),
                    all_variables,
                );

            // Try specific lambda function types for proper lambda function evaluation
            use octofhir_fhirpath_registry::lambda::LambdaFunction;
            use octofhir_fhirpath_registry::operations::collection::AllFunction;
            use octofhir_fhirpath_registry::operations::lambda::{
                AggregateFunction, RepeatFunction, SelectFunction, SortLambdaFunction,
                WhereFunction,
            };

            if let Some(where_func) = operation.as_any().downcast_ref::<WhereFunction>() {
                where_func
                    .evaluate_lambda(&func_data.args, &registry_context, self)
                    .await
                    .map_err(|e| EvaluationError::InvalidOperation {
                        message: format!("Lambda function error in {}: {}", func_data.name, e),
                    })
            } else if let Some(select_func) = operation.as_any().downcast_ref::<SelectFunction>() {
                select_func
                    .evaluate_lambda(&func_data.args, &registry_context, self)
                    .await
                    .map_err(|e| EvaluationError::InvalidOperation {
                        message: format!("Lambda function error in {}: {}", func_data.name, e),
                    })
            } else if let Some(sort_func) = operation.as_any().downcast_ref::<SortLambdaFunction>()
            {
                sort_func
                    .evaluate_lambda(&func_data.args, &registry_context, self)
                    .await
                    .map_err(|e| EvaluationError::InvalidOperation {
                        message: format!("Lambda function error in {}: {}", func_data.name, e),
                    })
            } else if let Some(aggregate_func) =
                operation.as_any().downcast_ref::<AggregateFunction>()
            {
                aggregate_func
                    .evaluate_lambda(&func_data.args, &registry_context, self)
                    .await
                    .map_err(|e| EvaluationError::InvalidOperation {
                        message: format!("Lambda function error in {}: {}", func_data.name, e),
                    })
            } else if let Some(repeat_func) = operation.as_any().downcast_ref::<RepeatFunction>() {
                repeat_func
                    .evaluate_lambda(&func_data.args, &registry_context, self)
                    .await
                    .map_err(|e| EvaluationError::InvalidOperation {
                        message: format!("Lambda function error in {}: {}", func_data.name, e),
                    })
            } else if let Some(all_func) = operation.as_any().downcast_ref::<AllFunction>() {
                all_func
                    .evaluate_lambda(&func_data.args, &registry_context, self)
                    .await
                    .map_err(|e| EvaluationError::InvalidOperation {
                        message: format!("Lambda function error in {}: {}", func_data.name, e),
                    })
            } else {
                Err(EvaluationError::InvalidOperation {
                    message: format!(
                        "Function {} is not a recognized lambda function",
                        func_data.name
                    ),
                })
            }
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Unknown lambda function: {}", func_data.name),
            })
        }
    }

    /// Evaluate where lambda function
    async fn evaluate_where_lambda(
        &self,
        func_data: &octofhir_fhirpath_ast::FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        if func_data.args.len() != 1 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "where() requires exactly 1 argument, got {}",
                    func_data.args.len()
                ),
            });
        }

        let predicate_expr = &func_data.args[0];

        match &input {
            FhirPathValue::Collection(items) => {
                let mut filtered_items = Vec::new();

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context with $this variable set to current item
                    let mut lambda_context = context.clone();
                    lambda_context.set_variable("$this".to_string(), item.clone());
                    lambda_context
                        .set_variable("$index".to_string(), FhirPathValue::Integer(index as i64));
                    lambda_context = lambda_context.with_input(item.clone());

                    // Evaluate predicate expression in lambda context
                    let predicate_result = self
                        .evaluate_node_async(
                            predicate_expr,
                            item.clone(),
                            &lambda_context,
                            depth + 1,
                        )
                        .await?;

                    // Check if predicate is true
                    if self.is_truthy(&predicate_result) {
                        filtered_items.push(item.clone());
                    }
                }

                // Return results
                if filtered_items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(filtered_items))
                }
            }
            single_item => {
                // Apply where to single item
                let mut lambda_context = context.clone();
                lambda_context.set_variable("$this".to_string(), single_item.clone());
                lambda_context.set_variable("$index".to_string(), FhirPathValue::Integer(0));
                lambda_context = lambda_context.with_input(single_item.clone());

                let predicate_result = self
                    .evaluate_node_async(
                        predicate_expr,
                        single_item.clone(),
                        &lambda_context,
                        depth + 1,
                    )
                    .await?;

                if self.is_truthy(&predicate_result) {
                    Ok(single_item.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    /// Evaluate select lambda function
    async fn evaluate_select_lambda(
        &self,
        func_data: &octofhir_fhirpath_ast::FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        if func_data.args.len() != 1 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "select() requires exactly 1 argument, got {}",
                    func_data.args.len()
                ),
            });
        }

        let transform_expr = &func_data.args[0];

        match &input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context with $this variable set to current item
                    let mut lambda_context = context.clone();
                    lambda_context.set_variable("$this".to_string(), item.clone());
                    lambda_context
                        .set_variable("$index".to_string(), FhirPathValue::Integer(index as i64));
                    lambda_context = lambda_context.with_input(item.clone());

                    // Evaluate transform expression in lambda context
                    let transform_result = self
                        .evaluate_node_async(
                            transform_expr,
                            item.clone(),
                            &lambda_context,
                            depth + 1,
                        )
                        .await?;

                    // Collect results
                    match transform_result {
                        FhirPathValue::Collection(sub_items) => {
                            results.extend(sub_items.iter().cloned());
                        }
                        FhirPathValue::Empty => {
                            // Skip empty results
                        }
                        single_value => {
                            results.push(single_value);
                        }
                    }
                }

                Ok(FhirPathValue::collection(results))
            }
            single_item => {
                // Apply select to single item
                let mut lambda_context = context.clone();
                lambda_context.set_variable("$this".to_string(), single_item.clone());
                lambda_context.set_variable("$index".to_string(), FhirPathValue::Integer(0));
                lambda_context = lambda_context.with_input(single_item.clone());

                let result = self
                    .evaluate_node_async(
                        transform_expr,
                        single_item.clone(),
                        &lambda_context,
                        depth + 1,
                    )
                    .await?;

                Ok(result)
            }
        }
    }

    /// Evaluate lambda expressions (inline lambda syntax)
    async fn evaluate_lambda_expression(
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
                LambdaType::Any => {
                    // Any: return true if any result is true
                    if self.is_truthy(&result) {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                            true,
                        )]));
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
            LambdaType::All => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                true,
            )])), // All were true
            LambdaType::Any => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                false,
            )])), // None were true
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
                    // Handle lambda method call - use raw expression arguments
                    if let Some(operation) = self.registry.get_operation(method_name).await {
                        // Try to downcast to specific lambda function types
                        use octofhir_fhirpath_registry::lambda::LambdaFunction;
                        use octofhir_fhirpath_registry::operations::collection::AllFunction;
                        use octofhir_fhirpath_registry::operations::lambda::{
                            AggregateFunction, RepeatFunction, SelectFunction, SortLambdaFunction,
                            WhereFunction,
                        };

                        // Create registry context with the object as input for the lambda and variables from engine context
                        let all_variables = context.variable_scope.collect_all_variables();
                        let registry_context = octofhir_fhirpath_registry::operations::EvaluationContext::with_variables(
                            object,
                            self.registry.clone(),
                            self.model_provider.clone(),
                            all_variables,
                        );

                        // Try each lambda function type
                        if let Some(where_func) = operation.as_any().downcast_ref::<WhereFunction>()
                        {
                            where_func
                                .evaluate_lambda(args, &registry_context, self)
                                .await
                                .map_err(|e| EvaluationError::InvalidOperation {
                                    message: format!("Lambda method error in {method_name}: {e}"),
                                })
                        } else if let Some(select_func) =
                            operation.as_any().downcast_ref::<SelectFunction>()
                        {
                            select_func
                                .evaluate_lambda(args, &registry_context, self)
                                .await
                                .map_err(|e| EvaluationError::InvalidOperation {
                                    message: format!("Lambda method error in {method_name}: {e}"),
                                })
                        } else if let Some(sort_func) =
                            operation.as_any().downcast_ref::<SortLambdaFunction>()
                        {
                            sort_func
                                .evaluate_lambda(args, &registry_context, self)
                                .await
                                .map_err(|e| EvaluationError::InvalidOperation {
                                    message: format!("Lambda method error in {method_name}: {e}"),
                                })
                        } else if let Some(aggregate_func) =
                            operation.as_any().downcast_ref::<AggregateFunction>()
                        {
                            aggregate_func
                                .evaluate_lambda(args, &registry_context, self)
                                .await
                                .map_err(|e| EvaluationError::InvalidOperation {
                                    message: format!("Lambda method error in {method_name}: {e}"),
                                })
                        } else if let Some(repeat_func) =
                            operation.as_any().downcast_ref::<RepeatFunction>()
                        {
                            repeat_func
                                .evaluate_lambda(args, &registry_context, self)
                                .await
                                .map_err(|e| EvaluationError::InvalidOperation {
                                    message: format!("Lambda method error in {method_name}: {e}"),
                                })
                        } else if let Some(all_func) =
                            operation.as_any().downcast_ref::<AllFunction>()
                        {
                            all_func
                                .evaluate_lambda(args, &registry_context, self)
                                .await
                                .map_err(|e| EvaluationError::InvalidOperation {
                                    message: format!("Lambda method error in {method_name}: {e}"),
                                })
                        } else {
                            Err(EvaluationError::InvalidOperation {
                                message: format!(
                                    "Method {method_name} is not a recognized lambda function"
                                ),
                            })
                        }
                    } else {
                        Err(EvaluationError::InvalidOperation {
                            message: format!("Unknown lambda method: {method_name}"),
                        })
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
                    if let Some(operation) = self.registry.get_operation(method_name).await {
                        // Create registry context with the object as input (context) for the method
                        let registry_context =
                            octofhir_fhirpath_registry::operations::EvaluationContext::new(
                                object,
                                self.registry.clone(),
                                self.model_provider.clone(),
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

    /// Check if a collection size is within limits
    /// This is used to prevent memory exhaustion from extremely large collections
    pub fn validate_collection_size(&self, size: usize) -> EvaluationResult<()> {
        if size > self.config.max_collection_size {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "Collection size {} exceeds maximum allowed size of {}",
                    size, self.config.max_collection_size
                ),
            });
        }
        Ok(())
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
            self.registry.clone(),
            self.model_provider.clone(),
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
                    octofhir_fhirpath_core::FhirPathError::EvaluationError { message }
                }
                EvaluationError::TypeError { expected, actual } => {
                    octofhir_fhirpath_core::FhirPathError::TypeError {
                        message: format!("Type mismatch: expected {expected}, got {actual}"),
                    }
                }
                EvaluationError::RuntimeError { message } => {
                    octofhir_fhirpath_core::FhirPathError::EvaluationError { message }
                }
                EvaluationError::Function(message) => {
                    octofhir_fhirpath_core::FhirPathError::FunctionError {
                        function_name: "unknown".to_string(),
                        message,
                    }
                }
                EvaluationError::Operator(message) => {
                    octofhir_fhirpath_core::FhirPathError::EvaluationError { message }
                }
                _ => octofhir_fhirpath_core::FhirPathError::EvaluationError {
                    message: e.to_string(),
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
