//! Function analyzer that integrates with the registry system

use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_model::types::TypeInfo;
use octofhir_fhirpath_registry::{
    FhirPathRegistry, OperationMetadata, OperationSpecificMetadata, OperationType,
};
use std::sync::Arc;

use crate::{
    error::{AnalysisError, ValidationError, ValidationErrorType},
    types::{Cardinality, FunctionCallAnalysis, FunctionSignature, ParameterInfo, TypeConstraint},
};

/// Function analyzer that integrates with the registry system
pub struct FunctionAnalyzer {
    registry: Arc<FhirPathRegistry>,
}

impl FunctionAnalyzer {
    pub fn new(registry: Arc<FhirPathRegistry>) -> Self {
        Self { registry }
    }

    /// Analyze function call for signature compliance
    pub async fn analyze_function(
        &self,
        name: &str,
        _args: &[ExpressionNode],
        arg_types: &[TypeInfo],
    ) -> Result<FunctionCallAnalysis, AnalysisError> {
        // Get function signature from registry
        let signature = self.get_function_signature(name).await.ok_or_else(|| {
            AnalysisError::FunctionAnalysisError {
                function_name: name.to_string(),
                message: "Function not found in registry".to_string(),
            }
        })?;

        // Validate parameters
        let validation_errors = self.validate_parameters(&signature, arg_types).await;

        // Determine return type based on signature and actual parameters
        let return_type = self.infer_return_type(&signature, arg_types).await;

        let node_id = 0; // Will be set by external mapping

        Ok(FunctionCallAnalysis {
            node_id,
            function_name: name.to_string(),
            signature,
            parameter_types: arg_types.to_vec(),
            return_type,
            validation_errors,
        })
    }

    /// Validate function parameter types against signature
    pub async fn validate_parameters(
        &self,
        signature: &FunctionSignature,
        actual_types: &[TypeInfo],
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check parameter count
        let required_params = signature
            .parameters
            .iter()
            .filter(|p| !p.is_optional)
            .count();

        if actual_types.len() < required_params {
            errors.push(ValidationError {
                message: format!(
                    "Function '{}' requires at least {} parameters, got {}",
                    signature.name,
                    required_params,
                    actual_types.len()
                ),
                error_type: ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: vec![format!(
                    "Expected signature: {}",
                    self.format_signature(signature)
                )],
            });
        }

        if actual_types.len() > signature.parameters.len() {
            errors.push(ValidationError {
                message: format!(
                    "Function '{}' accepts at most {} parameters, got {}",
                    signature.name,
                    signature.parameters.len(),
                    actual_types.len()
                ),
                error_type: ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: vec![format!(
                    "Expected signature: {}",
                    self.format_signature(signature)
                )],
            });
        }

        // Validate each parameter type
        for (i, (param_info, actual_type)) in signature
            .parameters
            .iter()
            .zip(actual_types.iter())
            .enumerate()
        {
            if !self
                .is_type_compatible(&param_info.type_constraint, actual_type)
                .await
            {
                errors.push(ValidationError {
                    message: format!(
                        "Parameter {} of function '{}' expects {:?}, got {:?}",
                        i + 1,
                        signature.name,
                        param_info.type_constraint,
                        actual_type
                    ),
                    error_type: ValidationErrorType::TypeMismatch,
                    location: None,
                    suggestions: vec![format!("Expected: {:?}", param_info.type_constraint)],
                });
            }
        }

        errors
    }

    /// Get function signature from registry
    pub async fn get_function_signature(&self, name: &str) -> Option<FunctionSignature> {
        // Get operation metadata from registry
        if let Some(metadata) = self.registry.get_metadata(name).await {
            // Only handle functions, not operators
            if matches!(metadata.basic.operation_type, OperationType::Function) {
                Some(self.convert_metadata_to_signature(metadata))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check type compatibility against constraint
    async fn is_type_compatible(
        &self,
        constraint: &TypeConstraint,
        actual_type: &TypeInfo,
    ) -> bool {
        match constraint {
            TypeConstraint::Any => true,
            TypeConstraint::Exact(expected) => &self.type_info_to_string(actual_type) == expected,
            TypeConstraint::OneOf(allowed_types) => {
                let actual_type_str = self.type_info_to_string(actual_type);
                allowed_types.iter().any(|t| &actual_type_str == t)
            }
            TypeConstraint::Numeric => {
                matches!(actual_type, TypeInfo::Integer | TypeInfo::Decimal)
            }
            TypeConstraint::Temporal => {
                matches!(
                    actual_type,
                    TypeInfo::Date | TypeInfo::DateTime | TypeInfo::Time
                )
            }
            TypeConstraint::Collection(inner_constraint) => {
                // For collection types, check if the inner type is compatible
                match actual_type {
                    TypeInfo::Collection(inner_type) => {
                        // Box the future to handle recursion
                        let inner_check =
                            Box::pin(self.is_type_compatible(inner_constraint, inner_type));
                        inner_check.await
                    }
                    _ => false,
                }
            }
        }
    }

    /// Infer return type based on function signature and actual parameters
    async fn infer_return_type(
        &self,
        signature: &FunctionSignature,
        _arg_types: &[TypeInfo],
    ) -> TypeInfo {
        // For now, return the signature's return type
        // Advanced implementations could infer based on actual parameter types
        signature.return_type.clone()
    }

    /// Convert registry metadata to our function signature format
    fn convert_metadata_to_signature(&self, metadata: OperationMetadata) -> FunctionSignature {
        let parameters = metadata
            .types
            .parameters
            .iter()
            .map(|param| ParameterInfo {
                name: param.name.clone(),
                type_constraint: self.convert_type_constraint(&param.constraint),
                cardinality: if param.optional {
                    Cardinality::ZeroToOne
                } else {
                    Cardinality::OneToOne
                },
                is_optional: param.optional,
            })
            .collect();

        // Extract function-specific metadata
        let is_aggregate =
            if let OperationSpecificMetadata::Function(_func_meta) = &metadata.specific {
                // For now, assume non-aggregate unless we have better information
                false
            } else {
                false
            };

        FunctionSignature {
            name: metadata.basic.name,
            parameters,
            return_type: self.convert_registry_type_to_type_info(&metadata.types.return_type),
            is_aggregate,
            description: metadata.basic.description,
        }
    }

    /// Convert registry TypeConstraint to our TypeConstraint
    fn convert_type_constraint(
        &self,
        constraint: &octofhir_fhirpath_registry::metadata::TypeConstraint,
    ) -> TypeConstraint {
        match constraint {
            octofhir_fhirpath_registry::metadata::TypeConstraint::Any => TypeConstraint::Any,
            octofhir_fhirpath_registry::metadata::TypeConstraint::Specific(fhir_type) => {
                TypeConstraint::Exact(self.convert_fhir_path_type_to_string(fhir_type))
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::OneOf(types) => {
                let type_names = types
                    .iter()
                    .map(|t| self.convert_fhir_path_type_to_string(t))
                    .collect();
                TypeConstraint::OneOf(type_names)
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Collection(inner) => {
                TypeConstraint::Collection(Box::new(self.convert_type_constraint(inner)))
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Numeric => {
                TypeConstraint::Numeric
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Comparable => TypeConstraint::Any, // Map to Any for now
        }
    }

    /// Convert FhirPathType to string
    fn convert_fhir_path_type_to_string(
        &self,
        fhir_type: &octofhir_fhirpath_registry::metadata::FhirPathType,
    ) -> String {
        match fhir_type {
            octofhir_fhirpath_registry::metadata::FhirPathType::Empty => "Empty".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Boolean => "Boolean".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Integer => "Integer".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Decimal => "Decimal".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::String => "String".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Date => "Date".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::DateTime => "DateTime".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Time => "Time".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Quantity => "Quantity".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Resource => "Resource".to_string(),
            octofhir_fhirpath_registry::metadata::FhirPathType::Collection => {
                "Collection".to_string()
            }
            octofhir_fhirpath_registry::metadata::FhirPathType::Any => "Any".to_string(),
        }
    }

    /// Convert registry TypeConstraint to string for return types
    fn convert_registry_type_to_string(
        &self,
        constraint: &octofhir_fhirpath_registry::metadata::TypeConstraint,
    ) -> String {
        match constraint {
            octofhir_fhirpath_registry::metadata::TypeConstraint::Any => "Any".to_string(),
            octofhir_fhirpath_registry::metadata::TypeConstraint::Specific(fhir_type) => {
                self.convert_fhir_path_type_to_string(fhir_type)
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::OneOf(types) => {
                if let Some(first_type) = types.first() {
                    self.convert_fhir_path_type_to_string(first_type)
                } else {
                    "Any".to_string()
                }
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Collection(_) => {
                "Collection".to_string()
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Numeric => "Numeric".to_string(),
            octofhir_fhirpath_registry::metadata::TypeConstraint::Comparable => "Any".to_string(),
        }
    }

    /// Convert TypeInfo enum to string representation
    fn type_info_to_string(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::Boolean => "Boolean".to_string(),
            TypeInfo::Integer => "Integer".to_string(),
            TypeInfo::Decimal => "Decimal".to_string(),
            TypeInfo::String => "String".to_string(),
            TypeInfo::Date => "Date".to_string(),
            TypeInfo::DateTime => "DateTime".to_string(),
            TypeInfo::Time => "Time".to_string(),
            TypeInfo::Quantity => "Quantity".to_string(),
            TypeInfo::Collection(inner) => {
                format!("Collection<{}>", self.type_info_to_string(inner))
            }
            TypeInfo::Resource(name) => name.clone(),
            TypeInfo::Any => "Any".to_string(),
            TypeInfo::Union(types) => {
                let type_strs: Vec<String> =
                    types.iter().map(|t| self.type_info_to_string(t)).collect();
                format!("Union<{}>", type_strs.join(", "))
            }
            TypeInfo::Optional(inner) => format!("Optional<{}>", self.type_info_to_string(inner)),
            TypeInfo::SimpleType => "SimpleType".to_string(),
            TypeInfo::ClassType => "ClassType".to_string(),
            TypeInfo::TypeInfo => "TypeInfo".to_string(),
            TypeInfo::Function {
                parameters,
                return_type,
            } => {
                let param_strs: Vec<String> = parameters
                    .iter()
                    .map(|t| self.type_info_to_string(t))
                    .collect();
                format!(
                    "Function<({}) -> {}>",
                    param_strs.join(", "),
                    self.type_info_to_string(return_type)
                )
            }
            TypeInfo::Tuple(types) => {
                let type_strs: Vec<String> =
                    types.iter().map(|t| self.type_info_to_string(t)).collect();
                format!("Tuple<{}>", type_strs.join(", "))
            }
            TypeInfo::Named { namespace, name } => {
                format!("{namespace}::{name}")
            }
        }
    }

    /// Convert registry TypeConstraint to TypeInfo
    fn convert_registry_type_to_type_info(
        &self,
        constraint: &octofhir_fhirpath_registry::metadata::TypeConstraint,
    ) -> TypeInfo {
        match constraint {
            octofhir_fhirpath_registry::metadata::TypeConstraint::Any => TypeInfo::Any,
            octofhir_fhirpath_registry::metadata::TypeConstraint::Specific(fhir_type) => {
                self.convert_fhir_path_type_to_type_info(fhir_type)
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::OneOf(types) => {
                let type_infos = types
                    .iter()
                    .map(|t| self.convert_fhir_path_type_to_type_info(t))
                    .collect();
                TypeInfo::Union(type_infos)
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Collection(inner) => {
                TypeInfo::Collection(Box::new(self.convert_registry_type_to_type_info(inner)))
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Numeric => {
                TypeInfo::Union(vec![TypeInfo::Integer, TypeInfo::Decimal])
            }
            octofhir_fhirpath_registry::metadata::TypeConstraint::Comparable => TypeInfo::Any,
        }
    }

    /// Convert FhirPathType to TypeInfo
    fn convert_fhir_path_type_to_type_info(
        &self,
        fhir_type: &octofhir_fhirpath_registry::metadata::FhirPathType,
    ) -> TypeInfo {
        match fhir_type {
            octofhir_fhirpath_registry::metadata::FhirPathType::Empty => TypeInfo::Any, // Empty collection
            octofhir_fhirpath_registry::metadata::FhirPathType::Boolean => TypeInfo::Boolean,
            octofhir_fhirpath_registry::metadata::FhirPathType::Integer => TypeInfo::Integer,
            octofhir_fhirpath_registry::metadata::FhirPathType::Decimal => TypeInfo::Decimal,
            octofhir_fhirpath_registry::metadata::FhirPathType::String => TypeInfo::String,
            octofhir_fhirpath_registry::metadata::FhirPathType::Date => TypeInfo::Date,
            octofhir_fhirpath_registry::metadata::FhirPathType::DateTime => TypeInfo::DateTime,
            octofhir_fhirpath_registry::metadata::FhirPathType::Time => TypeInfo::Time,
            octofhir_fhirpath_registry::metadata::FhirPathType::Quantity => TypeInfo::Quantity,
            octofhir_fhirpath_registry::metadata::FhirPathType::Resource => {
                TypeInfo::Resource("Resource".to_string())
            }
            octofhir_fhirpath_registry::metadata::FhirPathType::Collection => {
                TypeInfo::Collection(Box::new(TypeInfo::Any))
            }
            octofhir_fhirpath_registry::metadata::FhirPathType::Any => TypeInfo::Any,
        }
    }

    /// Format signature for error messages
    fn format_signature(&self, signature: &FunctionSignature) -> String {
        let params: Vec<String> = signature
            .parameters
            .iter()
            .map(|p| {
                let optional = if p.is_optional { "?" } else { "" };
                format!("{}: {:?}{}", p.name, p.type_constraint, optional)
            })
            .collect();

        format!(
            "{}({}) -> {}",
            signature.name,
            params.join(", "),
            self.type_info_to_string(&signature.return_type)
        )
    }
}
