// Optimized FHIRPath Data Model
//
// This module defines the optimized data model for FHIRPath values with:
// - Zero-copy string handling with lifetimes
// - Arena allocation support
// - String interning
// - Optimized collections

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::sync::Arc;

/// String interner for common property names and function names
#[derive(Debug, Default)]
pub struct StringInterner {
    strings: FxHashMap<String, Arc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        let mut interner = Self::default();

        // Pre-intern common FHIR property names
        let common_properties = [
            "resourceType", "id", "meta", "implicitRules", "language",
            "text", "contained", "extension", "modifierExtension",
            "identifier", "active", "name", "telecom", "gender",
            "birthDate", "address", "maritalStatus", "multipleBirth",
            "photo", "contact", "communication", "generalPractitioner",
            "managingOrganization", "link", "status", "category",
            "code", "subject", "encounter", "effective", "issued",
            "performer", "value", "dataAbsentReason", "interpretation",
            "note", "bodySite", "method", "specimen", "device",
            "referenceRange", "hasMember", "derivedFrom", "component",
        ];

        for prop in &common_properties {
            interner.intern(prop);
        }

        // Pre-intern common function names
        let common_functions = [
            "where", "select", "first", "last", "tail", "skip", "take",
            "exists", "empty", "count", "length", "distinct", "isDistinct",
            "sort", "descendants", "children", "repeat", "union", "combine",
            "intersect", "exclude", "subsetOf", "supersetOf", "is", "as",
            "contains", "startsWith", "endsWith", "substring", "indexOf",
            "replace", "matches", "split", "join", "abs", "ceiling",
            "floor", "round", "sqrt", "exp", "ln", "log", "power",
            "truncate", "type", "extension", "ofType", "conformsTo",
            "now", "today", "timeOfDay", "not", "all", "allTrue",
            "anyTrue", "allFalse", "anyFalse", "convertsToInteger",
            "convertsToString", "convertsToBoolean", "convertsToDecimal",
            "convertsToDate", "convertsToDateTime", "convertsToQuantity",
            "convertsToTime", "iif", "single", "trace", "aggregate",
            "toChars", "escape", "unescape", "toString", "toInteger",
            "toDecimal", "toQuantity", "toBoolean", "upper", "lower",
            "trim", "encode", "decode",
        ];

        for func in &common_functions {
            interner.intern(func);
        }

        interner
    }

    pub fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            interned.clone()
        } else {
            let interned: Arc<str> = s.into();
            self.strings.insert(s.to_string(), interned.clone());
            interned
        }
    }

    pub fn get(&self, s: &str) -> Option<Arc<str>> {
        self.strings.get(s).cloned()
    }
}

/// Optimized FHIRPath value types with lifetime parameters for zero-copy operations
#[derive(Debug, Clone, PartialEq)]
pub enum FhirPathValue<'a> {
    /// Empty value (no value)
    Empty,

    /// Boolean value
    Boolean(bool),

    /// Integer value
    Integer(i64),

    /// Decimal value
    Decimal(f64),

    /// String value with zero-copy support
    String(Cow<'a, str>),

    /// Date value (ISO8601) with zero-copy support
    Date(Cow<'a, str>),

    /// DateTime value (ISO8601) with zero-copy support
    DateTime(Cow<'a, str>),

    /// Time value (ISO8601) with zero-copy support
    Time(Cow<'a, str>),

    /// Quantity value with unit (zero-copy unit)
    Quantity {
        value: f64,
        unit: Cow<'a, str>
    },

    /// Collection of values optimized for small collections (most FHIR collections are small)
    Collection(SmallVec<[Box<FhirPathValue<'a>>; 4]>),

    /// FHIR resource or element
    Resource(FhirResource<'a>),
}

impl<'a> FhirPathValue<'a> {
    /// Convert to owned version (useful for caching)
    pub fn into_owned(self) -> FhirPathValue<'static> {
        match self {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Boolean(b) => FhirPathValue::Boolean(b),
            FhirPathValue::Integer(i) => FhirPathValue::Integer(i),
            FhirPathValue::Decimal(d) => FhirPathValue::Decimal(d),
            FhirPathValue::String(s) => FhirPathValue::String(Cow::Owned(s.into_owned())),
            FhirPathValue::Date(s) => FhirPathValue::Date(Cow::Owned(s.into_owned())),
            FhirPathValue::DateTime(s) => FhirPathValue::DateTime(Cow::Owned(s.into_owned())),
            FhirPathValue::Time(s) => FhirPathValue::Time(Cow::Owned(s.into_owned())),
            FhirPathValue::Quantity { value, unit } => FhirPathValue::Quantity {
                value,
                unit: Cow::Owned(unit.into_owned()),
            },
            FhirPathValue::Collection(items) => {
                FhirPathValue::Collection(
                    items.into_iter().map(|item| Box::new((*item).into_owned())).collect()
                )
            }
            FhirPathValue::Resource(resource) => FhirPathValue::Resource(resource.into_owned()),
        }
    }

    /// Check if the value is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, FhirPathValue::Empty)
    }

    /// Check if the value is a collection
    pub fn is_collection(&self) -> bool {
        matches!(self, FhirPathValue::Collection(_))
    }

    /// Get the type name for this value
    pub fn type_name(&self) -> &'static str {
        match self {
            FhirPathValue::Empty => "Empty",
            FhirPathValue::Boolean(_) => "Boolean",
            FhirPathValue::Integer(_) => "Integer",
            FhirPathValue::Decimal(_) => "Decimal",
            FhirPathValue::String(_) => "String",
            FhirPathValue::Date(_) => "Date",
            FhirPathValue::DateTime(_) => "DateTime",
            FhirPathValue::Time(_) => "Time",
            FhirPathValue::Quantity { .. } => "Quantity",
            FhirPathValue::Collection(_) => "Collection",
            FhirPathValue::Resource(_) => "Resource",
        }
    }
}

/// Optimized representation of a FHIR resource or element
#[derive(Debug, Clone, PartialEq)]
pub struct FhirResource<'a> {
    /// Resource type (e.g., "Patient", "Observation") with zero-copy support
    pub resource_type: Option<Cow<'a, str>>,

    /// Resource properties using faster hash map and zero-copy keys/values
    pub properties: FxHashMap<Cow<'a, str>, FhirPathValue<'a>>,
}

impl<'a> FhirResource<'a> {
    /// Creates a new empty FHIR resource
    pub fn new() -> Self {
        Self {
            resource_type: None,
            properties: FxHashMap::default(),
        }
    }

    /// Creates a new FHIR resource with a specific type
    pub fn with_type(resource_type: impl Into<Cow<'a, str>>) -> Self {
        Self {
            resource_type: Some(resource_type.into()),
            properties: FxHashMap::default(),
        }
    }

    /// Convert to owned version
    pub fn into_owned(self) -> FhirResource<'static> {
        FhirResource {
            resource_type: self.resource_type.map(|rt| Cow::Owned(rt.into_owned())),
            properties: self.properties
                .into_iter()
                .map(|(k, v)| (Cow::Owned(k.into_owned()), v.into_owned()))
                .collect(),
        }
    }

    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&FhirPathValue<'a>> {
        self.properties.get(name)
    }

    /// Set a property value
    pub fn set_property(&mut self, name: impl Into<Cow<'a, str>>, value: FhirPathValue<'a>) {
        self.properties.insert(name.into(), value);
    }

    /// Check if a property exists
    pub fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    /// Get all property names
    pub fn property_names(&self) -> impl Iterator<Item = &str> + use<'_, 'a> {
        self.properties.keys().map(|k| k.as_ref())
    }
}

impl<'a> Default for FhirResource<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized AST node with arena allocation support and compilation hints
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizedAstNode<'a> {
    /// Unique node ID for caching
    pub id: u32,

    /// Node kind
    pub kind: AstNodeKind<'a>,

    /// Compilation hints for optimization
    pub hints: CompilationHints,

    /// Source location for debugging
    pub source_span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeKind<'a> {
    /// Identifier with interned string
    Identifier(Cow<'a, str>),

    /// String literal with zero-copy support
    StringLiteral(Cow<'a, str>),

    /// Number literal
    NumberLiteral { value: f64, is_decimal: bool },

    /// Boolean literal
    BooleanLiteral(bool),

    /// DateTime literal with zero-copy support
    DateTimeLiteral(Cow<'a, str>),

    /// Quantity literal with zero-copy unit
    QuantityLiteral {
        value: f64,
        unit: Option<Cow<'a, str>>
    },

    /// Variable reference with interned name
    Variable(Cow<'a, str>),

    /// Path expression (optimized with node IDs)
    Path {
        base: u32,  // Node ID instead of Box
        path: u32,  // Node ID instead of Box
    },

    /// Function call with interned name
    FunctionCall {
        name: Cow<'a, str>,
        arguments: SmallVec<[u32; 2]>  // Node IDs, most functions have 0-2 args
    },

    /// Binary operation
    BinaryOp {
        op: BinaryOperator,
        left: u32,   // Node ID
        right: u32,  // Node ID
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOperator,
        operand: u32  // Node ID
    },

    /// Indexer operation
    Indexer {
        collection: u32,  // Node ID
        index: u32,       // Node ID
    },
}

/// Binary operators (same as before but with additional metadata)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    // Comparison operators
    Equals,
    NotEquals,
    Equivalent,
    NotEquivalent,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,

    // Arithmetic operators
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Div,
    Mod,

    // Logical operators
    And,
    Or,
    Xor,
    Implies,

    // Collection operators
    In,
    Contains,
    Union,

    // Type operators
    Is,
    As,

    // String operators
    Concatenation,
}

impl BinaryOperator {
    /// Check if this operator is commutative (for optimization)
    pub fn is_commutative(self) -> bool {
        matches!(self,
            BinaryOperator::Equals |
            BinaryOperator::NotEquals |
            BinaryOperator::Equivalent |
            BinaryOperator::NotEquivalent |
            BinaryOperator::Addition |
            BinaryOperator::Multiplication |
            BinaryOperator::And |
            BinaryOperator::Or |
            BinaryOperator::Xor |
            BinaryOperator::Union
        )
    }

    /// Check if this operator short-circuits
    pub fn short_circuits(self) -> bool {
        matches!(self, BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Implies)
    }

    /// Get operator precedence for parsing
    pub fn precedence(self) -> u8 {
        match self {
            BinaryOperator::Implies => 1,
            BinaryOperator::Or | BinaryOperator::Xor => 2,
            BinaryOperator::And => 3,
            BinaryOperator::In | BinaryOperator::Contains => 4,
            BinaryOperator::Equals | BinaryOperator::NotEquals |
            BinaryOperator::Equivalent | BinaryOperator::NotEquivalent => 5,
            BinaryOperator::LessThan | BinaryOperator::LessOrEqual |
            BinaryOperator::GreaterThan | BinaryOperator::GreaterOrEqual => 6,
            BinaryOperator::Union => 7,
            BinaryOperator::Is | BinaryOperator::As => 8,
            BinaryOperator::Addition | BinaryOperator::Subtraction |
            BinaryOperator::Concatenation => 9,
            BinaryOperator::Multiplication | BinaryOperator::Division |
            BinaryOperator::Div | BinaryOperator::Mod => 10,
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Positive,
    Negate,
    Not,
}

/// Compilation hints for optimization
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompilationHints {
    /// Whether this node is pure (no side effects)
    pub is_pure: bool,

    /// Whether this node can be constant-folded
    pub is_constant: bool,

    /// Whether this node is expensive to evaluate
    pub is_expensive: bool,

    /// Whether this node should be cached
    pub should_cache: bool,

    /// Expected result type (for optimization)
    pub expected_type: Option<&'static str>,

    /// Whether this node can be parallelized
    pub can_parallelize: bool,
}

/// Source location for debugging
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

/// Arena-based AST storage for efficient memory management
#[derive(Debug)]
pub struct AstArena<'a> {
    nodes: Vec<OptimizedAstNode<'a>>,
    next_id: u32,
}

impl<'a> AstArena<'a> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
        }
    }

    /// Allocate a new node in the arena
    pub fn alloc(&mut self, kind: AstNodeKind<'a>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let node = OptimizedAstNode {
            id,
            kind,
            hints: CompilationHints::default(),
            source_span: None,
        };

        self.nodes.push(node);
        id
    }

    /// Get a node by ID
    pub fn get(&self, id: u32) -> Option<&OptimizedAstNode<'a>> {
        self.nodes.get(id as usize)
    }

    /// Get a mutable node by ID
    pub fn get_mut(&mut self, id: u32) -> Option<&mut OptimizedAstNode<'a>> {
        self.nodes.get_mut(id as usize)
    }

    /// Get the root node (assumed to be the last allocated)
    pub fn root(&self) -> Option<&OptimizedAstNode<'a>> {
        self.nodes.last()
    }

    /// Clear the arena
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.next_id = 0;
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl<'a> Default for AstArena<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let s1 = interner.intern("test");
        let s2 = interner.intern("test");

        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(s1.as_ref(), "test");
    }

    #[test]
    fn test_fhir_path_value_type_name() {
        assert_eq!(FhirPathValue::Empty.type_name(), "Empty");
        assert_eq!(FhirPathValue::Boolean(true).type_name(), "Boolean");
        assert_eq!(FhirPathValue::Integer(42).type_name(), "Integer");
        assert_eq!(FhirPathValue::String(Cow::Borrowed("test")).type_name(), "String");
    }

    #[test]
    fn test_fhir_resource() {
        let mut resource = FhirResource::with_type("Patient");
        resource.set_property("id", FhirPathValue::String(Cow::Borrowed("123")));

        assert_eq!(resource.resource_type, Some(Cow::Borrowed("Patient")));
        assert!(resource.has_property("id"));
        assert_eq!(
            resource.get_property("id"),
            Some(&FhirPathValue::String(Cow::Borrowed("123")))
        );
    }

    #[test]
    fn test_ast_arena() {
        let mut arena = AstArena::new();

        let id1 = arena.alloc(AstNodeKind::BooleanLiteral(true));
        let id2 = arena.alloc(AstNodeKind::NumberLiteral { value: 42.0, is_decimal: false });

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(arena.len(), 2);

        let node1 = arena.get(id1).unwrap();
        assert_eq!(node1.kind, AstNodeKind::BooleanLiteral(true));
    }

    #[test]
    fn test_binary_operator_properties() {
        assert!(BinaryOperator::Addition.is_commutative());
        assert!(!BinaryOperator::Subtraction.is_commutative());
        assert!(BinaryOperator::And.short_circuits());
        assert!(!BinaryOperator::Addition.short_circuits());
        assert!(BinaryOperator::Multiplication.precedence() > BinaryOperator::Addition.precedence());
    }
}
