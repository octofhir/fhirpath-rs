//! Property validation for FHIRPath expressions
//!
//! This module implements comprehensive property validation for FHIRPath expressions,
//! including detection of invalid properties, typo suggestions, and FHIR compliance checking.

use crate::ast::expression::*;
use crate::analyzer::type_checker::TypeInfo;
use crate::analyzer::context::AnalysisContext;
use crate::analyzer::AnalysisWarning;
use crate::core::{Result, SourceLocation, ModelProvider};
use crate::diagnostics::DiagnosticSeverity;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Result of property validation analysis
#[derive(Debug, Clone)]
pub struct PropertyValidationResult {
    /// Analysis warnings for invalid properties
    pub warnings: Vec<AnalysisWarning>,
    /// Valid properties for encountered types
    pub valid_properties: HashMap<String, Vec<String>>,
    /// Property suggestions for typos and alternatives
    pub suggestions: Vec<PropertySuggestion>,
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
    /// Properties for FHIR resource types
    resource_properties: HashMap<String, Vec<PropertyInfo>>,
    /// Properties for FHIR element types
    element_properties: HashMap<String, Vec<PropertyInfo>>,
    /// Common typos mapping to correct properties
    common_typos: HashMap<String, String>,
    /// Cache for performance optimization
    property_cache: HashMap<String, HashSet<String>>,
    /// Model provider for dynamic property resolution
    model_provider: Option<Arc<dyn ModelProvider>>,
}

impl PropertyValidator {
    /// Create a new property validator with built-in FHIR knowledge
    pub fn new() -> Self {
        let mut validator = Self {
            resource_properties: HashMap::new(),
            element_properties: HashMap::new(),
            common_typos: HashMap::new(),
            property_cache: HashMap::new(),
            model_provider: None,
        };
        
        validator.initialize_builtin_properties();
        validator.initialize_common_typos();
        validator
    }

    /// Create a new property validator with a model provider
    pub fn with_model_provider(model_provider: Arc<dyn ModelProvider>) -> Self {
        let mut validator = Self::new();
        validator.model_provider = Some(model_provider);
        validator
    }

    /// Validate property access in a FHIRPath expression
    pub async fn validate(&self, expression: &ExpressionNode) -> Result<PropertyValidationResult> {
        let mut warnings = Vec::new();
        let mut valid_properties = HashMap::new();
        let mut suggestions = Vec::new();
        let mut context = AnalysisContext::new();

        self.validate_expression(expression, &mut context, &mut warnings, &mut suggestions).await?;

        // Collect valid properties for all encountered types
        for scope in &context.scopes {
            for (_var_name, type_info) in &scope.variables {
                if let Some(properties) = self.get_properties_for_type(type_info).await {
                    valid_properties.insert(
                        type_info.to_string(),
                        properties.iter().map(|p| p.name.clone()).collect()
                    );
                }
            }
        }

        Ok(PropertyValidationResult {
            warnings,
            valid_properties,
            suggestions,
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
                self.validate_property_access(access, context, warnings, suggestions).await?;
                self.validate_expression(&access.object, context, warnings, suggestions).await?;
            },
            ExpressionNode::FunctionCall(call) => {
                for arg in &call.arguments {
                    self.validate_expression(arg, context, warnings, suggestions).await?;
                }
            },
            ExpressionNode::MethodCall(method) => {
                self.validate_expression(&method.object, context, warnings, suggestions).await?;
                for arg in &method.arguments {
                    self.validate_expression(arg, context, warnings, suggestions).await?;
                }
            },
            ExpressionNode::BinaryOperation(binary) => {
                self.validate_expression(&binary.left, context, warnings, suggestions).await?;
                self.validate_expression(&binary.right, context, warnings, suggestions).await?;
            },
            ExpressionNode::UnaryOperation(unary) => {
                self.validate_expression(&unary.operand, context, warnings, suggestions).await?;
            },
            ExpressionNode::Filter(filter) => {
                self.validate_expression(&filter.base, context, warnings, suggestions).await?;
                self.validate_expression(&filter.condition, context, warnings, suggestions).await?;
            },
            ExpressionNode::IndexAccess(index) => {
                self.validate_expression(&index.object, context, warnings, suggestions).await?;
                self.validate_expression(&index.index, context, warnings, suggestions).await?;
            },
            ExpressionNode::Lambda(lambda) => {
                context.push_scope(crate::analyzer::context::ScopeType::Lambda { parameter: Some("item".to_string()) });
                self.validate_expression(&lambda.body, context, warnings, suggestions).await?;
                context.pop_scope();
            },
            ExpressionNode::Collection(coll) => {
                for element in &coll.elements {
                    self.validate_expression(element, context, warnings, suggestions).await?;
                }
            },
            ExpressionNode::Parenthesized(expr) => {
                self.validate_expression(expr, context, warnings, suggestions).await?;
            },
            _ => {}, // Literals and identifiers don't need property validation
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
                self.validate_resource_property(resource_type, &access.property, access.location.clone(), warnings, suggestions).await;
            },
            TypeInfo::BackboneElement { properties } => {
                self.validate_backbone_property(&access.property, properties, access.location.clone(), warnings, suggestions);
            },
            TypeInfo::Collection(inner_type) => {
                // Validate property access on collection elements
                match inner_type.as_ref() {
                    TypeInfo::Resource { resource_type } => {
                        self.validate_resource_property(resource_type, &access.property, access.location.clone(), warnings, suggestions).await;
                    },
                    TypeInfo::BackboneElement { properties } => {
                        self.validate_backbone_property(&access.property, properties, access.location.clone(), warnings, suggestions);
                    },
                    _ => {
                        warnings.push(AnalysisWarning {
                            code: "FP0114".to_string(),
                            message: format!("Property access '{}' on collection of non-object type: {}", 
                                           access.property, inner_type),
                            location: access.location.clone(),
                            severity: DiagnosticSeverity::Warning,
                            suggestion: Some("Property access is only valid on resources and elements".to_string()),
                        });
                    }
                }
            },
            TypeInfo::Any | TypeInfo::Unknown => {
                // Cannot validate - might be valid
                warnings.push(AnalysisWarning {
                    code: "FP0115".to_string(),
                    message: format!("Cannot validate property '{}' on unknown type", access.property),
                    location: access.location.clone(),
                    severity: DiagnosticSeverity::Info,
                    suggestion: Some("Consider providing type information for better validation".to_string()),
                });
            },
            _ => {
                warnings.push(AnalysisWarning {
                    code: "FP0116".to_string(),
                    message: format!("Invalid property access '{}' on primitive type: {}", 
                                   access.property, object_type),
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
            if !properties.iter().any(|p| p.name == property || p.aliases.contains(&property.to_string())) {
                // Property not found - generate suggestions
                let similar_properties = self.find_similar_properties(property, &properties);
                
                warnings.push(AnalysisWarning {
                    code: "FP0117".to_string(),
                    message: format!("Unknown property '{}' on resource '{}'", property, resource_name),
                    location: location.clone(),
                    severity: DiagnosticSeverity::Error,
                    suggestion: if !similar_properties.is_empty() {
                        Some(format!("Did you mean: {}?", similar_properties.join(", ")))
                    } else {
                        Some(format!("Valid properties for {}: {}", 
                                   resource_name, 
                                   properties.iter().map(|p| &p.name).take(5).cloned().collect::<Vec<_>>().join(", ")))
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

    fn validate_property_usage(&self, prop_info: &PropertyInfo, location: Option<SourceLocation>, warnings: &mut Vec<AnalysisWarning>) {
        // Additional validation rules based on cardinality and usage patterns
        match prop_info.cardinality {
            Cardinality::ZeroToOne | Cardinality::OneToOne => {
                // Single-valued properties - no special validation needed here
            },
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
            },
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

        if a_len == 0 { return b_len; }
        if b_len == 0 { return a_len; }

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
                let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i-1][j] + 1)
                    .min(matrix[i][j-1] + 1)
                    .min(matrix[i-1][j-1] + cost);
            }
        }

        matrix[a_len][b_len]
    }

    fn infer_object_type<'a>(&'a self, object: &'a ExpressionNode, context: &'a AnalysisContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TypeInfo>> + 'a>> {
        Box::pin(async move {
        // Simplified type inference for property validation
        match object {
            ExpressionNode::Identifier(id) => {
                if let Some(var_type) = context.lookup_variable(&id.name) {
                    Ok(var_type.clone())
                } else if self.is_known_resource_type(&id.name).await {
                    Ok(TypeInfo::Resource { resource_type: id.name.clone() })
                } else {
                    Ok(TypeInfo::Unknown)
                }
            },
            ExpressionNode::PropertyAccess(access) => {
                let parent_type = self.infer_object_type(&access.object, context).await?;
                self.infer_property_type(&parent_type, &access.property).await
            },
            ExpressionNode::FunctionCall(_) => Ok(TypeInfo::Any), // Would need full type inference
            ExpressionNode::MethodCall(_) => Ok(TypeInfo::Any), // Would need full type inference
            ExpressionNode::Filter(filter) => self.infer_object_type(&filter.base, context).await,
            _ => Ok(TypeInfo::Unknown),
        }
        })
    }

    fn infer_property_type<'a>(&'a self, object_type: &'a TypeInfo, property: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TypeInfo>> + 'a>> {
        Box::pin(async move {
        match object_type {
            TypeInfo::Resource { resource_type } => {
                if let Some(properties) = self.get_resource_properties(resource_type).await {
                    if let Some(prop) = properties.iter().find(|p| p.name == property) {
                        Ok(prop.data_type.clone())
                    } else {
                        Ok(TypeInfo::Unknown)
                    }
                } else {
                    Ok(TypeInfo::Unknown)
                }
            },
            TypeInfo::BackboneElement { properties } => {
                if let Some(prop_type) = properties.get(property) {
                    Ok(prop_type.clone())
                } else {
                    Ok(TypeInfo::Unknown)
                }
            },
            TypeInfo::Collection(inner) => {
                let inner_property_type = self.infer_property_type(inner, property).await?;
                Ok(TypeInfo::Collection(Box::new(inner_property_type)))
            },
            _ => Ok(TypeInfo::Unknown),
        }
        })
    }

    async fn get_properties_for_type(&self, type_info: &TypeInfo) -> Option<Vec<PropertyInfo>> {
        match type_info {
            TypeInfo::Resource { resource_type } => self.get_resource_properties(resource_type).await,
            TypeInfo::BackboneElement { properties } => {
                // Convert HashMap<String, TypeInfo> to Vec<PropertyInfo>
                let prop_infos: Vec<PropertyInfo> = properties.iter().map(|(name, type_info)| {
                    PropertyInfo {
                        name: name.clone(),
                        data_type: type_info.clone(),
                        cardinality: Cardinality::ZeroToOne, // Default assumption
                        required: false,
                        description: format!("Property {} of type {}", name, type_info),
                        aliases: vec![],
                    }
                }).collect();
                Some(prop_infos)
            },
            _ => None,
        }
    }

    async fn get_resource_properties(&self, resource_name: &str) -> Option<Vec<PropertyInfo>> {
        // First check built-in properties
        if let Some(properties) = self.resource_properties.get(resource_name) {
            return Some(properties.clone());
        }

        // If we have a model provider, use it for dynamic property resolution
        if let Some(_model_provider) = &self.model_provider {
            // TODO: Implement dynamic property resolution using model provider
            // This would be integrated with the actual FHIR schema
        }

        None
    }

    async fn is_known_resource_type(&self, name: &str) -> bool {
        self.resource_properties.contains_key(name) ||
            (self.model_provider.as_ref().map_or(false, |_| {
                // TODO: Check with model provider if this is a known resource type
                false
            }))
    }

    fn initialize_builtin_properties(&mut self) {
        // Patient resource properties
        let patient_properties = vec![
            PropertyInfo {
                name: "id".to_string(),
                data_type: TypeInfo::String,
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Logical id of this artifact".to_string(),
                aliases: vec!["identifier".to_string()],
            },
            PropertyInfo {
                name: "name".to_string(),
                data_type: TypeInfo::Collection(Box::new(TypeInfo::BackboneElement { 
                    properties: self.create_human_name_properties() 
                })),
                cardinality: Cardinality::ZeroToMany,
                required: false,
                description: "A name associated with the patient".to_string(),
                aliases: vec!["names".to_string()],
            },
            PropertyInfo {
                name: "gender".to_string(),
                data_type: TypeInfo::Code,
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Administrative gender".to_string(),
                aliases: vec!["sex".to_string()],
            },
            PropertyInfo {
                name: "birthDate".to_string(),
                data_type: TypeInfo::Date,
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "The date of birth for the individual".to_string(),
                aliases: vec!["dateOfBirth".to_string(), "dob".to_string()],
            },
            PropertyInfo {
                name: "active".to_string(),
                data_type: TypeInfo::Boolean,
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Whether this patient's record is in active use".to_string(),
                aliases: vec![],
            },
            PropertyInfo {
                name: "telecom".to_string(),
                data_type: TypeInfo::Collection(Box::new(TypeInfo::BackboneElement { 
                    properties: self.create_contact_point_properties() 
                })),
                cardinality: Cardinality::ZeroToMany,
                required: false,
                description: "A contact detail for the patient".to_string(),
                aliases: vec!["contact".to_string(), "phone".to_string(), "email".to_string()],
            },
        ];
        self.resource_properties.insert("Patient".to_string(), patient_properties);

        // Add more resource types as needed...
        // For now, we'll add a few more common ones
        self.add_observation_properties();
        self.add_encounter_properties();
    }

    fn create_human_name_properties(&self) -> HashMap<String, TypeInfo> {
        let mut properties = HashMap::new();
        properties.insert("use".to_string(), TypeInfo::Code);
        properties.insert("family".to_string(), TypeInfo::String);
        properties.insert("given".to_string(), TypeInfo::Collection(Box::new(TypeInfo::String)));
        properties.insert("prefix".to_string(), TypeInfo::Collection(Box::new(TypeInfo::String)));
        properties.insert("suffix".to_string(), TypeInfo::Collection(Box::new(TypeInfo::String)));
        properties
    }

    fn create_contact_point_properties(&self) -> HashMap<String, TypeInfo> {
        let mut properties = HashMap::new();
        properties.insert("system".to_string(), TypeInfo::Code);
        properties.insert("value".to_string(), TypeInfo::String);
        properties.insert("use".to_string(), TypeInfo::Code);
        properties.insert("rank".to_string(), TypeInfo::Integer);
        properties
    }

    fn add_observation_properties(&mut self) {
        let observation_properties = vec![
            PropertyInfo {
                name: "id".to_string(),
                data_type: TypeInfo::String,
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Logical id of this artifact".to_string(),
                aliases: vec![],
            },
            PropertyInfo {
                name: "status".to_string(),
                data_type: TypeInfo::Code,
                cardinality: Cardinality::OneToOne,
                required: true,
                description: "Status of the observation".to_string(),
                aliases: vec![],
            },
            PropertyInfo {
                name: "code".to_string(),
                data_type: TypeInfo::CodeableConcept,
                cardinality: Cardinality::OneToOne,
                required: true,
                description: "Type of observation".to_string(),
                aliases: vec![],
            },
            PropertyInfo {
                name: "subject".to_string(),
                data_type: TypeInfo::Reference { target_types: vec!["Patient".to_string(), "Group".to_string()] },
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Who and/or what the observation is about".to_string(),
                aliases: vec!["patient".to_string()],
            },
            PropertyInfo {
                name: "value".to_string(),
                data_type: TypeInfo::Choice(vec![
                    TypeInfo::Quantity,
                    TypeInfo::CodeableConcept,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                    TypeInfo::Integer,
                    TypeInfo::Range,
                ]),
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Actual result".to_string(),
                aliases: vec!["result".to_string()],
            },
        ];
        self.resource_properties.insert("Observation".to_string(), observation_properties);
    }

    fn add_encounter_properties(&mut self) {
        let encounter_properties = vec![
            PropertyInfo {
                name: "id".to_string(),
                data_type: TypeInfo::String,
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "Logical id of this artifact".to_string(),
                aliases: vec![],
            },
            PropertyInfo {
                name: "status".to_string(),
                data_type: TypeInfo::Code,
                cardinality: Cardinality::OneToOne,
                required: true,
                description: "Status of the encounter".to_string(),
                aliases: vec![],
            },
            PropertyInfo {
                name: "class".to_string(),
                data_type: TypeInfo::Coding,
                cardinality: Cardinality::OneToOne,
                required: true,
                description: "Classification of patient encounter".to_string(),
                aliases: vec!["encounterClass".to_string()],
            },
            PropertyInfo {
                name: "subject".to_string(),
                data_type: TypeInfo::Reference { target_types: vec!["Patient".to_string()] },
                cardinality: Cardinality::ZeroToOne,
                required: false,
                description: "The patient present at the encounter".to_string(),
                aliases: vec!["patient".to_string()],
            },
        ];
        self.resource_properties.insert("Encounter".to_string(), encounter_properties);
    }

    fn initialize_common_typos(&mut self) {
        // Common property name typos
        self.common_typos.insert("familly".to_string(), "family".to_string());
        self.common_typos.insert("givne".to_string(), "given".to_string());
        self.common_typos.insert("birthdate".to_string(), "birthDate".to_string());
        self.common_typos.insert("identfier".to_string(), "identifier".to_string());
        self.common_typos.insert("telecome".to_string(), "telecom".to_string());
        
        // Case variations
        self.common_typos.insert("firstname".to_string(), "given".to_string());
        self.common_typos.insert("lastname".to_string(), "family".to_string());
        self.common_typos.insert("patientid".to_string(), "id".to_string());

        // Common FHIR-specific typos
        self.common_typos.insert("ressource".to_string(), "resource".to_string());
        self.common_typos.insert("observaton".to_string(), "observation".to_string());
        self.common_typos.insert("encounteer".to_string(), "encounter".to_string());
    }
}

impl Default for PropertyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::{ExpressionNode, PropertyAccessNode, IdentifierNode};

    #[tokio::test]
    async fn test_similarity_calculation() {
        let validator = PropertyValidator::new();
        
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
        let validator = PropertyValidator::new();
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
        let validator = PropertyValidator::new();
        
        assert_eq!(validator.levenshtein_distance("", ""), 0);
        assert_eq!(validator.levenshtein_distance("", "abc"), 3);
        assert_eq!(validator.levenshtein_distance("abc", ""), 3);
        assert_eq!(validator.levenshtein_distance("abc", "abc"), 0);
        assert_eq!(validator.levenshtein_distance("abc", "ab"), 1);
        assert_eq!(validator.levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(validator.levenshtein_distance("family", "familly"), 1);
    }

    #[test]
    fn test_builtin_properties_initialization() {
        let validator = PropertyValidator::new();
        
        // Check that Patient properties are initialized
        assert!(validator.resource_properties.contains_key("Patient"));
        
        let patient_props = validator.resource_properties.get("Patient").unwrap();
        assert!(!patient_props.is_empty());
        
        // Check for key properties
        assert!(patient_props.iter().any(|p| p.name == "name"));
        assert!(patient_props.iter().any(|p| p.name == "birthDate"));
        assert!(patient_props.iter().any(|p| p.name == "gender"));
    }

    #[test]
    fn test_common_typos() {
        let validator = PropertyValidator::new();
        
        assert_eq!(validator.common_typos.get("familly"), Some(&"family".to_string()));
        assert_eq!(validator.common_typos.get("birthdate"), Some(&"birthDate".to_string()));
        assert_eq!(validator.common_typos.get("firstname"), Some(&"given".to_string()));
    }
}