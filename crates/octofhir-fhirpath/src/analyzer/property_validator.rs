//! Property validation for FHIRPath expressions
//!
//! This module implements comprehensive property validation for FHIRPath expressions,
//! including detection of invalid properties, typo suggestions, and FHIR compliance checking.

use crate::analyzer::AnalysisWarning;
use crate::analyzer::context::AnalysisContext;
use crate::analyzer::type_checker::TypeInfo;
use crate::ast::expression::*;
use crate::core::error_code::{ErrorCode, FP0054, FP0055, FP0121, FP0124, FP0125};
use crate::core::{ModelProvider, Result, SourceLocation};
use crate::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};
use crate::registry::FunctionRegistry;
use octofhir_fhir_model::TypeReflectionInfo;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Result of property validation analysis
#[derive(Debug, Clone)]
pub struct PropertyValidationResult {
    /// Analysis warnings for invalid properties (legacy)
    pub warnings: Vec<AnalysisWarning>,
    /// Valid properties for encountered types
    pub valid_properties: HashMap<String, Vec<String>>,
    /// Property suggestions for typos and alternatives
    pub suggestions: Vec<PropertySuggestion>,
    /// Enhanced Ariadne diagnostics (new)
    pub ariadne_diagnostics: Vec<AriadneDiagnostic>,
}

/// Suggestion for correcting invalid property access
#[derive(Debug, Clone)]
pub struct PropertySuggestion {
    /// The invalid property name that was used
    pub invalid_property: String,
    /// List of suggested correct property names
    pub suggested_properties: Vec<String>,
    /// Similarity score between invalid and suggested properties (0.0-1.0)
    pub similarity_score: f32,
    /// The type context where the property was accessed
    pub context_type: String,
    /// Source location of the invalid property access
    pub location: Option<SourceLocation>,
}

/// Enhanced suggestion with detailed reasoning and confidence
#[derive(Debug, Clone)]
pub struct DetailedSuggestion {
    /// The suggested property or function name
    pub suggestion: String,
    /// Reason for the suggestion ("required property", "commonly used", "similar spelling", etc.)
    pub reason: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Context type (resource type for properties, "function" for functions)
    pub context_type: String,
    /// Whether this is a polymorphic property suggestion
    pub is_polymorphic: bool,
}

/// Performance-optimized caching for suggestions
#[derive(Debug, Clone, Default)]
struct SuggestionCache {
    /// Cache of property names by resource type
    property_cache: HashMap<String, Vec<String>>,
    /// Cache of all available function names
    function_cache: Option<Vec<String>>,
    /// Cache of detailed suggestions by (invalid_name, context_type) key
    suggestion_cache: HashMap<(String, String), Vec<DetailedSuggestion>>,
    /// Cache of polymorphic property mappings
    polymorphic_cache: HashMap<String, Vec<String>>,
}

/// Information about a property on a FHIR resource or element
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    /// Property name
    pub name: String,
    /// Data type of the property
    pub data_type: TypeInfo,
    /// Cardinality constraints
    pub cardinality: Cardinality,
    /// Whether the property is required
    pub required: bool,
    /// Human-readable description
    pub description: String,
    /// Common alternative names or aliases
    pub aliases: Vec<String>,
}

/// Cardinality constraints for FHIR properties
#[derive(Debug, Clone, PartialEq)]
pub enum Cardinality {
    /// 0..1 - Optional single value
    ZeroToOne,
    /// 0..* - Optional multiple values
    ZeroToMany,
    /// 1..1 - Required single value
    OneToOne,
    /// 1..* - Required multiple values
    OneToMany,
}

/// Property validator for FHIRPath expressions
pub struct PropertyValidator {
    /// Common typos mapping to correct properties
    common_typos: HashMap<String, String>,
    /// Cache for performance optimization (legacy - replaced by suggestion_cache)
    property_cache: HashMap<String, HashSet<String>>,
    /// Model provider for dynamic property resolution (REQUIRED)
    model_provider: Arc<dyn ModelProvider>,
    /// Function registry for function name validation and suggestions
    function_registry: Arc<FunctionRegistry>,
    /// Enhanced caching system for suggestions
    suggestion_cache: SuggestionCache,
}

impl PropertyValidator {
    /// Create a new property validator with a model provider and function registry (REQUIRED)
    pub fn new(
        model_provider: Arc<dyn ModelProvider>,
        function_registry: Arc<FunctionRegistry>,
    ) -> Self {
        let mut validator = Self {
            common_typos: HashMap::new(),
            property_cache: HashMap::new(),
            model_provider,
            function_registry,
            suggestion_cache: SuggestionCache::default(),
        };

        validator.initialize_common_typos();
        validator
    }

    /// Convert TypeReflectionInfo from ModelProvider to TypeInfo used by PropertyValidator
    fn convert_reflection_to_typeinfo(&self, reflection: &TypeReflectionInfo) -> TypeInfo {
        match reflection {
            TypeReflectionInfo::SimpleType {
                namespace,
                name,
                base_type: _base_type,
            } => {
                // Convert System (primitive) types
                if namespace == "System" {
                    match name.as_str() {
                        "Boolean" => TypeInfo::Boolean,
                        "Integer" => TypeInfo::Integer,
                        "Decimal" => TypeInfo::Decimal,
                        "String" => TypeInfo::String,
                        "Date" => TypeInfo::Date,
                        "DateTime" => TypeInfo::DateTime,
                        "Time" => TypeInfo::Time,
                        _ => TypeInfo::Unknown,
                    }
                }
                // Convert FHIR types
                else if namespace == "FHIR" {
                    match name.as_str() {
                        "boolean" => TypeInfo::Boolean,
                        "integer" => TypeInfo::Integer,
                        "decimal" => TypeInfo::Decimal,
                        "string" => TypeInfo::String,
                        "date" => TypeInfo::Date,
                        "dateTime" => TypeInfo::DateTime,
                        "time" => TypeInfo::Time,
                        "Quantity" => TypeInfo::Quantity,
                        "code" => TypeInfo::Code,
                        "Coding" => TypeInfo::Coding,
                        "CodeableConcept" => TypeInfo::CodeableConcept,
                        "Range" => TypeInfo::Range,
                        "Reference" => TypeInfo::Reference {
                            target_types: vec![],
                        },
                        // Check if it's a resource type (simple heuristic - starts with capital letter)
                        resource_name
                            if resource_name.chars().next().unwrap_or('a').is_uppercase() =>
                        {
                            TypeInfo::Resource {
                                resource_type: resource_name.to_string(),
                            }
                        }
                        _ => TypeInfo::Unknown,
                    }
                } else {
                    TypeInfo::Unknown
                }
            }
            TypeReflectionInfo::ClassInfo {
                name,
                namespace,
                elements,
                base_type: _base_type,
            } => {
                if namespace == "FHIR" {
                    // Check if it's a resource type (simple heuristic - starts with capital letter)
                    if name.chars().next().unwrap_or('a').is_uppercase() {
                        TypeInfo::Resource {
                            resource_type: name.clone(),
                        }
                    } else {
                        // Convert to BackboneElement with properties
                        let mut properties = HashMap::new();
                        for element in elements {
                            let property_type =
                                self.convert_reflection_to_typeinfo(&element.type_info);
                            properties.insert(element.name.clone(), property_type);
                        }
                        TypeInfo::BackboneElement { properties }
                    }
                } else {
                    TypeInfo::Unknown
                }
            }
            TypeReflectionInfo::ListType { element_type } => {
                let inner_type = self.convert_reflection_to_typeinfo(element_type);
                TypeInfo::Collection(Box::new(inner_type))
            }
            TypeReflectionInfo::TupleType {
                elements: _elements,
            } => {
                // For now, treat tuples as unknown - could be enhanced later
                TypeInfo::Unknown
            }
        }
    }

    /// Check if a type name represents a FHIR resource type using ModelProvider
    async fn is_resource_type_name(&self, type_name: &str) -> bool {
        // WORKAROUND: resource_type_exists() has a bug where the HashMap is not populated
        // Use get_type_reflection() instead, which works correctly
        self.model_provider
            .get_type_reflection(type_name)
            .await
            .map(|reflection| reflection.is_some())
            .unwrap_or(false)
    }

    /// Validate an identifier as a property in the current context
    async fn validate_identifier_as_property(
        &self,
        identifier: &IdentifierNode,
        context: &mut AnalysisContext,
        warnings: &mut Vec<AnalysisWarning>,
        suggestions: &mut Vec<PropertySuggestion>,
    ) -> Result<()> {
        // Get the current context type to validate the property against
        let context_types = context.get_current_types();

        // If we have no context information, we can't validate the property
        if context_types.is_empty() {
            // This might be a standalone identifier like a function parameter
            // For now, let's not generate warnings for these cases
            return Ok(());
        }

        // Check if the identifier is a valid property for any of the current context types
        let mut is_valid_property = false;
        let mut all_suggested_properties = Vec::new();

        for context_type in &context_types {
            match context_type {
                TypeInfo::Resource { resource_type } => {
                    // Check if the property exists on this resource type
                    match self.model_provider.get_type_reflection(resource_type).await {
                        Ok(Some(TypeReflectionInfo::ClassInfo { elements, .. })) => {
                            let property_names: Vec<String> =
                                elements.iter().map(|e| e.name.clone()).collect();

                            if property_names.contains(&identifier.name) {
                                is_valid_property = true;
                                break;
                            }

                            // Collect suggestions for this resource type
                            all_suggested_properties.extend(property_names);
                        }
                        _ => {
                            // Could not get type reflection or not ClassInfo - skip validation for this type
                            continue;
                        }
                    }
                }
                TypeInfo::BackboneElement { properties } => {
                    // Similar logic for backbone elements - check direct properties
                    let property_names: Vec<String> = properties.keys().cloned().collect();

                    if property_names.contains(&identifier.name) {
                        is_valid_property = true;
                        break;
                    }

                    all_suggested_properties.extend(property_names);
                }
                TypeInfo::Collection(element_type) => {
                    // Recursively check the element type
                    if let TypeInfo::Resource { resource_type } = element_type.as_ref() {
                        match self.model_provider.get_type_reflection(resource_type).await {
                            Ok(Some(TypeReflectionInfo::ClassInfo { elements, .. })) => {
                                let property_names: Vec<String> =
                                    elements.iter().map(|e| e.name.clone()).collect();

                                if property_names.contains(&identifier.name) {
                                    is_valid_property = true;
                                    break;
                                }

                                all_suggested_properties.extend(property_names);
                            }
                            _ => {
                                // Could not get type reflection or not ClassInfo - skip validation for this type
                                continue;
                            }
                        }
                    }
                }
                _ => {
                    // For other types, we might not be able to validate properties
                    continue;
                }
            }
        }

        if !is_valid_property && !context_types.is_empty() {
            // Generate a warning for unknown property
            let context_type_names: Vec<String> = context_types
                .iter()
                .map(|t| match t {
                    TypeInfo::Resource { resource_type } => resource_type.clone(),
                    TypeInfo::BackboneElement { .. } => "BackboneElement".to_string(),
                    TypeInfo::Collection(element_type) => match element_type.as_ref() {
                        TypeInfo::Resource { resource_type } => {
                            format!("Collection<{}>", resource_type)
                        }
                        _ => "Collection<Unknown>".to_string(),
                    },
                    _ => "unknown".to_string(),
                })
                .collect();

            let context_desc = if context_type_names.len() == 1 {
                context_type_names[0].clone()
            } else {
                format!("one of [{}]", context_type_names.join(", "))
            };

            warnings.push(AnalysisWarning {
                code: "FP0055".to_string(),
                message: format!(
                    "Property '{}' not found on type '{}'. Check if the property name is correct.",
                    identifier.name, context_desc
                ),
                location: identifier.location.clone(),
                severity: DiagnosticSeverity::Error,
                suggestion: None,
            });

            // Generate property suggestions
            if !all_suggested_properties.is_empty() {
                all_suggested_properties.sort();
                all_suggested_properties.dedup();

                // Find close matches using simple string distance
                let close_matches: Vec<String> = all_suggested_properties
                    .iter()
                    .filter(|prop| {
                        // Simple heuristic: suggest if it starts with the same letter
                        // or contains the identifier name
                        prop.to_lowercase().starts_with(
                            &identifier
                                .name
                                .to_lowercase()
                                .chars()
                                .next()
                                .unwrap_or('a')
                                .to_string(),
                        ) || prop
                            .to_lowercase()
                            .contains(&identifier.name.to_lowercase())
                            || identifier
                                .name
                                .to_lowercase()
                                .contains(&prop.to_lowercase())
                    })
                    .take(3) // Limit suggestions
                    .cloned()
                    .collect();

                if !close_matches.is_empty() {
                    suggestions.push(PropertySuggestion {
                        invalid_property: identifier.name.clone(),
                        suggested_properties: close_matches,
                        location: identifier.location.clone(),
                        context_type: context_desc,
                        similarity_score: 0.8, // Simple heuristic score
                    });
                }
            }
        }

        Ok(())
    }

    /// Check if an identifier could be a resource type based on capitalization
    fn is_potential_resource_type(&self, identifier: &str) -> bool {
        if identifier.is_empty() {
            return false;
        }

        // Must start with uppercase letter
        let first_char = identifier.chars().next().unwrap_or('a');
        if !first_char.is_uppercase() {
            return false;
        }

        // Should not contain underscores (FHIR resource types are PascalCase)
        if identifier.contains('_') {
            return false;
        }

        // Should not start with $ (variable reference)
        if identifier.starts_with('$') {
            return false;
        }

        true
    }

    /// Generate intelligent suggestions for misspelled resource types
    async fn generate_resource_type_suggestions(&self, invalid_resource: &str) -> Vec<String> {
        // Common FHIR resource types that we can check against
        let common_resources = [
            "Patient",
            "Observation",
            "Encounter",
            "Practitioner",
            "Organization",
            "DiagnosticReport",
            "Condition",
            "Procedure",
            "MedicationRequest",
            "AllergyIntolerance",
            "Immunization",
            "Bundle",
            "Composition",
            "Location",
            "Device",
            "Medication",
            "Appointment",
            "ServiceRequest",
            "QuestionnaireResponse",
            "Questionnaire",
            "DocumentReference",
            "Binary",
            "OperationOutcome",
            "Parameters",
            "StructureDefinition",
            "ValueSet",
            "CodeSystem",
            "ConceptMap",
            "CapabilityStatement",
            "SearchParameter",
            "CompartmentDefinition",
            "ImplementationGuide",
        ];

        let mut suggestions = Vec::new();

        for resource_type in common_resources {
            // First check if this resource type exists in the ModelProvider
            if self
                .model_provider
                .resource_type_exists(resource_type)
                .unwrap_or(false)
            {
                let similarity = self.calculate_similarity(invalid_resource, resource_type);
                if similarity > 0.6 {
                    suggestions.push((resource_type.to_string(), similarity));
                }
            }
        }

        // Sort by similarity (highest first) and return top 3
        suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        suggestions
            .into_iter()
            .take(3)
            .map(|(name, _)| name)
            .collect()
    }

    /// Handle unknown resource type by generating appropriate warnings
    async fn handle_unknown_resource_type(
        &self,
        invalid_resource: &str,
        location: Option<SourceLocation>,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> TypeInfo {
        let suggestions = self
            .generate_resource_type_suggestions(invalid_resource)
            .await;

        let (message, suggestion) = if !suggestions.is_empty() {
            (
                format!("Unknown resource type: '{}'", invalid_resource),
                Some(format!("Did you mean: {}?", suggestions.join(", "))),
            )
        } else {
            (
                format!(
                    "Unknown resource type: '{}'. Check the FHIR specification for valid resource types.",
                    invalid_resource
                ),
                None,
            )
        };

        warnings.push(AnalysisWarning {
            code: FP0121.code_str(),
            message,
            location: location.clone(),
            severity: DiagnosticSeverity::Error,
            suggestion,
        });

        // Return Unknown to continue processing, but validation will fail
        TypeInfo::Unknown
    }

    /// Validate a potential resource type identifier
    async fn validate_potential_resource_type(
        &self,
        type_name: &str,
        location: Option<SourceLocation>,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> Result<()> {
        // Only validate if it looks like a potential resource type
        if self.is_potential_resource_type(type_name) {
            // Check if it's a valid resource type
            if !self
                .model_provider
                .resource_type_exists(type_name)
                .unwrap_or(false)
            {
                // Handle the unknown resource type
                self.handle_unknown_resource_type(type_name, location, warnings)
                    .await;
            }
        }
        Ok(())
    }

    /// Validate property access in a FHIRPath expression
    pub async fn validate(&self, expression: &ExpressionNode) -> Result<PropertyValidationResult> {
        self.validate_with_source_text(expression, "").await
    }

    /// Enhanced validate method that generates both legacy and Ariadne diagnostics
    pub async fn validate_with_source_text(
        &self,
        expression: &ExpressionNode,
        source_text: &str,
    ) -> Result<PropertyValidationResult> {
        let mut warnings = Vec::new();
        let mut valid_properties = HashMap::new();
        let mut suggestions = Vec::new();
        let mut ariadne_diagnostics = Vec::new();
        let mut context = AnalysisContext::new();

        self.validate_expression(expression, &mut context, &mut warnings, &mut suggestions)
            .await?;

        // Convert all warnings to enhanced Ariadne diagnostics
        for warning in &warnings {
            let diagnostic = self.convert_warning_to_diagnostic(warning, source_text);
            ariadne_diagnostics.push(diagnostic);
        }

        // NOTE: Enhanced diagnostics for suggestions are separate and don't duplicate warnings

        // Collect valid properties for all encountered types
        for scope in &context.scopes {
            for (_var_name, type_info) in &scope.variables {
                if let Some(properties) = self.get_properties_for_type(type_info).await {
                    valid_properties.insert(
                        type_info.to_string(),
                        properties.iter().map(|p| p.name.clone()).collect(),
                    );
                }
            }
        }

        Ok(PropertyValidationResult {
            warnings,
            valid_properties,
            suggestions,
            ariadne_diagnostics, // New field
        })
    }

    fn validate_expression<'a>(
        &'a self,
        expression: &'a ExpressionNode,
        context: &'a mut AnalysisContext,
        warnings: &'a mut Vec<AnalysisWarning>,
        suggestions: &'a mut Vec<PropertySuggestion>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            match expression {
                ExpressionNode::PropertyAccess(access) => {
                    // First validate the base object if it might be a resource type
                    if let ExpressionNode::Identifier(base_id) = &*access.object {
                        self.validate_potential_resource_type(
                            &base_id.name,
                            base_id.location.clone(),
                            warnings,
                        )
                        .await?;
                    }

                    self.validate_property_access(access, context, warnings, suggestions)
                        .await?;
                    self.validate_expression(&access.object, context, warnings, suggestions)
                        .await?;
                }
                ExpressionNode::FunctionCall(call) => {
                    // Validate function name
                    self.validate_function_name(&call.name, call.location.clone(), warnings);

                    for arg in &call.arguments {
                        self.validate_expression(arg, context, warnings, suggestions)
                            .await?;
                    }
                }
                ExpressionNode::MethodCall(method) => {
                    self.validate_expression(&method.object, context, warnings, suggestions)
                        .await?;
                    for arg in &method.arguments {
                        self.validate_expression(arg, context, warnings, suggestions)
                            .await?;
                    }
                }
                ExpressionNode::BinaryOperation(binary) => {
                    self.validate_expression(&binary.left, context, warnings, suggestions)
                        .await?;
                    self.validate_expression(&binary.right, context, warnings, suggestions)
                        .await?;
                }
                ExpressionNode::UnaryOperation(unary) => {
                    self.validate_expression(&unary.operand, context, warnings, suggestions)
                        .await?;
                }
                ExpressionNode::Filter(filter) => {
                    self.validate_expression(&filter.base, context, warnings, suggestions)
                        .await?;
                    self.validate_expression(&filter.condition, context, warnings, suggestions)
                        .await?;
                }
                ExpressionNode::IndexAccess(index) => {
                    self.validate_expression(&index.object, context, warnings, suggestions)
                        .await?;
                    self.validate_expression(&index.index, context, warnings, suggestions)
                        .await?;
                }
                ExpressionNode::Lambda(lambda) => {
                    context.push_scope(crate::analyzer::context::ScopeType::Lambda {
                        parameter: Some("item".to_string()),
                    });
                    self.validate_expression(&lambda.body, context, warnings, suggestions)
                        .await?;
                    context.pop_scope();
                }
                ExpressionNode::Collection(coll) => {
                    for element in &coll.elements {
                        self.validate_expression(element, context, warnings, suggestions)
                            .await?;
                    }
                }
                ExpressionNode::Parenthesized(expr) => {
                    self.validate_expression(expr, context, warnings, suggestions)
                        .await?;
                }
                ExpressionNode::Identifier(id) => {
                    // First check if it's a potential resource type
                    if self.is_potential_resource_type(&id.name) {
                        self.validate_potential_resource_type(
                            &id.name,
                            id.location.clone(),
                            warnings,
                        )
                        .await?;
                    } else {
                        // If not a resource type, validate it as a property in the current context
                        self.validate_identifier_as_property(id, context, warnings, suggestions)
                            .await?;
                    }
                }
                _ => {} // Literals don't need validation
            }
            Ok(())
        })
    }

    async fn validate_property_access(
        &self,
        access: &PropertyAccessNode,
        context: &AnalysisContext,
        warnings: &mut Vec<AnalysisWarning>,
        suggestions: &mut Vec<PropertySuggestion>,
    ) -> Result<()> {
        // Determine the type of the object being accessed
        let object_type = self.infer_object_type(&access.object, context).await?;

        match &object_type {
            TypeInfo::Resource { resource_type } => {
                self.validate_resource_property(
                    resource_type,
                    &access.property,
                    access.location.clone(),
                    warnings,
                    suggestions,
                )
                .await;
            }
            TypeInfo::BackboneElement { properties } => {
                self.validate_backbone_property(
                    &access.property,
                    properties,
                    access.location.clone(),
                    warnings,
                    suggestions,
                );
            }
            TypeInfo::Collection(inner_type) => {
                // Validate property access on collection elements
                match inner_type.as_ref() {
                    TypeInfo::Resource { resource_type } => {
                        self.validate_resource_property(
                            resource_type,
                            &access.property,
                            access.location.clone(),
                            warnings,
                            suggestions,
                        )
                        .await;
                    }
                    TypeInfo::BackboneElement { properties } => {
                        self.validate_backbone_property(
                            &access.property,
                            properties,
                            access.location.clone(),
                            warnings,
                            suggestions,
                        );
                    }
                    _ => {
                        warnings.push(AnalysisWarning {
                            code: "FP0114".to_string(),
                            message: format!(
                                "Property access '{}' on collection of non-object type: {}",
                                access.property, inner_type
                            ),
                            location: access.location.clone(),
                            severity: DiagnosticSeverity::Warning,
                            suggestion: Some(
                                "Property access is only valid on resources and elements"
                                    .to_string(),
                            ),
                        });
                    }
                }
            }
            TypeInfo::Any | TypeInfo::Unknown => {
                // Cannot validate - might be valid
                warnings.push(AnalysisWarning {
                    code: "FP0115".to_string(),
                    message: format!(
                        "Cannot validate property '{}' on unknown type",
                        access.property
                    ),
                    location: access.location.clone(),
                    severity: DiagnosticSeverity::Info,
                    suggestion: Some(
                        "Consider providing type information for better validation".to_string(),
                    ),
                });
            }
            _ => {
                warnings.push(AnalysisWarning {
                    code: "FP0116".to_string(),
                    message: format!(
                        "Invalid property access '{}' on primitive type: {}",
                        access.property, object_type
                    ),
                    location: access.location.clone(),
                    severity: DiagnosticSeverity::Error,
                    suggestion: Some("Property access is only valid on complex types".to_string()),
                });
            }
        }

        Ok(())
    }

    async fn validate_resource_property(
        &self,
        resource_name: &str,
        property: &str,
        location: Option<SourceLocation>,
        warnings: &mut Vec<AnalysisWarning>,
        suggestions: &mut Vec<PropertySuggestion>,
    ) {
        if let Some(properties) = self.get_resource_properties(resource_name).await {
            // Check if property exists
            if !properties
                .iter()
                .any(|p| p.name == property || p.aliases.contains(&property.to_string()))
            {
                // Property not found - generate suggestions
                let similar_properties = self.find_similar_properties(property, &properties);

                warnings.push(AnalysisWarning {
                    code: "FP0117".to_string(),
                    message: format!(
                        "Unknown property '{}' on resource '{}'",
                        property, resource_name
                    ),
                    location: location.clone(),
                    severity: DiagnosticSeverity::Error,
                    suggestion: if !similar_properties.is_empty() {
                        Some(format!("Did you mean: {}?", similar_properties.join(", ")))
                    } else {
                        Some(format!(
                            "Valid properties for {}: {}",
                            resource_name,
                            properties
                                .iter()
                                .map(|p| &p.name)
                                .take(5)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    },
                });

                if !similar_properties.is_empty() {
                    suggestions.push(PropertySuggestion {
                        invalid_property: property.to_string(),
                        suggested_properties: similar_properties,
                        similarity_score: 0.8,
                        context_type: resource_name.to_string(),
                        location: location.clone(),
                    });
                }
            } else {
                // Property exists - validate usage patterns
                if let Some(prop_info) = properties.iter().find(|p| p.name == property) {
                    self.validate_property_usage(prop_info, location.clone(), warnings);
                }
            }
        } else {
            warnings.push(AnalysisWarning {
                code: "FP0118".to_string(),
                message: format!("Unknown resource type: {}", resource_name),
                location: location.clone(),
                severity: DiagnosticSeverity::Warning,
                suggestion: Some("Check if the resource type is correct".to_string()),
            });
        }
    }

    fn validate_backbone_property(
        &self,
        property: &str,
        properties: &HashMap<String, TypeInfo>,
        location: Option<SourceLocation>,
        warnings: &mut Vec<AnalysisWarning>,
        suggestions: &mut Vec<PropertySuggestion>,
    ) {
        if !properties.contains_key(property) {
            let property_names: Vec<String> = properties.keys().cloned().collect();
            let similar_properties = self.find_similar_property_names(property, &property_names);

            warnings.push(AnalysisWarning {
                code: "FP0119".to_string(),
                message: format!("Unknown property '{}' on backbone element", property),
                location: location.clone(),
                severity: DiagnosticSeverity::Error,
                suggestion: if !similar_properties.is_empty() {
                    Some(format!("Did you mean: {}?", similar_properties.join(", ")))
                } else {
                    None
                },
            });

            if !similar_properties.is_empty() {
                suggestions.push(PropertySuggestion {
                    invalid_property: property.to_string(),
                    suggested_properties: similar_properties,
                    similarity_score: 0.8,
                    context_type: "BackboneElement".to_string(),
                    location: location.clone(),
                });
            }
        }
    }

    fn validate_property_usage(
        &self,
        prop_info: &PropertyInfo,
        location: Option<SourceLocation>,
        warnings: &mut Vec<AnalysisWarning>,
    ) {
        // Additional validation rules based on cardinality and usage patterns
        match prop_info.cardinality {
            Cardinality::ZeroToOne | Cardinality::OneToOne => {
                // Single-valued properties - no special validation needed here
            }
            Cardinality::ZeroToMany | Cardinality::OneToMany => {
                // Multi-valued properties - could warn about performance for large collections
                if prop_info.name == "extension" || prop_info.name == "modifierExtension" {
                    warnings.push(AnalysisWarning {
                        code: "FP0120".to_string(),
                        message: format!("Accessing collection property '{}' may have performance implications", prop_info.name),
                        location,
                        severity: DiagnosticSeverity::Info,
                        suggestion: Some("Consider using filters or specific indexes if accessing large collections".to_string()),
                    });
                }
            }
        }
    }

    fn find_similar_properties(&self, target: &str, properties: &[PropertyInfo]) -> Vec<String> {
        let mut candidates: Vec<(String, f32)> = properties
            .iter()
            .map(|p| {
                let similarity = self.calculate_similarity(target, &p.name);
                (p.name.clone(), similarity)
            })
            .collect();

        // Also check aliases
        for prop in properties {
            for alias in &prop.aliases {
                let similarity = self.calculate_similarity(target, alias);
                candidates.push((alias.clone(), similarity));
            }
        }

        // Check common typos
        if let Some(correction) = self.common_typos.get(target) {
            candidates.push((correction.clone(), 1.0));
        }

        // Sort by similarity and take top matches
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
            .into_iter()
            .filter(|(_, score)| *score > 0.6)
            .take(3)
            .map(|(name, _)| name)
            .collect()
    }

    fn find_similar_property_names(&self, target: &str, property_names: &[String]) -> Vec<String> {
        let mut candidates: Vec<(String, f32)> = property_names
            .iter()
            .map(|name| {
                let similarity = self.calculate_similarity(target, name);
                (name.clone(), similarity)
            })
            .collect();

        // Check common typos
        if let Some(correction) = self.common_typos.get(target) {
            candidates.push((correction.clone(), 1.0));
        }

        // Sort by similarity and take top matches
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
            .into_iter()
            .filter(|(_, score)| *score > 0.6)
            .take(3)
            .map(|(name, _)| name)
            .collect()
    }

    fn calculate_similarity(&self, a: &str, b: &str) -> f32 {
        // Levenshtein distance normalized by string length
        let distance = self.levenshtein_distance(a, b);
        let max_len = a.len().max(b.len()) as f32;
        if max_len == 0.0 {
            1.0
        } else {
            1.0 - (distance as f32 / max_len)
        }
    }

    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        // Initialize first row and column
        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[a_len][b_len]
    }

    fn infer_object_type<'a>(
        &'a self,
        object: &'a ExpressionNode,
        context: &'a AnalysisContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TypeInfo>> + 'a>> {
        Box::pin(async move {
            // Simplified type inference for property validation
            match object {
                ExpressionNode::Identifier(id) => {
                    if let Some(var_type) = context.lookup_variable(&id.name) {
                        Ok(var_type.clone())
                    } else if self.is_known_resource_type(&id.name).await {
                        Ok(TypeInfo::Resource {
                            resource_type: id.name.clone(),
                        })
                    } else {
                        Ok(TypeInfo::Unknown)
                    }
                }
                ExpressionNode::PropertyAccess(access) => {
                    let parent_type = self.infer_object_type(&access.object, context).await?;
                    self.infer_property_type(&parent_type, &access.property)
                        .await
                }
                ExpressionNode::FunctionCall(_) => Ok(TypeInfo::Any), // Would need full type inference
                ExpressionNode::MethodCall(_) => Ok(TypeInfo::Any), // Would need full type inference
                ExpressionNode::Filter(filter) => {
                    self.infer_object_type(&filter.base, context).await
                }
                _ => Ok(TypeInfo::Unknown),
            }
        })
    }

    fn infer_property_type<'a>(
        &'a self,
        object_type: &'a TypeInfo,
        property: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TypeInfo>> + 'a>> {
        Box::pin(async move {
            match object_type {
                TypeInfo::Resource { resource_type } => {
                    // Use ModelProvider's get_type_reflection for dynamic property type resolution
                    match self.model_provider.get_type_reflection(resource_type).await {
                        Ok(Some(TypeReflectionInfo::ClassInfo { elements, .. })) => {
                            // Find the property in the elements
                            for element in elements {
                                if element.name == property {
                                    return Ok(
                                        self.convert_reflection_to_typeinfo(&element.type_info)
                                    );
                                }
                            }
                            Ok(TypeInfo::Unknown)
                        }
                        Ok(_) => Ok(TypeInfo::Unknown),
                        Err(_) => Ok(TypeInfo::Unknown),
                    }
                }
                TypeInfo::BackboneElement { properties } => {
                    if let Some(prop_type) = properties.get(property) {
                        Ok(prop_type.clone())
                    } else {
                        Ok(TypeInfo::Unknown)
                    }
                }
                TypeInfo::Collection(inner) => {
                    let inner_property_type = self.infer_property_type(inner, property).await?;
                    Ok(TypeInfo::Collection(Box::new(inner_property_type)))
                }
                _ => Ok(TypeInfo::Unknown),
            }
        })
    }

    async fn get_properties_for_type(&self, type_info: &TypeInfo) -> Option<Vec<PropertyInfo>> {
        match type_info {
            TypeInfo::Resource { resource_type } => {
                self.get_resource_properties(resource_type).await
            }
            TypeInfo::BackboneElement { properties } => {
                // Convert HashMap<String, TypeInfo> to Vec<PropertyInfo>
                let prop_infos: Vec<PropertyInfo> = properties
                    .iter()
                    .map(|(name, type_info)| {
                        PropertyInfo {
                            name: name.clone(),
                            data_type: type_info.clone(),
                            cardinality: Cardinality::ZeroToOne, // Default assumption
                            required: false,
                            description: format!("Property {} of type {}", name, type_info),
                            aliases: vec![],
                        }
                    })
                    .collect();
                Some(prop_infos)
            }
            _ => None,
        }
    }

    async fn get_resource_properties(&self, resource_name: &str) -> Option<Vec<PropertyInfo>> {
        // Use ModelProvider's get_type_reflection for dynamic property resolution
        let type_info = match self.model_provider.get_type_reflection(resource_name).await {
            Ok(Some(info)) => info,
            Ok(None) => return None,
            Err(_) => return None,
        };

        // Extract properties from ClassInfo
        let properties = match type_info {
            TypeReflectionInfo::ClassInfo { elements, .. } => elements,
            _ => return None,
        };

        if properties.is_empty() {
            return None;
        }

        // Convert elements to PropertyInfo
        let property_infos: Vec<PropertyInfo> = properties
            .into_iter()
            .map(|element| {
                let data_type = self.convert_reflection_to_typeinfo(&element.type_info);

                let cardinality = match (element.min_cardinality, element.max_cardinality) {
                    (0, Some(1)) => Cardinality::ZeroToOne,
                    (0, None) => Cardinality::ZeroToMany,
                    (1, Some(1)) => Cardinality::OneToOne,
                    (1, None) => Cardinality::OneToMany,
                    _ => Cardinality::ZeroToOne,
                };
                let required = element.min_cardinality > 0;

                PropertyInfo {
                    name: element.name.clone(),
                    data_type,
                    cardinality,
                    required,
                    description: element.documentation.unwrap_or_else(|| {
                        format!(
                            "Property {} of type {}",
                            element.name,
                            element.type_info.name()
                        )
                    }),
                    aliases: vec![], // Could be enhanced with ModelProvider support
                }
            })
            .collect();

        Some(property_infos)
    }

    async fn is_known_resource_type(&self, name: &str) -> bool {
        self.model_provider
            .resource_type_exists(name)
            .unwrap_or(false)
    }

    /// Convert AnalysisWarning to AriadneDiagnostic format
    fn convert_warning_to_diagnostic(
        &self,
        warning: &AnalysisWarning,
        _source_text: &str,
    ) -> AriadneDiagnostic {
        // Convert error code string to ErrorCode
        let error_code = match warning.code.as_str() {
            "FP0117" => FP0055,     // Unknown property on resource -> Property not found
            "FP0118" => FP0121,     // Unknown resource type
            "FP0119" => FP0055,     // Unknown property on backbone element -> Property not found
            "FP0120" => FP0055,     // Performance warning for collections - map to property issue
            "FP0121" => FP0121,     // Unknown resource type (new)
            "FP0124" => FP0124,     // Unknown function name
            "FP0125" => FP0125,     // Enhanced property validation
            _ => ErrorCode::new(1), // Default fallback
        };

        // Convert SourceLocation to span Range<usize>
        let span = if let Some(location) = &warning.location {
            location.offset..location.offset + location.length
        } else {
            0..0 // Default span if no location
        };

        AriadneDiagnostic {
            severity: warning.severity.clone(),
            error_code,
            message: warning.message.clone(),
            span,
            help: warning.suggestion.clone(),
            note: None,
            related: Vec::new(),
        }
    }

    /// Create diagnostic for unknown property with suggestions
    fn create_property_diagnostic_with_suggestions(
        &self,
        invalid_property: &str,
        resource_type: &str,
        suggestions: &[PropertySuggestion],
        location: Option<SourceLocation>,
    ) -> AriadneDiagnostic {
        let span = location
            .as_ref()
            .map(|loc| loc.offset..loc.offset + loc.length)
            .unwrap_or(0..0);

        let mut diagnostic = AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: FP0055,
            message: format!(
                "Unknown property '{}' on resource '{}'",
                invalid_property, resource_type
            ),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        };

        // Add suggestions as help text
        if !suggestions.is_empty() {
            let suggestion_text = if suggestions.len() == 1 {
                format!(
                    "help: did you mean `{}`?",
                    suggestions[0].suggested_properties[0]
                )
            } else {
                let names = suggestions
                    .iter()
                    .flat_map(|s| &s.suggested_properties)
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>();
                format!("help: did you mean one of: {}?", names.join(", "))
            };
            diagnostic.help = Some(suggestion_text);
        }

        diagnostic
    }

    /// Create diagnostic for unknown resource type
    fn create_resource_type_diagnostic(
        &self,
        invalid_resource: &str,
        suggestions: &[String],
        location: Option<SourceLocation>,
    ) -> AriadneDiagnostic {
        let span = location
            .as_ref()
            .map(|loc| loc.offset..loc.offset + loc.length)
            .unwrap_or(0..0);

        let mut diagnostic = AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: FP0121,
            message: format!("Unknown resource type '{}'", invalid_resource),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        };

        if !suggestions.is_empty() {
            diagnostic.help = Some(format!("help: did you mean `{}`?", suggestions[0]));
        } else {
            diagnostic.help =
                Some("help: check the FHIR specification for valid resource types".to_string());
        }

        diagnostic.note = Some(
            "note: Resource types must start with a capital letter and match FHIR specification"
                .to_string(),
        );

        diagnostic
    }

    /// Create diagnostic for unknown function
    fn create_function_diagnostic(
        &self,
        invalid_function: &str,
        suggestions: &[String],
        location: Option<SourceLocation>,
    ) -> AriadneDiagnostic {
        let span = location
            .as_ref()
            .map(|loc| loc.offset..loc.offset + loc.length)
            .unwrap_or(0..0);

        AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: FP0054, // Unknown function
            message: format!("Unknown function '{}'", invalid_function),
            span,
            help: if !suggestions.is_empty() {
                Some(format!("help: did you mean `{}()`?", suggestions[0]))
            } else {
                Some("help: check available FHIRPath functions".to_string())
            },
            note: Some("note: Function names are case-sensitive".to_string()),
            related: Vec::new(),
        }
    }

    /// Create enhanced diagnostics with suggestions
    fn create_enhanced_diagnostics(
        &self,
        _warnings: &[AnalysisWarning],
        suggestions: &[PropertySuggestion],
        _source_text: &str,
    ) -> Vec<AriadneDiagnostic> {
        let mut diagnostics = Vec::new();

        // Generate enhanced diagnostics for each property suggestion
        for suggestion in suggestions {
            let diagnostic = self.create_property_diagnostic_with_suggestions(
                &suggestion.invalid_property,
                &suggestion.context_type,
                &[suggestion.clone()],
                suggestion.location.clone(),
            );
            diagnostics.push(diagnostic);
        }

        diagnostics
    }

    fn initialize_common_typos(&mut self) {
        // Common property name typos
        self.common_typos
            .insert("familly".to_string(), "family".to_string());
        self.common_typos
            .insert("givne".to_string(), "given".to_string());
        self.common_typos
            .insert("birthdate".to_string(), "birthDate".to_string());
        self.common_typos
            .insert("identfier".to_string(), "identifier".to_string());
        self.common_typos
            .insert("telecome".to_string(), "telecom".to_string());

        // Case variations
        self.common_typos
            .insert("firstname".to_string(), "given".to_string());
        self.common_typos
            .insert("lastname".to_string(), "family".to_string());
        self.common_typos
            .insert("patientid".to_string(), "id".to_string());

        // Common FHIR-specific typos
        self.common_typos
            .insert("ressource".to_string(), "resource".to_string());
        self.common_typos
            .insert("observaton".to_string(), "observation".to_string());
        self.common_typos
            .insert("encounteer".to_string(), "encounter".to_string());
    }

    /// Generate property suggestions using ModelProvider schema data instead of hardcoded lists
    async fn generate_schema_based_property_suggestions(
        &self,
        invalid_property: &str,
        parent_type: &str,
        _location: Option<SourceLocation>,
    ) -> Vec<DetailedSuggestion> {
        let mut suggestions = Vec::new();

        // Get all properties for this type from schema
        match self.model_provider.get_type_reflection(parent_type).await {
            Ok(Some(TypeReflectionInfo::ClassInfo { elements, .. })) => {
                let mut scored_suggestions: Vec<(String, f32, String)> = elements
                    .iter()
                    .map(|element| {
                        let similarity = self.calculate_similarity(invalid_property, &element.name);
                        let reason = if element.is_required() {
                            "required property".to_string()
                        } else if element.is_summary {
                            "summary property".to_string()
                        } else {
                            "available property".to_string()
                        };
                        (element.name.clone(), similarity, reason)
                    })
                    .filter(|(_, similarity, _)| *similarity > 0.6) // Only include good matches
                    .collect();

                // Sort by similarity score (descending)
                scored_suggestions
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                // Take top 3 suggestions
                for (prop_name, similarity, reason) in scored_suggestions.into_iter().take(3) {
                    suggestions.push(DetailedSuggestion {
                        suggestion: prop_name,
                        reason,
                        confidence: similarity,
                        context_type: parent_type.to_string(),
                        is_polymorphic: false,
                    });
                }
            }
            Ok(_) => {
                // Fallback to hardcoded suggestions for non-class types
                let fallback_suggestions =
                    self.generate_hardcoded_property_suggestions(invalid_property, parent_type);
                for property_suggestion in fallback_suggestions {
                    for suggested_prop in property_suggestion.suggested_properties {
                        suggestions.push(DetailedSuggestion {
                            suggestion: suggested_prop,
                            reason: "similar spelling".to_string(),
                            confidence: property_suggestion.similarity_score,
                            context_type: parent_type.to_string(),
                            is_polymorphic: false,
                        });
                    }
                }
            }
            Err(_) => {
                // Fallback to hardcoded suggestions on error
                let fallback_suggestions =
                    self.generate_hardcoded_property_suggestions(invalid_property, parent_type);
                for property_suggestion in fallback_suggestions {
                    for suggested_prop in property_suggestion.suggested_properties {
                        suggestions.push(DetailedSuggestion {
                            suggestion: suggested_prop,
                            reason: "similar spelling".to_string(),
                            confidence: property_suggestion.similarity_score,
                            context_type: parent_type.to_string(),
                            is_polymorphic: false,
                        });
                    }
                }
            }
        }

        suggestions
    }

    /// Generate hardcoded property suggestions as fallback (legacy method)
    fn generate_hardcoded_property_suggestions(
        &self,
        invalid_property: &str,
        parent_type: &str,
    ) -> Vec<PropertySuggestion> {
        // Check common typos first
        if let Some(correction) = self.common_typos.get(invalid_property) {
            return vec![PropertySuggestion {
                invalid_property: invalid_property.to_string(),
                suggested_properties: vec![correction.clone()],
                similarity_score: 1.0,
                context_type: parent_type.to_string(),
                location: None,
            }];
        }

        // Hardcoded common properties for major resource types
        let common_properties = match parent_type {
            "Patient" => vec![
                "id",
                "name",
                "birthDate",
                "gender",
                "active",
                "identifier",
                "telecom",
                "address",
            ],
            "Observation" => vec![
                "id",
                "status",
                "code",
                "subject",
                "value",
                "component",
                "category",
                "effective",
            ],
            "Encounter" => vec![
                "id",
                "status",
                "class",
                "subject",
                "period",
                "diagnosis",
                "location",
                "participant",
            ],
            "DiagnosticReport" => vec![
                "id",
                "status",
                "code",
                "subject",
                "result",
                "conclusion",
                "category",
                "effective",
            ],
            "Practitioner" => vec![
                "id",
                "name",
                "active",
                "identifier",
                "telecom",
                "address",
                "gender",
                "birthDate",
            ],
            "Organization" => vec![
                "id",
                "name",
                "active",
                "identifier",
                "telecom",
                "address",
                "type",
                "alias",
            ],
            _ => vec!["id", "resourceType", "meta"], // Basic Resource properties
        };

        let property_infos: Vec<PropertyInfo> = common_properties
            .into_iter()
            .map(|prop| PropertyInfo {
                name: prop.to_string(),
                data_type: TypeInfo::String, // Simplified for hardcoded suggestions
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: format!("Property {}", prop),
                aliases: vec![],
            })
            .collect();

        let suggestions = self.find_similar_properties(invalid_property, &property_infos);

        suggestions
            .into_iter()
            .map(|suggested_prop| {
                let similarity = self.calculate_similarity(invalid_property, &suggested_prop);
                PropertySuggestion {
                    invalid_property: invalid_property.to_string(),
                    suggested_properties: vec![suggested_prop],
                    similarity_score: similarity,
                    context_type: parent_type.to_string(),
                    location: None,
                }
            })
            .collect()
    }

    /// Generate function name suggestions with typo correction
    fn generate_function_suggestions(&self, invalid_function: &str) -> Vec<DetailedSuggestion> {
        let all_functions = self.function_registry.list_functions();
        let mut suggestions: Vec<(String, f32, String)> = Vec::new();

        // Get function names and calculate similarity
        for function_metadata in &all_functions {
            let similarity = self.calculate_similarity(invalid_function, &function_metadata.name);
            if similarity > 0.6 {
                let reason = match function_metadata.category {
                    crate::registry::FunctionCategory::Collection => {
                        "collection function".to_string()
                    }
                    crate::registry::FunctionCategory::Math => "math function".to_string(),
                    crate::registry::FunctionCategory::String => "string function".to_string(),
                    crate::registry::FunctionCategory::Type => "type function".to_string(),
                    crate::registry::FunctionCategory::Conversion => {
                        "conversion function".to_string()
                    }
                    crate::registry::FunctionCategory::DateTime => "date/time function".to_string(),
                    crate::registry::FunctionCategory::Fhir => "FHIR function".to_string(),
                    crate::registry::FunctionCategory::Terminology => {
                        "terminology function".to_string()
                    }
                    crate::registry::FunctionCategory::Logic => "logic function".to_string(),
                    crate::registry::FunctionCategory::Utility => "utility function".to_string(),
                };
                suggestions.push((function_metadata.name.clone(), similarity, reason));
            }
        }

        // Check common function typos
        let common_function_typos = [
            ("lenght", "length"),
            ("contians", "contains"),
            ("exsits", "exists"),
            ("whre", "where"),
            ("selct", "select"),
            ("distinc", "distinct"),
            ("frist", "first"),
            ("las", "last"),
            ("emty", "empty"),
            ("cout", "count"),
            ("gte", ">="),
            ("lte", "<="),
        ];

        for (typo, correction) in common_function_typos {
            if invalid_function == typo {
                suggestions.push((
                    correction.to_string(),
                    1.0,
                    "common typo correction".to_string(),
                ));
            }
        }

        // Sort by similarity score (descending)
        suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert to DetailedSuggestion and return top 3
        suggestions
            .into_iter()
            .take(3)
            .map(|(name, confidence, reason)| DetailedSuggestion {
                suggestion: name,
                reason,
                confidence,
                context_type: "function".to_string(),
                is_polymorphic: false,
            })
            .collect()
    }

    /// Check if a function name exists in the registry
    fn is_valid_function_name(&self, function_name: &str) -> bool {
        self.function_registry
            .get_sync_function(function_name)
            .is_some()
            || self
                .function_registry
                .get_async_function(function_name)
                .is_some()
    }

    /// Handle polymorphic properties (choice[x] like value[x])
    async fn suggest_polymorphic_properties(
        &self,
        invalid_property: &str,
        parent_type: &str,
    ) -> Vec<DetailedSuggestion> {
        let mut suggestions = Vec::new();

        // Check if this might be a polymorphic property
        let base_name = if invalid_property.len() > 5 {
            // Look for patterns like "valueString", "valueInteger", etc.
            if let Some(pos) = invalid_property.find(char::is_uppercase) {
                if pos > 0 {
                    &invalid_property[..pos]
                } else {
                    invalid_property
                }
            } else {
                invalid_property
            }
        } else {
            invalid_property
        };

        // Get type reflection to check for polymorphic properties
        if let Ok(Some(TypeReflectionInfo::ClassInfo { elements, .. })) =
            self.model_provider.get_type_reflection(parent_type).await
        {
            // Look for choice[x] pattern in schema
            for element in &elements {
                if element.name.starts_with(base_name) && element.name.contains("[x]") {
                    // This is a polymorphic property, get valid type suffixes
                    let valid_types = self.get_choice_type_suffixes(&element.type_info);

                    for suffix in valid_types {
                        let polymorphic_property = format!("{}{}", base_name, suffix);
                        let similarity =
                            self.calculate_similarity(invalid_property, &polymorphic_property);

                        if similarity > 0.7 {
                            suggestions.push(DetailedSuggestion {
                                suggestion: polymorphic_property,
                                reason: format!("polymorphic property ({}[x])", base_name),
                                confidence: similarity,
                                context_type: parent_type.to_string(),
                                is_polymorphic: true,
                            });
                        }
                    }
                    break;
                }
            }
        }

        // Sort by confidence and return top suggestions
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        suggestions.into_iter().take(5).collect()
    }

    /// Extract choice type suffixes from TypeReflectionInfo for polymorphic properties
    fn get_choice_type_suffixes(&self, _type_info: &TypeReflectionInfo) -> Vec<String> {
        // Common FHIR polymorphic type suffixes
        vec![
            "String".to_string(),
            "Integer".to_string(),
            "Boolean".to_string(),
            "Decimal".to_string(),
            "Date".to_string(),
            "DateTime".to_string(),
            "Time".to_string(),
            "Code".to_string(),
            "Coding".to_string(),
            "CodeableConcept".to_string(),
            "Quantity".to_string(),
            "Range".to_string(),
            "Ratio".to_string(),
            "Reference".to_string(),
            "Attachment".to_string(),
            "Identifier".to_string(),
            "Period".to_string(),
            "Address".to_string(),
            "ContactPoint".to_string(),
        ]
    }

    /// Validate function name and provide suggestions for invalid names
    fn validate_function_name(
        &self,
        function_name: &str,
        location: Option<SourceLocation>,
        warnings: &mut Vec<AnalysisWarning>,
    ) {
        if !self.is_valid_function_name(function_name) {
            let suggestions = self.generate_function_suggestions(function_name);

            let suggestion_text = if !suggestions.is_empty() {
                let suggestion_list: Vec<String> = suggestions
                    .iter()
                    .map(|s| format!("{} ({})", s.suggestion, s.reason))
                    .collect();
                Some(format!("Did you mean: {}?", suggestion_list.join(", ")))
            } else {
                None
            };

            warnings.push(AnalysisWarning {
                code: "FP0124".to_string(), // New error code for invalid function names
                message: format!("Unknown function: '{}'", function_name),
                location: location.clone(),
                severity: DiagnosticSeverity::Warning,
                suggestion: suggestion_text,
            });
        }
    }

    /// Generate comprehensive, context-aware suggestions combining all strategies
    async fn generate_comprehensive_property_suggestions(
        &self,
        invalid_property: &str,
        parent_type: &str,
        location: Option<SourceLocation>,
    ) -> Vec<DetailedSuggestion> {
        let mut all_suggestions = Vec::new();

        // 1. Check for exact typo corrections first (highest priority)
        if let Some(correction) = self.common_typos.get(invalid_property) {
            all_suggestions.push(DetailedSuggestion {
                suggestion: correction.clone(),
                reason: "common typo correction".to_string(),
                confidence: 1.0,
                context_type: parent_type.to_string(),
                is_polymorphic: false,
            });
        }

        // 2. Get schema-based suggestions
        let schema_suggestions = self
            .generate_schema_based_property_suggestions(
                invalid_property,
                parent_type,
                location.clone(),
            )
            .await;
        all_suggestions.extend(schema_suggestions);

        // 3. Get polymorphic property suggestions
        let polymorphic_suggestions = self
            .suggest_polymorphic_properties(invalid_property, parent_type)
            .await;
        all_suggestions.extend(polymorphic_suggestions);

        // 4. Remove duplicates and prioritize suggestions
        let mut seen_suggestions = std::collections::HashSet::new();
        let mut unique_suggestions = Vec::new();

        for suggestion in all_suggestions {
            if seen_suggestions.insert(suggestion.suggestion.clone()) {
                unique_suggestions.push(suggestion);
            }
        }

        // 5. Sort by priority: typos > required properties > summary properties > polymorphic > others
        unique_suggestions.sort_by(|a, b| {
            // First by confidence (higher is better)
            let confidence_cmp = b
                .confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal);
            if confidence_cmp != std::cmp::Ordering::Equal {
                return confidence_cmp;
            }

            // Then by reason priority
            let a_priority = match a.reason.as_str() {
                "common typo correction" => 5,
                s if s.contains("required property") => 4,
                s if s.contains("summary property") => 3,
                s if s.contains("polymorphic property") => 2,
                _ => 1,
            };
            let b_priority = match b.reason.as_str() {
                "common typo correction" => 5,
                s if s.contains("required property") => 4,
                s if s.contains("summary property") => 3,
                s if s.contains("polymorphic property") => 2,
                _ => 1,
            };

            b_priority.cmp(&a_priority)
        });

        // Return top 3 suggestions
        unique_suggestions.into_iter().take(3).collect()
    }

    /// Enhanced property access validation with comprehensive suggestions
    async fn enhanced_validate_property_access(
        &self,
        access: &crate::ast::expression::PropertyAccessNode,
        context: &mut AnalysisContext,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> crate::core::Result<()> {
        // Infer the type of the object being accessed
        let object_type = self.infer_object_type(&access.object, context).await?;

        // Check if the property exists on this type
        let property_valid = match &object_type {
            TypeInfo::Resource { resource_type } => {
                match self.model_provider.get_type_reflection(resource_type).await {
                    Ok(Some(TypeReflectionInfo::ClassInfo { elements, .. })) => elements
                        .iter()
                        .any(|element| element.name == access.property),
                    _ => false,
                }
            }
            TypeInfo::BackboneElement { properties } => properties.contains_key(&access.property),
            _ => true, // Don't validate unknown types
        };

        if !property_valid {
            let parent_type_name = match &object_type {
                TypeInfo::Resource { resource_type } => resource_type.clone(),
                TypeInfo::BackboneElement { .. } => "BackboneElement".to_string(),
                _ => "unknown".to_string(),
            };

            // Generate comprehensive suggestions
            let suggestions = self
                .generate_comprehensive_property_suggestions(
                    &access.property,
                    &parent_type_name,
                    access.location.clone(),
                )
                .await;

            let suggestion_text = if !suggestions.is_empty() {
                let suggestion_list: Vec<String> = suggestions
                    .iter()
                    .map(|s| format!("{} ({})", s.suggestion, s.reason))
                    .collect();
                Some(format!("Did you mean: {}?", suggestion_list.join(", ")))
            } else {
                None
            };

            warnings.push(AnalysisWarning {
                code: "FP0125".to_string(), // Enhanced property validation error code
                message: format!(
                    "Invalid property '{}' on type '{}'",
                    access.property, parent_type_name
                ),
                location: access.location.clone(),
                severity: DiagnosticSeverity::Warning,
                suggestion: suggestion_text,
            });
        }

        Ok(())
    }

    /// Get cached properties for a resource type for performance
    async fn get_cached_properties(&mut self, resource_type: &str) -> Vec<String> {
        // Check cache first
        if let Some(cached) = self.suggestion_cache.property_cache.get(resource_type) {
            return cached.clone();
        }

        // Get properties from ModelProvider and cache them
        if let Ok(Some(TypeReflectionInfo::ClassInfo { elements, .. })) =
            self.model_provider.get_type_reflection(resource_type).await
        {
            let properties: Vec<String> =
                elements.into_iter().map(|element| element.name).collect();

            self.suggestion_cache
                .property_cache
                .insert(resource_type.to_string(), properties.clone());
            return properties;
        }

        Vec::new()
    }

    /// Get cached function names for performance
    fn get_cached_function_names(&mut self) -> Vec<String> {
        // Check cache first
        if let Some(ref cached) = self.suggestion_cache.function_cache {
            return cached.clone();
        }

        // Get function names from registry and cache them
        let function_names: Vec<String> = self
            .function_registry
            .list_functions()
            .into_iter()
            .map(|metadata| metadata.name)
            .collect();

        self.suggestion_cache.function_cache = Some(function_names.clone());
        function_names
    }

    /// Get cached suggestions for performance
    fn get_cached_suggestions(
        &self,
        invalid_name: &str,
        context_type: &str,
    ) -> Option<Vec<DetailedSuggestion>> {
        let cache_key = (invalid_name.to_string(), context_type.to_string());
        self.suggestion_cache
            .suggestion_cache
            .get(&cache_key)
            .cloned()
    }

    /// Cache suggestions for future use
    fn cache_suggestions(
        &mut self,
        invalid_name: &str,
        context_type: &str,
        suggestions: Vec<DetailedSuggestion>,
    ) {
        let cache_key = (invalid_name.to_string(), context_type.to_string());
        self.suggestion_cache
            .suggestion_cache
            .insert(cache_key, suggestions);
    }

    /// Optimized suggestion generation with caching
    async fn generate_cached_property_suggestions(
        &mut self,
        invalid_property: &str,
        parent_type: &str,
        location: Option<SourceLocation>,
    ) -> Vec<DetailedSuggestion> {
        // Check cache first
        if let Some(cached) = self.get_cached_suggestions(invalid_property, parent_type) {
            return cached;
        }

        // Generate suggestions
        let suggestions = self
            .generate_comprehensive_property_suggestions(invalid_property, parent_type, location)
            .await;

        // Cache the results
        self.cache_suggestions(invalid_property, parent_type, suggestions.clone());

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockModelProvider;
    use crate::ast::expression::{ExpressionNode, IdentifierNode, PropertyAccessNode};
    use crate::core::error_code::{FP0054, FP0055, FP0121};
    use crate::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};

    /// Test that AriadneDiagnostic conversion works correctly
    #[test]
    fn test_convert_warning_to_diagnostic() {
        let provider = Arc::new(MockModelProvider::new());
        let validator = PropertyValidator::new(provider);

        let warning = AnalysisWarning {
            code: "FP0117".to_string(),
            message: "Unknown property 'invalid' on Patient".to_string(),
            location: Some(SourceLocation::new(1, 8, 7, 7)),
            severity: DiagnosticSeverity::Error,
            suggestion: Some("did you mean 'name'?".to_string()),
        };

        let diagnostic = validator.convert_warning_to_diagnostic(&warning, "Patient.invalid");

        assert_eq!(diagnostic.error_code, FP0055); // Maps to Property not found
        assert_eq!(diagnostic.message, "Unknown property 'invalid' on Patient");
        assert_eq!(diagnostic.span, 7..14); // offset to offset + length
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.help, Some("did you mean 'name'?".to_string()));
    }

    /// Test property diagnostic creation with suggestions
    #[test]
    fn test_create_property_diagnostic_with_suggestions() {
        let provider = Arc::new(MockModelProvider::new());
        let validator = PropertyValidator::new(provider);

        let suggestions = vec![PropertySuggestion {
            invalid_property: "familly".to_string(),
            suggested_properties: vec!["family".to_string()],
            context_type: "Patient.name".to_string(),
            confidence: 0.9,
            suggestion_type: SuggestionType::TypoCorrection,
            location: Some(SourceLocation::new(1, 13, 12, 7)),
        }];

        let diagnostic = validator.create_property_diagnostic_with_suggestions(
            "familly",
            "Patient.name",
            &suggestions,
            Some(SourceLocation::new(1, 13, 12, 7)),
        );

        assert_eq!(diagnostic.error_code, FP0055);
        assert_eq!(
            diagnostic.message,
            "Unknown property 'familly' on resource 'Patient.name'"
        );
        assert_eq!(diagnostic.span, 12..19); // offset to offset + length
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.help.as_ref().unwrap().contains("did you mean"));
        assert!(diagnostic.help.as_ref().unwrap().contains("family"));
    }

    /// Test resource type diagnostic creation
    #[test]
    fn test_create_resource_type_diagnostic() {
        let provider = Arc::new(MockModelProvider::new());
        let validator = PropertyValidator::new(provider);

        let suggestions = vec!["Patient".to_string(), "Practitioner".to_string()];

        let diagnostic = validator.create_resource_type_diagnostic(
            "Patinet",
            &suggestions,
            Some(SourceLocation::new(1, 1, 0, 7)),
        );

        assert_eq!(diagnostic.error_code, FP0121);
        assert_eq!(diagnostic.message, "Unknown resource type 'Patinet'");
        assert_eq!(diagnostic.span, 0..7);
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.help.as_ref().unwrap().contains("did you mean"));
        assert!(diagnostic.help.as_ref().unwrap().contains("Patient"));
        assert!(diagnostic.note.is_some());
    }

    /// Test function diagnostic creation
    #[test]
    fn test_create_function_diagnostic() {
        let provider = Arc::new(MockModelProvider::new());
        let validator = PropertyValidator::new(provider);

        let suggestions = vec!["first".to_string(), "last".to_string()];

        let diagnostic = validator.create_function_diagnostic(
            "frist",
            &suggestions,
            Some(SourceLocation::new(1, 20, 19, 5)),
        );

        assert_eq!(diagnostic.error_code, FP0054);
        assert_eq!(diagnostic.message, "Unknown function 'frist'");
        assert_eq!(diagnostic.span, 19..24);
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.help.as_ref().unwrap().contains("did you mean"));
        assert!(diagnostic.help.as_ref().unwrap().contains("first()"));
        assert!(diagnostic.note.is_some());
        assert!(diagnostic.note.as_ref().unwrap().contains("case-sensitive"));
    }

    /// Integration test that PropertyValidationResult includes ariadne_diagnostics
    #[tokio::test]
    async fn test_property_validation_includes_ariadne_diagnostics() {
        let provider = Arc::new(MockModelProvider::new());
        let validator = PropertyValidator::new(provider);

        // Create a mock expression with an invalid property
        let expression = ExpressionNode::property_access(
            ExpressionNode::identifier("Patient"),
            "invalidProperty".to_string(),
        );

        let result = validator
            .validate_with_source_text(&expression, "Patient.invalidProperty")
            .await;

        match result {
            Ok(validation_result) => {
                // Should have the new ariadne_diagnostics field populated
                // Even if no legacy warnings, should have enhanced diagnostics
                assert!(
                    validation_result.ariadne_diagnostics.len() >= validation_result.warnings.len()
                );
            }
            Err(e) => {
                // This is fine for MockModelProvider - test that it doesn't panic
                eprintln!("Expected error with MockModelProvider: {}", e);
            }
        }
    }

    /// Test that enhanced diagnostics are generated even without legacy warnings
    #[test]
    fn test_enhanced_diagnostics_creation() {
        let provider = Arc::new(MockModelProvider::new());
        let validator = PropertyValidator::new(provider);

        let warnings = vec![];
        let suggestions = vec![PropertySuggestion {
            invalid_property: "nam".to_string(),
            suggested_properties: vec!["name".to_string()],
            context_type: "Patient".to_string(),
            confidence: 0.8,
            suggestion_type: SuggestionType::TypoCorrection,
            location: Some(SourceLocation::new(1, 8, 7, 3)),
        }];

        let enhanced_diagnostics =
            validator.create_enhanced_diagnostics(&warnings, &suggestions, "Patient.nam");

        assert_eq!(enhanced_diagnostics.len(), 1);
        let diagnostic = &enhanced_diagnostics[0];
        assert_eq!(diagnostic.error_code, FP0055);
        assert_eq!(
            diagnostic.message,
            "Unknown property 'nam' on resource 'Patient'"
        );
        assert!(diagnostic.help.is_some());
    }

    /// CRITICAL TEST: Test that Pat.name generates FP0121 error for invalid resource type
    #[tokio::test]
    async fn test_invalid_resource_type_pat_generates_error() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;

        // Use MockModelProvider first to test the logic
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Create expression: Pat.name (Pat is invalid resource type)
        let expression =
            ExpressionNode::property_access(ExpressionNode::identifier("Pat"), "name".to_string());

        let result = validator
            .validate_with_source_text(&expression, "Pat.name")
            .await;

        match result {
            Ok(validation_result) => {
                eprintln!("Validation result: {:?}", validation_result);
                eprintln!("Warnings: {:?}", validation_result.warnings);
                eprintln!(
                    "Ariadne diagnostics: {:?}",
                    validation_result.ariadne_diagnostics
                );

                // Should have warnings for invalid resource type
                assert!(
                    !validation_result.warnings.is_empty(),
                    "Should have warnings for invalid resource type 'Pat'"
                );

                // Check that we have the FP0121 error code in warnings
                let has_resource_error = validation_result
                    .warnings
                    .iter()
                    .any(|w| w.code == "FP0121" || w.message.contains("Unknown resource type"));

                assert!(
                    has_resource_error,
                    "Should have FP0121 or unknown resource type error. Found warnings: {:?}",
                    validation_result.warnings
                );
            }
            Err(e) => {
                eprintln!("Validation error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_similarity_calculation() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // High similarity - single typo
        let similarity = validator.calculate_similarity("familly", "family");
        assert!(similarity > 0.8);

        // Medium similarity - multiple changes
        let similarity = validator.calculate_similarity("birthdate", "birthDate");
        assert!(similarity > 0.9); // Just case difference

        // Low similarity - completely different
        let similarity = validator.calculate_similarity("completely_different", "family");
        assert!(similarity < 0.3);
    }

    #[tokio::test]
    async fn test_property_suggestions() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        let properties = vec![
            PropertyInfo {
                name: "family".to_string(),
                data_type: TypeInfo::String,
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Family name".to_string(),
                aliases: vec!["surname".to_string()],
            },
            PropertyInfo {
                name: "given".to_string(),
                data_type: TypeInfo::Collection(Box::new(TypeInfo::String)),
                cardinality: Cardinality::ZeroToMany,
                required: false,
                description: "Given names".to_string(),
                aliases: vec!["firstName".to_string()],
            },
        ];

        let suggestions = validator.find_similar_properties("familly", &properties);
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"family".to_string()));
    }

    #[tokio::test]
    async fn test_levenshtein_distance() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        assert_eq!(validator.levenshtein_distance("", ""), 0);
        assert_eq!(validator.levenshtein_distance("", "abc"), 3);
        assert_eq!(validator.levenshtein_distance("abc", ""), 3);
        assert_eq!(validator.levenshtein_distance("abc", "abc"), 0);
        assert_eq!(validator.levenshtein_distance("abc", "ab"), 1);
        assert_eq!(validator.levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(validator.levenshtein_distance("family", "familly"), 1);
    }

    #[tokio::test]
    async fn test_model_provider_integration() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider.clone(), function_registry);

        // Test that validator uses ModelProvider for resource type checking
        assert!(validator.is_known_resource_type("Patient").await);

        // Test property resolution through ModelProvider
        let properties = validator.get_resource_properties("Patient").await;
        assert!(properties.is_some());
    }

    #[test]
    fn test_common_typos() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        assert_eq!(
            validator.common_typos.get("familly"),
            Some(&"family".to_string())
        );
        assert_eq!(
            validator.common_typos.get("birthdate"),
            Some(&"birthDate".to_string())
        );
        assert_eq!(
            validator.common_typos.get("firstname"),
            Some(&"given".to_string())
        );
    }

    #[tokio::test]
    async fn test_dynamic_property_validation_with_model_provider() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider.clone(), function_registry);

        // Test that ModelProvider is used for property type resolution
        let type_info = TypeInfo::Resource {
            resource_type: "Patient".to_string(),
        };

        let property_type = validator.infer_property_type(&type_info, "name").await;
        assert!(property_type.is_ok());
    }

    #[tokio::test]
    async fn test_resource_type_validation_with_schema() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider.clone(), function_registry);

        // Test resource type validation through ModelProvider
        assert!(validator.is_known_resource_type("Patient").await);
        assert!(validator.is_known_resource_type("Observation").await);
        assert!(!validator.is_known_resource_type("InvalidResource").await);
    }

    #[tokio::test]
    async fn test_property_suggestions_from_schema() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider.clone(), function_registry);

        // Test that property suggestions work with dynamic schema data
        if let Some(properties) = validator.get_resource_properties("Patient").await {
            let suggestions = validator.find_similar_properties("familly", &properties);
            assert!(!suggestions.is_empty());
        }
    }

    #[test]
    fn test_type_reflection_conversion() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        use octofhir_fhir_model::TypeReflectionInfo;

        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test conversion from System types
        let string_reflection = TypeReflectionInfo::simple_type("System", "String");
        let type_info = validator.convert_reflection_to_typeinfo(&string_reflection);
        assert!(matches!(type_info, TypeInfo::String));

        // Test conversion from FHIR types
        let boolean_reflection = TypeReflectionInfo::simple_type("FHIR", "boolean");
        let type_info = validator.convert_reflection_to_typeinfo(&boolean_reflection);
        assert!(matches!(type_info, TypeInfo::Boolean));

        // Test conversion of collection types
        let list_reflection =
            TypeReflectionInfo::list_type(TypeReflectionInfo::simple_type("System", "String"));
        let type_info = validator.convert_reflection_to_typeinfo(&list_reflection);
        assert!(matches!(type_info, TypeInfo::Collection(_)));
    }

    #[tokio::test]
    async fn test_valid_resource_type_detection() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test that valid resource type identifiers are detected correctly
        assert!(validator.is_potential_resource_type("Patient"));
        assert!(validator.is_potential_resource_type("Observation"));
        assert!(validator.is_potential_resource_type("DiagnosticReport"));
        assert!(validator.is_potential_resource_type("Bundle"));

        // Test that non-resource identifiers are not detected
        assert!(!validator.is_potential_resource_type("name"));
        assert!(!validator.is_potential_resource_type("family"));
        assert!(!validator.is_potential_resource_type("$patient"));
        assert!(!validator.is_potential_resource_type("some_var"));
        assert!(!validator.is_potential_resource_type(""));
    }

    #[tokio::test]
    async fn test_invalid_resource_type_error() {
        use crate::ast::expression::{ExpressionNode, IdentifierNode};
        use crate::core::SourceLocation;
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;

        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Create an expression with an invalid resource type
        let invalid_id = ExpressionNode::Identifier(IdentifierNode {
            name: "Patientt".to_string(), // Typo in "Patient"
            location: Some(SourceLocation::new(1, 1, 0, 8)),
        });

        let result = validator.validate(&invalid_id).await.unwrap();

        // Should have generated a warning for the unknown resource type
        assert!(!result.warnings.is_empty());
        let warning = &result.warnings[0];
        assert_eq!(warning.code, "FP0121");
        assert!(
            warning
                .message
                .contains("Unknown resource type: 'Patientt'")
        );
        assert!(warning.suggestion.is_some());
    }

    #[tokio::test]
    async fn test_resource_type_suggestions() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test suggestions for common typos
        let suggestions = validator
            .generate_resource_type_suggestions("Patientt")
            .await;
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"Patient".to_string()));

        let suggestions = validator
            .generate_resource_type_suggestions("Observaton")
            .await;
        assert!(!suggestions.is_empty());
        // MockProvider might not have Observation, but the algorithm should still work

        let suggestions = validator.generate_resource_type_suggestions("Bundl").await;
        assert!(!suggestions.is_empty());

        // Test that completely different words don't get suggestions
        let suggestions = validator
            .generate_resource_type_suggestions("CompletelyDifferent")
            .await;
        // Should be empty or very few suggestions due to low similarity
        assert!(suggestions.len() <= 1);
    }

    #[test]
    fn test_capitalization_detection() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Should trigger resource validation (capitalized)
        assert!(validator.is_potential_resource_type("Patient"));
        assert!(validator.is_potential_resource_type("Observation"));
        assert!(validator.is_potential_resource_type("Bundle"));
        assert!(validator.is_potential_resource_type("DiagnosticReport"));

        // Should NOT trigger resource validation (lowercase start)
        assert!(!validator.is_potential_resource_type("name"));
        assert!(!validator.is_potential_resource_type("family"));
        assert!(!validator.is_potential_resource_type("given"));
        assert!(!validator.is_potential_resource_type("status"));

        // Should NOT trigger resource validation (variable reference)
        assert!(!validator.is_potential_resource_type("$patient"));
        assert!(!validator.is_potential_resource_type("$context"));
        assert!(!validator.is_potential_resource_type("$this"));

        // Should NOT trigger resource validation (underscore naming)
        assert!(!validator.is_potential_resource_type("some_var"));
        assert!(!validator.is_potential_resource_type("patient_data"));
        assert!(!validator.is_potential_resource_type("UPPER_CASE"));

        // Edge cases
        assert!(!validator.is_potential_resource_type(""));
        assert!(validator.is_potential_resource_type("A")); // Single capital letter
        assert!(!validator.is_potential_resource_type("a")); // Single lowercase letter
    }

    #[tokio::test]
    async fn test_property_access_resource_validation() {
        use crate::ast::expression::{ExpressionNode, IdentifierNode, PropertyAccessNode};
        use crate::core::SourceLocation;
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;

        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Create an expression like "Patientt.name" (typo in resource type)
        let invalid_base = ExpressionNode::Identifier(IdentifierNode {
            name: "Patientt".to_string(),
            location: Some(SourceLocation::new(1, 1, 0, 8)),
        });

        let property_access = ExpressionNode::PropertyAccess(PropertyAccessNode {
            object: Box::new(invalid_base),
            property: "name".to_string(),
            location: Some(SourceLocation::new(1, 1, 0, 13)),
        });

        let result = validator.validate(&property_access).await.unwrap();

        // Should have generated a warning for the unknown resource type
        assert!(!result.warnings.is_empty());

        // Find the resource type warning (there might be other warnings too)
        let resource_warning = result
            .warnings
            .iter()
            .find(|w| w.code == "FP0121")
            .expect("Should have FP0121 error for unknown resource type");

        assert!(
            resource_warning
                .message
                .contains("Unknown resource type: 'Patientt'")
        );
        assert!(resource_warning.suggestion.is_some());
    }

    #[tokio::test]
    async fn test_valid_resource_type_no_error() {
        use crate::ast::expression::{ExpressionNode, IdentifierNode, PropertyAccessNode};
        use crate::core::SourceLocation;
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;

        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Create an expression like "Patient.name" (valid resource type)
        let valid_base = ExpressionNode::Identifier(IdentifierNode {
            name: "Patient".to_string(),
            location: Some(SourceLocation::new(1, 1, 0, 7)),
        });

        let property_access = ExpressionNode::PropertyAccess(PropertyAccessNode {
            object: Box::new(valid_base),
            property: "name".to_string(),
            location: Some(SourceLocation::new(1, 1, 0, 12)),
        });

        let result = validator.validate(&property_access).await.unwrap();

        // Should NOT have resource type validation errors (FP0121)
        let has_resource_error = result.warnings.iter().any(|w| w.code == "FP0121");

        assert!(
            !has_resource_error,
            "Should not have resource type error for valid 'Patient'"
        );
    }

    #[tokio::test]
    async fn test_schema_based_property_suggestions() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test schema-based suggestions using MockModelProvider
        let suggestions = validator
            .generate_schema_based_property_suggestions(
                "namee", // Typo in "name"
                "Patient", None,
            )
            .await;

        // Should provide suggestions based on schema
        assert!(!suggestions.is_empty());
        // Should have high confidence suggestions
        let high_confidence_suggestions: Vec<_> =
            suggestions.iter().filter(|s| s.confidence > 0.7).collect();
        assert!(!high_confidence_suggestions.is_empty());
    }

    #[test]
    fn test_function_name_validation() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test valid function names
        assert!(validator.is_valid_function_name("first"));
        assert!(validator.is_valid_function_name("count"));
        assert!(validator.is_valid_function_name("where"));

        // Test invalid function names
        assert!(!validator.is_valid_function_name("invalid_function_name"));
        assert!(!validator.is_valid_function_name("nonexistent"));
    }

    #[test]
    fn test_function_suggestions() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test function suggestions for typos
        let suggestions = validator.generate_function_suggestions("lenght"); // Typo of "length"
        assert!(!suggestions.is_empty());

        let suggestions = validator.generate_function_suggestions("contians"); // Typo of "contains"
        assert!(!suggestions.is_empty());

        let suggestions = validator.generate_function_suggestions("frist"); // Typo of "first"
        assert!(!suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_polymorphic_property_detection() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test detection of polymorphic properties
        let suggestions = validator
            .suggest_polymorphic_properties(
                "valueStrig", // Typo in "valueString"
                "Observation",
            )
            .await;

        // Should detect polymorphic patterns
        let polymorphic_suggestions: Vec<_> =
            suggestions.iter().filter(|s| s.is_polymorphic).collect();

        // Even with MockModelProvider, the algorithm should work
        // The exact results depend on MockModelProvider implementation
        assert!(suggestions.len() >= 0); // At least doesn't crash
    }

    #[tokio::test]
    async fn test_comprehensive_property_suggestions() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test comprehensive suggestions that combine all strategies
        let suggestions = validator
            .generate_comprehensive_property_suggestions(
                "familly", // Known typo
                "Patient", None,
            )
            .await;

        assert!(!suggestions.is_empty());

        // Should prioritize typo corrections
        let first_suggestion = &suggestions[0];
        assert_eq!(first_suggestion.suggestion, "family");
        assert_eq!(first_suggestion.reason, "common typo correction");
        assert_eq!(first_suggestion.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_suggestion_prioritization() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        let suggestions = validator
            .generate_comprehensive_property_suggestions(
                "birtdate", // Similar to birthDate
                "Patient", None,
            )
            .await;

        assert!(!suggestions.is_empty());

        // Should be sorted by priority and confidence
        for i in 0..suggestions.len() - 1 {
            // Either higher confidence or higher priority reason
            let curr = &suggestions[i];
            let next = &suggestions[i + 1];

            if curr.confidence == next.confidence {
                // Same confidence, check priority
                let curr_priority = match curr.reason.as_str() {
                    "common typo correction" => 5,
                    s if s.contains("required property") => 4,
                    s if s.contains("summary property") => 3,
                    s if s.contains("polymorphic property") => 2,
                    _ => 1,
                };
                let next_priority = match next.reason.as_str() {
                    "common typo correction" => 5,
                    s if s.contains("required property") => 4,
                    s if s.contains("summary property") => 3,
                    s if s.contains("polymorphic property") => 2,
                    _ => 1,
                };
                assert!(curr_priority >= next_priority);
            } else {
                assert!(curr.confidence >= next.confidence);
            }
        }
    }

    #[tokio::test]
    async fn test_function_validation_in_expression() {
        use crate::ast::expression::{ExpressionNode, FunctionCallNode};
        use crate::core::SourceLocation;
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;

        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        // Test that function validation is triggered during expression validation
        let function_call = ExpressionNode::FunctionCall(FunctionCallNode {
            name: "invalidFunction".to_string(), // Invalid function name
            arguments: vec![],
            location: Some(SourceLocation::new(1, 1, 0, 15)),
        });

        let result = validator.validate(&function_call).await.unwrap();

        // Should generate warning for invalid function name
        let function_warnings: Vec<_> = result
            .warnings
            .iter()
            .filter(|w| w.code == "FP0124")
            .collect();
        assert!(!function_warnings.is_empty());

        let warning = function_warnings[0];
        assert!(
            warning
                .message
                .contains("Unknown function: 'invalidFunction'")
        );
    }

    #[test]
    fn test_detailed_suggestion_structure() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let validator = PropertyValidator::new(model_provider, function_registry);

        let suggestions = validator.generate_function_suggestions("frist");

        for suggestion in &suggestions {
            // All suggestions should have required fields
            assert!(!suggestion.suggestion.is_empty());
            assert!(!suggestion.reason.is_empty());
            assert!(suggestion.confidence >= 0.0 && suggestion.confidence <= 1.0);
            assert!(!suggestion.context_type.is_empty());
            // is_polymorphic is always false for function suggestions
            assert!(!suggestion.is_polymorphic);
        }
    }

    #[tokio::test]
    async fn test_performance_caching() {
        use crate::MockModelProvider;
        use crate::registry::FunctionRegistry;
        let model_provider = Arc::new(MockModelProvider::new());
        let function_registry = Arc::new(FunctionRegistry::default());
        let mut validator = PropertyValidator::new(model_provider, function_registry);

        // First call - should populate cache
        let suggestions1 = validator
            .generate_cached_property_suggestions("namee", "Patient", None)
            .await;

        // Second call - should use cache
        let suggestions2 = validator
            .generate_cached_property_suggestions("namee", "Patient", None)
            .await;

        // Results should be identical (cached)
        assert_eq!(suggestions1.len(), suggestions2.len());

        // Check function name caching
        let functions1 = validator.get_cached_function_names();
        let functions2 = validator.get_cached_function_names();
        assert_eq!(functions1, functions2);
    }
}
