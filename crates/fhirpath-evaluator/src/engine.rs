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
use crate::evaluators::polymorphic::PolymorphicNavigationEngine;

// Import the new modular components
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{FhirPathValue, ModelProvider};
use octofhir_fhirpath_registry::traits::EvaluationContext as RegistryEvaluationContext;
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
    registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,
    /// Model provider (Send + Sync)
    model_provider: Arc<dyn ModelProvider>,
    /// Evaluation configuration
    config: EvaluationConfig,
    /// Optional polymorphic navigation engine for FHIR choice types
    polymorphic_engine: Option<Arc<PolymorphicNavigationEngine>>,
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
    /// use octofhir_fhirpath_registry::FunctionRegistry;
    /// use octofhir_fhirpath_model::MockModelProvider;
    /// use std::sync::Arc;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = Arc::new(FunctionRegistry::new());
    /// let model_provider = Arc::new(MockModelProvider::new());
    ///
    /// let engine = FhirPathEngine::new(registry, model_provider);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            registry,
            model_provider,
            config: EvaluationConfig::default(),
            polymorphic_engine: None,
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
    pub fn registry(&self) -> &Arc<octofhir_fhirpath_registry::FunctionRegistry> {
        &self.registry
    }

    /// Get the model provider reference  
    pub fn model_provider(&self) -> &Arc<dyn ModelProvider> {
        &self.model_provider
    }

    /// Get the polymorphic navigation engine reference (if enabled)
    pub fn polymorphic_engine(&self) -> Option<&Arc<PolymorphicNavigationEngine>> {
        self.polymorphic_engine.as_ref()
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

    /// Enables polymorphic navigation for FHIR choice types.
    ///
    /// This method creates and enables a polymorphic navigation engine that provides
    /// enhanced support for FHIR choice types (value[x] patterns), allowing expressions
    /// like `Observation.value.unit` to correctly resolve to `Observation.valueQuantity.unit`
    /// based on the actual data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use sonic_rs::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?
    ///     .with_polymorphic_navigation();
    ///
    /// let observation = json!({
    ///     "resourceType": "Observation",
    ///     "valueQuantity": {
    ///         "value": 185,
    ///         "unit": "lbs"
    ///     }
    /// });
    ///
    /// // Now this works correctly due to polymorphic navigation
    /// let result = engine.evaluate("Observation.value.unit", observation).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_polymorphic_navigation(mut self) -> Self {
        use crate::evaluators::polymorphic::PolymorphicNavigationFactory;

        let polymorphic_engine =
            PolymorphicNavigationFactory::create_r4_navigation_engine(self.model_provider.clone());
        self.polymorphic_engine = Some(Arc::new(polymorphic_engine));
        self
    }

    /// Enables polymorphic navigation with a custom navigation engine.
    ///
    /// This method allows you to provide a custom polymorphic navigation engine
    /// for specialized use cases or custom choice type mappings.
    ///
    /// # Arguments
    ///
    /// * `engine` - Custom polymorphic navigation engine
    ///
    /// # Examples
    ///
    /// ```rust
    /// use octofhir_fhirpath_evaluator::FhirPathEngine;
    /// use octofhir_fhirpath_evaluator::evaluators::polymorphic::PolymorphicNavigationFactory;
    /// use std::sync::Arc;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let engine = FhirPathEngine::with_mock_provider().await?;
    /// let custom_nav_engine = PolymorphicNavigationFactory::create_r4_navigation_engine(
    ///     engine.model_provider().clone()
    /// );
    ///
    /// let enhanced_engine = engine.with_custom_polymorphic_navigation(Arc::new(custom_nav_engine));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_custom_polymorphic_navigation(
        mut self,
        engine: Arc<PolymorphicNavigationEngine>,
    ) -> Self {
        self.polymorphic_engine = Some(engine);
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

        let registry = octofhir_fhirpath_registry::create_standard_registry().await;

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
        let registry = octofhir_fhirpath_registry::create_standard_registry().await;
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

        Ok(FhirPathValue::Boolean(matches_type))
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

        // Create evaluation context for the operation
        let eval_context = RegistryEvaluationContext {
            input: expr_result,
            root: context.root.clone(),
            variables: context.variable_scope.variables.as_ref().clone(),
            model_provider: self.model_provider().clone(),
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

        self.registry
            .evaluate("as", &args, &eval_context)
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
    pub fn is_system_variable(name: &str) -> bool {
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
                        return Ok(FhirPathValue::Boolean(false));
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

    /// Extract defineVariable calls from an expression tree and return accumulated context
    fn extract_define_variable_context<'a>(
        &'a self,
        expr: &'a ExpressionNode,
        input: FhirPathValue,
        context: &'a LocalEvaluationContext,
        depth: usize,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = EvaluationResult<(LocalEvaluationContext, FhirPathValue)>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            match expr {
                ExpressionNode::FunctionCall(func_data) if func_data.name == "defineVariable" => {
                    // This is a defineVariable call - extract the variable definition
                    if func_data.args.is_empty() {
                        return Err(EvaluationError::InvalidOperation {
                            message: "defineVariable() requires at least 1 argument".to_string(),
                        });
                    }

                    // Get variable name
                    let name_value = self
                        .evaluate_node_async(&func_data.args[0], input.clone(), context, depth + 1)
                        .await?;

                    let var_name = match name_value {
                        FhirPathValue::String(name) => name.to_string(),
                        _ => {
                            return Err(EvaluationError::InvalidOperation {
                                message: "defineVariable() first argument must be a string"
                                    .to_string(),
                            });
                        }
                    };

                    // Check for system variable protection
                    if Self::is_system_variable(&var_name) {
                        return Err(EvaluationError::InvalidOperation {
                            message: format!("Cannot override system variable '{var_name}'"),
                        });
                    }

                    // Check for redefinition
                    if context.variable_scope.get_variable(&var_name).is_some() {
                        return Err(EvaluationError::InvalidOperation {
                            message: format!(
                                "Variable '{var_name}' is already defined in current scope"
                            ),
                        });
                    }

                    // Get variable value
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

                    // Create new context with the variable
                    let mut new_context = context.clone();
                    new_context.variable_scope.set_variable(var_name, var_value);

                    Ok((new_context, input))
                }
                ExpressionNode::MethodCall(method_data) => {
                    // Recursively check the base for defineVariable calls
                    let (base_context, base_result) = self
                        .extract_define_variable_context(
                            &method_data.base,
                            input,
                            context,
                            depth + 1,
                        )
                        .await?;

                    // Check if this method call is also a defineVariable
                    if method_data.method == "defineVariable" {
                        // This is a chained defineVariable call
                        if method_data.args.is_empty() {
                            return Err(EvaluationError::InvalidOperation {
                                message: "defineVariable() requires at least 1 argument"
                                    .to_string(),
                            });
                        }

                        // Get variable name for the chained call
                        let name_value = self
                            .evaluate_node_async(
                                &method_data.args[0],
                                base_result.clone(),
                                &base_context,
                                depth + 1,
                            )
                            .await?;

                        let var_name = match name_value {
                            FhirPathValue::String(name) => name.to_string(),
                            _ => {
                                return Err(EvaluationError::InvalidOperation {
                                    message: "defineVariable() first argument must be a string"
                                        .to_string(),
                                });
                            }
                        };

                        // Check for system variable protection
                        if Self::is_system_variable(&var_name) {
                            return Err(EvaluationError::InvalidOperation {
                                message: format!("Cannot override system variable '{var_name}'"),
                            });
                        }

                        // Check for redefinition in the accumulated context
                        if base_context
                            .variable_scope
                            .get_variable(&var_name)
                            .is_some()
                        {
                            return Err(EvaluationError::InvalidOperation {
                                message: format!(
                                    "Variable '{var_name}' is already defined in current scope"
                                ),
                            });
                        }

                        // Get variable value for the chained call
                        let var_value = if method_data.args.len() == 2 {
                            self.evaluate_node_async(
                                &method_data.args[1],
                                base_result.clone(),
                                &base_context,
                                depth + 1,
                            )
                            .await?
                        } else {
                            base_result.clone()
                        };

                        // Create new context with the additional variable
                        let mut new_context = base_context;
                        new_context.variable_scope.set_variable(var_name, var_value);

                        Ok((new_context, base_result))
                    } else {
                        // Not a defineVariable call, return the base context
                        Ok((base_context, base_result))
                    }
                }
                _ => {
                    // Not a defineVariable-related expression
                    Ok((context.clone(), input))
                }
            }
        })
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
                // First evaluate the defineVariable function - this will validate and return input unchanged
                let object = self
                    .evaluate_define_variable_function(func_data, input.clone(), context, depth + 1)
                    .await?;

                // Extract the variable name and value for context propagation
                if !func_data.args.is_empty() {
                    let name_value = self
                        .evaluate_node_async(&func_data.args[0], input.clone(), context, depth + 1)
                        .await?;

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
                    } else {
                        return Err(EvaluationError::InvalidOperation {
                            message: "defineVariable() first argument must be a string".to_string(),
                        });
                    }
                }
            }
        } else if let ExpressionNode::MethodCall(base_method) = &method_data.base {
            // Handle chained defineVariable calls like defineVariable().defineVariable()
            if base_method.method == "defineVariable" {
                // Extract all defineVariable calls from the chain
                let (accumulated_context, base_result) = self
                    .extract_define_variable_context(
                        &method_data.base,
                        input.clone(),
                        context,
                        depth + 1,
                    )
                    .await?;

                // Continue with the accumulated context
                return self
                    .evaluate_method_call_with_object(
                        &method_data.method,
                        &method_data.args,
                        base_result,
                        &accumulated_context,
                        depth,
                        input,
                    )
                    .await;
            }
        }

        // First evaluate the base expression normally
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
            "empty" => Ok(FhirPathValue::Boolean(self.is_empty(&object))),
            "exists" => {
                // If exists has arguments, use lambda version for conditional evaluation
                if !args.is_empty() {
                    let func_data = octofhir_fhirpath_ast::FunctionCallData {
                        name: method_name.to_string(),
                        args: args.iter().cloned().collect(),
                    };
                    self.evaluate_exists_lambda(&func_data, object, context, depth)
                        .await
                } else {
                    // Handle Empty propagation correctly for exists()
                    match &object {
                        FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                        _ => Ok(FhirPathValue::Boolean(!self.is_empty(&object))),
                    }
                }
            }
            "count" => Ok(FhirPathValue::Integer(self.count(&object))),
            "toString" => Ok(FhirPathValue::String(self.to_string_value(&object))),

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
                        "iif" => {
                            self.evaluate_iif_function(&func_data, object, context, depth)
                                .await
                        }
                        _ => {
                            // Fallback to registry for other lambda functions not yet moved to engine
                            if self.registry().has_function(method_name).await {
                                // Create registry context with the object as input for the lambda and variables from engine context
                                let all_variables = context.variable_scope.collect_all_variables();
                                let registry_context =
                                    octofhir_fhirpath_registry::traits::EvaluationContext {
                                        input: object,
                                        root: context.root.clone(),
                                        variables: all_variables,
                                        model_provider: self.model_provider().clone(),
                                    };

                                // Use generic lambda function evaluation for remaining functions
                                self.registry()
                                    .evaluate(method_name, &[], &registry_context)
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
                        // Special handling for type identifiers in type checking methods
                        if matches!(method_name, "is" | "as" | "ofType")
                            && Self::is_type_identifier_expression(arg_expr)
                        {
                            // Convert type identifier to TypeInfoObject
                            let type_arg = match arg_expr {
                                ExpressionNode::Identifier(type_name) => {
                                    if self.is_type_identifier(type_name) {
                                        // Create a TypeInfoObject for known type identifiers
                                        let (namespace, name) = if type_name.contains('.') {
                                            let parts: Vec<&str> = type_name.split('.').collect();
                                            (parts[0], parts[1])
                                        } else {
                                            // Handle common FHIRPath types
                                            match type_name.to_lowercase().as_str() {
                                                "boolean" | "integer" | "decimal" | "string"
                                                | "date" | "datetime" | "time" | "quantity"
                                                | "collection" => ("System", type_name.as_str()),
                                                "code" | "uri" | "url" | "canonical" | "oid"
                                                | "uuid" | "id" | "markdown" | "base64binary"
                                                | "instant" | "positiveint" | "unsignedint"
                                                | "xhtml" => ("FHIR", type_name.as_str()),
                                                _ => ("System", type_name.as_str()),
                                            }
                                        };
                                        FhirPathValue::TypeInfoObject {
                                            namespace: Arc::from(namespace),
                                            name: Arc::from(name),
                                        }
                                    } else {
                                        // Treat as string literal for backward compatibility
                                        FhirPathValue::String(type_name.clone().into())
                                    }
                                }
                                ExpressionNode::Path { base, path } => {
                                    // Handle qualified type names like FHIR.uuid, System.Boolean
                                    if let ExpressionNode::Identifier(namespace) = base.as_ref() {
                                        if matches!(namespace.as_str(), "FHIR" | "System") {
                                            FhirPathValue::TypeInfoObject {
                                                namespace: Arc::from(namespace.as_str()),
                                                name: Arc::from(path.as_str()),
                                            }
                                        } else {
                                            // Evaluate as normal path expression
                                            self.evaluate_node_async(
                                                arg_expr,
                                                input.clone(),
                                                context,
                                                depth + 1,
                                            )
                                            .await?
                                        }
                                    } else {
                                        // Evaluate as normal path expression
                                        self.evaluate_node_async(
                                            arg_expr,
                                            input.clone(),
                                            context,
                                            depth + 1,
                                        )
                                        .await?
                                    }
                                }
                                _ => {
                                    // For other type expressions, evaluate normally
                                    self.evaluate_node_async(
                                        arg_expr,
                                        input.clone(),
                                        context,
                                        depth + 1,
                                    )
                                    .await?
                                }
                            };
                            evaluated_args.push(type_arg);
                        } else {
                            // Standard argument evaluation
                            let arg_context = object.clone();
                            let arg_value = self
                                .evaluate_node_async(arg_expr, arg_context, context, depth + 1)
                                .await?;
                            evaluated_args.push(arg_value);
                        }
                    }

                    // Get method from registry and evaluate
                    if self.registry().has_function(method_name).await {
                        // Create registry context with the object as input (context) for the method
                        let registry_context =
                            octofhir_fhirpath_registry::traits::EvaluationContext {
                                input: object,
                                root: context.root.clone(),
                                variables: context.variable_scope.collect_all_variables(),
                                model_provider: self.model_provider().clone(),
                            };

                        self.registry()
                            .evaluate(method_name, &evaluated_args, &registry_context)
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
            FhirPathValue::Date(date) => date.to_string().into(),
            FhirPathValue::DateTime(datetime) => datetime.to_string().into(),
            FhirPathValue::Time(time) => time.to_string().into(),
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
