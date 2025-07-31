//! Type system definitions for FHIRPath

use serde::{Deserialize, Serialize};
use std::fmt;
/// Type information for FHIRPath values
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeInfo {
    /// Primitive types
    /// Boolean value (true/false)
    Boolean,
    /// Integer numeric value
    Integer,
    /// Decimal numeric value with arbitrary precision
    Decimal,
    /// String value
    String,
    /// Date value (YYYY-MM-DD)
    Date,
    /// DateTime value with timezone information
    DateTime,
    /// Time value (HH:MM:SS)
    Time,
    /// Quantity value with unit and magnitude
    Quantity,

    /// Collection type with element type
    Collection(Box<TypeInfo>),

    /// FHIR resource type
    Resource(String),

    /// Any type (used for polymorphic functions)
    Any,

    /// Union of multiple types
    Union(Vec<TypeInfo>),

    /// Optional type (may be empty)
    Optional(Box<TypeInfo>),

    /// System types
    /// Simple primitive type
    SimpleType,
    /// Complex class type
    ClassType,

    /// Type information object type (for reflection)
    TypeInfo,

    /// Function type (for higher-order functions)
    Function {
        /// Function parameter types
        parameters: Vec<TypeInfo>,
        /// Function return type
        return_type: Box<TypeInfo>,
    },

    /// Tuple type
    Tuple(Vec<TypeInfo>),

    /// Named type with namespace
    Named {
        /// Type namespace
        namespace: String,
        /// Type name
        name: String,
    },
}

impl TypeInfo {
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &TypeInfo) -> bool {
        match (self, other) {
            (TypeInfo::Any, _) | (_, TypeInfo::Any) => true,
            (TypeInfo::Optional(a), TypeInfo::Optional(b)) => a.is_compatible_with(b),
            (TypeInfo::Optional(a), b) => a.is_compatible_with(b),
            (a, TypeInfo::Optional(b)) => a.is_compatible_with(b),
            (TypeInfo::Collection(a), TypeInfo::Collection(b)) => a.is_compatible_with(b),
            (TypeInfo::Union(types), other) => types.iter().any(|t| t == other),
            (other, TypeInfo::Union(types)) => types.iter().any(|t| other == t),
            // Numeric type conversions
            (TypeInfo::Integer, TypeInfo::Decimal) | (TypeInfo::Decimal, TypeInfo::Integer) => true,
            // Type coercion for strings and primitives
            (TypeInfo::String, TypeInfo::Boolean) | (TypeInfo::Boolean, TypeInfo::String) => true,
            (TypeInfo::String, TypeInfo::Integer) | (TypeInfo::Integer, TypeInfo::String) => true,
            (TypeInfo::String, TypeInfo::Decimal) | (TypeInfo::Decimal, TypeInfo::String) => true,
            // Named types with same name are compatible
            (TypeInfo::Named { name: n1, .. }, TypeInfo::Named { name: n2, .. }) => n1 == n2,
            // Resource types with inheritance
            (TypeInfo::Resource(r1), TypeInfo::Resource(r2)) => self.is_resource_subtype(r1, r2),
            _ => self == other,
        }
    }

    /// Get the element type if this is a collection
    pub fn element_type(&self) -> Option<&TypeInfo> {
        match self {
            TypeInfo::Collection(elem) => Some(elem),
            TypeInfo::Optional(inner) => inner.element_type(),
            _ => None,
        }
    }

    /// Check if this type is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            TypeInfo::Boolean
                | TypeInfo::Integer
                | TypeInfo::Decimal
                | TypeInfo::String
                | TypeInfo::Date
                | TypeInfo::DateTime
                | TypeInfo::Time
                | TypeInfo::Quantity
        )
    }

    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            TypeInfo::Integer | TypeInfo::Decimal | TypeInfo::Quantity
        )
    }

    /// Check if this type can be converted to boolean
    pub fn can_convert_to_boolean(&self) -> bool {
        matches!(
            self,
            TypeInfo::Boolean
                | TypeInfo::Integer
                | TypeInfo::Decimal
                | TypeInfo::String
                | TypeInfo::Collection(_)
        )
    }

    /// Check if this type can be converted to string
    pub fn can_convert_to_string(&self) -> bool {
        self.is_primitive() || matches!(self, TypeInfo::Resource(_))
    }

    /// Check if this type supports equality comparison
    pub fn supports_equality(&self) -> bool {
        self.is_primitive() || matches!(self, TypeInfo::Resource(_) | TypeInfo::TypeInfo)
    }

    /// Check if this type supports ordering comparison
    pub fn supports_ordering(&self) -> bool {
        matches!(
            self,
            TypeInfo::Integer
                | TypeInfo::Decimal
                | TypeInfo::String
                | TypeInfo::Date
                | TypeInfo::DateTime
                | TypeInfo::Time
                | TypeInfo::Quantity
        )
    }

    /// Check if this type is a collection type
    pub fn is_collection(&self) -> bool {
        matches!(self, TypeInfo::Collection(_))
    }

    /// Check if this type is optional
    pub fn is_optional(&self) -> bool {
        matches!(self, TypeInfo::Optional(_))
    }

    /// Get the name of this type for display
    pub fn type_name(&self) -> String {
        match self {
            TypeInfo::Boolean => "Boolean".to_string(),
            TypeInfo::Integer => "Integer".to_string(),
            TypeInfo::Decimal => "Decimal".to_string(),
            TypeInfo::String => "String".to_string(),
            TypeInfo::Date => "Date".to_string(),
            TypeInfo::DateTime => "DateTime".to_string(),
            TypeInfo::Time => "Time".to_string(),
            TypeInfo::Quantity => "Quantity".to_string(),
            TypeInfo::Collection(elem) => format!("Collection<{}>", elem.type_name()),
            TypeInfo::Resource(name) => name.clone(),
            TypeInfo::Any => "Any".to_string(),
            TypeInfo::Union(types) => {
                let type_names: Vec<String> = types.iter().map(|t| t.type_name()).collect();
                format!("Union<{}>", type_names.join(", "))
            }
            TypeInfo::Optional(inner) => format!("Optional<{}>", inner.type_name()),
            TypeInfo::SimpleType => "SimpleType".to_string(),
            TypeInfo::ClassType => "ClassType".to_string(),
            TypeInfo::TypeInfo => "TypeInfo".to_string(),
            TypeInfo::Function {
                parameters,
                return_type,
            } => {
                let param_names: Vec<String> = parameters.iter().map(|t| t.type_name()).collect();
                format!(
                    "Function<({}) -> {}>",
                    param_names.join(", "),
                    return_type.type_name()
                )
            }
            TypeInfo::Tuple(types) => {
                let type_names: Vec<String> = types.iter().map(|t| t.type_name()).collect();
                format!("Tuple<{}>", type_names.join(", "))
            }
            TypeInfo::Named { namespace, name } => {
                if namespace.is_empty() {
                    name.clone()
                } else {
                    format!("{namespace}.{name}")
                }
            }
        }
    }

    /// Create a collection type
    pub fn collection(element_type: TypeInfo) -> Self {
        TypeInfo::Collection(Box::new(element_type))
    }

    /// Create an optional type
    pub fn optional(inner_type: TypeInfo) -> Self {
        TypeInfo::Optional(Box::new(inner_type))
    }

    /// Create a union type
    pub fn union(types: Vec<TypeInfo>) -> Self {
        // Flatten nested unions and remove duplicates
        let mut flattened = Vec::new();
        for t in types {
            match t {
                TypeInfo::Union(inner) => flattened.extend(inner),
                other => {
                    if !flattened.contains(&other) {
                        flattened.push(other);
                    }
                }
            }
        }

        if flattened.len() == 1 {
            flattened.into_iter().next().unwrap()
        } else {
            TypeInfo::Union(flattened)
        }
    }

    /// Create a function type
    pub fn function(parameters: Vec<TypeInfo>, return_type: TypeInfo) -> Self {
        TypeInfo::Function {
            parameters,
            return_type: Box::new(return_type),
        }
    }

    /// Create a tuple type
    pub fn tuple(types: Vec<TypeInfo>) -> Self {
        TypeInfo::Tuple(types)
    }

    /// Create a named type
    pub fn named(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        TypeInfo::Named {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    /// Get the fully qualified name for a type
    pub fn qualified_name(&self) -> String {
        match self {
            TypeInfo::Named { namespace, name } => {
                if namespace.is_empty() {
                    name.clone()
                } else {
                    format!("{namespace}.{name}")
                }
            }
            TypeInfo::Resource(name) => format!("FHIR.{name}"),
            _ => self.type_name(),
        }
    }

    /// Check if one resource type is a subtype of another
    fn is_resource_subtype(&self, child: &str, parent: &str) -> bool {
        // Basic FHIR resource hierarchy - in a complete implementation,
        // this would consult the FHIR structure definitions
        if child == parent {
            return true;
        }

        // Example hierarchies
        match (child, parent) {
            ("Patient", "DomainResource") => true,
            ("Observation", "DomainResource") => true,
            ("DomainResource", "Resource") => true,
            _ => false,
        }
    }

    /// Get the conversion priority for type coercion
    pub fn conversion_priority(&self) -> u8 {
        match self {
            TypeInfo::String => 1,
            TypeInfo::Boolean => 2,
            TypeInfo::Integer => 3,
            TypeInfo::Decimal => 4,
            TypeInfo::Quantity => 5,
            TypeInfo::Date => 6,
            TypeInfo::DateTime => 7,
            TypeInfo::Time => 8,
            TypeInfo::Collection(_) => 9,
            TypeInfo::Resource(_) => 10,
            _ => 255,
        }
    }
}

/// Type registry for FHIRPath type checking
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    /// Registered types
    types: std::collections::HashMap<String, TypeDefinition>,
}

/// Definition of a type in the registry
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Type name
    pub name: String,
    /// Base type (for inheritance)
    pub base_type: Option<String>,
    /// Properties of this type
    pub properties: std::collections::HashMap<String, PropertyDefinition>,
}

/// Definition of a property
#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    /// Property name
    pub name: String,
    /// Property type
    pub type_info: TypeInfo,
    /// Minimum cardinality
    pub min_cardinality: u32,
    /// Maximum cardinality (None for unbounded)
    pub max_cardinality: Option<u32>,
}

impl TypeRegistry {
    /// Create a new empty type registry
    pub fn new() -> Self {
        Self {
            types: std::collections::HashMap::new(),
        }
    }

    /// Register a type
    pub fn register_type(&mut self, type_def: TypeDefinition) {
        self.types.insert(type_def.name.clone(), type_def);
    }

    /// Get a type definition
    pub fn get_type(&self, name: &str) -> Option<&TypeDefinition> {
        self.types.get(name)
    }

    /// Check if a type exists
    pub fn has_type(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Get property type information
    pub fn get_property_type(&self, type_name: &str, property: &str) -> Option<&TypeInfo> {
        self.get_type(type_name)?
            .properties
            .get(property)
            .map(|p| &p.type_info)
    }

    /// Check if a type is a subtype of another
    pub fn is_subtype_of(&self, child: &str, parent: &str) -> bool {
        if child == parent {
            return true;
        }

        if let Some(type_def) = self.get_type(child) {
            if let Some(base) = &type_def.base_type {
                return self.is_subtype_of(base, parent);
            }
        }

        false
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_compatibility() {
        assert!(TypeInfo::Integer.is_compatible_with(&TypeInfo::Integer));
        assert!(TypeInfo::Any.is_compatible_with(&TypeInfo::Integer));
        assert!(TypeInfo::Integer.is_compatible_with(&TypeInfo::Any));

        let opt_int = TypeInfo::optional(TypeInfo::Integer);
        assert!(opt_int.is_compatible_with(&TypeInfo::Integer));
        assert!(TypeInfo::Integer.is_compatible_with(&opt_int));
    }

    #[test]
    fn test_union_types() {
        let union = TypeInfo::union(vec![TypeInfo::Integer, TypeInfo::String]);
        assert!(union.is_compatible_with(&TypeInfo::Integer));
        assert!(union.is_compatible_with(&TypeInfo::String));
        assert!(!union.is_compatible_with(&TypeInfo::Boolean));

        // Test union flattening
        let nested_union = TypeInfo::union(vec![
            TypeInfo::Boolean,
            TypeInfo::union(vec![TypeInfo::Integer, TypeInfo::String]),
        ]);
        match nested_union {
            TypeInfo::Union(types) => {
                assert_eq!(types.len(), 3);
                assert!(types.contains(&TypeInfo::Boolean));
                assert!(types.contains(&TypeInfo::Integer));
                assert!(types.contains(&TypeInfo::String));
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_type_registry() {
        let mut registry = TypeRegistry::new();

        let mut patient_type = TypeDefinition {
            name: "Patient".to_string(),
            base_type: Some("DomainResource".to_string()),
            properties: std::collections::HashMap::new(),
        };

        patient_type.properties.insert(
            "active".to_string(),
            PropertyDefinition {
                name: "active".to_string(),
                type_info: TypeInfo::Boolean,
                min_cardinality: 0,
                max_cardinality: Some(1),
            },
        );

        registry.register_type(patient_type);

        assert!(registry.has_type("Patient"));
        assert_eq!(
            registry.get_property_type("Patient", "active"),
            Some(&TypeInfo::Boolean)
        );
    }
}
