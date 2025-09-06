//! FHIRPath type checking implementation
//!
//! This module provides comprehensive type checking capabilities for FHIRPath expressions,
//! including type inference, validation, and compatibility checking.

use crate::analyzer::context::AnalysisContext;
use crate::analyzer::visitor::ExpressionVisitor;
use crate::ast::expression::*;
use crate::ast::literal::LiteralValue;
use crate::ast::operator::{BinaryOperator, UnaryOperator};
use crate::core::{ModelProvider, Result};
use crate::registry::FunctionRegistry;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Comprehensive type information for FHIRPath values
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    /// Type could not be determined
    Unknown,
    /// Boolean type
    Boolean,
    /// Integer type
    Integer,
    /// Decimal type
    Decimal,
    /// String type
    String,
    /// Date type (YYYY, YYYY-MM, YYYY-MM-DD)
    Date,
    /// DateTime type with optional timezone
    DateTime,
    /// Time type (HH:MM:SS[.fff][+|-ZZ:ZZ])
    Time,
    /// Quantity type with value and unit
    Quantity,
    /// Code type (system|code)
    Code,
    /// Coding type (system, code, display)
    Coding,
    /// CodeableConcept type
    CodeableConcept,
    /// Range type (low/high values)
    Range,
    /// Reference type (reference to another resource)
    Reference { target_types: Vec<String> },
    /// FHIR Resource type
    Resource { resource_type: String },
    /// FHIR BackboneElement or nested object
    BackboneElement {
        properties: HashMap<String, TypeInfo>,
    },
    /// Collection of a specific type
    Collection(Box<TypeInfo>),
    /// Union of multiple possible types
    Union(Vec<TypeInfo>),
    /// Empty collection (no elements)
    Empty,
    /// Choice type (e.g., value[x] in FHIR)
    Choice(Vec<TypeInfo>),
    /// Function type for lambda expressions and function references
    Function {
        parameters: Vec<TypeInfo>,
        return_type: Box<TypeInfo>,
    },
    /// Any type (top type in type hierarchy)
    Any,
    /// Cardinality constraints
    Constrained {
        base_type: Box<TypeInfo>,
        min_cardinality: u32,
        max_cardinality: Option<u32>, // None means unbounded
    },
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeInfo::Unknown => write!(f, "Unknown"),
            TypeInfo::Boolean => write!(f, "Boolean"),
            TypeInfo::Integer => write!(f, "Integer"),
            TypeInfo::Decimal => write!(f, "Decimal"),
            TypeInfo::String => write!(f, "String"),
            TypeInfo::Date => write!(f, "Date"),
            TypeInfo::DateTime => write!(f, "DateTime"),
            TypeInfo::Time => write!(f, "Time"),
            TypeInfo::Quantity => write!(f, "Quantity"),
            TypeInfo::Code => write!(f, "Code"),
            TypeInfo::Coding => write!(f, "Coding"),
            TypeInfo::CodeableConcept => write!(f, "CodeableConcept"),
            TypeInfo::Reference { target_types } => {
                if target_types.is_empty() {
                    write!(f, "Reference")
                } else {
                    write!(f, "Reference<{}>", target_types.join(" | "))
                }
            }
            TypeInfo::Resource { resource_type } => write!(f, "{}", resource_type),
            TypeInfo::BackboneElement { .. } => write!(f, "BackboneElement"),
            TypeInfo::Collection(inner) => write!(f, "Collection<{}>", inner),
            TypeInfo::Union(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            TypeInfo::Empty => write!(f, "Empty"),
            TypeInfo::Choice(choices) => {
                write!(f, "Choice<")?;
                for (i, t) in choices.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ">")
            }
            TypeInfo::Function {
                parameters,
                return_type,
            } => {
                write!(f, "(")?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            TypeInfo::Any => write!(f, "Any"),
            TypeInfo::Range => write!(f, "Range"),
            TypeInfo::Constrained {
                base_type,
                min_cardinality,
                max_cardinality,
            } => match max_cardinality {
                Some(max) => write!(f, "{}[{}..{}]", base_type, min_cardinality, max),
                None => write!(f, "{}[{}..*]", base_type, min_cardinality),
            },
        }
    }
}

impl TypeInfo {
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
                | TypeInfo::Code
        )
    }

    /// Check if this type is a collection
    pub fn is_collection(&self) -> bool {
        matches!(self, TypeInfo::Collection(_))
    }

    /// Check if this type is a resource
    pub fn is_resource(&self) -> bool {
        matches!(self, TypeInfo::Resource { .. })
    }

    /// Check if this type is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, TypeInfo::Empty)
    }

    /// Check if this type is unknown
    pub fn is_unknown(&self) -> bool {
        matches!(self, TypeInfo::Unknown)
    }

    /// Check if this type can be converted to another type
    pub fn is_compatible_with(&self, other: &TypeInfo) -> bool {
        match (self, other) {
            // Exact matches
            (a, b) if a == b => true,

            // Any is compatible with everything
            (TypeInfo::Any, _) | (_, TypeInfo::Any) => true,

            // Empty is compatible with collections
            (TypeInfo::Empty, TypeInfo::Collection(_))
            | (TypeInfo::Collection(_), TypeInfo::Empty) => true,

            // Unknown requires explicit handling
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => false,

            // Numeric compatibility
            (TypeInfo::Integer, TypeInfo::Decimal) | (TypeInfo::Decimal, TypeInfo::Integer) => true,

            // String conversion compatibility
            (
                TypeInfo::Boolean
                | TypeInfo::Integer
                | TypeInfo::Decimal
                | TypeInfo::Date
                | TypeInfo::DateTime
                | TypeInfo::Time
                | TypeInfo::Code
                | TypeInfo::Quantity,
                TypeInfo::String,
            ) => true,

            // Collection compatibility
            (TypeInfo::Collection(a), TypeInfo::Collection(b)) => a.is_compatible_with(b),
            (TypeInfo::Collection(inner), other) | (other, TypeInfo::Collection(inner)) => {
                inner.as_ref().is_compatible_with(other)
            }

            // Union compatibility
            (TypeInfo::Union(types), other) => types.iter().any(|t| t.is_compatible_with(other)),
            (other, TypeInfo::Union(types)) => types.iter().any(|t| other.is_compatible_with(t)),

            // Choice compatibility
            (TypeInfo::Choice(choices), other) => {
                choices.iter().any(|c| c.is_compatible_with(other))
            }
            (other, TypeInfo::Choice(choices)) => {
                choices.iter().any(|c| other.is_compatible_with(c))
            }

            // Coding hierarchy
            (TypeInfo::Code, TypeInfo::Coding) => true,
            (TypeInfo::Coding, TypeInfo::CodeableConcept) => true,
            (TypeInfo::Code, TypeInfo::CodeableConcept) => true,

            // Reference compatibility
            (TypeInfo::Reference { target_types }, TypeInfo::Resource { resource_type }) => {
                target_types.is_empty() || target_types.contains(resource_type)
            }

            // Constrained type compatibility
            (TypeInfo::Constrained { base_type, .. }, other) => base_type.is_compatible_with(other),
            (other, TypeInfo::Constrained { base_type, .. }) => other.is_compatible_with(base_type),

            _ => false,
        }
    }

    /// Infer the result type of a binary operation
    pub fn infer_binary_result(
        operator: &BinaryOperator,
        left: &TypeInfo,
        right: &TypeInfo,
    ) -> TypeInfo {
        use BinaryOperator::*;
        match operator {
            Add => match (left, right) {
                (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
                (TypeInfo::String, _) | (_, TypeInfo::String) => TypeInfo::String,
                (TypeInfo::DateTime, TypeInfo::Quantity) => TypeInfo::DateTime,
                (TypeInfo::Quantity, TypeInfo::DateTime) => TypeInfo::DateTime,
                (TypeInfo::Quantity, TypeInfo::Quantity) => TypeInfo::Quantity,
                _ => TypeInfo::Any,
            },
            Subtract => match (left, right) {
                (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
                (TypeInfo::DateTime, TypeInfo::DateTime) => TypeInfo::Quantity,
                (TypeInfo::DateTime, TypeInfo::Quantity) => TypeInfo::DateTime,
                (TypeInfo::Quantity, TypeInfo::Quantity) => TypeInfo::Quantity,
                _ => TypeInfo::Any,
            },
            Multiply | Divide => match (left, right) {
                (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                (TypeInfo::Quantity, TypeInfo::Integer) => TypeInfo::Quantity,
                (TypeInfo::Integer, TypeInfo::Quantity) => TypeInfo::Quantity,
                (TypeInfo::Quantity, TypeInfo::Decimal) => TypeInfo::Quantity,
                (TypeInfo::Decimal, TypeInfo::Quantity) => TypeInfo::Quantity,
                (TypeInfo::Quantity, TypeInfo::Quantity) => TypeInfo::Quantity,
                (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
                _ => TypeInfo::Any,
            },
            Modulo => match (left, right) {
                (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
                _ => TypeInfo::Any,
            },
            IntegerDivide => match (left, right) {
                (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Integer,
                _ => TypeInfo::Any,
            },
            Equal | NotEqual | Equivalent | NotEquivalent | LessThan | LessThanOrEqual
            | GreaterThan | GreaterThanOrEqual => TypeInfo::Boolean,
            And | Or | Xor | Implies => {
                match (left, right) {
                    (TypeInfo::Boolean, TypeInfo::Boolean) => TypeInfo::Boolean,
                    _ => TypeInfo::Boolean, // FHIRPath coerces to boolean
                }
            }
            In | Contains => TypeInfo::Boolean,
            Is => TypeInfo::Boolean,
            As => TypeInfo::Any, // Type depends on the cast target
            Concatenate => TypeInfo::String,
            Union => {
                TypeInfo::Collection(Box::new(TypeInfo::Union(vec![left.clone(), right.clone()])))
            }
        }
    }

    /// Infer the result type of a unary operation
    pub fn infer_unary_result(operator: &UnaryOperator, operand: &TypeInfo) -> TypeInfo {
        use UnaryOperator::*;
        match operator {
            Not => TypeInfo::Boolean,
            Negate => match operand {
                TypeInfo::Integer => TypeInfo::Integer,
                TypeInfo::Decimal => TypeInfo::Decimal,
                TypeInfo::Quantity => TypeInfo::Quantity,
                _ => TypeInfo::Any,
            },
            Positive => operand.clone(), // Unary plus returns the same type
        }
    }

    /// Get the common type between this and another type
    pub fn common_type(&self, other: &TypeInfo) -> TypeInfo {
        if self == other {
            return self.clone();
        }

        match (self, other) {
            (TypeInfo::Any, _) | (_, TypeInfo::Any) => TypeInfo::Any,
            (TypeInfo::Empty, other) | (other, TypeInfo::Empty) => other.clone(),
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => TypeInfo::Unknown,

            // Numeric types
            (TypeInfo::Integer, TypeInfo::Decimal) | (TypeInfo::Decimal, TypeInfo::Integer) => {
                TypeInfo::Decimal
            }

            // Collections
            (TypeInfo::Collection(a), TypeInfo::Collection(b)) => {
                TypeInfo::Collection(Box::new(a.common_type(b)))
            }

            // Create union type for incompatible types
            _ => TypeInfo::Union(vec![self.clone(), other.clone()]),
        }
    }

    /// Create a collection type of this type
    pub fn collection_of(inner: TypeInfo) -> Self {
        TypeInfo::Collection(Box::new(inner))
    }

    /// Unwrap collection to get inner type
    pub fn unwrap_collection(&self) -> Option<&TypeInfo> {
        match self {
            TypeInfo::Collection(inner) => Some(inner),
            _ => None,
        }
    }
}

/// Node identifier for tracking types through the AST
pub type NodeId = usize;

/// Type checker for FHIRPath expressions
pub struct TypeChecker {
    function_registry: Arc<FunctionRegistry>,
    model_provider: Arc<dyn ModelProvider>,
    function_signatures: HashMap<String, FunctionSignature>,
    type_cache: HashMap<String, TypeInfo>, // Cache for property types
    node_counter: std::cell::RefCell<NodeId>,
}

impl TypeChecker {
    /// Create a new type checker with function registry and model provider
    pub fn new(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        let mut checker = Self {
            function_registry,
            model_provider,
            function_signatures: HashMap::new(),
            type_cache: HashMap::new(),
            node_counter: std::cell::RefCell::new(0),
        };

        checker.initialize_builtin_function_signatures();
        checker
    }

    /// Perform comprehensive type analysis on an expression
    pub fn analyze(&self, expression: &ExpressionNode) -> Result<TypeAnalysisResult> {
        let mut context = AnalysisContext::new();
        let mut type_info = HashMap::new();
        let mut warnings = Vec::new();

        // Initialize with built-in variables
        context.define_global_variable("this".to_string(), TypeInfo::Unknown);
        context.define_global_variable(
            "context".to_string(),
            TypeInfo::Resource {
                resource_type: "Resource".to_string(),
            },
        );
        context.define_global_variable("resource".to_string(), TypeInfo::Unknown);
        context.define_global_variable("rootResource".to_string(), TypeInfo::Unknown);
        context.define_global_variable("ucum".to_string(), TypeInfo::String);

        let mut visitor = TypeInferenceVisitor::new(
            self,
            &mut context,
            &mut type_info,
            &mut warnings,
            &self.node_counter,
        );

        let return_type = visitor.visit_expression(expression)?;

        Ok(TypeAnalysisResult {
            type_info,
            warnings,
            return_type,
            context,
        })
    }

    /// Check type compatibility between two types
    pub fn is_compatible(&self, from: &TypeInfo, to: &TypeInfo) -> bool {
        use TypeInfo::*;

        match (from, to) {
            // Same types are always compatible
            (a, b) if a == b => true,

            // Unknown is compatible with anything
            (Unknown, _) | (_, Unknown) => true,

            // Empty is compatible with any collection
            (Empty, Collection(_)) | (Collection(_), Empty) => true,

            // Numeric compatibility
            (Integer, Decimal) | (Decimal, Integer) => true,

            // String compatibility (many things can convert to string)
            (Boolean, String)
            | (Integer, String)
            | (Decimal, String)
            | (Date, String)
            | (DateTime, String)
            | (Time, String)
            | (Code, String)
            | (Quantity, String) => true,

            // Collection compatibility
            (Collection(a), Collection(b)) => self.is_compatible(a, b),

            // Union compatibility
            (Union(types), target) => types.iter().any(|t| self.is_compatible(t, target)),
            (source, Union(types)) => types.iter().any(|t| self.is_compatible(source, t)),

            // Choice compatibility
            (Choice(choices), target) => choices.iter().any(|c| self.is_compatible(c, target)),
            (source, Choice(choices)) => choices.iter().any(|c| self.is_compatible(source, c)),

            // Coding hierarchy
            (Coding, CodeableConcept) => true,
            (Code, Coding) => true,
            (Code, CodeableConcept) => true,

            // Resource hierarchy
            (Resource { .. }, BackboneElement { .. }) => true,

            _ => false,
        }
    }

    /// Get the most specific common type between two types
    pub fn common_type(&self, a: &TypeInfo, b: &TypeInfo) -> TypeInfo {
        use TypeInfo::*;

        if a == b {
            return a.clone();
        }

        match (a, b) {
            (Unknown, t) | (t, Unknown) => t.clone(),
            (Empty, Collection(t)) | (Collection(t), Empty) => Collection(t.clone()),
            (Integer, Decimal) | (Decimal, Integer) => Decimal,
            (Collection(a), Collection(b)) => Collection(Box::new(self.common_type(a, b))),
            _ => Union(vec![a.clone(), b.clone()]),
        }
    }

    /// Generate a unique node ID
    #[allow(dead_code)]
    fn next_node_id(&self) -> NodeId {
        let mut counter = self.node_counter.borrow_mut();
        *counter += 1;
        *counter
    }

    /// Initialize builtin function signatures with proper type constraints
    fn initialize_builtin_function_signatures(&mut self) {
        // Collection functions
        self.function_signatures.insert(
            "first".to_string(),
            FunctionSignature {
                name: "first".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Any, // Returns first element of collection
                description: "Returns the first element of a collection".to_string(),
            },
        );

        self.function_signatures.insert(
            "last".to_string(),
            FunctionSignature {
                name: "last".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Any, // Returns last element of collection
                description: "Returns the last element of a collection".to_string(),
            },
        );

        self.function_signatures.insert(
            "count".to_string(),
            FunctionSignature {
                name: "count".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Integer,
                description: "Returns the count of elements in a collection".to_string(),
            },
        );

        self.function_signatures.insert(
            "empty".to_string(),
            FunctionSignature {
                name: "empty".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Boolean,
                description: "Returns true if the collection is empty".to_string(),
            },
        );

        self.function_signatures.insert(
            "exists".to_string(),
            FunctionSignature {
                name: "exists".to_string(),
                parameters: vec![TypeConstraint {
                    required_type: TypeInfo::Function {
                        parameters: vec![TypeInfo::Any],
                        return_type: Box::new(TypeInfo::Boolean),
                    },
                    optional: true,
                    description: "Optional condition function".to_string(),
                }],
                return_type: TypeInfo::Boolean,
                description: "Returns true if any element exists (optionally matching condition)"
                    .to_string(),
            },
        );

        self.function_signatures.insert(
            "all".to_string(),
            FunctionSignature {
                name: "all".to_string(),
                parameters: vec![TypeConstraint {
                    required_type: TypeInfo::Function {
                        parameters: vec![TypeInfo::Any],
                        return_type: Box::new(TypeInfo::Boolean),
                    },
                    optional: false,
                    description: "Condition function".to_string(),
                }],
                return_type: TypeInfo::Boolean,
                description: "Returns true if all elements match the condition".to_string(),
            },
        );

        // String functions
        self.function_signatures.insert(
            "length".to_string(),
            FunctionSignature {
                name: "length".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Integer,
                description: "Returns the length of a string".to_string(),
            },
        );

        self.function_signatures.insert(
            "substring".to_string(),
            FunctionSignature {
                name: "substring".to_string(),
                parameters: vec![
                    TypeConstraint {
                        required_type: TypeInfo::Integer,
                        optional: false,
                        description: "Start index (0-based)".to_string(),
                    },
                    TypeConstraint {
                        required_type: TypeInfo::Integer,
                        optional: true,
                        description: "Length of substring".to_string(),
                    },
                ],
                return_type: TypeInfo::String,
                description: "Returns a substring starting at the given index".to_string(),
            },
        );

        self.function_signatures.insert(
            "contains".to_string(),
            FunctionSignature {
                name: "contains".to_string(),
                parameters: vec![TypeConstraint {
                    required_type: TypeInfo::String,
                    optional: false,
                    description: "Substring to search for".to_string(),
                }],
                return_type: TypeInfo::Boolean,
                description: "Returns true if string contains the given substring".to_string(),
            },
        );

        // Math functions
        self.function_signatures.insert(
            "abs".to_string(),
            FunctionSignature {
                name: "abs".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Union(vec![TypeInfo::Integer, TypeInfo::Decimal]),
                description: "Returns the absolute value of a number".to_string(),
            },
        );

        self.function_signatures.insert(
            "ceiling".to_string(),
            FunctionSignature {
                name: "ceiling".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Integer,
                description: "Returns the smallest integer greater than or equal to the input"
                    .to_string(),
            },
        );

        self.function_signatures.insert(
            "floor".to_string(),
            FunctionSignature {
                name: "floor".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Integer,
                description: "Returns the largest integer less than or equal to the input"
                    .to_string(),
            },
        );

        // Type functions
        self.function_signatures.insert(
            "is".to_string(),
            FunctionSignature {
                name: "is".to_string(),
                parameters: vec![TypeConstraint {
                    required_type: TypeInfo::String,
                    optional: false,
                    description: "Type name to check".to_string(),
                }],
                return_type: TypeInfo::Boolean,
                description: "Tests if the input is of the specified type".to_string(),
            },
        );

        self.function_signatures.insert(
            "as".to_string(),
            FunctionSignature {
                name: "as".to_string(),
                parameters: vec![TypeConstraint {
                    required_type: TypeInfo::String,
                    optional: false,
                    description: "Type to cast to".to_string(),
                }],
                return_type: TypeInfo::Any,
                description: "Casts the input to the specified type".to_string(),
            },
        );

        // Filtering functions
        self.function_signatures.insert(
            "where".to_string(),
            FunctionSignature {
                name: "where".to_string(),
                parameters: vec![TypeConstraint {
                    required_type: TypeInfo::Function {
                        parameters: vec![TypeInfo::Any],
                        return_type: Box::new(TypeInfo::Boolean),
                    },
                    optional: false,
                    description: "Filter condition".to_string(),
                }],
                return_type: TypeInfo::Collection(Box::new(TypeInfo::Any)),
                description: "Filters collection elements based on condition".to_string(),
            },
        );

        self.function_signatures.insert(
            "select".to_string(),
            FunctionSignature {
                name: "select".to_string(),
                parameters: vec![TypeConstraint {
                    required_type: TypeInfo::Function {
                        parameters: vec![TypeInfo::Any],
                        return_type: Box::new(TypeInfo::Any),
                    },
                    optional: false,
                    description: "Transformation function".to_string(),
                }],
                return_type: TypeInfo::Collection(Box::new(TypeInfo::Any)),
                description: "Transforms collection elements using the given function".to_string(),
            },
        );
    }

    /// Get property type for FHIR resources and elements using ModelProvider
    pub fn get_property_type(&self, base_type: &TypeInfo, property: &str) -> TypeInfo {
        match base_type {
            TypeInfo::Resource { resource_type: _ } => {
                // Try to use ModelProvider for navigation if possible
                // For now, fallback to essential property mapping
                match property {
                    "id" => TypeInfo::String,
                    "meta" => TypeInfo::BackboneElement {
                        properties: HashMap::new(),
                    },
                    "resourceType" => TypeInfo::String,
                    _ => {
                        // In future, could use a sync ModelProvider method here
                        // For now, return Any to indicate we need more type information
                        TypeInfo::Any
                    }
                }
            }
            TypeInfo::BackboneElement { properties } => properties
                .get(property)
                .cloned()
                .unwrap_or(TypeInfo::Unknown),
            TypeInfo::Collection(inner) => {
                // Property access on collection returns collection of property type
                TypeInfo::Collection(Box::new(self.get_property_type(inner, property)))
            }
            _ => TypeInfo::Unknown,
        }
    }

    /// Async version of get_property_type that uses full ModelProvider capabilities
    pub async fn get_property_type_async(&self, base_type: &TypeInfo, property: &str) -> TypeInfo {
        match base_type {
            TypeInfo::Resource { resource_type } => {
                self.get_property_type_from_model(resource_type, property)
                    .await
            }
            TypeInfo::BackboneElement { properties } => properties
                .get(property)
                .cloned()
                .unwrap_or(TypeInfo::Unknown),
            TypeInfo::Collection(inner) => {
                // Property access on collection returns collection of property type
                let inner_type = Box::pin(self.get_property_type_async(inner, property)).await;
                TypeInfo::Collection(Box::new(inner_type))
            }
            _ => TypeInfo::Unknown,
        }
    }

    /// Get property type for FHIR resources using ModelProvider
    async fn get_resource_property_type_from_model(
        &self,
        resource_type: &str,
        property: &str,
    ) -> TypeInfo {
        // Cache key for property lookup
        let cache_key = format!("{}.{}", resource_type, property);

        if let Some(cached_type) = self.type_cache.get(&cache_key) {
            return cached_type.clone();
        }

        // Use async ModelProvider to get property type information
        match self
            .model_provider
            .get_navigation_result_type(resource_type, property)
            .await
        {
            Ok(Some(type_reflection)) => {
                let type_info = self.convert_type_reflection_to_type_info(type_reflection);
                // Note: We can't mutate the cache from &self, but that's okay for now
                type_info
            }
            Ok(None) => {
                // Fallback to essential system properties only
                match property {
                    "id" => TypeInfo::String,
                    "resourceType" => TypeInfo::String,
                    "meta" => TypeInfo::BackboneElement {
                        properties: HashMap::new(),
                    },
                    _ => TypeInfo::Unknown,
                }
            }
            Err(_) => {
                // On error, fallback to essential system properties only
                match property {
                    "id" => TypeInfo::String,
                    "resourceType" => TypeInfo::String,
                    "meta" => TypeInfo::BackboneElement {
                        properties: HashMap::new(),
                    },
                    _ => TypeInfo::Unknown,
                }
            }
        }
    }

    /// Get property type using ModelProvider instead of hardcoded logic
    async fn get_property_type_from_model(&self, resource_type: &str, property: &str) -> TypeInfo {
        // Use ModelProvider to get navigation result type
        match self
            .model_provider
            .get_navigation_result_type(resource_type, property)
            .await
        {
            Ok(Some(type_reflection)) => self.convert_type_reflection_to_type_info(type_reflection),
            Ok(None) => TypeInfo::Unknown,
            Err(_) => {
                // Fallback to basic inference for critical system properties only
                match property {
                    "id" => TypeInfo::String,
                    "resourceType" => TypeInfo::String,
                    "meta" => TypeInfo::BackboneElement {
                        properties: HashMap::new(),
                    },
                    _ => TypeInfo::Unknown,
                }
            }
        }
    }

    /// Convert TypeReflectionInfo from ModelProvider to our TypeInfo
    fn convert_type_reflection_to_type_info(
        &self,
        type_reflection: octofhir_fhir_model::reflection::TypeReflectionInfo,
    ) -> TypeInfo {
        use octofhir_fhir_model::reflection::TypeReflectionInfo as TRI;

        match &type_reflection {
            TRI::SimpleType {
                namespace, name, ..
            } => {
                match (namespace.as_str(), name.as_str()) {
                    // System primitive types
                    ("System", "Boolean") => TypeInfo::Boolean,
                    ("System", "Integer") => TypeInfo::Integer,
                    ("System", "Decimal") => TypeInfo::Decimal,
                    ("System", "String") => TypeInfo::String,
                    ("System", "Date") => TypeInfo::Date,
                    ("System", "DateTime") => TypeInfo::DateTime,
                    ("System", "Time") => TypeInfo::Time,

                    // FHIR primitive types
                    ("FHIR", "boolean") => TypeInfo::Boolean,
                    ("FHIR", "integer") => TypeInfo::Integer,
                    ("FHIR", "decimal") => TypeInfo::Decimal,
                    ("FHIR", "string") => TypeInfo::String,
                    ("FHIR", "date") => TypeInfo::Date,
                    ("FHIR", "dateTime") => TypeInfo::DateTime,
                    ("FHIR", "time") => TypeInfo::Time,
                    ("FHIR", "code") => TypeInfo::Code,

                    // FHIR complex types
                    ("FHIR", "Quantity") => TypeInfo::Quantity,
                    ("FHIR", "Coding") => TypeInfo::Coding,
                    ("FHIR", "CodeableConcept") => TypeInfo::CodeableConcept,
                    ("FHIR", "Range") => TypeInfo::Range,
                    ("FHIR", "Reference") => TypeInfo::Reference {
                        target_types: vec![],
                    },

                    // Resource types
                    ("FHIR", resource_type) => TypeInfo::Resource {
                        resource_type: resource_type.to_string(),
                    },

                    // Unknown types
                    _ => TypeInfo::Unknown,
                }
            }

            TRI::ClassInfo {
                namespace,
                name,
                elements,
                ..
            } => {
                match (namespace.as_str(), name.as_str()) {
                    // FHIR Resource types
                    ("FHIR", resource_type) => TypeInfo::Resource {
                        resource_type: resource_type.to_string(),
                    },

                    // BackboneElements or complex types
                    _ => {
                        let mut property_map = HashMap::new();
                        for element in elements {
                            let element_type = self
                                .convert_type_reflection_to_type_info(element.type_info.clone());
                            property_map.insert(element.name.clone(), element_type);
                        }
                        TypeInfo::BackboneElement {
                            properties: property_map,
                        }
                    }
                }
            }

            TRI::ListType { element_type } => {
                let inner_type =
                    self.convert_type_reflection_to_type_info((**element_type).clone());
                TypeInfo::Collection(Box::new(inner_type))
            }

            TRI::TupleType { elements } => {
                // Convert tuple to union of element types for now
                let type_infos: Vec<TypeInfo> = elements
                    .iter()
                    .map(|elem| self.convert_type_reflection_to_type_info(elem.type_info.clone()))
                    .collect();
                if type_infos.len() == 1 {
                    type_infos.into_iter().next().unwrap()
                } else {
                    TypeInfo::Union(type_infos)
                }
            }
        }
    }

    /// Check if a type name represents a known FHIR resource using ModelProvider
    pub fn is_known_resource_type(&self, type_name: &str) -> bool {
        // For synchronous API, use standard resource type checking
        // Async version available via is_known_resource_type_from_model
        self.is_standard_fhir_resource_type(type_name)
    }

    /// Check if a type name represents a known FHIR resource using ModelProvider
    async fn is_known_resource_type_from_model(&self, type_name: &str) -> bool {
        // First check standard types for quick lookup
        if self.is_standard_fhir_resource_type(type_name) {
            return true;
        }

        // Use async ModelProvider to get type reflection information
        match self.model_provider.get_type_reflection(type_name).await {
            Ok(Some(type_reflection)) => {
                // Check if this type is a resource (it should inherit from Resource or be a known resource type)
                match &type_reflection {
                    octofhir_fhir_model::reflection::TypeReflectionInfo::SimpleType {
                        namespace,
                        name,
                        base_type,
                    }
                    | octofhir_fhir_model::reflection::TypeReflectionInfo::ClassInfo {
                        namespace,
                        name,
                        base_type,
                        ..
                    } => {
                        // FHIR namespace resource types
                        if namespace == "FHIR" {
                            // Check if it's Resource or inherits from Resource
                            if name == "Resource" || name == "DomainResource" {
                                return true;
                            }
                            if let Some(base) = base_type {
                                if base == "Resource" || base == "DomainResource" {
                                    return true;
                                }
                            }
                        }
                        false
                    }
                    _ => false,
                }
            }
            Ok(None) | Err(_) => false,
        }
    }

    /// Check if a type is a standard FHIR resource type using ModelProvider
    fn is_standard_fhir_resource_type(&self, type_name: &str) -> bool {
        // Use ModelProvider's synchronous resource type check if available
        match self.model_provider.resource_type_exists(type_name) {
            Ok(exists) => exists,
            Err(_) => {
                // Fallback to basic type checking only for essential types
                matches!(
                    type_name,
                    "Resource" | "DomainResource" | "Bundle" | "Patient" | "Observation"
                )
            }
        }
    }

    /// Advanced type inference for choice types (e.g., value[x] in FHIR)
    pub async fn resolve_choice_type(
        &self,
        base_type: &TypeInfo,
        property_name: &str,
    ) -> Result<TypeInfo> {
        match base_type {
            TypeInfo::Resource { resource_type } => {
                // Use ModelProvider to resolve choice types
                match self.model_provider.get_type_reflection(resource_type).await {
                    Ok(Some(type_reflection)) => {
                        // Check if this property is a choice type
                        if let Some(property_type) = self
                            .extract_property_type_from_reflection(&type_reflection, property_name)
                        {
                            // If it's a choice type, return the choice options
                            Ok(property_type)
                        } else {
                            Ok(TypeInfo::Any)
                        }
                    }
                    _ => Ok(TypeInfo::Any),
                }
            }
            _ => Ok(TypeInfo::Any),
        }
    }

    /// Extract property type information from type reflection
    fn extract_property_type_from_reflection(
        &self,
        _type_reflection: &octofhir_fhir_model::reflection::TypeReflectionInfo,
        property_name: &str,
    ) -> Option<TypeInfo> {
        // This is a simplified implementation
        // In production, this would parse the type reflection to find choice types
        if property_name.ends_with("[x]") || property_name.contains("value") {
            // Common choice types in FHIR
            Some(TypeInfo::Choice(vec![
                TypeInfo::String,
                TypeInfo::Integer,
                TypeInfo::Boolean,
                TypeInfo::DateTime,
                TypeInfo::Quantity,
                TypeInfo::CodeableConcept,
            ]))
        } else {
            None
        }
    }

    /// Validate reference types
    pub async fn validate_reference_type(
        &self,
        reference_type: &TypeInfo,
        target_resource: &str,
    ) -> Result<bool> {
        match reference_type {
            TypeInfo::Reference { target_types } => {
                // Check if the target resource is in the allowed list
                Ok(target_types.contains(&target_resource.to_string())
                    || target_types.contains(&"*".to_string()))
            }
            _ => Ok(false),
        }
    }

    /// Apply cardinality constraints to a type
    pub fn apply_cardinality_constraints(
        &self,
        base_type: TypeInfo,
        min: u32,
        max: Option<u32>,
    ) -> TypeInfo {
        TypeInfo::Constrained {
            base_type: Box::new(base_type),
            min_cardinality: min,
            max_cardinality: max,
        }
    }

    /// Validate cardinality constraints
    pub fn validate_cardinality(&self, type_info: &TypeInfo, actual_count: u32) -> bool {
        match type_info {
            TypeInfo::Constrained {
                min_cardinality,
                max_cardinality,
                ..
            } => {
                if actual_count < *min_cardinality {
                    return false;
                }
                if let Some(max) = max_cardinality {
                    if actual_count > *max {
                        return false;
                    }
                }
                true
            }
            TypeInfo::Collection(_) => actual_count >= 0, // Collections can be empty
            _ => actual_count == 1,                       // Single values must have exactly 1 item
        }
    }

    /// Resolve polymorphic types based on context
    pub async fn resolve_polymorphic_type(
        &self,
        union_type: &TypeInfo,
        context: &AnalysisContext,
    ) -> Result<TypeInfo> {
        match union_type {
            TypeInfo::Union(types) => {
                // Try to narrow down based on context
                if let Some(expected_type) = context.variables.get("this") {
                    for candidate_type in types {
                        if self.is_compatible(candidate_type, expected_type) {
                            return Ok(candidate_type.clone());
                        }
                    }
                }
                // If we can't narrow down, return the union as-is
                Ok(union_type.clone())
            }
            TypeInfo::Choice(types) => {
                // Similar logic for choice types
                if let Some(expected_type) = context.variables.get("this") {
                    for candidate_type in types {
                        if self.is_compatible(candidate_type, expected_type) {
                            return Ok(candidate_type.clone());
                        }
                    }
                }
                Ok(TypeInfo::Choice(types.clone()))
            }
            _ => Ok(union_type.clone()),
        }
    }

    /// Enhanced type compatibility checking with advanced features
    pub fn is_compatible_advanced(&self, from: &TypeInfo, to: &TypeInfo) -> bool {
        match (from, to) {
            // Handle constrained types
            (TypeInfo::Constrained { base_type, .. }, to_type) => {
                self.is_compatible_advanced(base_type, to_type)
            }
            (from_type, TypeInfo::Constrained { base_type, .. }) => {
                self.is_compatible_advanced(from_type, base_type)
            }

            // Handle choice types
            (from_type, TypeInfo::Choice(choice_types)) => choice_types
                .iter()
                .any(|choice_type| self.is_compatible_advanced(from_type, choice_type)),
            (TypeInfo::Choice(choice_types), to_type) => choice_types
                .iter()
                .any(|choice_type| self.is_compatible_advanced(choice_type, to_type)),

            // Handle reference types
            (
                TypeInfo::Reference {
                    target_types: from_targets,
                },
                TypeInfo::Reference {
                    target_types: to_targets,
                },
            ) => {
                // References are compatible if target types overlap or either accepts any
                from_targets.iter().any(|from_target| {
                    to_targets.contains(from_target)
                        || to_targets.contains(&"*".to_string())
                        || from_target == "*"
                })
            }

            // Fallback to basic compatibility
            _ => self.is_compatible(from, to),
        }
    }
}

/// Result of type analysis
#[derive(Debug)]
pub struct TypeAnalysisResult {
    /// Type information for each node in the AST
    pub type_info: HashMap<NodeId, TypeInfo>,
    /// Type-related warnings
    pub warnings: Vec<TypeWarning>,
    /// Inferred return type of the expression
    pub return_type: TypeInfo,
    /// Analysis context
    pub context: AnalysisContext,
}

/// Type-related warning
#[derive(Debug, Clone)]
pub struct TypeWarning {
    pub code: String,
    pub message: String,
    pub node_id: Option<NodeId>,
    pub suggestion: Option<String>,
}

/// Type constraint for function parameters
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub required_type: TypeInfo,
    pub optional: bool,
    pub description: String,
}

/// Function signature with type constraints
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<TypeConstraint>,
    pub return_type: TypeInfo,
    pub description: String,
}

/// Type mismatch error
#[derive(Debug, Clone)]
pub struct TypeMismatch {
    pub expected: String,
    pub actual: String,
    pub context: String,
}

impl fmt::Display for TypeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Type mismatch in {}: expected {}, got {}",
            self.context, self.expected, self.actual
        )
    }
}

impl FunctionSignature {
    /// Check if this signature accepts the given argument types
    pub fn accepts_args(&self, arg_types: &[TypeInfo]) -> Result<TypeInfo> {
        // Check minimum required parameters
        let required_count = self.parameters.iter().filter(|p| !p.optional).count();
        if arg_types.len() < required_count {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0157,
                format!(
                    "Function '{}' requires at least {} arguments, got {}",
                    self.name,
                    required_count,
                    arg_types.len()
                ),
            ));
        }

        // Check maximum parameters
        if arg_types.len() > self.parameters.len() {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0157,
                format!(
                    "Function '{}' accepts at most {} arguments, got {}",
                    self.name,
                    self.parameters.len(),
                    arg_types.len()
                ),
            ));
        }

        // Check each argument type
        for (i, (arg_type, param)) in arg_types.iter().zip(self.parameters.iter()).enumerate() {
            if !arg_type.is_compatible_with(&param.required_type) {
                return Err(crate::core::FhirPathError::TypeError {
                    error_code: crate::core::error_code::FP0156,
                    message: format!(
                        "Function '{}' argument {} type mismatch: expected {}, got {}",
                        self.name,
                        i + 1,
                        param.required_type,
                        arg_type
                    ),
                    expected_type: Some(param.required_type.to_string()),
                    actual_type: Some(arg_type.to_string()),
                    location: None,
                });
            }
        }

        Ok(self.return_type.clone())
    }
}

/// Visitor for type inference
#[allow(dead_code)]
struct TypeInferenceVisitor<'a> {
    type_checker: &'a TypeChecker,
    context: &'a mut AnalysisContext,
    type_info: &'a mut HashMap<NodeId, TypeInfo>,
    warnings: &'a mut Vec<TypeWarning>,
    node_counter: &'a std::cell::RefCell<NodeId>,
}

impl<'a> TypeInferenceVisitor<'a> {
    fn new(
        type_checker: &'a TypeChecker,
        context: &'a mut AnalysisContext,
        type_info: &'a mut HashMap<NodeId, TypeInfo>,
        warnings: &'a mut Vec<TypeWarning>,
        node_counter: &'a std::cell::RefCell<NodeId>,
    ) -> Self {
        Self {
            type_checker,
            context,
            type_info,
            warnings,
            node_counter,
        }
    }

    #[allow(dead_code)]
    fn next_node_id(&self) -> NodeId {
        let mut counter = self.node_counter.borrow_mut();
        *counter += 1;
        *counter
    }

    fn record_type(&mut self, node_id: NodeId, type_info: TypeInfo) {
        self.type_info.insert(node_id, type_info);
    }

    fn add_warning(&mut self, code: &str, message: String, node_id: Option<NodeId>) {
        self.warnings.push(TypeWarning {
            code: code.to_string(),
            message,
            node_id,
            suggestion: None,
        });
    }
}

impl<'a> ExpressionVisitor for TypeInferenceVisitor<'a> {
    type Output = Result<TypeInfo>;

    fn visit_literal(&mut self, literal: &LiteralNode) -> Self::Output {
        let node_id = self.next_node_id();
        let type_info = match &literal.value {
            LiteralValue::Boolean(_) => TypeInfo::Boolean,
            LiteralValue::Integer(_) => TypeInfo::Integer,
            LiteralValue::Decimal(_) => TypeInfo::Decimal,
            LiteralValue::String(_) => TypeInfo::String,
            LiteralValue::Date(_) => TypeInfo::Date,
            LiteralValue::DateTime(_) => TypeInfo::DateTime,
            LiteralValue::Time(_) => TypeInfo::Time,
            LiteralValue::Quantity { .. } => TypeInfo::Quantity,
        };

        self.record_type(node_id, type_info.clone());
        Ok(type_info)
    }

    fn visit_identifier(&mut self, identifier: &IdentifierNode) -> Self::Output {
        let node_id = self.next_node_id();

        // Check if it's a known resource type or resolve within context
        let type_info = if self.type_checker.is_known_resource_type(&identifier.name) {
            TypeInfo::Resource {
                resource_type: identifier.name.clone(),
            }
        } else if let Some(resource_type) = &self.context.resource_type {
            // Use ModelProvider-based property type inference where possible
            match identifier.name.as_str() {
                "id" => TypeInfo::String,
                "meta" => TypeInfo::BackboneElement {
                    properties: HashMap::new(),
                },
                "resourceType" => TypeInfo::String,
                "true" | "false" => TypeInfo::Boolean,
                _ => {
                    // Use sync ModelProvider property lookup if available
                    let base_type = TypeInfo::Resource {
                        resource_type: resource_type.clone(),
                    };
                    self.type_checker
                        .get_property_type(&base_type, &identifier.name)
                }
            }
        } else {
            match identifier.name.as_str() {
                "true" | "false" => TypeInfo::Boolean,
                _ => TypeInfo::Any,
            }
        };

        self.record_type(node_id, type_info.clone());
        Ok(type_info)
    }

    fn visit_function_call(&mut self, call: &FunctionCallNode) -> Self::Output {
        let node_id = self.next_node_id();

        // Infer types of arguments
        let mut arg_types = Vec::new();
        for arg in &call.arguments {
            arg_types.push(self.visit_expression(arg)?);
        }

        // Look up function signature and infer return type
        let return_type =
            if let Some(signature) = self.type_checker.function_signatures.get(&call.name) {
                signature.accepts_args(&arg_types).unwrap_or_else(|err| {
                    self.add_warning("W003", err.to_string(), Some(node_id));
                    TypeInfo::Any
                })
            } else {
                self.add_warning(
                    "W004",
                    format!("Unknown function '{}'", call.name),
                    Some(node_id),
                );
                self.infer_function_return_type(&call.name, &arg_types)
            };

        self.record_type(node_id, return_type.clone());
        Ok(return_type)
    }

    fn visit_method_call(&mut self, call: &MethodCallNode) -> Self::Output {
        let node_id = self.next_node_id();

        // Infer object type first
        let object_type = self.visit_expression(&call.object)?;

        // Infer argument types
        let mut arg_types = Vec::new();
        for arg in &call.arguments {
            arg_types.push(self.visit_expression(arg)?);
        }

        let return_type =
            if let Some(signature) = self.type_checker.function_signatures.get(&call.method) {
                // Method calls are similar to function calls but with implicit first argument
                let mut all_arg_types = vec![object_type.clone()];
                all_arg_types.extend(arg_types.clone());
                signature
                    .accepts_args(&all_arg_types)
                    .unwrap_or_else(|err| {
                        self.add_warning("W003", err.to_string(), Some(node_id));
                        TypeInfo::Any
                    })
            } else {
                // Handle built-in collection methods and other method calls
                self.infer_method_return_type(&call.method, &object_type, &arg_types)
            };

        self.record_type(node_id, return_type.clone());
        Ok(return_type)
    }

    fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Self::Output {
        let node_id = self.next_node_id();

        let object_type = self.visit_expression(&access.object)?;

        // Track path for nested property access
        self.context.push_path(access.property.clone());
        let property_type = self.infer_property_type(&object_type, &access.property);
        self.context.pop_path();

        self.record_type(node_id, property_type.clone());
        Ok(property_type)
    }

    fn visit_index_access(&mut self, access: &IndexAccessNode) -> Self::Output {
        let node_id = self.next_node_id();

        let object_type = self.visit_expression(&access.object)?;
        let index_type = self.visit_expression(&access.index)?;

        // Index should be integer
        if !matches!(index_type, TypeInfo::Integer) {
            self.add_warning(
                "W001",
                "Index should be an integer".to_string(),
                Some(node_id),
            );
        }

        // Return element type of collection
        let element_type = match object_type {
            TypeInfo::Collection(inner) => *inner,
            TypeInfo::String => TypeInfo::String, // String indexing returns string
            _ => {
                self.add_warning(
                    "W002",
                    "Index access on non-collection type".to_string(),
                    Some(node_id),
                );
                TypeInfo::Unknown
            }
        };

        self.record_type(node_id, element_type.clone());
        Ok(element_type)
    }

    fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Self::Output {
        let node_id = self.next_node_id();

        let left_type = self.visit_expression(&binary.left)?;
        let right_type = self.visit_expression(&binary.right)?;

        let result_type = TypeInfo::infer_binary_result(&binary.operator, &left_type, &right_type);

        self.record_type(node_id, result_type.clone());
        Ok(result_type)
    }

    fn visit_unary_operation(&mut self, unary: &UnaryOperationNode) -> Self::Output {
        let node_id = self.next_node_id();

        let operand_type = self.visit_expression(&unary.operand)?;
        let result_type = TypeInfo::infer_unary_result(&unary.operator, &operand_type);

        self.record_type(node_id, result_type.clone());
        Ok(result_type)
    }

    fn visit_lambda(&mut self, lambda: &LambdaNode) -> Self::Output {
        let node_id = self.next_node_id();

        // Push lambda scope
        self.context
            .push_scope(crate::analyzer::context::ScopeType::Lambda {
                parameter: lambda.parameter.clone(),
            });

        // Define lambda parameter if provided
        if let Some(param) = &lambda.parameter {
            self.context
                .define_variable(param.clone(), TypeInfo::Unknown);
        }

        let body_type = self.visit_expression(&lambda.body)?;

        // Pop lambda scope
        self.context.pop_scope();

        self.record_type(node_id, body_type.clone());
        Ok(body_type)
    }

    fn visit_collection(&mut self, collection: &CollectionNode) -> Self::Output {
        let node_id = self.next_node_id();

        if collection.elements.is_empty() {
            self.record_type(node_id, TypeInfo::Empty);
            return Ok(TypeInfo::Empty);
        }

        // Infer element types
        let mut element_types = Vec::new();
        for element in &collection.elements {
            element_types.push(self.visit_expression(element)?);
        }

        // Find common type
        let mut common_type = element_types[0].clone();
        for element_type in &element_types[1..] {
            // For now, use simple union if types differ
            if &common_type != element_type {
                common_type = TypeInfo::Union(element_types.clone());
                break;
            }
        }

        let collection_type = TypeInfo::Collection(Box::new(common_type));
        self.record_type(node_id, collection_type.clone());
        Ok(collection_type)
    }

    fn visit_parenthesized(&mut self, expr: &ExpressionNode) -> Self::Output {
        // Parentheses don't change the type
        self.visit_expression(expr)
    }

    fn visit_type_cast(&mut self, cast: &TypeCastNode) -> Self::Output {
        let node_id = self.next_node_id();

        let _source_type = self.visit_expression(&cast.expression)?;
        let target_type = self.parse_type_string(&cast.target_type);

        self.record_type(node_id, target_type.clone());
        Ok(target_type)
    }

    fn visit_filter(&mut self, filter: &FilterNode) -> Self::Output {
        let node_id = self.next_node_id();

        let base_type = self.visit_expression(&filter.base)?;

        // Push filter scope
        self.context
            .push_scope(crate::analyzer::context::ScopeType::Filter);

        let condition_type = self.visit_expression(&filter.condition)?;

        // Condition should be boolean
        if !matches!(condition_type, TypeInfo::Boolean) {
            self.add_warning(
                "W003",
                "Filter condition should return boolean".to_string(),
                Some(node_id),
            );
        }

        // Pop filter scope
        self.context.pop_scope();

        // Filter preserves the collection structure
        self.record_type(node_id, base_type.clone());
        Ok(base_type)
    }

    fn visit_union(&mut self, union: &UnionNode) -> Self::Output {
        let node_id = self.next_node_id();

        let left_type = self.visit_expression(&union.left)?;
        let right_type = self.visit_expression(&union.right)?;

        // Union creates a collection containing both types
        let result_type = match (&left_type, &right_type) {
            (TypeInfo::Collection(a), TypeInfo::Collection(b)) => {
                TypeInfo::Collection(Box::new(TypeInfo::Union(vec![
                    (**a).clone(),
                    (**b).clone(),
                ])))
            }
            (TypeInfo::Collection(a), b) => {
                TypeInfo::Collection(Box::new(TypeInfo::Union(vec![(**a).clone(), b.clone()])))
            }
            (a, TypeInfo::Collection(b)) => {
                TypeInfo::Collection(Box::new(TypeInfo::Union(vec![a.clone(), (**b).clone()])))
            }
            (a, b) => TypeInfo::Collection(Box::new(TypeInfo::Union(vec![a.clone(), b.clone()]))),
        };

        self.record_type(node_id, result_type.clone());
        Ok(result_type)
    }

    fn visit_type_check(&mut self, check: &TypeCheckNode) -> Self::Output {
        let node_id = self.next_node_id();

        let _expr_type = self.visit_expression(&check.expression)?;

        // Type check always returns boolean
        self.record_type(node_id, TypeInfo::Boolean);
        Ok(TypeInfo::Boolean)
    }

    fn visit_variable(&mut self, variable: &VariableNode) -> Self::Output {
        let node_id = self.next_node_id();

        let var_type = self
            .context
            .lookup_variable(&variable.name)
            .cloned()
            .unwrap_or(TypeInfo::Unknown);

        self.record_type(node_id, var_type.clone());
        Ok(var_type)
    }

    fn visit_path(&mut self, _path: &PathNode) -> Self::Output {
        let node_id = self.next_node_id();

        // For now, treat path as property access chain
        // This would need more sophisticated implementation
        let path_type = TypeInfo::Unknown;

        self.record_type(node_id, path_type.clone());
        Ok(path_type)
    }
}

impl<'a> TypeInferenceVisitor<'a> {
    fn infer_method_return_type(
        &mut self,
        method_name: &str,
        object_type: &TypeInfo,
        _arg_types: &[TypeInfo],
    ) -> TypeInfo {
        match method_name {
            // Collection methods
            "where" | "select" | "all" | "any" | "exists" => match object_type {
                TypeInfo::Collection(_) => TypeInfo::Collection(Box::new(TypeInfo::Any)),
                _ => {
                    self.add_warning(
                        "W005",
                        format!("Method '{}' can only be called on collections", method_name),
                        None,
                    );
                    TypeInfo::Any
                }
            },
            "first" | "last" | "single" => match object_type {
                TypeInfo::Collection(inner_type) => inner_type.as_ref().clone(),
                _ => object_type.clone(),
            },
            "count" => TypeInfo::Integer,
            "empty" => TypeInfo::Boolean,
            _ => {
                self.add_warning("W004", format!("Unknown method '{}'", method_name), None);
                TypeInfo::Any
            }
        }
    }

    fn infer_function_return_type(
        &mut self,
        function_name: &str,
        _arg_types: &[TypeInfo],
    ) -> TypeInfo {
        match function_name {
            // Boolean-returning functions
            "exists" | "empty" | "all" | "any" | "contains" | "startsWith" | "endsWith"
            | "matches" | "hasValue" => TypeInfo::Boolean,

            // String-returning functions
            "toString" | "substring" | "replace" | "trim" | "lower" | "upper" => TypeInfo::String,

            // Numeric functions
            "count" | "length" => TypeInfo::Integer,
            "sum" | "avg" | "min" | "max" => TypeInfo::Decimal,

            // Collection functions
            "first" | "last" | "single" => {
                // These would need to analyze the input collection type
                TypeInfo::Unknown
            }

            "where" | "select" => {
                // These preserve collection structure but may change element type
                TypeInfo::Unknown
            }

            _ => TypeInfo::Unknown,
        }
    }

    fn infer_property_type(&self, _object_type: &TypeInfo, _property: &str) -> TypeInfo {
        // This would require FHIR schema integration
        // For now, return Unknown
        TypeInfo::Unknown
    }

    fn infer_binary_operation_type(
        &self,
        operator: &BinaryOperator,
        left: &TypeInfo,
        right: &TypeInfo,
    ) -> TypeInfo {
        match operator {
            // Equality operators - can compare most types
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                // These operators return boolean and work on most type combinations
                TypeInfo::Boolean
            }

            // Equivalence operators - stricter than equality
            BinaryOperator::Equivalent | BinaryOperator::NotEquivalent => {
                // For now, treat same as equality, but could be stricter
                TypeInfo::Boolean
            }

            // Comparison operators - require ordered types
            BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual => {
                // Check if both operands are orderable types
                let orderable_types = [
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::String,
                    TypeInfo::Date,
                    TypeInfo::DateTime,
                    TypeInfo::Time,
                ];

                let left_orderable = orderable_types.iter().any(|t| left.is_compatible_with(t));
                let right_orderable = orderable_types.iter().any(|t| right.is_compatible_with(t));

                if left_orderable && right_orderable {
                    TypeInfo::Boolean
                } else {
                    // Add warning about incompatible comparison
                    TypeInfo::Boolean // Still return boolean but might be runtime error
                }
            }

            // Logical operators - require boolean operands
            BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
                // These should work on boolean types
                match (left, right) {
                    (TypeInfo::Boolean, TypeInfo::Boolean) => TypeInfo::Boolean,
                    _ => {
                        // In FHIRPath, non-boolean values are converted to boolean in logical context
                        // For now, we'll be permissive and return boolean
                        TypeInfo::Boolean
                    }
                }
            }

            // Arithmetic operators
            BinaryOperator::Add => {
                match (left, right) {
                    // Numeric addition
                    (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                    (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,

                    // Quantity arithmetic
                    (TypeInfo::Quantity, TypeInfo::Quantity) => TypeInfo::Quantity,

                    // String concatenation (if + is used for concatenation)
                    (TypeInfo::String, _) | (_, TypeInfo::String) => TypeInfo::String,

                    // Date/time arithmetic
                    (TypeInfo::DateTime, TypeInfo::Quantity)
                    | (TypeInfo::Quantity, TypeInfo::DateTime) => TypeInfo::DateTime,
                    (TypeInfo::Date, TypeInfo::Quantity) | (TypeInfo::Quantity, TypeInfo::Date) => {
                        TypeInfo::Date
                    }
                    (TypeInfo::Time, TypeInfo::Quantity) | (TypeInfo::Quantity, TypeInfo::Time) => {
                        TypeInfo::Time
                    }

                    _ => TypeInfo::Unknown,
                }
            }

            BinaryOperator::Subtract => {
                match (left, right) {
                    // Numeric subtraction
                    (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                    (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,

                    // Quantity arithmetic
                    (TypeInfo::Quantity, TypeInfo::Quantity) => TypeInfo::Quantity,

                    // Date/time arithmetic
                    (TypeInfo::DateTime, TypeInfo::Quantity) => TypeInfo::DateTime,
                    (TypeInfo::Date, TypeInfo::Quantity) => TypeInfo::Date,
                    (TypeInfo::Time, TypeInfo::Quantity) => TypeInfo::Time,
                    (TypeInfo::DateTime, TypeInfo::DateTime) => TypeInfo::Quantity, // Duration
                    (TypeInfo::Date, TypeInfo::Date) => TypeInfo::Quantity,
                    (TypeInfo::Time, TypeInfo::Time) => TypeInfo::Quantity,

                    _ => TypeInfo::Unknown,
                }
            }

            BinaryOperator::Multiply => {
                match (left, right) {
                    // Numeric multiplication
                    (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                    // Quantity multiplication (before general decimal match)
                    (TypeInfo::Quantity, TypeInfo::Integer)
                    | (TypeInfo::Integer, TypeInfo::Quantity) => TypeInfo::Quantity,
                    (TypeInfo::Quantity, TypeInfo::Decimal)
                    | (TypeInfo::Decimal, TypeInfo::Quantity) => TypeInfo::Quantity,
                    
                    (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,

                    _ => TypeInfo::Unknown,
                }
            }

            BinaryOperator::Divide => {
                match (left, right) {
                    // Numeric division - always results in decimal for precision
                    (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Decimal,
                    
                    // Quantity division (before general decimal match)
                    (TypeInfo::Quantity, TypeInfo::Integer)
                    | (TypeInfo::Quantity, TypeInfo::Decimal) => TypeInfo::Quantity,
                    
                    (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
                    (TypeInfo::Quantity, TypeInfo::Quantity) => TypeInfo::Decimal, // Ratio

                    _ => TypeInfo::Unknown,
                }
            }

            BinaryOperator::IntegerDivide => {
                match (left, right) {
                    // Integer division always returns integer
                    (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                    (TypeInfo::Decimal, TypeInfo::Decimal) => TypeInfo::Integer,
                    (TypeInfo::Integer, TypeInfo::Decimal)
                    | (TypeInfo::Decimal, TypeInfo::Integer) => TypeInfo::Integer,
                    _ => TypeInfo::Unknown,
                }
            }

            BinaryOperator::Modulo => {
                match (left, right) {
                    // Modulo preserves the type of the left operand
                    (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
                    (TypeInfo::Decimal, _) => TypeInfo::Decimal,
                    _ => TypeInfo::Unknown,
                }
            }

            // Collection operations
            BinaryOperator::Union => {
                // Union of two collections or values creates a collection
                let union_type = left.common_type(right);
                TypeInfo::Collection(Box::new(union_type))
            }

            // Any other operators
            _ => TypeInfo::Unknown,
        }
    }

    fn infer_unary_operation_type(&self, operator: &UnaryOperator, operand: &TypeInfo) -> TypeInfo {
        match operator {
            UnaryOperator::Not => {
                // Logical NOT always returns boolean
                // In FHIRPath, any value can be converted to boolean for NOT operation
                TypeInfo::Boolean
            }

            UnaryOperator::Negate => {
                // Numeric negation - preserves numeric type
                match operand {
                    TypeInfo::Integer => TypeInfo::Integer,
                    TypeInfo::Decimal => TypeInfo::Decimal,
                    TypeInfo::Quantity => TypeInfo::Quantity,
                    _ => {
                        // If non-numeric type, it's likely an error but return Unknown
                        // Could add warning here
                        TypeInfo::Unknown
                    }
                }
            }

            UnaryOperator::Positive => {
                // Unary plus - preserves numeric type (essentially a no-op)
                match operand {
                    TypeInfo::Integer => TypeInfo::Integer,
                    TypeInfo::Decimal => TypeInfo::Decimal,
                    TypeInfo::Quantity => TypeInfo::Quantity,
                    _ => {
                        // If non-numeric type, it's likely an error but return Unknown
                        // Could add warning here
                        TypeInfo::Unknown
                    }
                }
            }
        }
    }

    fn parse_type_string(&self, type_str: &str) -> TypeInfo {
        match type_str.to_lowercase().as_str() {
            "boolean" => TypeInfo::Boolean,
            "integer" => TypeInfo::Integer,
            "decimal" => TypeInfo::Decimal,
            "string" => TypeInfo::String,
            "date" => TypeInfo::Date,
            "datetime" => TypeInfo::DateTime,
            "time" => TypeInfo::Time,
            "quantity" => TypeInfo::Quantity,
            "code" => TypeInfo::Code,
            "coding" => TypeInfo::Coding,
            "codeableconcept" => TypeInfo::CodeableConcept,
            _ => TypeInfo::Unknown,
        }
    }
}

impl Default for TypeInfo {
    fn default() -> Self {
        Self::Unknown
    }
}
