//! # Type System Foundation
//!
//! Implements the FHIRPath and FHIR type system for static analysis, including
//! type inference, type checking, and type compatibility validation.

use std::sync::Arc;
use std::fmt;

use octofhir_fhirpath_core::FhirPathValue;
use octofhir_fhirpath_diagnostics::{SourceLocation, Span, Position};

use crate::providers::fhir_provider::FhirProvider;
use super::error::AnalysisError;

/// FHIR type representation for analysis
#[derive(Debug, Clone, PartialEq)]
pub enum FhirType {
    /// Primitive types (boolean, string, integer, etc.)
    Primitive(PrimitiveType),
    
    /// Resource types (Patient, Observation, etc.)
    Resource(ResourceType),
    
    /// BackboneElement types
    BackboneElement(BackboneElementType),
    
    /// Collection of another type
    Collection(Box<FhirType>),
    
    /// Union of multiple types (choice types)
    Union(Vec<FhirType>),
    
    /// Unknown/inferred type
    Unknown,
}

/// FHIRPath primitive types
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Boolean,
    Integer,
    Decimal,
    String,
    Date,
    DateTime,
    Time,
    Quantity,
}

/// Resource type information
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceType {
    pub name: String,
    pub base_type: Option<String>,
}

/// BackboneElement type information
#[derive(Debug, Clone, PartialEq)]
pub struct BackboneElementType {
    pub name: String,
    pub resource_context: String,
}

/// Type cardinality constraints
#[derive(Debug, Clone, PartialEq)]
pub struct Cardinality {
    pub min: usize,
    pub max: Option<usize>,
}

impl Default for Cardinality {
    fn default() -> Self {
        Self { min: 0, max: Some(1) }
    }
}

/// Type constraint information
#[derive(Debug, Clone, PartialEq)]
pub struct TypeConstraint {
    pub constraint_type: ConstraintType,
    pub description: String,
    pub severity: ConstraintSeverity,
}

/// Types of constraints
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Cardinality,
    Type,
    Value,
    Invariant,
    Binding,
    Reference,
}

/// Constraint severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintSeverity {
    Error,
    Warning,
    Info,
}

/// Comprehensive type information for analysis
#[derive(Debug, Clone)]
pub struct TypeInformation {
    /// Primary type
    pub primary_type: FhirType,
    
    /// Cardinality constraints
    pub cardinality: Cardinality,
    
    /// Alternative possible types (for union types)
    pub possible_types: Vec<FhirType>,
    
    /// Type constraints
    pub constraints: Vec<TypeConstraint>,
    
    /// Source location where type was inferred
    pub source_location: SourceLocation,
}

/// Type system for analysis
pub struct TypeSystem {
    provider: Arc<dyn FhirProvider>,
    type_cache: dashmap::DashMap<String, FhirType>,
}

impl TypeSystem {
    /// Create a new type system
    pub fn new(provider: Arc<dyn FhirProvider>) -> Self {
        Self {
            provider,
            type_cache: dashmap::DashMap::new(),
        }
    }
    
    /// Resolve a type by name
    pub async fn resolve_type(&self, type_name: &str) -> Result<FhirType, AnalysisError> {
        // Check cache first
        if let Some(cached_type) = self.type_cache.get(type_name) {
            return Ok(cached_type.clone());
        }
        
        // Try primitive types first
        if let Some(primitive) = self.resolve_primitive_type(type_name) {
            let fhir_type = FhirType::Primitive(primitive);
            self.type_cache.insert(type_name.to_string(), fhir_type.clone());
            return Ok(fhir_type);
        }
        
        // Check if it's a resource type
        if self.provider.has_resource_type(type_name).await.unwrap_or(false) {
            let resource_type = ResourceType {
                name: type_name.to_string(),
                base_type: None, // TODO: Get from provider
            };
            let fhir_type = FhirType::Resource(resource_type);
            self.type_cache.insert(type_name.to_string(), fhir_type.clone());
            return Ok(fhir_type);
        }
        
        // Default to unknown
        Ok(FhirType::Unknown)
    }
    
    /// Resolve primitive type by name
    fn resolve_primitive_type(&self, type_name: &str) -> Option<PrimitiveType> {
        match type_name {
            "boolean" => Some(PrimitiveType::Boolean),
            "integer" => Some(PrimitiveType::Integer),
            "decimal" => Some(PrimitiveType::Decimal),
            "string" => Some(PrimitiveType::String),
            "date" => Some(PrimitiveType::Date),
            "dateTime" => Some(PrimitiveType::DateTime),
            "time" => Some(PrimitiveType::Time),
            "Quantity" => Some(PrimitiveType::Quantity),
            _ => None,
        }
    }
    
    /// Check if two types are compatible
    pub fn is_compatible(&self, from: &FhirType, to: &FhirType) -> bool {
        match (from, to) {
            // Identical types are always compatible
            (a, b) if a == b => true,
            
            // Unknown type is compatible with anything
            (FhirType::Unknown, _) | (_, FhirType::Unknown) => true,
            
            // Collection compatibility
            (FhirType::Collection(from_inner), FhirType::Collection(to_inner)) => {
                self.is_compatible(from_inner, to_inner)
            }
            
            // Union type compatibility
            (from_type, FhirType::Union(union_types)) => {
                union_types.iter().any(|union_type| self.is_compatible(from_type, union_type))
            }
            
            // Resource type inheritance (simplified)
            (FhirType::Resource(from_res), FhirType::Resource(to_res)) => {
                from_res.name == to_res.name ||
                from_res.base_type.as_ref() == Some(&to_res.name)
            }
            
            // Default: incompatible
            _ => false,
        }
    }
    
    /// Find common supertype for multiple types
    pub fn common_supertype(&self, types: &[FhirType]) -> Option<FhirType> {
        if types.is_empty() {
            return None;
        }
        
        if types.len() == 1 {
            return Some(types[0].clone());
        }
        
        // For now, simple implementation
        let first_type = &types[0];
        
        // If all types are the same, return that type
        if types.iter().all(|t| t == first_type) {
            return Some(first_type.clone());
        }
        
        // If any type is Unknown, return Unknown
        if types.iter().any(|t| matches!(t, FhirType::Unknown)) {
            return Some(FhirType::Unknown);
        }
        
        // For collections, find common element type
        if types.iter().all(|t| matches!(t, FhirType::Collection(_))) {
            let element_types: Vec<FhirType> = types.iter()
                .filter_map(|t| match t {
                    FhirType::Collection(inner) => Some((**inner).clone()),
                    _ => None,
                })
                .collect();
            
            if let Some(common_element) = self.common_supertype(&element_types) {
                return Some(FhirType::Collection(Box::new(common_element)));
            }
        }
        
        // Default: create union type
        Some(FhirType::Union(types.to_vec()))
    }
    
    /// Infer type information from a value
    pub fn infer_from_value(&self, value: &FhirPathValue) -> TypeInformation {
        let primary_type = match value {
            FhirPathValue::Boolean(_) => FhirType::Primitive(PrimitiveType::Boolean),
            FhirPathValue::Integer(_) => FhirType::Primitive(PrimitiveType::Integer),
            FhirPathValue::Decimal(_) => FhirType::Primitive(PrimitiveType::Decimal),
            FhirPathValue::String(_) => FhirType::Primitive(PrimitiveType::String),
            FhirPathValue::Date { .. } => FhirType::Primitive(PrimitiveType::Date),
            FhirPathValue::DateTime { .. } => FhirType::Primitive(PrimitiveType::DateTime),
            FhirPathValue::Time { .. } => FhirType::Primitive(PrimitiveType::Time),
            FhirPathValue::Quantity { .. } => FhirType::Primitive(PrimitiveType::Quantity),
            _ => FhirType::Unknown,
        };
        
        TypeInformation {
            primary_type,
            cardinality: Cardinality::default(),
            possible_types: Vec::new(),
            constraints: Vec::new(),
            source_location: SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0))),
        }
    }
}

impl fmt::Display for FhirType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FhirType::Primitive(p) => write!(f, "{:?}", p),
            FhirType::Resource(r) => write!(f, "{}", r.name),
            FhirType::BackboneElement(b) => write!(f, "{}.{}", b.resource_context, b.name),
            FhirType::Collection(inner) => write!(f, "Collection<{}>", inner),
            FhirType::Union(types) => {
                write!(f, "Union<")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, " | ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ">")
            }
            FhirType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl TypeInformation {
    /// Create new type information
    pub fn new(primary_type: FhirType, source_location: SourceLocation) -> Self {
        Self {
            primary_type,
            cardinality: Cardinality::default(),
            possible_types: Vec::new(),
            constraints: Vec::new(),
            source_location,
        }
    }
    
    /// Check if this is a collection type
    pub fn is_collection(&self) -> bool {
        matches!(self.primary_type, FhirType::Collection(_)) || 
        self.cardinality.max.map_or(true, |max| max > 1)
    }
    
    /// Check if this type is optional
    pub fn is_optional(&self) -> bool {
        self.cardinality.min == 0
    }
    
    /// Get the display name for this type
    pub fn display_name(&self) -> String {
        self.primary_type.to_string()
    }
}