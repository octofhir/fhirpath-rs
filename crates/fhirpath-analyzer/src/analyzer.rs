//! Main analyzer implementation

use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_model::provider::ModelProvider;
use octofhir_fhirpath_parser::parse;
use octofhir_fhirpath_registry::FunctionRegistry;
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
        function_registry: Arc<FunctionRegistry>,
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
                    // Check if it's a lambda function first - these are implemented as plain Rust functions
                    if self.is_lambda_function(&data.method) {
                        // Validate lambda function signature and arguments
                        self.validate_lambda_function_signature(
                            &data.method,
                            &data.args,
                            validation_errors,
                        );

                        // Additional semantic validation for where() clauses
                        if data.method == "where" && data.args.len() == 1 {
                            self.validate_where_clause_arguments(&data.args, validation_errors)
                                .await;
                        }
                    } else if let Some(func_analyzer) = &self.function_analyzer {
                        // Convert MethodCall to FunctionCallData for registry function analysis
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

    /// Check if a function name corresponds to a lambda function (implemented as plain Rust functions)
    fn is_lambda_function(&self, function_name: &str) -> bool {
        matches!(
            function_name,
            "where" | "select" | "sort" | "repeat" | "aggregate" | "all" | "exists" | "iif"
        )
    }

    /// Validate arguments in where() clause for resource type validation
    async fn validate_where_clause_arguments(
        &self,
        args: &[ExpressionNode],
        validation_errors: &mut Vec<crate::error::ValidationError>,
    ) {
        if args.len() != 1 {
            return; // where() should have exactly 1 argument, but that's a separate validation
        }

        // Look for patterns like resourceType='SomeType' in the where condition
        self.validate_resource_type_comparisons(&args[0], validation_errors)
            .await;
    }

    /// Recursively validate resource type comparisons in expressions
    fn validate_resource_type_comparisons<'a>(
        &'a self,
        expr: &'a ExpressionNode,
        validation_errors: &'a mut Vec<crate::error::ValidationError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            match expr {
                ExpressionNode::BinaryOp(data) => {
                    // Check for resourceType = 'SomeString' patterns
                    if let (
                        ExpressionNode::Identifier(field),
                        ExpressionNode::Literal(octofhir_fhirpath_ast::LiteralValue::String(value)),
                    ) = (&data.left, &data.right)
                    {
                        if field == "resourceType"
                            && matches!(data.op, octofhir_fhirpath_ast::BinaryOperator::Equal)
                        {
                            self.validate_resource_type_string(value, validation_errors)
                                .await;
                        }
                    }
                    // Also check the reverse: 'SomeString' = resourceType
                    if let (
                        ExpressionNode::Literal(octofhir_fhirpath_ast::LiteralValue::String(value)),
                        ExpressionNode::Identifier(field),
                    ) = (&data.left, &data.right)
                    {
                        if field == "resourceType"
                            && matches!(data.op, octofhir_fhirpath_ast::BinaryOperator::Equal)
                        {
                            self.validate_resource_type_string(value, validation_errors)
                                .await;
                        }
                    }

                    // Recursively check both sides
                    self.validate_resource_type_comparisons(&data.left, validation_errors)
                        .await;
                    self.validate_resource_type_comparisons(&data.right, validation_errors)
                        .await;
                }
                // Handle other expression types that might contain nested comparisons
                ExpressionNode::UnaryOp { operand, .. } => {
                    self.validate_resource_type_comparisons(operand, validation_errors)
                        .await;
                }
                ExpressionNode::FunctionCall(data) => {
                    for arg in &data.args {
                        self.validate_resource_type_comparisons(arg, validation_errors)
                            .await;
                    }
                }
                ExpressionNode::MethodCall(data) => {
                    self.validate_resource_type_comparisons(&data.base, validation_errors)
                        .await;
                    for arg in &data.args {
                        self.validate_resource_type_comparisons(arg, validation_errors)
                            .await;
                    }
                }
                _ => {} // Other node types don't need resource type validation
            }
        })
    }

    /// Validate that a resource type string is a valid FHIR resource type
    async fn validate_resource_type_string(
        &self,
        resource_type: &str,
        validation_errors: &mut Vec<crate::error::ValidationError>,
    ) {
        // Check if the resource type exists in the model provider
        if let None = self.model_provider.get_type_reflection(resource_type).await {
            // Generate suggestions for similar resource types
            let suggestions = self.get_resource_type_suggestions(resource_type).await;

            validation_errors.push(crate::error::ValidationError {
                message: format!("Unknown FHIR resource type: '{resource_type}'"),
                error_type: crate::error::ValidationErrorType::InvalidResourceType,
                location: None,
                suggestions,
            });
        }
    }

    /// Get suggestions for similar resource types
    async fn get_resource_type_suggestions(&self, unknown_type: &str) -> Vec<String> {
        // For now, provide common FHIR resource types as suggestions
        // In a full implementation, we'd query the model provider for all resource types
        let common_resources = vec![
            "Patient",
            "Observation",
            "Medication",
            "MedicationRequest",
            "Practitioner",
            "Organization",
            "Encounter",
            "Procedure",
            "DiagnosticReport",
            "Condition",
            "Bundle",
            "AllergyIntolerance",
            "Immunization",
            "Specimen",
            "Location",
        ];

        // Simple similarity matching
        common_resources
            .into_iter()
            .filter(|resource| {
                // Check for similarity: starts with same letter, contains substring, or edit distance
                let resource_lower = resource.to_lowercase();
                let unknown_lower = unknown_type.to_lowercase();

                resource_lower.starts_with(&unknown_lower[..1.min(unknown_lower.len())])
                    || resource_lower.contains(&unknown_lower)
                    || unknown_lower.contains(&resource_lower[..3.min(resource_lower.len())])
            })
            .take(3)
            .map(|s| s.to_string())
            .collect()
    }

    /// Validate lambda function signature and parameter count
    fn validate_lambda_function_signature(
        &self,
        function_name: &str,
        args: &[ExpressionNode],
        validation_errors: &mut Vec<crate::error::ValidationError>,
    ) {
        let (min_args, max_args, description) = match function_name {
            "where" => (
                1,
                1,
                "where(condition: expression) - filters collection based on condition",
            ),
            "select" => (
                1,
                1,
                "select(projection: expression) - transforms each item in collection",
            ),
            "sort" => (
                0,
                usize::MAX,
                "sort() or sort(expression1, expression2, ...) - sorts collection",
            ),
            "repeat" => (
                1,
                1,
                "repeat(expression) - repeatedly applies expression until empty result",
            ),
            "aggregate" => (
                1,
                2,
                "aggregate(iterator: expression) or aggregate(iterator: expression, init: expression) - accumulates values",
            ),
            "all" => (
                1,
                1,
                "all(condition: expression) - returns true if all items match condition",
            ),
            "exists" => (
                0,
                1,
                "exists() or exists(condition: expression) - checks if any items exist or match condition",
            ),
            "iif" => (
                2,
                3,
                "iif(condition: expression, then: expression) or iif(condition: expression, then: expression, else: expression) - conditional expression",
            ),
            _ => return, // Not a recognized lambda function
        };

        let arg_count = args.len();

        if arg_count < min_args {
            validation_errors.push(crate::error::ValidationError {
                message: format!(
                    "Function '{}()' requires at least {} argument{}, got {}. Usage: {}",
                    function_name,
                    min_args,
                    if min_args == 1 { "" } else { "s" },
                    arg_count,
                    description
                ),
                error_type: crate::error::ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: vec![format!(
                    "Add {} more argument{}",
                    min_args - arg_count,
                    if min_args - arg_count == 1 { "" } else { "s" }
                )],
            });
        } else if arg_count > max_args && max_args != usize::MAX {
            validation_errors.push(crate::error::ValidationError {
                message: format!(
                    "Function '{}()' accepts at most {} argument{}, got {}. Usage: {}",
                    function_name,
                    max_args,
                    if max_args == 1 { "" } else { "s" },
                    arg_count,
                    description
                ),
                error_type: crate::error::ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: vec![format!(
                    "Remove {} argument{}",
                    arg_count - max_args,
                    if arg_count - max_args == 1 { "" } else { "s" }
                )],
            });
        }

        // Additional parameter type validation for specific functions
        match function_name {
            "iif" => self.validate_iif_parameters(args, validation_errors),
            "aggregate" => self.validate_aggregate_parameters(args, validation_errors),
            _ => {} // Other functions have flexible parameter types
        }
    }

    /// Validate iif() function parameters
    fn validate_iif_parameters(
        &self,
        args: &[ExpressionNode],
        validation_errors: &mut Vec<crate::error::ValidationError>,
    ) {
        if args.len() >= 2 {
            // The first parameter should be a boolean condition
            // We can add more sophisticated type checking here in the future
            if let ExpressionNode::Literal(octofhir_fhirpath_ast::LiteralValue::String(_)) =
                &args[0]
            {
                validation_errors.push(crate::error::ValidationError {
                    message: "iif() condition parameter should be a boolean expression, not a string literal".to_string(),
                    error_type: crate::error::ValidationErrorType::TypeMismatch,
                    location: None,
                    suggestions: vec![
                        "Use a boolean expression like 'field = value' instead of a string literal".to_string(),
                        "Remove quotes if you meant to reference a field".to_string()
                    ],
                });
            }
        }
    }

    /// Validate aggregate() function parameters
    fn validate_aggregate_parameters(
        &self,
        args: &[ExpressionNode],
        validation_errors: &mut Vec<crate::error::ValidationError>,
    ) {
        if !args.is_empty() {
            // The first parameter should be an iterator expression
            // We can add more validation for the iterator expression here
            match &args[0] {
                ExpressionNode::Literal(_) => {
                    validation_errors.push(crate::error::ValidationError {
                        message: "aggregate() iterator parameter should be an expression that uses $this or $total, not a literal value".to_string(),
                        error_type: crate::error::ValidationErrorType::InvalidFunction,
                        location: None,
                        suggestions: vec![
                            "Use an expression like '$this + $total' or '$this.value + $total'".to_string(),
                            "Reference $this (current item) and/or $total (accumulator) in your expression".to_string()
                        ],
                    });
                }
                _ => {} // Other expressions are potentially valid
            }
        }
    }
}
