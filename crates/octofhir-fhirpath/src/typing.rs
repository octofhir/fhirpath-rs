//! Type resolution system using ModelProvider for accurate FHIR type information
//! 
//! This module provides integration with ModelProvider to resolve accurate FHIR types
//! during evaluation, replacing generic types with specific FHIR type names.

use std::sync::Arc;

use crate::core::{FhirPathError, Result, ModelProvider};
use crate::path::CanonicalPath;

/// Type resolver that integrates with ModelProvider for accurate FHIR type information
#[derive(Debug, Clone)]
pub struct TypeResolver {
    model_provider: Arc<dyn ModelProvider>,
}

impl TypeResolver {
    /// Create a new type resolver with the given ModelProvider
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }
    
    /// Resolve FHIR type for a property of a parent type
    /// 
    /// # Arguments
    /// * `parent_type` - The FHIR type of the parent (e.g., "Patient", "HumanName")
    /// * `property` - The property name (e.g., "name", "given", "family")
    /// 
    /// # Returns
    /// The resolved FHIR type name or an error if resolution fails
    pub async fn resolve_property_type(
        &self,
        parent_type: &str,
        property: &str,
    ) -> Result<String> {
        // Handle special cases for primitive types
        if is_primitive_type(parent_type) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0052,
                format!("Cannot access property '{}' on primitive type '{}'", property, parent_type),
            ));
        }
        
        // Use ModelProvider to get type reflection
        match self.model_provider.get_type_reflection(parent_type).await {
            Ok(Some(type_reflection)) => {
                use octofhir_fhir_model::TypeReflectionInfo;
                match type_reflection {
                    TypeReflectionInfo::ClassInfo { elements, .. } => {
                        // Look for the property in elements
                        for element in elements.iter() {
                            if element.name == property {
                                // Found the property - return its type
                                return Ok(element.type_info.name().to_string());
                            }
                        }
                        
                        // Property not found - check if it's a choice element
                        if let Some(choice_type) = self.resolve_choice_element(parent_type, property).await? {
                            Ok(choice_type)
                        } else {
                            // Property doesn't exist
                            Err(FhirPathError::evaluation_error(
                                crate::core::error_code::FP0052,
                                format!("Property '{}' not found on type '{}'", property, parent_type),
                            ))
                        }
                    }
                    _ => {
                        // Not a class type - cannot access properties
                        Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0052,
                            format!("Cannot access property '{}' on non-class type '{}'", property, parent_type),
                        ))
                    }
                }
            }
            Ok(None) => {
                // Type not found
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0052,
                    format!("Unknown type '{}'", parent_type),
                ))
            }
            Err(_) => {
                // Error getting type reflection
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0052,
                    format!("Failed to resolve type '{}'", parent_type),
                ))
            }
        }
    }
    
    /// Resolve the element type for collection access
    /// Strips array wrapper to get the underlying element type
    pub async fn resolve_element_type(&self, collection_type: &str) -> Result<String> {
        // Handle array types
        if collection_type.starts_with("Array<") && collection_type.ends_with(">") {
            let element_type = &collection_type[6..collection_type.len() - 1];
            return Ok(element_type.to_string());
        }
        
        // For non-array types, return as-is
        Ok(collection_type.to_string())
    }
    
    /// Check if a property represents a choice element (value[x])
    async fn resolve_choice_element(
        &self,
        parent_type: &str,
        property: &str,
    ) -> Result<Option<String>> {
        // FHIR choice elements follow the pattern: baseProperty + TypeName
        // e.g., valueQuantity, valueString, valueBoolean for value[x]
        
        // Common choice element bases
        let choice_bases = ["value", "onset", "effective", "occurs", "multipleBirth"];
        
        for base in &choice_bases {
            if property.starts_with(base) && property.len() > base.len() {
                let type_suffix = &property[base.len()..];
                
                // Check if the base element exists as a choice element
                if let Ok(Some(type_reflection)) = self.model_provider.get_type_reflection(parent_type).await {
                    use octofhir_fhir_model::TypeReflectionInfo;
                    if let TypeReflectionInfo::ClassInfo { elements, .. } = type_reflection {
                        for element in elements.iter() {
                            if element.name == *base {
                                // The element exists as a choice - resolve the specific type
                                return Ok(Some(type_suffix.to_string()));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Resolve type from a canonical path
    /// Walks the path from root to leaf to determine the final type
    pub async fn resolve_type_by_path(&self, path: &CanonicalPath) -> Result<String> {
        let segments = path.segments();
        if segments.is_empty() {
            return Ok("unknown".to_string());
        }
        
        // Start with root type
        let root_segment = &segments[0];
        let mut current_type = match root_segment {
            crate::path::PathSegment::Root(resource_type) => {
                // Validate that this is a known resource type
                if self.model_provider.resource_type_exists(resource_type).unwrap_or(false) {
                    resource_type.clone()
                } else {
                    return Ok("unknown".to_string());
                }
            }
            _ => return Ok("unknown".to_string()),
        };
        
        // Walk through property segments (skip index segments)
        for segment in &segments[1..] {
            match segment {
                crate::path::PathSegment::Property(property) => {
                    current_type = self.resolve_property_type(&current_type, property).await?;
                }
                crate::path::PathSegment::Index(_) => {
                    // Index access doesn't change the type, just access array element
                    current_type = self.resolve_element_type(&current_type).await?;
                }
                crate::path::PathSegment::Wildcard => {
                    // Wildcard represents collection access
                    current_type = format!("Array<{}>", current_type);
                }
                crate::path::PathSegment::Root(_) => {
                    // Should not have multiple roots in a path
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0052,
                        "Invalid path: multiple root segments".to_string(),
                    ));
                }
            }
        }
        
        Ok(current_type)
    }
    
    /// Resolve type for a resource by examining its resourceType
    pub async fn resolve_resource_type(&self, resource_json: &serde_json::Value) -> Result<String> {
        if let Some(resource_type) = resource_json.get("resourceType")
            .and_then(|rt| rt.as_str()) {
            
            // Validate that this is a known resource type
            if self.model_provider.resource_type_exists(resource_type).unwrap_or(false) {
                Ok(resource_type.to_string())
            } else {
                Ok("unknown".to_string())
            }
        } else {
            Ok("unknown".to_string())
        }
    }
    
    /// Check if a type is a FHIR primitive type
    pub fn is_primitive_type(&self, type_name: &str) -> bool {
        is_primitive_type(type_name)
    }
    
    /// Check if a type is a FHIR resource type
    pub async fn is_resource_type(&self, type_name: &str) -> bool {
        self.model_provider.resource_type_exists(type_name).unwrap_or(false)
    }
    
    /// Get a reference to the underlying ModelProvider
    pub fn model_provider(&self) -> &Arc<dyn ModelProvider> {
        &self.model_provider
    }
}

/// Check if a type name represents a FHIR primitive type
pub fn is_primitive_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "boolean" | "integer" | "string" | "decimal" | "uri" | "url" | "canonical" |
        "base64Binary" | "instant" | "date" | "dateTime" | "time" | "code" | "oid" |
        "id" | "markdown" | "unsignedInt" | "positiveInt" | "uuid" | "xhtml"
    )
}

/// Type resolution context for caching and optimization
#[derive(Debug, Clone)]
pub struct TypeResolutionContext {
    resolver: TypeResolver,
    // Cache for resolved types to improve performance
    type_cache: std::collections::HashMap<String, String>,
}

impl TypeResolutionContext {
    /// Create a new type resolution context
    pub fn new(resolver: TypeResolver) -> Self {
        Self {
            resolver,
            type_cache: std::collections::HashMap::new(),
        }
    }
    
    /// Resolve type with caching
    pub async fn resolve_cached(&mut self, path: &CanonicalPath) -> Result<String> {
        let path_key = path.to_string();
        
        if let Some(cached_type) = self.type_cache.get(&path_key) {
            return Ok(cached_type.clone());
        }
        
        let resolved_type = self.resolver.resolve_type_by_path(path).await?;
        self.type_cache.insert(path_key, resolved_type.clone());
        
        Ok(resolved_type)
    }
    
    /// Clear the type cache
    pub fn clear_cache(&mut self) {
        self.type_cache.clear();
    }
}

/// Utility functions for type operations
pub mod type_utils {
    use super::*;
    
    /// Create type resolver from ModelProvider
    pub fn create_resolver(model_provider: Arc<dyn ModelProvider>) -> TypeResolver {
        TypeResolver::new(model_provider)
    }
    
    /// Check if two FHIR types are compatible for operations
    pub fn are_types_compatible(type1: &str, type2: &str) -> bool {
        // Same type is always compatible
        if type1 == type2 {
            return true;
        }
        
        // Numeric types are compatible with each other
        let numeric_types = ["integer", "decimal", "unsignedInt", "positiveInt"];
        let type1_numeric = numeric_types.contains(&type1);
        let type2_numeric = numeric_types.contains(&type2);
        if type1_numeric && type2_numeric {
            return true;
        }
        
        // String-like types are compatible
        let string_types = ["string", "code", "id", "markdown", "uri", "url", "canonical"];
        let type1_string = string_types.contains(&type1);
        let type2_string = string_types.contains(&type2);
        if type1_string && type2_string {
            return true;
        }
        
        // Date/time types are compatible
        let datetime_types = ["date", "dateTime", "instant", "time"];
        let type1_datetime = datetime_types.contains(&type1);
        let type2_datetime = datetime_types.contains(&type2);
        if type1_datetime && type2_datetime {
            return true;
        }
        
        false
    }
    
    /// Get the most specific common type for a collection of types
    pub fn get_common_type(types: &[String]) -> String {
        if types.is_empty() {
            return "unknown".to_string();
        }
        
        if types.len() == 1 {
            return types[0].clone();
        }
        
        // Check if all types are the same
        let first_type = &types[0];
        if types.iter().all(|t| t == first_type) {
            return first_type.clone();
        }
        
        // Check for compatible numeric types
        let all_numeric = types.iter().all(|t| {
            matches!(t.as_str(), "integer" | "decimal" | "unsignedInt" | "positiveInt")
        });
        if all_numeric {
            // If any is decimal, result is decimal; otherwise integer
            if types.iter().any(|t| t == "decimal") {
                return "decimal".to_string();
            } else {
                return "integer".to_string();
            }
        }
        
        // Check for compatible string types
        let all_string = types.iter().all(|t| {
            matches!(t.as_str(), "string" | "code" | "id" | "markdown" | "uri" | "url" | "canonical")
        });
        if all_string {
            return "string".to_string();
        }
        
        // Default to unknown for mixed types
        "unknown".to_string()
    }
    
    /// Convert FhirPathValue type to FHIR type name
    pub fn fhirpath_value_to_fhir_type(value: &crate::core::FhirPathValue) -> String {
        match value {
            crate::core::FhirPathValue::Boolean(_) => "boolean".to_string(),
            crate::core::FhirPathValue::Integer(_) => "integer".to_string(),
            crate::core::FhirPathValue::Decimal(_) => "decimal".to_string(),
            crate::core::FhirPathValue::String(_) => "string".to_string(),
            crate::core::FhirPathValue::Date(_) => "date".to_string(),
            crate::core::FhirPathValue::DateTime(_) => "dateTime".to_string(),
            crate::core::FhirPathValue::Time(_) => "time".to_string(),
            crate::core::FhirPathValue::Quantity { .. } => "Quantity".to_string(),
            crate::core::FhirPathValue::Resource(_) => "Resource".to_string(),
            crate::core::FhirPathValue::JsonValue(_) => "unknown".to_string(),
            crate::core::FhirPathValue::Id(_) => "id".to_string(),
            crate::core::FhirPathValue::Base64Binary(_) => "base64Binary".to_string(),
            crate::core::FhirPathValue::Uri(_) => "uri".to_string(),
            crate::core::FhirPathValue::Url(_) => "url".to_string(),
            crate::core::FhirPathValue::Collection(_) => "Collection".to_string(),
            crate::core::FhirPathValue::TypeInfoObject { .. } => "TypeInfo".to_string(),
            crate::core::FhirPathValue::Empty => "empty".to_string(),
        }
    }
}

/// Factory for creating type resolvers
pub struct TypeResolverFactory;

impl TypeResolverFactory {
    /// Create a type resolver from a ModelProvider
    pub fn create(model_provider: Arc<dyn ModelProvider>) -> TypeResolver {
        TypeResolver::new(model_provider)
    }
    
    /// Create a type resolution context with caching
    pub fn create_context(model_provider: Arc<dyn ModelProvider>) -> TypeResolutionContext {
        let resolver = Self::create(model_provider);
        TypeResolutionContext::new(resolver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::CanonicalPath;
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;
    
    fn create_test_resolver() -> TypeResolver {
        let provider = Arc::new(EmptyModelProvider);
        TypeResolver::new(provider)
    }
    
    #[test]
    fn test_primitive_type_check() {
        assert!(is_primitive_type("string"));
        assert!(is_primitive_type("boolean"));
        assert!(is_primitive_type("integer"));
        assert!(!is_primitive_type("Patient"));
        assert!(!is_primitive_type("HumanName"));
    }
    
    #[test]
    fn test_type_compatibility() {
        assert!(type_utils::are_types_compatible("string", "string"));
        assert!(type_utils::are_types_compatible("integer", "decimal"));
        assert!(type_utils::are_types_compatible("string", "code"));
        assert!(type_utils::are_types_compatible("date", "dateTime"));
        assert!(!type_utils::are_types_compatible("string", "integer"));
        assert!(!type_utils::are_types_compatible("Patient", "Observation"));
    }
    
    #[test]
    fn test_common_type_resolution() {
        let types = vec!["string".to_string(), "code".to_string()];
        assert_eq!(type_utils::get_common_type(&types), "string");
        
        let types = vec!["integer".to_string(), "decimal".to_string()];
        assert_eq!(type_utils::get_common_type(&types), "decimal");
        
        let types = vec!["Patient".to_string(), "Observation".to_string()];
        assert_eq!(type_utils::get_common_type(&types), "unknown");
    }
    
    #[test]
    fn test_fhirpath_value_type_mapping() {
        use crate::core::FhirPathValue;
        
        assert_eq!(
            type_utils::fhirpath_value_to_fhir_type(&FhirPathValue::String("test".to_string())),
            "string"
        );
        assert_eq!(
            type_utils::fhirpath_value_to_fhir_type(&FhirPathValue::Integer(42)),
            "integer"
        );
        assert_eq!(
            type_utils::fhirpath_value_to_fhir_type(&FhirPathValue::Boolean(true)),
            "boolean"
        );
    }
    
    #[tokio::test]
    async fn test_element_type_resolution() {
        let resolver = create_test_resolver();
        
        // Test array type resolution
        let result = resolver.resolve_element_type("Array<string>").await.unwrap();
        assert_eq!(result, "string");
        
        // Test non-array type
        let result = resolver.resolve_element_type("HumanName").await.unwrap();
        assert_eq!(result, "HumanName");
    }
    
    #[test]
    fn test_type_resolution_context() {
        let resolver = create_test_resolver();
        let mut context = TypeResolutionContext::new(resolver);
        
        // Test cache functionality
        assert!(context.type_cache.is_empty());
        context.clear_cache();
        assert!(context.type_cache.is_empty());
    }
}