//! FHIRPath Expression AST with type-safe node definitions
//!
//! This module provides a comprehensive Abstract Syntax Tree for FHIRPath expressions
//! with proper type safety, source location tracking, and rich error information.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::literal::LiteralValue;
use super::operator::{BinaryOperator, UnaryOperator};
use crate::core::{FP0001, FhirPathError, SourceLocation};

/// The main expression node representing any FHIRPath expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionNode {
    /// Literal value (string, number, boolean, etc.)
    Literal(LiteralNode),

    /// Identifier/property access (e.g., "name", "resourceType")
    Identifier(IdentifierNode),

    /// Function call (e.g., "first()", "where(condition)", "substring(1, 3)")
    FunctionCall(FunctionCallNode),

    /// Method call (e.g., "Patient.name.first()", "name.where(use = 'official')")
    MethodCall(MethodCallNode),

    /// Property navigation (e.g., "Patient.name", "name.family")
    PropertyAccess(PropertyAccessNode),

    /// Index access (e.g., "name[0]", "telecom[1]")
    IndexAccess(IndexAccessNode),

    /// Binary operation (e.g., "age > 18", "name = 'John'")
    BinaryOperation(BinaryOperationNode),

    /// Unary operation (e.g., "-5", "not active")
    UnaryOperation(UnaryOperationNode),

    /// Lambda expression (e.g., "$this.age > 18")
    Lambda(LambdaNode),

    /// Collection literal (e.g., "{1, 2, 3}")
    Collection(CollectionNode),

    /// Parenthesized expression
    Parenthesized(Box<ExpressionNode>),

    /// Type cast expression (e.g., "value as string")
    TypeCast(TypeCastNode),

    /// Filter expression (e.g., "name.where(use = 'official')")
    Filter(FilterNode),

    /// Union expression (e.g., "given | family")
    Union(UnionNode),

    /// Type check expression (e.g., "value is string")
    TypeCheck(TypeCheckNode),

    /// Variable reference (e.g., "$this", "$index")
    Variable(VariableNode),

    /// Path expression (advanced path navigation)
    Path(PathNode),

    /// Type information (e.g., "System.Integer", "FHIR.Patient")
    TypeInfo(TypeInfoNode),
}

/// Literal value with source location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteralNode {
    /// Literal value
    pub value: LiteralValue,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Identifier with validation and source tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifierNode {
    /// Identifier name
    pub name: String,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Function call with arguments and validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallNode {
    /// Function name
    pub name: String,
    /// Function arguments
    pub arguments: Vec<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Method call on an object (e.g., Patient.name.first())
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodCallNode {
    /// Method object
    pub object: Box<ExpressionNode>,
    /// Method name
    pub method: String,
    /// Method arguments
    pub arguments: Vec<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Property access for navigation (dot notation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyAccessNode {
    /// Property object
    pub object: Box<ExpressionNode>,
    /// Property name
    pub property: String,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Index access with bounds checking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexAccessNode {
    /// Index object
    pub object: Box<ExpressionNode>,
    /// Index expression
    pub index: Box<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Binary operation with operator precedence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryOperationNode {
    /// Left operand
    pub left: Box<ExpressionNode>,
    /// Binary operator
    pub operator: BinaryOperator,
    /// Right operand
    pub right: Box<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Unary operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryOperationNode {
    /// Unary operator
    pub operator: UnaryOperator,
    /// Operand expression
    pub operand: Box<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Lambda expression for filtering and mapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LambdaNode {
    /// Parameter name (e.g., "$this")
    pub parameter: Option<String>,
    /// Lambda body expression
    pub body: Box<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Collection literal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionNode {
    /// Collection elements
    pub elements: Vec<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Type cast expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeCastNode {
    /// Expression to cast
    pub expression: Box<ExpressionNode>,
    /// Target type name
    pub target_type: String,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Filter expression (where clause)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterNode {
    /// Base expression to filter
    pub base: Box<ExpressionNode>,
    /// Filter condition
    pub condition: Box<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Union expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnionNode {
    /// Left union operand
    pub left: Box<ExpressionNode>,
    /// Right union operand
    pub right: Box<ExpressionNode>,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Type check expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeCheckNode {
    /// Expression to check
    pub expression: Box<ExpressionNode>,
    /// Target type name
    pub target_type: String,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Variable reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableNode {
    /// Variable name
    pub name: String,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Path expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathNode {
    /// Base expression
    pub base: Box<ExpressionNode>,
    /// Path string
    pub path: String,
    /// Source location
    pub location: Option<SourceLocation>,
}

/// Type information node for type expressions like System.Integer or FHIR.Patient
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeInfoNode {
    /// Type namespace (e.g., "System", "FHIR")
    pub namespace: String,
    /// Type name (e.g., "Integer", "Patient")
    pub name: String,
    /// Source location
    pub location: Option<SourceLocation>,
}

impl ExpressionNode {
    /// Get the source location for this node, if available
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            Self::Literal(n) => n.location.as_ref(),
            Self::Identifier(n) => n.location.as_ref(),
            Self::FunctionCall(n) => n.location.as_ref(),
            Self::MethodCall(n) => n.location.as_ref(),
            Self::PropertyAccess(n) => n.location.as_ref(),
            Self::IndexAccess(n) => n.location.as_ref(),
            Self::BinaryOperation(n) => n.location.as_ref(),
            Self::UnaryOperation(n) => n.location.as_ref(),
            Self::Lambda(n) => n.location.as_ref(),
            Self::Collection(n) => n.location.as_ref(),
            Self::TypeCast(n) => n.location.as_ref(),
            Self::Filter(n) => n.location.as_ref(),
            Self::Union(n) => n.location.as_ref(),
            Self::TypeCheck(n) => n.location.as_ref(),
            Self::Variable(n) => n.location.as_ref(),
            Self::Path(n) => n.location.as_ref(),
            Self::TypeInfo(n) => n.location.as_ref(),
            Self::Parenthesized(_) => None,
        }
    }

    /// Set the source location for this node
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        match &mut self {
            Self::Literal(n) => n.location = Some(location),
            Self::Identifier(n) => n.location = Some(location),
            Self::FunctionCall(n) => n.location = Some(location),
            Self::MethodCall(n) => n.location = Some(location),
            Self::PropertyAccess(n) => n.location = Some(location),
            Self::IndexAccess(n) => n.location = Some(location),
            Self::BinaryOperation(n) => n.location = Some(location),
            Self::UnaryOperation(n) => n.location = Some(location),
            Self::Lambda(n) => n.location = Some(location),
            Self::Collection(n) => n.location = Some(location),
            Self::TypeCast(n) => n.location = Some(location),
            Self::Filter(n) => n.location = Some(location),
            Self::Union(n) => n.location = Some(location),
            Self::TypeCheck(n) => n.location = Some(location),
            Self::Variable(n) => n.location = Some(location),
            Self::Path(n) => n.location = Some(location),
            Self::TypeInfo(n) => n.location = Some(location),
            Self::Parenthesized(_) => {}
        }
        self
    }

    /// Get a human-readable description of the node type
    pub fn node_type(&self) -> &'static str {
        match self {
            Self::Literal(_) => "literal",
            Self::Identifier(_) => "identifier",
            Self::FunctionCall(_) => "function call",
            Self::MethodCall(_) => "method call",
            Self::PropertyAccess(_) => "property access",
            Self::IndexAccess(_) => "index access",
            Self::BinaryOperation(_) => "binary operation",
            Self::UnaryOperation(_) => "unary operation",
            Self::Lambda(_) => "lambda",
            Self::Collection(_) => "collection",
            Self::Parenthesized(_) => "parenthesized expression",
            Self::TypeCast(_) => "type cast",
            Self::Filter(_) => "filter",
            Self::Union(_) => "union",
            Self::TypeCheck(_) => "type check",
            Self::Variable(_) => "variable",
            Self::Path(_) => "path",
            Self::TypeInfo(_) => "type info",
        }
    }

    /// Validate the AST node for semantic correctness
    pub fn validate(&self) -> Result<(), FhirPathError> {
        match self {
            Self::FunctionCall(func) => {
                // Validate function name is not empty
                if func.name.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Function name cannot be empty",
                        "anonymous function call",
                        func.location.clone(),
                    ));
                }

                // Recursively validate arguments
                for arg in &func.arguments {
                    arg.validate()?;
                }
            }
            Self::MethodCall(method) => {
                // Validate method name is not empty
                if method.method.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Method name cannot be empty",
                        "anonymous method call",
                        method.location.clone(),
                    ));
                }

                // Validate object and arguments
                method.object.validate()?;
                for arg in &method.arguments {
                    arg.validate()?;
                }
            }
            Self::Identifier(ident) => {
                // Validate identifier is not empty
                if ident.name.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Identifier cannot be empty",
                        "empty identifier",
                        ident.location.clone(),
                    ));
                }
            }
            Self::PropertyAccess(prop) => {
                prop.object.validate()?;
                if prop.property.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Property name cannot be empty",
                        "empty property access",
                        prop.location.clone(),
                    ));
                }
            }
            Self::BinaryOperation(bin) => {
                bin.left.validate()?;
                bin.right.validate()?;
            }
            Self::UnaryOperation(un) => {
                un.operand.validate()?;
            }
            Self::Lambda(lambda) => {
                lambda.body.validate()?;
            }
            Self::Collection(coll) => {
                for element in &coll.elements {
                    element.validate()?;
                }
            }
            Self::IndexAccess(idx) => {
                idx.object.validate()?;
                idx.index.validate()?;
            }
            Self::TypeCast(cast) => {
                cast.expression.validate()?;
                if cast.target_type.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Type cast target cannot be empty",
                        "empty type cast",
                        cast.location.clone(),
                    ));
                }
            }
            Self::Filter(filter) => {
                filter.base.validate()?;
                filter.condition.validate()?;
            }
            Self::Union(union) => {
                union.left.validate()?;
                union.right.validate()?;
            }
            Self::TypeCheck(check) => {
                check.expression.validate()?;
                if check.target_type.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Type check target cannot be empty",
                        "empty type check",
                        check.location.clone(),
                    ));
                }
            }
            Self::Variable(var) => {
                if var.name.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Variable name cannot be empty",
                        "empty variable reference",
                        var.location.clone(),
                    ));
                }
            }
            Self::Path(path) => {
                path.base.validate()?;
                if path.path.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Path cannot be empty",
                        "empty path",
                        path.location.clone(),
                    ));
                }
            }
            Self::TypeInfo(type_info) => {
                if type_info.namespace.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Type namespace cannot be empty",
                        "empty type namespace",
                        type_info.location.clone(),
                    ));
                }
                if type_info.name.is_empty() {
                    return Err(FhirPathError::parse_error(
                        FP0001,
                        "Type name cannot be empty",
                        "empty type name",
                        type_info.location.clone(),
                    ));
                }
            }
            Self::Parenthesized(expr) => {
                expr.validate()?;
            }
            Self::Literal(_) => {
                // Literals are always valid
            }
        }
        Ok(())
    }

    /// Count the total number of nodes in this AST subtree
    pub fn node_count(&self) -> usize {
        1 + match self {
            Self::PropertyAccess(n) => n.object.node_count(),
            Self::IndexAccess(n) => n.object.node_count() + n.index.node_count(),
            Self::BinaryOperation(n) => n.left.node_count() + n.right.node_count(),
            Self::UnaryOperation(n) => n.operand.node_count(),
            Self::Lambda(n) => n.body.node_count(),
            Self::Collection(n) => n.elements.iter().map(|e| e.node_count()).sum(),
            Self::FunctionCall(n) => n.arguments.iter().map(|a| a.node_count()).sum(),
            Self::MethodCall(n) => {
                n.object.node_count() + n.arguments.iter().map(|a| a.node_count()).sum::<usize>()
            }
            Self::Parenthesized(expr) => expr.node_count(),
            Self::TypeCast(n) => n.expression.node_count(),
            Self::Filter(n) => n.base.node_count() + n.condition.node_count(),
            Self::Union(n) => n.left.node_count() + n.right.node_count(),
            Self::TypeCheck(n) => n.expression.node_count(),
            Self::Variable(_) => 0,
            Self::Path(n) => n.base.node_count(),
            Self::TypeInfo(_) => 0,
            _ => 0,
        }
    }
}

impl fmt::Display for ExpressionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(n) => write!(f, "{}", n.value),
            Self::Identifier(n) => write!(f, "{}", n.name),
            Self::FunctionCall(n) => {
                write!(f, "{}(", n.name)?;
                for (i, arg) in n.arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Self::MethodCall(n) => {
                write!(f, "{}.{}(", n.object, n.method)?;
                for (i, arg) in n.arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Self::PropertyAccess(n) => write!(f, "{}.{}", n.object, n.property),
            Self::IndexAccess(n) => write!(f, "{}[{}]", n.object, n.index),
            Self::BinaryOperation(n) => write!(f, "{} {} {}", n.left, n.operator, n.right),
            Self::UnaryOperation(n) => write!(f, "{}{}", n.operator, n.operand),
            Self::Lambda(n) => {
                if let Some(param) = &n.parameter {
                    write!(f, "{} -> {}", param, n.body)
                } else {
                    write!(f, "{}", n.body)
                }
            }
            Self::Collection(n) => {
                write!(f, "{{")?;
                for (i, element) in n.elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{element}")?;
                }
                write!(f, "}}")
            }
            Self::Filter(n) => write!(f, "{}.where({})", n.base, n.condition),
            Self::Union(n) => write!(f, "{} | {}", n.left, n.right),
            Self::TypeCheck(n) => write!(f, "{} is {}", n.expression, n.target_type),
            Self::Variable(n) => write!(f, "${}", n.name),
            Self::Path(n) => write!(f, "{}.{}", n.base, n.path),
            Self::TypeInfo(n) => write!(f, "{}.{}", n.namespace, n.name),
            Self::Parenthesized(expr) => write!(f, "({expr})"),
            Self::TypeCast(n) => write!(f, "{} as {}", n.expression, n.target_type),
        }
    }
}

// Convenience constructors
impl ExpressionNode {
    /// Create a literal node
    pub fn literal(value: LiteralValue) -> Self {
        Self::Literal(LiteralNode {
            value,
            location: None,
        })
    }

    /// Create an identifier node
    pub fn identifier(name: impl Into<String>) -> Self {
        Self::Identifier(IdentifierNode {
            name: name.into(),
            location: None,
        })
    }

    /// Create a function call node
    pub fn function_call(name: impl Into<String>, arguments: Vec<ExpressionNode>) -> Self {
        Self::FunctionCall(FunctionCallNode {
            name: name.into(),
            arguments,
            location: None,
        })
    }

    /// Create a method call node
    pub fn method_call(
        object: ExpressionNode,
        method: impl Into<String>,
        arguments: Vec<ExpressionNode>,
    ) -> Self {
        Self::MethodCall(MethodCallNode {
            object: Box::new(object),
            method: method.into(),
            arguments,
            location: None,
        })
    }

    /// Create a variable reference node
    pub fn variable(name: impl Into<String>) -> Self {
        Self::Variable(VariableNode {
            name: name.into(),
            location: None,
        })
    }

    /// Create a path navigation node
    pub fn path(base: ExpressionNode, path: impl Into<String>) -> Self {
        Self::Path(PathNode {
            base: Box::new(base),
            path: path.into(),
            location: None,
        })
    }

    /// Create a type info node for type expressions like System.Integer
    pub fn type_info(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self::TypeInfo(TypeInfoNode {
            namespace: namespace.into(),
            name: name.into(),
            location: None,
        })
    }

    /// Create a property access node
    pub fn property_access(object: ExpressionNode, property: impl Into<String>) -> Self {
        Self::PropertyAccess(PropertyAccessNode {
            object: Box::new(object),
            property: property.into(),
            location: None,
        })
    }

    /// Create a binary operation node
    pub fn binary_op(
        left: ExpressionNode,
        operator: BinaryOperator,
        right: ExpressionNode,
    ) -> Self {
        Self::BinaryOperation(BinaryOperationNode {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            location: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_construction() {
        let expr = ExpressionNode::identifier("Patient");
        assert_eq!(expr.node_type(), "identifier");
        assert_eq!(expr.to_string(), "Patient");
    }

    #[test]
    fn test_property_access() {
        let expr = ExpressionNode::property_access(ExpressionNode::identifier("Patient"), "name");
        assert_eq!(expr.to_string(), "Patient.name");
        assert_eq!(expr.node_count(), 2);
    }

    #[test]
    fn test_function_call() {
        let expr = ExpressionNode::function_call("first", vec![ExpressionNode::identifier("name")]);
        assert_eq!(expr.to_string(), "first(name)");
        assert_eq!(expr.node_count(), 2);
    }

    #[test]
    fn test_method_call() {
        let expr =
            ExpressionNode::method_call(ExpressionNode::identifier("Patient"), "first", vec![]);
        assert_eq!(expr.to_string(), "Patient.first()");
        assert_eq!(expr.node_count(), 2);
    }

    #[test]
    fn test_variable() {
        let expr = ExpressionNode::variable("this");
        assert_eq!(expr.to_string(), "$this");
        assert_eq!(expr.node_count(), 1);
    }

    #[test]
    fn test_validation() {
        let valid_expr = ExpressionNode::identifier("valid");
        assert!(valid_expr.validate().is_ok());

        let invalid_expr = ExpressionNode::identifier("");
        assert!(invalid_expr.validate().is_err());
    }
}
