//! ModelProvider extension for children() function analysis

use crate::types::UnionTypeInfo;
use async_trait::async_trait;
use octofhir_fhirpath_model::{
    error::ModelError,
    provider::{ModelProvider, TypeReflectionInfo},
    types::TypeInfo,
};
use std::collections::HashMap;

/// Extension to ModelProvider for children() function analysis
#[async_trait]
pub trait ModelProviderChildrenExt: ModelProvider {
    /// Get all child element types for a given parent type
    /// Returns union type information including all possible child types
    async fn get_children_types(&self, parent_type: &str) -> Result<UnionTypeInfo, ModelError>;

    /// Validate type filtering operations (is, as, ofType) against available child types
    async fn validate_type_filter(
        &self,
        union_type: &UnionTypeInfo,
        target_type: &str,
    ) -> Result<bool, ModelError>;

    /// Generate suggestions for invalid type operations
    async fn suggest_valid_types(&self, union_type: &UnionTypeInfo) -> Vec<String>;
}

/// Implementation for any ModelProvider using FHIRSchema
#[async_trait]
impl<T: ?Sized> ModelProviderChildrenExt for T
where
    T: ModelProvider + Send + Sync,
{
    async fn get_children_types(&self, parent_type: &str) -> Result<UnionTypeInfo, ModelError> {
        // Get type reflection for parent
        let _parent_reflection = self.get_type_reflection(parent_type).await.ok_or_else(|| {
            ModelError::TypeNotFound {
                type_name: parent_type.to_string(),
            }
        })?;

        // Get all properties of the parent type
        let properties = self.get_properties(parent_type).await;

        // Extract unique child types
        let mut constituent_types = Vec::new();
        let mut seen_types = std::collections::HashSet::new();

        for (property_name, type_reflection_info) in properties {
            // Skip non-element properties (like extensions, etc.)
            if self.is_structural_property(&property_name) {
                continue;
            }

            let type_name = self.reflection_info_to_string(&type_reflection_info);
            if seen_types.insert(type_name.clone()) {
                // Convert TypeReflectionInfo to TypeInfo for our Union type
                let type_info = self.convert_reflection_to_type_info(&type_reflection_info);
                constituent_types.push(type_info);
            }
        }

        // Create model context for downstream operations
        let mut model_context = HashMap::new();
        model_context.insert("parent_type".to_string(), parent_type.to_string());
        model_context.insert("operation".to_string(), "children".to_string());

        Ok(UnionTypeInfo {
            constituent_types,
            is_collection: true, // children() always returns a collection
            model_context,
        })
    }

    async fn validate_type_filter(
        &self,
        union_type: &UnionTypeInfo,
        target_type: &str,
    ) -> Result<bool, ModelError> {
        // Check if target type is among the constituent types
        let is_valid = union_type
            .constituent_types
            .iter()
            .any(|t| self.type_info_to_string(t) == target_type);

        // Also check inheritance hierarchy
        if !is_valid {
            for constituent_type in &union_type.constituent_types {
                let constituent_type_name = self.type_info_to_string(constituent_type);
                if self
                    .is_subtype_of(target_type, &constituent_type_name)
                    .await
                {
                    return Ok(true);
                }
            }
        }

        Ok(is_valid)
    }

    async fn suggest_valid_types(&self, union_type: &UnionTypeInfo) -> Vec<String> {
        union_type
            .constituent_types
            .iter()
            .map(|t| self.type_info_to_string(t))
            .collect()
    }
}

/// Helper trait for structural property detection and type conversion
trait StructuralPropertyDetector {
    fn is_structural_property(&self, property_name: &str) -> bool {
        // Filter out FHIR structural properties that aren't "children"
        matches!(
            property_name,
            "id" | "meta"
                | "implicitRules"
                | "language"
                | "extension"
                | "modifierExtension"
                | "resourceType"
        )
    }

    fn reflection_info_to_string(&self, reflection_info: &TypeReflectionInfo) -> String {
        // Extract the actual type name from TypeReflectionInfo enum
        match reflection_info {
            TypeReflectionInfo::SimpleType { name, .. } => name.clone(),
            TypeReflectionInfo::ClassInfo { name, .. } => name.clone(),
            TypeReflectionInfo::ListType { element_type } => {
                format!(
                    "Collection<{}>",
                    self.reflection_info_to_string(element_type)
                )
            }
            TypeReflectionInfo::TupleType { elements } => {
                if elements.is_empty() {
                    "Tuple<>".to_string()
                } else {
                    let element_names: Vec<String> = elements
                        .iter()
                        .map(|e| self.reflection_info_to_string(&e.type_info))
                        .collect();
                    format!("Tuple<{}>", element_names.join(", "))
                }
            }
        }
    }

    fn convert_reflection_to_type_info(&self, reflection_info: &TypeReflectionInfo) -> TypeInfo {
        // Convert TypeReflectionInfo to TypeInfo by extracting type name
        // and mapping to appropriate TypeInfo variant
        match reflection_info {
            TypeReflectionInfo::SimpleType { name, .. } => {
                let type_name = name.as_str();
                match type_name {
                    "boolean" | "Boolean" => TypeInfo::Boolean,
                    "integer" | "Integer" => TypeInfo::Integer,
                    "decimal" | "Decimal" => TypeInfo::Decimal,
                    "string" | "String" => TypeInfo::String,
                    "date" | "Date" => TypeInfo::Date,
                    "dateTime" | "DateTime" => TypeInfo::DateTime,
                    "time" | "Time" => TypeInfo::Time,
                    "Quantity" => TypeInfo::Quantity,
                    // FHIR primitive types
                    "uri" | "url" | "canonical" | "oid" | "uuid" | "id" | "markdown"
                    | "base64Binary" | "code" => TypeInfo::String,
                    "positiveInt" | "unsignedInt" => TypeInfo::Integer,
                    "instant" => TypeInfo::DateTime,
                    // FHIR complex types and resources - use Resource type
                    _ => {
                        if type_name.chars().next().unwrap_or('a').is_uppercase() {
                            // Capitalized names are typically FHIR resources or complex types
                            TypeInfo::Resource(type_name.to_string())
                        } else {
                            TypeInfo::Any
                        }
                    }
                }
            }
            TypeReflectionInfo::ClassInfo { name, .. } => {
                // FHIR complex types and resources
                TypeInfo::Resource(name.clone())
            }
            TypeReflectionInfo::ListType { element_type } => {
                // Collection of elements
                let inner_type = self.convert_reflection_to_type_info(element_type);
                TypeInfo::Collection(Box::new(inner_type))
            }
            TypeReflectionInfo::TupleType { elements } => {
                // Convert tuple elements to individual TypeInfo
                let tuple_types: Vec<TypeInfo> = elements
                    .iter()
                    .map(|e| self.convert_reflection_to_type_info(&e.type_info))
                    .collect();
                TypeInfo::Tuple(tuple_types)
            }
        }
    }

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
}

impl<T: ModelProvider + ?Sized> StructuralPropertyDetector for T {}
