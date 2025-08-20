//! Specialized analyzer for children() function with union type support

use octofhir_fhirpath_ast::{ExpressionNode, FunctionCallData};
use std::sync::Arc;

use crate::{
    error::{AnalysisError, ValidationError, ValidationErrorType},
    model_provider_ext::ModelProviderChildrenExt,
    types::{FunctionCallAnalysis, FunctionSignature, UnionTypeInfo},
};
use octofhir_fhirpath_model::types::TypeInfo;

/// Specialized analyzer for children() function
#[derive(Debug)]
pub struct ChildrenFunctionAnalyzer<T: ModelProviderChildrenExt + ?Sized> {
    model_provider: Arc<T>,
}

impl<T: ModelProviderChildrenExt + ?Sized> ChildrenFunctionAnalyzer<T> {
    /// Create new children function analyzer
    pub fn new(model_provider: Arc<T>) -> Self {
        Self { model_provider }
    }

    /// Analyze children() function call and create union type
    pub async fn analyze_children_call(
        &self,
        _base_expression: &ExpressionNode,
        base_type: &str,
    ) -> Result<UnionTypeInfo, AnalysisError> {
        // Get all child types from model provider
        let union_type = self
            .model_provider
            .get_children_types(base_type)
            .await
            .map_err(|e| AnalysisError::UnionTypeError {
                message: format!("Failed to get children types for '{base_type}': {e}"),
            })?;

        // Validate that the base type actually has children
        if union_type.constituent_types.is_empty() {
            return Err(AnalysisError::UnionTypeError {
                message: format!("Type '{base_type}' has no child elements"),
            });
        }

        Ok(union_type)
    }

    /// Analyze type filtering operations on children() results
    pub async fn analyze_type_filter_on_children(
        &self,
        union_type: &UnionTypeInfo,
        filter_operation: &str, // "ofType", "is", "as"
        target_type: &str,
    ) -> Result<Vec<ValidationError>, AnalysisError> {
        let mut errors = Vec::new();

        // Validate that the target type is valid for this union
        let is_valid = self
            .model_provider
            .validate_type_filter(union_type, target_type)
            .await
            .map_err(|e| AnalysisError::TypeInferenceFailed {
                message: format!("Failed to validate type filter: {e}"),
            })?;

        if !is_valid {
            let suggestions = self.model_provider.suggest_valid_types(union_type).await;

            errors.push(ValidationError {
                message: format!(
                    "Type '{target_type}' is not a valid child type for {filter_operation} operation on children()"
                ),
                error_type: ValidationErrorType::InvalidTypeOperation,
                location: None,
                suggestions,
            });
        }

        Ok(errors)
    }

    /// Analyze chained operations after children()
    pub async fn analyze_children_chain(
        &self,
        children_union: &UnionTypeInfo,
        chain: &[ExpressionNode],
    ) -> Result<Vec<ValidationError>, AnalysisError> {
        let mut errors = Vec::new();

        for node in chain {
            match node {
                ExpressionNode::MethodCall(method_data) if method_data.method == "ofType" => {
                    // Handle .children().ofType(SomeType)
                    if let Some(type_arg) = method_data.args.first() {
                        if let ExpressionNode::Identifier(type_name) = type_arg {
                            let filter_errors = self
                                .analyze_type_filter_on_children(
                                    children_union,
                                    "ofType",
                                    type_name,
                                )
                                .await?;
                            errors.extend(filter_errors);
                        }
                    }
                }
                ExpressionNode::TypeCheck { type_name, .. } => {
                    // Handle .children() is SomeType
                    let filter_errors = self
                        .analyze_type_filter_on_children(children_union, "is", type_name)
                        .await?;
                    errors.extend(filter_errors);
                }
                ExpressionNode::TypeCast { type_name, .. } => {
                    // Handle .children() as SomeType
                    let filter_errors = self
                        .analyze_type_filter_on_children(children_union, "as", type_name)
                        .await?;
                    errors.extend(filter_errors);
                }
                _ => {
                    // Other operations on children() result
                    // Could analyze property access, method calls, etc.
                }
            }
        }

        Ok(errors)
    }
}

/// Integration with main analyzer
impl<T: ModelProviderChildrenExt + ?Sized> ChildrenFunctionAnalyzer<T> {
    /// Create function call analysis for children() function
    pub async fn create_children_analysis(
        &self,
        function_data: &FunctionCallData,
        base_type: &str,
        node_id: u64,
    ) -> Result<FunctionCallAnalysis, AnalysisError> {
        // children() takes no parameters
        let mut validation_errors = Vec::new();
        if !function_data.args.is_empty() {
            validation_errors.push(ValidationError {
                message: format!(
                    "Function 'children' expects 0 parameters, got {}",
                    function_data.args.len()
                ),
                error_type: ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: vec!["children() takes no arguments".to_string()],
            });
        }

        // Get union type for return type
        let union_type = match self.model_provider.get_children_types(base_type).await {
            Ok(union) => union,
            Err(e) => {
                validation_errors.push(ValidationError {
                    message: format!("Cannot determine children types: {e}"),
                    error_type: ValidationErrorType::TypeMismatch,
                    location: None,
                    suggestions: vec![format!(
                        "Ensure '{}' is a valid FHIR type with child elements",
                        base_type
                    )],
                });

                // Return empty union as fallback
                UnionTypeInfo {
                    constituent_types: Vec::new(),
                    is_collection: true,
                    model_context: std::collections::HashMap::new(),
                }
            }
        };

        // Create synthetic return type representing the union
        let return_type = TypeInfo::Union(union_type.constituent_types.clone());

        Ok(FunctionCallAnalysis {
            node_id,
            function_name: "children".to_string(),
            signature: FunctionSignature {
                name: "children".to_string(),
                parameters: Vec::new(), // No parameters
                return_type: return_type.clone(),
                is_aggregate: false,
                description: "Returns all child elements of the current element".to_string(),
            },
            parameter_types: Vec::new(),
            return_type,
            validation_errors,
        })
    }
}
