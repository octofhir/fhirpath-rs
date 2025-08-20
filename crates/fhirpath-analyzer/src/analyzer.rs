//! Main analyzer implementation

use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_model::provider::ModelProvider;
use octofhir_fhirpath_parser::parse;
use octofhir_fhirpath_registry::FhirPathRegistry;
use std::sync::Arc;

use crate::{
    cache::{AnalysisCache, ExpressionAnalysisMap},
    children_analyzer::ChildrenFunctionAnalyzer,
    config::AnalyzerConfig,
    error::AnalysisError,
    function_analyzer::FunctionAnalyzer,
    types::{AnalysisContext, AnalysisResult, Cardinality, ConfidenceLevel, SemanticInfo},
};

/// Main analyzer for FHIRPath expressions
pub struct FhirPathAnalyzer {
    model_provider: Arc<dyn ModelProvider>,
    cache: Arc<AnalysisCache>,
    config: AnalyzerConfig,
    function_analyzer: Option<FunctionAnalyzer>,
}

impl FhirPathAnalyzer {
    /// Create new analyzer with ModelProvider
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            model_provider,
            cache: Arc::new(AnalysisCache::new()),
            config: AnalyzerConfig::default(),
            function_analyzer: None,
        }
    }

    /// Create analyzer with custom configuration
    pub fn with_config(model_provider: Arc<dyn ModelProvider>, config: AnalyzerConfig) -> Self {
        Self {
            model_provider,
            cache: Arc::new(AnalysisCache::with_capacity(config.cache_size)),
            config,
            function_analyzer: None,
        }
    }

    /// Create analyzer with function registry
    pub fn with_function_registry(
        model_provider: Arc<dyn ModelProvider>,
        function_registry: Arc<FhirPathRegistry>,
    ) -> Self {
        let function_analyzer = Some(FunctionAnalyzer::new(function_registry));

        Self {
            model_provider,
            cache: Arc::new(AnalysisCache::new()),
            config: AnalyzerConfig::default(),
            function_analyzer,
        }
    }

    /// Analyze expression and enrich with semantic type information
    pub async fn analyze(&self, expression: &str) -> Result<AnalysisResult, AnalysisError> {
        // Check cache first
        if let Some(cached) = self.cache.get_analysis(expression) {
            return Ok(cached);
        }

        // Parse expression to AST
        let ast = parse(expression).map_err(|e| AnalysisError::InvalidExpression {
            message: format!("Parse error: {e}"),
        })?;

        // Create analysis context
        let context = AnalysisContext {
            root_type: None,
            variables: std::collections::HashMap::new(),
            environment: std::collections::HashMap::new(),
            settings: self.config.settings.clone(),
        };

        // Perform analysis
        let result = self.analyze_ast(&ast, &context).await?;

        // Cache result
        self.cache
            .cache_analysis(expression.to_string(), result.clone());

        Ok(result)
    }

    /// Analyze expression with specific context
    pub async fn analyze_with_context(
        &self,
        expression: &str,
        context: &AnalysisContext,
    ) -> Result<AnalysisResult, AnalysisError> {
        let ast = parse(expression).map_err(|e| AnalysisError::InvalidExpression {
            message: format!("Parse error: {e}"),
        })?;

        self.analyze_ast(&ast, context).await
    }

    /// Internal AST analysis
    async fn analyze_ast(
        &self,
        ast: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<AnalysisResult, AnalysisError> {
        let mut analysis_map = ExpressionAnalysisMap::new();
        let mut validation_errors = Vec::new();

        // Analyze the AST tree
        self.analyze_node_recursive(ast, context, &mut analysis_map, &mut validation_errors)
            .await?;

        // Convert analysis map to result
        let type_annotations = analysis_map
            .get_all_analyses()
            .iter()
            .enumerate()
            .map(|(i, (_, info))| (i as u64, info.clone()))
            .collect();

        let function_calls = analysis_map
            .get_all_function_analyses()
            .values()
            .cloned()
            .collect();

        let union_types = analysis_map
            .get_all_union_analyses()
            .iter()
            .enumerate()
            .map(|(i, (_, union))| (i as u64, union.clone()))
            .collect();

        Ok(AnalysisResult {
            validation_errors,
            type_annotations,
            function_calls,
            union_types,
        })
    }

    /// Recursive node analysis
    fn analyze_node_recursive<'a>(
        &'a self,
        node: &'a ExpressionNode,
        context: &'a AnalysisContext,
        analysis_map: &'a mut ExpressionAnalysisMap,
        validation_errors: &'a mut Vec<crate::error::ValidationError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AnalysisError>> + Send + 'a>>
    {
        Box::pin(async move {
            // Basic type inference for current node
            if let Some(semantic_info) = self.infer_basic_type(node, context).await? {
                analysis_map.attach_analysis(node, semantic_info);
            }

            // Recursively analyze child nodes
            match node {
                ExpressionNode::Path { base, .. } => {
                    self.analyze_node_recursive(base, context, analysis_map, validation_errors)
                        .await?;
                }
                ExpressionNode::BinaryOp(data) => {
                    self.analyze_node_recursive(
                        &data.left,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                    self.analyze_node_recursive(
                        &data.right,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                }
                ExpressionNode::UnaryOp { operand, .. } => {
                    self.analyze_node_recursive(operand, context, analysis_map, validation_errors)
                        .await?;
                }
                ExpressionNode::FunctionCall(data) => {
                    // Analyze function call if we have function analyzer
                    if let Some(func_analyzer) = &self.function_analyzer {
                        // Create a FunctionCall node for analysis
                        let func_node = ExpressionNode::FunctionCall(data.clone());
                        match self
                            .analyze_function_call_with_node(
                                data,
                                &func_node,
                                analysis_map,
                                func_analyzer,
                            )
                            .await
                        {
                            Ok(func_validation_errors) => {
                                // Add function validation errors to main validation errors
                                validation_errors.extend(func_validation_errors);
                            }
                            Err(e) => {
                                // Add as validation error if function analysis fails
                                validation_errors.push(crate::error::ValidationError {
                                    message: format!("Function analysis failed: {e}"),
                                    error_type: crate::error::ValidationErrorType::InvalidFunction,
                                    location: None,
                                    suggestions: vec![],
                                });
                            }
                        }
                    }

                    // Analyze arguments recursively
                    for arg in &data.args {
                        self.analyze_node_recursive(arg, context, analysis_map, validation_errors)
                            .await?;
                    }
                }
                ExpressionNode::MethodCall(data) => {
                    // Analyze method call as a function if we have function analyzer
                    if let Some(func_analyzer) = &self.function_analyzer {
                        // Convert MethodCall to FunctionCallData for analysis
                        let function_data = octofhir_fhirpath_ast::FunctionCallData {
                            name: data.method.clone(),
                            args: data.args.clone(),
                        };

                        match self
                            .analyze_function_call_with_node(
                                &function_data,
                                node,
                                analysis_map,
                                func_analyzer,
                            )
                            .await
                        {
                            Ok(func_validation_errors) => {
                                // Add function validation errors to main validation errors
                                validation_errors.extend(func_validation_errors);
                            }
                            Err(e) => {
                                // Add as validation error if function analysis fails
                                validation_errors.push(crate::error::ValidationError {
                                    message: format!("Method analysis failed: {e}"),
                                    error_type: crate::error::ValidationErrorType::InvalidFunction,
                                    location: None,
                                    suggestions: vec![],
                                });
                            }
                        }
                    }

                    self.analyze_node_recursive(
                        &data.base,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                    for arg in &data.args {
                        self.analyze_node_recursive(arg, context, analysis_map, validation_errors)
                            .await?;
                    }
                }
                ExpressionNode::Index { base, index } => {
                    self.analyze_node_recursive(base, context, analysis_map, validation_errors)
                        .await?;
                    self.analyze_node_recursive(index, context, analysis_map, validation_errors)
                        .await?;
                }
                ExpressionNode::Filter { base, condition } => {
                    self.analyze_node_recursive(base, context, analysis_map, validation_errors)
                        .await?;
                    self.analyze_node_recursive(
                        condition,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                }
                ExpressionNode::Union { left, right } => {
                    self.analyze_node_recursive(left, context, analysis_map, validation_errors)
                        .await?;
                    self.analyze_node_recursive(right, context, analysis_map, validation_errors)
                        .await?;
                }
                ExpressionNode::TypeCheck { expression, .. } => {
                    self.analyze_node_recursive(
                        expression,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                }
                ExpressionNode::TypeCast { expression, .. } => {
                    self.analyze_node_recursive(
                        expression,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                }
                ExpressionNode::Lambda(data) => {
                    self.analyze_node_recursive(
                        &data.body,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                }
                ExpressionNode::Conditional(data) => {
                    self.analyze_node_recursive(
                        &data.condition,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                    self.analyze_node_recursive(
                        &data.then_expr,
                        context,
                        analysis_map,
                        validation_errors,
                    )
                    .await?;
                    if let Some(else_expr) = &data.else_expr {
                        self.analyze_node_recursive(
                            else_expr,
                            context,
                            analysis_map,
                            validation_errors,
                        )
                        .await?;
                    }
                }
                // Leaf nodes are handled in infer_basic_type
                _ => {}
            }

            Ok(())
        })
    }

    /// Basic type inference for literals and identifiers
    async fn infer_basic_type(
        &self,
        node: &ExpressionNode,
        _context: &AnalysisContext,
    ) -> Result<Option<SemanticInfo>, AnalysisError> {
        let semantic_info = match node {
            ExpressionNode::Literal(literal) => {
                use octofhir_fhirpath_ast::LiteralValue;

                let (fhir_path_type, model_type) = match literal {
                    LiteralValue::String(_) => ("String".to_string(), None),
                    LiteralValue::Integer(_) => ("Integer".to_string(), None),
                    LiteralValue::Decimal(_) => ("Decimal".to_string(), None),
                    LiteralValue::Boolean(_) => ("Boolean".to_string(), None),
                    LiteralValue::Date(_) => ("Date".to_string(), None),
                    LiteralValue::DateTime(_) => ("DateTime".to_string(), None),
                    LiteralValue::Time(_) => ("Time".to_string(), None),
                    LiteralValue::Quantity { .. } => ("Quantity".to_string(), None),
                    LiteralValue::Null => ("Null".to_string(), None),
                };

                Some(SemanticInfo {
                    fhir_path_type: Some(fhir_path_type),
                    model_type,
                    cardinality: Cardinality::OneToOne,
                    confidence: ConfidenceLevel::High,
                    scope_info: None,
                    function_info: None,
                })
            }
            ExpressionNode::Identifier(name) => {
                // Try to resolve identifier type through ModelProvider
                if let Some(_type_info) = self.model_provider.get_type_reflection(name).await {
                    Some(SemanticInfo {
                        fhir_path_type: Some("Resource".to_string()), // Generic for now
                        model_type: Some(name.clone()),
                        cardinality: Cardinality::ZeroToOne, // Default assumption
                        confidence: ConfidenceLevel::Medium,
                        scope_info: None,
                        function_info: None,
                    })
                } else {
                    // Unknown identifier
                    Some(SemanticInfo {
                        fhir_path_type: None,
                        model_type: None,
                        cardinality: Cardinality::ZeroToMany, // Most permissive
                        confidence: ConfidenceLevel::Low,
                        scope_info: None,
                        function_info: None,
                    })
                }
            }
            ExpressionNode::FunctionCall(_) | ExpressionNode::MethodCall(_) => {
                // Extract function name from either FunctionCall or MethodCall
                let function_name = match node {
                    ExpressionNode::FunctionCall(data) => &data.name,
                    ExpressionNode::MethodCall(data) => &data.method,
                    _ => unreachable!(),
                };

                // Provide basic type inference for common functions even without function analyzer
                let basic_type = match function_name.as_str() {
                    "count" => Some(("Integer", "Integer")),
                    "empty" | "exists" => Some(("Boolean", "Boolean")),
                    "first" | "last" | "single" => Some(("Any", "Any")), // Return type depends on input
                    "children" => Some(("Collection", "Any")), // Collection of child elements
                    "substring" | "upper" | "lower" => Some(("String", "String")),
                    "length" => Some(("Integer", "Integer")),
                    _ => None,
                };

                if let Some((fhir_path_type, model_type)) = basic_type {
                    Some(SemanticInfo {
                        fhir_path_type: Some(fhir_path_type.to_string()),
                        model_type: Some(model_type.to_string()),
                        cardinality: match function_name.as_str() {
                            "count" | "length" => Cardinality::OneToOne,
                            "empty" | "exists" => Cardinality::OneToOne,
                            "children" => Cardinality::ZeroToMany,
                            _ => Cardinality::ZeroToOne,
                        },
                        confidence: ConfidenceLevel::Medium,
                        scope_info: None,
                        function_info: None,
                    })
                } else {
                    // Unknown function
                    Some(SemanticInfo {
                        fhir_path_type: Some("Any".to_string()),
                        model_type: None,
                        cardinality: Cardinality::ZeroToMany, // Most permissive
                        confidence: ConfidenceLevel::Low,
                        scope_info: None,
                        function_info: None,
                    })
                }
            }
            _ => None, // Advanced types handled in later tasks
        };

        Ok(semantic_info)
    }

    /// Get detailed type information for a specific node
    pub async fn get_type_info(
        &self,
        node: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Option<SemanticInfo> {
        // Try cache first
        let node_hash = ExpressionAnalysisMap::hash_node(node);
        if let Some(cached) = self.cache.get_semantic_info(node_hash) {
            return Some(cached);
        }

        // Perform fresh analysis
        if let Ok(Some(info)) = self.infer_basic_type(node, context).await {
            self.cache.cache_semantic_info(node_hash, info.clone());
            Some(info)
        } else {
            None
        }
    }

    /// Basic validation (comprehensive validation in later tasks)
    pub async fn validate(
        &self,
        expression: &str,
    ) -> Result<Vec<crate::error::ValidationError>, AnalysisError> {
        let analysis = self.analyze(expression).await?;
        Ok(analysis.validation_errors)
    }

    /// Analyze function call during AST traversal
    async fn analyze_function_call_with_node(
        &self,
        function_data: &octofhir_fhirpath_ast::FunctionCallData,
        original_node: &ExpressionNode,
        analysis_map: &mut ExpressionAnalysisMap,
        func_analyzer: &FunctionAnalyzer,
    ) -> Result<Vec<crate::error::ValidationError>, AnalysisError> {
        // Special handling for children() function
        if function_data.name == "children" {
            return self
                .analyze_children_function_call_with_errors(
                    function_data,
                    original_node,
                    analysis_map,
                )
                .await;
        }

        // Infer actual argument types
        let arg_types: Vec<octofhir_fhirpath_model::types::TypeInfo> = function_data
            .args
            .iter()
            .map(|arg| self.infer_argument_type(arg))
            .collect();

        // Analyze function call
        let analysis = func_analyzer
            .analyze_function(&function_data.name, &function_data.args, &arg_types)
            .await?;

        // Extract validation errors from function analysis
        let validation_errors = analysis.validation_errors.clone();

        // Store analysis with original node reference
        analysis_map.attach_function_analysis(original_node, analysis);

        Ok(validation_errors)
    }

    /// Analyze function call during AST traversal (backwards compatibility)
    async fn analyze_function_call(
        &self,
        function_data: &octofhir_fhirpath_ast::FunctionCallData,
        analysis_map: &mut ExpressionAnalysisMap,
        func_analyzer: &FunctionAnalyzer,
    ) -> Result<(), AnalysisError> {
        // Special handling for children() function
        if function_data.name == "children" {
            // Create a function call node for backwards compatibility
            let node = ExpressionNode::FunctionCall(Box::new(function_data.clone()));
            return self
                .analyze_children_function_call(function_data, &node, analysis_map)
                .await;
        }

        // Infer actual argument types
        let arg_types: Vec<octofhir_fhirpath_model::types::TypeInfo> = function_data
            .args
            .iter()
            .map(|arg| self.infer_argument_type(arg))
            .collect();

        // Analyze function call
        let analysis = func_analyzer
            .analyze_function(&function_data.name, &function_data.args, &arg_types)
            .await?;

        // Store analysis in external mapping
        let node = ExpressionNode::FunctionCall(Box::new(function_data.clone()));
        analysis_map.attach_function_analysis(&node, analysis);

        Ok(())
    }

    /// Analyze children() function call with union type support (with error propagation)
    async fn analyze_children_function_call_with_errors(
        &self,
        function_data: &octofhir_fhirpath_ast::FunctionCallData,
        original_node: &ExpressionNode,
        analysis_map: &mut ExpressionAnalysisMap,
    ) -> Result<Vec<crate::error::ValidationError>, AnalysisError> {
        // For children() function, we need to determine the base type
        // This is a simplified implementation - in practice we'd need to track the evaluation context
        let base_type = "Patient"; // Default for now - would need proper context tracking

        // Create a children analyzer with the model provider
        // Note: We need to cast our model provider to support children extension
        let children_analyzer = ChildrenFunctionAnalyzer::new(self.model_provider.clone());

        // Generate node ID for analysis mapping
        let node_id = analysis_map.get_next_node_id();

        // Create children analysis
        let analysis = children_analyzer
            .create_children_analysis(function_data, base_type, node_id)
            .await?;

        // Extract validation errors from children analysis
        let validation_errors = analysis.validation_errors.clone();

        // Try to create union type
        let dummy_base = ExpressionNode::Identifier(base_type.to_string());
        match children_analyzer
            .analyze_children_call(&dummy_base, base_type)
            .await
        {
            Ok(union_type) => {
                // Store both function analysis and union type using original node
                analysis_map.attach_function_analysis(original_node, analysis);
                analysis_map.attach_union_analysis(original_node, union_type);
            }
            Err(_) => {
                // Just store function analysis if union type creation fails using original node
                analysis_map.attach_function_analysis(original_node, analysis);
            }
        }

        Ok(validation_errors)
    }

    /// Analyze children() function call with union type support (backwards compatibility)
    async fn analyze_children_function_call(
        &self,
        function_data: &octofhir_fhirpath_ast::FunctionCallData,
        original_node: &ExpressionNode,
        analysis_map: &mut ExpressionAnalysisMap,
    ) -> Result<(), AnalysisError> {
        // For children() function, we need to determine the base type
        // This is a simplified implementation - in practice we'd need to track the evaluation context
        let base_type = "Patient"; // Default for now - would need proper context tracking

        // Create a children analyzer with the model provider
        // Note: We need to cast our model provider to support children extension
        let children_analyzer = ChildrenFunctionAnalyzer::new(self.model_provider.clone());

        // Generate node ID for analysis mapping
        let node_id = analysis_map.get_next_node_id();

        // Create children analysis
        let analysis = children_analyzer
            .create_children_analysis(function_data, base_type, node_id)
            .await?;

        // Try to create union type
        let dummy_base = ExpressionNode::Identifier(base_type.to_string());
        match children_analyzer
            .analyze_children_call(&dummy_base, base_type)
            .await
        {
            Ok(union_type) => {
                // Store both function analysis and union type using original node
                analysis_map.attach_function_analysis(original_node, analysis);
                analysis_map.attach_union_analysis(original_node, union_type);
            }
            Err(_) => {
                // Just store function analysis if union type creation fails using original node
                analysis_map.attach_function_analysis(original_node, analysis);
            }
        }

        Ok(())
    }

    /// Infer type for function argument
    fn infer_argument_type(
        &self,
        arg: &ExpressionNode,
    ) -> octofhir_fhirpath_model::types::TypeInfo {
        match arg {
            ExpressionNode::Literal(literal) => {
                use octofhir_fhirpath_ast::LiteralValue;
                match literal {
                    LiteralValue::String(_) => octofhir_fhirpath_model::types::TypeInfo::String,
                    LiteralValue::Integer(_) => octofhir_fhirpath_model::types::TypeInfo::Integer,
                    LiteralValue::Decimal(_) => octofhir_fhirpath_model::types::TypeInfo::Decimal,
                    LiteralValue::Boolean(_) => octofhir_fhirpath_model::types::TypeInfo::Boolean,
                    LiteralValue::Date(_) => octofhir_fhirpath_model::types::TypeInfo::Date,
                    LiteralValue::DateTime(_) => octofhir_fhirpath_model::types::TypeInfo::DateTime,
                    LiteralValue::Time(_) => octofhir_fhirpath_model::types::TypeInfo::Time,
                    LiteralValue::Quantity { .. } => {
                        octofhir_fhirpath_model::types::TypeInfo::Quantity
                    }
                    LiteralValue::Null => octofhir_fhirpath_model::types::TypeInfo::Any,
                }
            }
            // For more complex expressions, we'd need full type inference
            // For now, default to Any for non-literals
            _ => octofhir_fhirpath_model::types::TypeInfo::Any,
        }
    }
}
