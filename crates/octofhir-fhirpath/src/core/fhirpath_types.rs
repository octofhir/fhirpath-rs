//! FHIRPath type system
//!
//! Defines the core type system for FHIRPath expressions, following the
//! FHIRPath specification.

use serde::{Deserialize, Serialize};
use std::fmt;

/// FHIRPath type enumeration following the specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FhirPathType {
    // System types
    /// Boolean type (true/false)
    Boolean,
    /// Integer type (64-bit signed integer)
    Integer,
    /// Long type (64-bit signed integer with extended range)
    Long,
    /// Decimal type (high-precision decimal)
    Decimal,
    /// String type (UTF-8 string)
    String,
    /// Date type (ISO 8601 date)
    Date,
    /// DateTime type (ISO 8601 datetime with timezone)
    DateTime,
    /// Time type (ISO 8601 time)
    Time,
    /// Quantity type (value with unit)
    Quantity,

    // Collection and special types
    /// Collection type (ordered list of values)
    Collection(Box<FhirPathType>), // Parameterized by element type
    /// Empty type (represents absence of value)
    Empty,
    /// Any type (supertype of all types)
    Any,

    // FHIR types
    /// FHIR Resource type (generic)
    Resource,
    /// FHIR Element type (base for all FHIR types)
    Element,
    /// BackboneElement type
    BackboneElement,
    /// Specific FHIR resource type by name
    FhirResource(String), // e.g., Patient, Observation
    /// FHIR primitive type
    FhirPrimitive(String), // e.g., code, uri, id
    /// FHIR complex type
    FhirComplex(String), // e.g., CodeableConcept, Coding

    // Type operators
    /// Type identifier (used in is/as operations)
    TypeSpecifier(Box<FhirPathType>),
    /// Union type (for choice elements)
    Union(Vec<FhirPathType>),

    // Navigation types
    /// Polymorphic type (for choice types like value[x])
    Polymorphic(Vec<FhirPathType>),
    /// Unknown type (for error cases)
    Unknown,
}

impl FhirPathType {
    /// Check if this type is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::Boolean
                | Self::Integer
                | Self::Long
                | Self::Decimal
                | Self::String
                | Self::Date
                | Self::DateTime
                | Self::Time
                | Self::FhirPrimitive(_)
        )
    }

    /// Check if this type is a collection type
    pub fn is_collection(&self) -> bool {
        matches!(self, Self::Collection(_))
    }

    /// Check if this type is a FHIR type
    pub fn is_fhir_type(&self) -> bool {
        matches!(
            self,
            Self::Resource
                | Self::Element
                | Self::BackboneElement
                | Self::FhirResource(_)
                | Self::FhirPrimitive(_)
                | Self::FhirComplex(_)
        )
    }

    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Long | Self::Decimal | Self::Quantity
        )
    }

    /// Check if this type is temporal
    pub fn is_temporal(&self) -> bool {
        matches!(self, Self::Date | Self::DateTime | Self::Time)
    }

    /// Check if this type is ordered (supports comparison operators)
    pub fn is_ordered(&self) -> bool {
        matches!(
            self,
            Self::Integer
                | Self::Long
                | Self::Decimal
                | Self::String
                | Self::Date
                | Self::DateTime
                | Self::Time
                | Self::Quantity
        )
    }

    /// Get the element type for a collection
    pub fn element_type(&self) -> Option<&FhirPathType> {
        match self {
            Self::Collection(element_type) => Some(element_type),
            _ => None,
        }
    }

    /// Get the type name as used in FHIRPath expressions
    pub fn type_name(&self) -> String {
        match self {
            Self::Boolean => "Boolean".to_string(),
            Self::Integer => "Integer".to_string(),
            Self::Long => "Long".to_string(),
            Self::Decimal => "Decimal".to_string(),
            Self::String => "String".to_string(),
            Self::Date => "Date".to_string(),
            Self::DateTime => "DateTime".to_string(),
            Self::Time => "Time".to_string(),
            Self::Quantity => "Quantity".to_string(),
            Self::Collection(element_type) => format!("Collection<{}>", element_type.type_name()),
            Self::Empty => "Empty".to_string(),
            Self::Any => "Any".to_string(),
            Self::Resource => "Resource".to_string(),
            Self::Element => "Element".to_string(),
            Self::BackboneElement => "BackboneElement".to_string(),
            Self::FhirResource(name) => name.clone(),
            Self::FhirPrimitive(name) => name.clone(),
            Self::FhirComplex(name) => name.clone(),
            Self::TypeSpecifier(inner) => format!("TypeSpecifier<{}>", inner.type_name()),
            Self::Union(types) => {
                let type_names: Vec<String> = types.iter().map(|t| t.type_name()).collect();
                format!("Union<{}>", type_names.join(" | "))
            }
            Self::Polymorphic(types) => {
                let type_names: Vec<String> = types.iter().map(|t| t.type_name()).collect();
                format!("Polymorphic<{}>", type_names.join(" | "))
            }
            Self::Unknown => "Unknown".to_string(),
        }
    }

    /// Get the namespace for this type
    pub fn namespace(&self) -> Option<&'static str> {
        match self {
            Self::Boolean
            | Self::Integer
            | Self::Long
            | Self::Decimal
            | Self::String
            | Self::Date
            | Self::DateTime
            | Self::Time
            | Self::Quantity
            | Self::Collection(_)
            | Self::Empty
            | Self::Any => Some("System"),
            Self::Resource
            | Self::Element
            | Self::BackboneElement
            | Self::FhirResource(_)
            | Self::FhirPrimitive(_)
            | Self::FhirComplex(_) => Some("FHIR"),
            _ => None,
        }
    }

    /// Get the fully qualified type name (with namespace)
    pub fn qualified_name(&self) -> String {
        if let Some(namespace) = self.namespace() {
            format!("{}.{}", namespace, self.type_name())
        } else {
            self.type_name()
        }
    }

    /// Check if this type is assignable from another type
    pub fn is_assignable_from(&self, other: &Self) -> bool {
        // Exact match
        if self == other {
            return true;
        }

        // Any accepts everything
        if matches!(self, Self::Any) {
            return true;
        }

        // Empty can be assigned to any type
        if matches!(other, Self::Empty) {
            return true;
        }

        // Numeric type promotions
        matches!(
            (self, other),
            (Self::Decimal, Self::Integer | Self::Long)
                | (Self::Long, Self::Integer)
                | (Self::Quantity, Self::Integer | Self::Long | Self::Decimal)
        )
    }

    /// Parse a type from its string representation
    pub fn from_string(type_str: &str) -> Self {
        match type_str {
            "Boolean" | "boolean" => Self::Boolean,
            "Integer" | "integer" => Self::Integer,
            "Long" | "long" => Self::Long,
            "Decimal" | "decimal" => Self::Decimal,
            "String" | "string" => Self::String,
            "Date" | "date" => Self::Date,
            "DateTime" | "dateTime" => Self::DateTime,
            "Time" | "time" => Self::Time,
            "Quantity" | "quantity" => Self::Quantity,
            "Empty" | "empty" => Self::Empty,
            "Any" | "any" => Self::Any,
            "Resource" | "resource" => Self::Resource,
            "Element" | "element" => Self::Element,
            "BackboneElement" => Self::BackboneElement,
            _ => {
                // Check for system types with namespace
                if let Some(bare_type) = type_str.strip_prefix("System.") {
                    return Self::from_string(bare_type);
                }

                // Check for FHIR types with namespace
                if let Some(bare_type) = type_str.strip_prefix("FHIR.") {
                    return Self::FhirResource(bare_type.to_string());
                }

                // Collection type with generics
                if type_str.starts_with("Collection<") && type_str.ends_with('>') {
                    let inner_type_str = &type_str[11..type_str.len() - 1];
                    let inner_type = Self::from_string(inner_type_str);
                    return Self::Collection(Box::new(inner_type));
                }

                // Default to unknown FHIR resource type
                Self::FhirResource(type_str.to_string())
            }
        }
    }

    /// Create a collection type from this type
    pub fn to_collection(&self) -> Self {
        Self::Collection(Box::new(self.clone()))
    }

    /// Get common supertype of two types
    pub fn common_supertype(&self, other: &Self) -> Self {
        if self == other {
            return self.clone();
        }

        // If either is Any, return Any
        if matches!(self, Self::Any) || matches!(other, Self::Any) {
            return Self::Any;
        }

        // Numeric type hierarchy
        match (self, other) {
            (Self::Integer, Self::Decimal) | (Self::Decimal, Self::Integer) => Self::Decimal,
            (Self::Integer, Self::Long) | (Self::Long, Self::Integer) => Self::Long,
            (Self::Long, Self::Decimal) | (Self::Decimal, Self::Long) => Self::Decimal,
            (Self::Integer | Self::Long | Self::Decimal, Self::Quantity)
            | (Self::Quantity, Self::Integer | Self::Long | Self::Decimal) => Self::Quantity,
            _ => Self::Any, // Default to Any for incompatible types
        }
    }

    /// Create system type constants
    pub fn system_boolean() -> Self {
        Self::Boolean
    }

    pub fn system_integer() -> Self {
        Self::Integer
    }

    pub fn system_decimal() -> Self {
        Self::Decimal
    }

    pub fn system_string() -> Self {
        Self::String
    }

    pub fn system_date() -> Self {
        Self::Date
    }

    pub fn system_datetime() -> Self {
        Self::DateTime
    }

    pub fn system_time() -> Self {
        Self::Time
    }

    pub fn system_quantity() -> Self {
        Self::Quantity
    }

    pub fn system_any() -> Self {
        Self::Any
    }

    pub fn system_empty() -> Self {
        Self::Empty
    }

    /// Create FHIR resource type
    pub fn fhir_resource(resource_type: &str) -> Self {
        Self::FhirResource(resource_type.to_string())
    }

    /// Create FHIR primitive type
    pub fn fhir_primitive(primitive_type: &str) -> Self {
        Self::FhirPrimitive(primitive_type.to_string())
    }

    /// Create collection of this type
    pub fn collection() -> Self {
        Self::Collection(Box::new(Self::Any))
    }

    /// Create typed collection
    pub fn collection_of(element_type: Self) -> Self {
        Self::Collection(Box::new(element_type))
    }
}

impl fmt::Display for FhirPathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_name())
    }
}

/// Type signature for functions and operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeSignature {
    /// Input parameter types
    pub parameters: Vec<FhirPathType>,
    /// Return type
    pub return_type: FhirPathType,
    /// Whether this signature is polymorphic
    pub polymorphic: bool,
}

impl TypeSignature {
    /// Create a new type signature
    pub fn new(parameters: Vec<FhirPathType>, return_type: FhirPathType) -> Self {
        Self {
            parameters,
            return_type,
            polymorphic: false,
        }
    }

    /// Create a polymorphic type signature
    pub fn polymorphic(parameters: Vec<FhirPathType>, return_type: FhirPathType) -> Self {
        Self {
            parameters,
            return_type,
            polymorphic: true,
        }
    }

    /// Check if the given argument types match this signature
    pub fn matches(&self, argument_types: &[FhirPathType]) -> bool {
        if self.parameters.len() != argument_types.len() {
            return false;
        }

        self.parameters
            .iter()
            .zip(argument_types)
            .all(|(param_type, arg_type)| param_type.is_assignable_from(arg_type))
    }

    /// Get the return type for the given argument types
    pub fn resolve_return_type(&self, _argument_types: &[FhirPathType]) -> FhirPathType {
        // For now, return the declared return type
        // In a more advanced implementation, this could resolve generic types
        self.return_type.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_names() {
        assert_eq!(FhirPathType::Boolean.type_name(), "Boolean");
        assert_eq!(FhirPathType::Integer.type_name(), "Integer");
        assert_eq!(FhirPathType::String.type_name(), "String");
        assert_eq!(
            FhirPathType::FhirResource("Patient".to_string()).type_name(),
            "Patient"
        );
    }

    #[test]
    fn test_qualified_names() {
        assert_eq!(FhirPathType::Boolean.qualified_name(), "System.Boolean");
        assert_eq!(
            FhirPathType::FhirResource("Patient".to_string()).qualified_name(),
            "FHIR.Patient"
        );
    }

    #[test]
    fn test_type_checks() {
        assert!(FhirPathType::Integer.is_numeric());
        assert!(FhirPathType::Date.is_temporal());
        assert!(FhirPathType::Integer.is_ordered());
        assert!(FhirPathType::FhirResource("Patient".to_string()).is_fhir_type());
    }

    #[test]
    fn test_assignability() {
        assert!(FhirPathType::Any.is_assignable_from(&FhirPathType::Integer));
        assert!(FhirPathType::Decimal.is_assignable_from(&FhirPathType::Integer));
        assert!(!FhirPathType::Integer.is_assignable_from(&FhirPathType::String));
    }

    #[test]
    fn test_from_string() {
        assert_eq!(FhirPathType::from_string("Boolean"), FhirPathType::Boolean);
        assert_eq!(
            FhirPathType::from_string("System.Integer"),
            FhirPathType::Integer
        );
        assert_eq!(
            FhirPathType::from_string("FHIR.Patient"),
            FhirPathType::FhirResource("Patient".to_string())
        );
    }

    #[test]
    fn test_common_supertype() {
        assert_eq!(
            FhirPathType::Integer.common_supertype(&FhirPathType::Decimal),
            FhirPathType::Decimal
        );
        assert_eq!(
            FhirPathType::Boolean.common_supertype(&FhirPathType::String),
            FhirPathType::Any
        );
    }
}
