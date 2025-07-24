//! Operator registry for FHIRPath operators

use std::collections::HashMap;
use std::sync::Arc;
use crate::error::{FhirPathError, Result};
use crate::value_ext::FhirPathValue;
use rust_decimal::Decimal;
use super::TypeInfo;

/// Associativity of an operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

/// Trait for implementing FHIRPath operators
pub trait FhirPathOperator: Send + Sync {
    /// Get the operator symbol
    fn symbol(&self) -> &str;
    
    /// Get the operator precedence (higher = tighter binding)
    fn precedence(&self) -> u8;
    
    /// Get the associativity
    fn associativity(&self) -> Associativity;
    
    /// Evaluate the operator
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue>;
    
    /// Get the return type based on operand types
    fn return_type(&self, left_type: &TypeInfo, right_type: &TypeInfo) -> TypeInfo;
}

/// Registry for FHIRPath operators
#[derive(Clone)]
pub struct OperatorRegistry {
    pub(crate) binary_ops: HashMap<String, Arc<dyn FhirPathOperator>>,
    pub(crate) unary_ops: HashMap<String, Arc<dyn FhirPathOperator>>,
}

impl OperatorRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            binary_ops: HashMap::new(),
            unary_ops: HashMap::new(),
        }
    }
    
    /// Register a binary operator
    pub fn register_binary<O: FhirPathOperator + 'static>(&mut self, operator: O) {
        let symbol = operator.symbol().to_string();
        self.binary_ops.insert(symbol, Arc::new(operator));
    }
    
    /// Register a unary operator
    pub fn register_unary<O: FhirPathOperator + 'static>(&mut self, operator: O) {
        let symbol = operator.symbol().to_string();
        self.unary_ops.insert(symbol, Arc::new(operator));
    }
    
    /// Get a binary operator by symbol
    pub fn get_binary(&self, symbol: &str) -> Option<Arc<dyn FhirPathOperator>> {
        self.binary_ops.get(symbol).cloned()
    }
    
    /// Get a unary operator by symbol
    pub fn get_unary(&self, symbol: &str) -> Option<Arc<dyn FhirPathOperator>> {
        self.unary_ops.get(symbol).cloned()
    }
}

/// Register all built-in FHIRPath operators
pub fn register_builtin_operators(registry: &mut OperatorRegistry) {
    // Arithmetic operators
    registry.register_binary(AddOperator);
    registry.register_binary(SubtractOperator);
    registry.register_binary(MultiplyOperator);
    registry.register_binary(DivideOperator);
    registry.register_binary(IntegerDivideOperator);
    registry.register_binary(ModuloOperator);
    
    // Comparison operators
    registry.register_binary(EqualOperator);
    registry.register_binary(NotEqualOperator);
    registry.register_binary(LessThanOperator);
    registry.register_binary(LessThanOrEqualOperator);
    registry.register_binary(GreaterThanOperator);
    registry.register_binary(GreaterThanOrEqualOperator);
    
    // Equivalence operators
    registry.register_binary(EquivalentOperator);
    registry.register_binary(NotEquivalentOperator);
    
    // Logical operators
    registry.register_binary(AndOperator);
    registry.register_binary(OrOperator);
    registry.register_binary(XorOperator);
    registry.register_binary(ImpliesOperator);
    
    // String operators
    registry.register_binary(ConcatenateOperator);
    registry.register_binary(ContainsOperator);
    registry.register_binary(InOperator);
    
    // Collection operators
    registry.register_binary(UnionOperator);
    
    // Type operators
    registry.register_binary(IsOperator);
    
    // Unary operators
    registry.register_unary(NotOperator);
    registry.register_unary(UnaryMinusOperator);
    registry.register_unary(UnaryPlusOperator);
}

// Arithmetic operator implementations

struct AddOperator;
impl FhirPathOperator for AddOperator {
    fn symbol(&self) -> &str { "+" }
    fn precedence(&self) -> u8 { 6 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, left_type: &TypeInfo, right_type: &TypeInfo) -> TypeInfo {
        match (left_type, right_type) {
            (TypeInfo::String, TypeInfo::String) => TypeInfo::String,
            (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
            (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
            (TypeInfo::Quantity, _) | (_, TypeInfo::Quantity) => TypeInfo::Quantity,
            _ => TypeInfo::Any,
        }
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            // String concatenation
            (FhirPathValue::String(l), FhirPathValue::String(r)) => {
                Ok(FhirPathValue::String(format!("{}{}", l, r)))
            }
            
            // Integer addition
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                l.checked_add(*r)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::arithmetic_overflow("integer addition"))
            }
            
            // Decimal addition
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(FhirPathValue::Decimal(l + r))
            }
            
            // Mixed numeric addition
            (FhirPathValue::Integer(i), FhirPathValue::Decimal(d)) |
            (FhirPathValue::Decimal(d), FhirPathValue::Integer(i)) => {
                let i_decimal = Decimal::from(*i);
                Ok(FhirPathValue::Decimal(i_decimal + d))
            }
            
            // Quantity addition
            (FhirPathValue::Quantity { value: v1, unit: u1, .. }, 
             FhirPathValue::Quantity { value: v2, unit: u2, .. }) => {
                if u1 == u2 {
                    Ok(FhirPathValue::quantity(v1 + v2, u1.clone()))
                } else if u1.is_none() && u2.is_none() {
                    Ok(FhirPathValue::quantity(v1 + v2, None))
                } else {
                    Err(FhirPathError::incompatible_units(
                        u1.as_deref().unwrap_or(""),
                        u2.as_deref().unwrap_or("")
                    ))
                }
            }
            
            // Empty propagation
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Empty)
            }
            
            _ => Err(FhirPathError::invalid_operand_types("+", left.type_name(), right.type_name()))
        }
    }
}

struct SubtractOperator;
impl FhirPathOperator for SubtractOperator {
    fn symbol(&self) -> &str { "-" }
    fn precedence(&self) -> u8 { 6 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, left_type: &TypeInfo, right_type: &TypeInfo) -> TypeInfo {
        match (left_type, right_type) {
            (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
            (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
            (TypeInfo::Quantity, _) | (_, TypeInfo::Quantity) => TypeInfo::Quantity,
            _ => TypeInfo::Any,
        }
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            // Integer subtraction
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                l.checked_sub(*r)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::arithmetic_overflow("integer subtraction"))
            }
            
            // Decimal subtraction
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(FhirPathValue::Decimal(l - r))
            }
            
            // Mixed numeric subtraction
            (FhirPathValue::Integer(i), FhirPathValue::Decimal(d)) => {
                let i_decimal = Decimal::from(*i);
                Ok(FhirPathValue::Decimal(i_decimal - d))
            }
            (FhirPathValue::Decimal(d), FhirPathValue::Integer(i)) => {
                let i_decimal = Decimal::from(*i);
                Ok(FhirPathValue::Decimal(d - i_decimal))
            }
            
            // Quantity subtraction
            (FhirPathValue::Quantity { value: v1, unit: u1, .. }, 
             FhirPathValue::Quantity { value: v2, unit: u2, .. }) => {
                if u1 == u2 {
                    Ok(FhirPathValue::quantity(v1 - v2, u1.clone()))
                } else if u1.is_none() && u2.is_none() {
                    Ok(FhirPathValue::quantity(v1 - v2, None))
                } else {
                    Err(FhirPathError::incompatible_units(
                        u1.as_deref().unwrap_or(""),
                        u2.as_deref().unwrap_or("")
                    ))
                }
            }
            
            // Empty propagation
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Empty)
            }
            
            _ => Err(FhirPathError::invalid_operand_types("-", left.type_name(), right.type_name()))
        }
    }
}

struct MultiplyOperator;
impl FhirPathOperator for MultiplyOperator {
    fn symbol(&self) -> &str { "*" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, left_type: &TypeInfo, right_type: &TypeInfo) -> TypeInfo {
        match (left_type, right_type) {
            (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
            (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
            (TypeInfo::Quantity, TypeInfo::Decimal) | (TypeInfo::Decimal, TypeInfo::Quantity) => TypeInfo::Quantity,
            _ => TypeInfo::Any,
        }
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            // Integer multiplication
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                l.checked_mul(*r)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::arithmetic_overflow("integer multiplication"))
            }
            
            // Decimal multiplication
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(FhirPathValue::Decimal(l * r))
            }
            
            // Mixed numeric multiplication
            (FhirPathValue::Integer(i), FhirPathValue::Decimal(d)) |
            (FhirPathValue::Decimal(d), FhirPathValue::Integer(i)) => {
                let i_decimal = Decimal::from(*i);
                Ok(FhirPathValue::Decimal(i_decimal * d))
            }
            
            // Quantity multiplication by scalar
            (FhirPathValue::Quantity { value, unit, .. }, FhirPathValue::Integer(i)) |
            (FhirPathValue::Integer(i), FhirPathValue::Quantity { value, unit, .. }) => {
                let i_decimal = Decimal::from(*i);
                Ok(FhirPathValue::quantity(value * i_decimal, unit.clone()))
            }
            
            (FhirPathValue::Quantity { value, unit, .. }, FhirPathValue::Decimal(d)) |
            (FhirPathValue::Decimal(d), FhirPathValue::Quantity { value, unit, .. }) => {
                Ok(FhirPathValue::quantity(value * d, unit.clone()))
            }
            
            // Empty propagation
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Empty)
            }
            
            _ => Err(FhirPathError::invalid_operand_types("*", left.type_name(), right.type_name()))
        }
    }
}

struct DivideOperator;
impl FhirPathOperator for DivideOperator {
    fn symbol(&self) -> &str { "/" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Decimal
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        let (l_decimal, r_decimal) = match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                (Decimal::from(*l), Decimal::from(*r))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                (*l, *r)
            }
            (FhirPathValue::Integer(i), FhirPathValue::Decimal(d)) => {
                (Decimal::from(*i), *d)
            }
            (FhirPathValue::Decimal(d), FhirPathValue::Integer(i)) => {
                (*d, Decimal::from(*i))
            }
            (FhirPathValue::Quantity { value: v1, unit: u1, .. }, 
             FhirPathValue::Quantity { value: v2, unit: u2, .. }) => {
                if u1 == u2 {
                    // Division of same units results in unitless decimal
                    (*v1, *v2)
                } else {
                    return Err(FhirPathError::incompatible_units(
                        u1.as_deref().unwrap_or(""),
                        u2.as_deref().unwrap_or("")
                    ));
                }
            }
            (FhirPathValue::Quantity { value, .. }, FhirPathValue::Integer(i)) => {
                (*value, Decimal::from(*i))
            }
            (FhirPathValue::Quantity { value, .. }, FhirPathValue::Decimal(d)) => {
                (*value, *d)
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                return Ok(FhirPathValue::Empty);
            }
            _ => {
                return Err(FhirPathError::invalid_operand_types("/", left.type_name(), right.type_name()));
            }
        };
        
        if r_decimal.is_zero() {
            Err(FhirPathError::division_by_zero())
        } else {
            match (left, right) {
                (FhirPathValue::Quantity { unit, .. }, FhirPathValue::Integer(_)) |
                (FhirPathValue::Quantity { unit, .. }, FhirPathValue::Decimal(_)) => {
                    Ok(FhirPathValue::quantity(l_decimal / r_decimal, unit.clone()))
                }
                _ => Ok(FhirPathValue::Decimal(l_decimal / r_decimal))
            }
        }
    }
}

struct IntegerDivideOperator;
impl FhirPathOperator for IntegerDivideOperator {
    fn symbol(&self) -> &str { "div" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Integer
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                if *r == 0 {
                    Err(FhirPathError::division_by_zero())
                } else {
                    Ok(FhirPathValue::Integer(l / r))
                }
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Empty)
            }
            _ => Err(FhirPathError::invalid_operand_types("div", left.type_name(), right.type_name()))
        }
    }
}

struct ModuloOperator;
impl FhirPathOperator for ModuloOperator {
    fn symbol(&self) -> &str { "mod" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Integer
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                if *r == 0 {
                    Err(FhirPathError::division_by_zero())
                } else {
                    Ok(FhirPathValue::Integer(l % r))
                }
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Empty)
            }
            _ => Err(FhirPathError::invalid_operand_types("mod", left.type_name(), right.type_name()))
        }
    }
}

// Comparison operator implementations

struct EqualOperator;
impl FhirPathOperator for EqualOperator {
    fn symbol(&self) -> &str { "=" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath equality is strict - types must match
        let result = match (left, right) {
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l == r,
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l == r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => l == r,
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l == r,
            (FhirPathValue::Quantity { value: v1, unit: u1, .. }, 
             FhirPathValue::Quantity { value: v2, unit: u2, .. }) => {
                v1 == v2 && u1 == u2
            }
            (FhirPathValue::Empty, FhirPathValue::Empty) => true,
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| {
                    matches!(self.evaluate(a, b), Ok(FhirPathValue::Boolean(true)))
                })
            }
            _ => false,
        };
        
        Ok(FhirPathValue::Boolean(result))
    }
}

struct NotEqualOperator;
impl FhirPathOperator for NotEqualOperator {
    fn symbol(&self) -> &str { "!=" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match EqualOperator.evaluate(left, right)? {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            _ => unreachable!("Equal operator should always return boolean"),
        }
    }
}

struct LessThanOperator;
impl FhirPathOperator for LessThanOperator {
    fn symbol(&self) -> &str { "<" }
    fn precedence(&self) -> u8 { 5 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l < r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l < r,
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l < r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l < r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => l < r,
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l < r,
            
            // Mixed numeric comparisons
            (FhirPathValue::Integer(i), FhirPathValue::Decimal(d)) => {
                Decimal::from(*i) < *d
            }
            (FhirPathValue::Decimal(d), FhirPathValue::Integer(i)) => {
                *d < Decimal::from(*i)
            }
            
            // Quantity comparisons (only for same units)
            (FhirPathValue::Quantity { value: v1, unit: u1, .. }, 
             FhirPathValue::Quantity { value: v2, unit: u2, .. }) => {
                if u1 == u2 {
                    v1 < v2
                } else {
                    return Err(FhirPathError::incompatible_units(
                        u1.as_deref().unwrap_or(""),
                        u2.as_deref().unwrap_or("")
                    ));
                }
            }
            
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                return Ok(FhirPathValue::Empty);
            }
            
            _ => return Err(FhirPathError::invalid_operand_types("<", left.type_name(), right.type_name())),
        };
        
        Ok(FhirPathValue::Boolean(result))
    }
}

struct LessThanOrEqualOperator;
impl FhirPathOperator for LessThanOrEqualOperator {
    fn symbol(&self) -> &str { "<=" }
    fn precedence(&self) -> u8 { 5 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // a <= b is equivalent to !(a > b)
        match GreaterThanOperator.evaluate(left, right)? {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => unreachable!("GreaterThan operator should return boolean or empty"),
        }
    }
}

struct GreaterThanOperator;
impl FhirPathOperator for GreaterThanOperator {
    fn symbol(&self) -> &str { ">" }
    fn precedence(&self) -> u8 { 5 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // a > b is equivalent to b < a
        LessThanOperator.evaluate(right, left)
    }
}

struct GreaterThanOrEqualOperator;
impl FhirPathOperator for GreaterThanOrEqualOperator {
    fn symbol(&self) -> &str { ">=" }
    fn precedence(&self) -> u8 { 5 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // a >= b is equivalent to !(a < b)
        match LessThanOperator.evaluate(left, right)? {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => unreachable!("LessThan operator should return boolean or empty"),
        }
    }
}

// Equivalence operators

struct EquivalentOperator;
impl FhirPathOperator for EquivalentOperator {
    fn symbol(&self) -> &str { "~" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Equivalence is looser than equality - considers empty as equivalent to missing
        let result = match (left, right) {
            // Empty is equivalent to empty
            (FhirPathValue::Empty, FhirPathValue::Empty) => true,
            
            // Collections with only empty elements are equivalent to empty
            (FhirPathValue::Collection(items), FhirPathValue::Empty) |
            (FhirPathValue::Empty, FhirPathValue::Collection(items)) => {
                items.is_empty() || items.iter().all(|v| v.is_empty())
            }
            
            // String equivalence is case-insensitive and whitespace-normalized
            (FhirPathValue::String(l), FhirPathValue::String(r)) => {
                l.trim().eq_ignore_ascii_case(r.trim())
            }
            
            // Numeric equivalence allows type conversion
            (FhirPathValue::Integer(i), FhirPathValue::Decimal(d)) |
            (FhirPathValue::Decimal(d), FhirPathValue::Integer(i)) => {
                *d == Decimal::from(*i)
            }
            
            // Otherwise delegate to equality
            _ => match EqualOperator.evaluate(left, right)? {
                FhirPathValue::Boolean(b) => b,
                _ => false,
            }
        };
        
        Ok(FhirPathValue::Boolean(result))
    }
}

struct NotEquivalentOperator;
impl FhirPathOperator for NotEquivalentOperator {
    fn symbol(&self) -> &str { "!~" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match EquivalentOperator.evaluate(left, right)? {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            _ => unreachable!("Equivalent operator should always return boolean"),
        }
    }
}

// Logical operators

struct AndOperator;
impl FhirPathOperator for AndOperator {
    fn symbol(&self) -> &str { "and" }
    fn precedence(&self) -> u8 { 3 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Three-valued logic: true, false, empty
        match (left.to_boolean(), right.to_boolean()) {
            (Some(false), _) | (_, Some(false)) => Ok(FhirPathValue::Boolean(false)),
            (Some(true), Some(true)) => Ok(FhirPathValue::Boolean(true)),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

struct OrOperator;
impl FhirPathOperator for OrOperator {
    fn symbol(&self) -> &str { "or" }
    fn precedence(&self) -> u8 { 1 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Three-valued logic: true, false, empty
        match (left.to_boolean(), right.to_boolean()) {
            (Some(true), _) | (_, Some(true)) => Ok(FhirPathValue::Boolean(true)),
            (Some(false), Some(false)) => Ok(FhirPathValue::Boolean(false)),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

struct XorOperator;
impl FhirPathOperator for XorOperator {
    fn symbol(&self) -> &str { "xor" }
    fn precedence(&self) -> u8 { 2 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left.to_boolean(), right.to_boolean()) {
            (Some(l), Some(r)) => Ok(FhirPathValue::Boolean(l != r)),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

struct ImpliesOperator;
impl FhirPathOperator for ImpliesOperator {
    fn symbol(&self) -> &str { "implies" }
    fn precedence(&self) -> u8 { 0 }
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // a implies b is equivalent to (not a) or b
        match left.to_boolean() {
            Some(false) => Ok(FhirPathValue::Boolean(true)),
            Some(true) => match right.to_boolean() {
                Some(b) => Ok(FhirPathValue::Boolean(b)),
                None => Ok(FhirPathValue::Empty),
            },
            None => Ok(FhirPathValue::Empty),
        }
    }
}

// String operators

struct ConcatenateOperator;
impl FhirPathOperator for ConcatenateOperator {
    fn symbol(&self) -> &str { "&" }
    fn precedence(&self) -> u8 { 3 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::String
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        let l_str = left.to_string_value().unwrap_or_default();
        let r_str = right.to_string_value().unwrap_or_default();
        Ok(FhirPathValue::String(format!("{}{}", l_str, r_str)))
    }
}

struct ContainsOperator;
impl FhirPathOperator for ContainsOperator {
    fn symbol(&self) -> &str { "contains" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            // String contains
            (FhirPathValue::String(haystack), FhirPathValue::String(needle)) => {
                Ok(FhirPathValue::Boolean(haystack.contains(needle)))
            }
            
            // Collection contains
            (FhirPathValue::Collection(items), value) => {
                for item in items {
                    if let Ok(FhirPathValue::Boolean(true)) = EqualOperator.evaluate(item, value) {
                        return Ok(FhirPathValue::Boolean(true));
                    }
                }
                Ok(FhirPathValue::Boolean(false))
            }
            
            // Empty propagation
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Empty)
            }
            
            _ => Err(FhirPathError::invalid_operand_types("contains", left.type_name(), right.type_name()))
        }
    }
}

struct InOperator;
impl FhirPathOperator for InOperator {
    fn symbol(&self) -> &str { "in" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // "in" is the reverse of "contains"
        ContainsOperator.evaluate(right, left)
    }
}

// Collection operators

struct UnionOperator;
impl FhirPathOperator for UnionOperator {
    fn symbol(&self) -> &str { "|" }
    fn precedence(&self) -> u8 { 0 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Collection(Box::new(TypeInfo::Any))
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        let mut result = left.clone().to_collection();
        result.extend(right.clone().to_collection());
        Ok(FhirPathValue::collection(result))
    }
}

// Type operators

struct IsOperator;
impl FhirPathOperator for IsOperator {
    fn symbol(&self) -> &str { "is" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Right side should be a type identifier
        if let FhirPathValue::String(type_name) = right {
            let is_type = match (left, type_name.as_str()) {
                (FhirPathValue::Boolean(_), "Boolean") => true,
                (FhirPathValue::Integer(_), "Integer") => true,
                (FhirPathValue::Decimal(_), "Decimal") => true,
                (FhirPathValue::String(_), "String") => true,
                (FhirPathValue::Date(_), "Date") => true,
                (FhirPathValue::DateTime(_), "DateTime") => true,
                (FhirPathValue::Time(_), "Time") => true,
                (FhirPathValue::Quantity { .. }, "Quantity") => true,
                (FhirPathValue::Resource(res), type_name) => {
                    res.resource_type() == Some(type_name)
                }
                _ => false,
            };
            Ok(FhirPathValue::Boolean(is_type))
        } else {
            Err(FhirPathError::invalid_type_specifier())
        }
    }
}

// Unary operators

struct NotOperator;
impl FhirPathOperator for NotOperator {
    fn symbol(&self) -> &str { "not" }
    fn precedence(&self) -> u8 { 10 }
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn return_type(&self, _left_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, operand: &FhirPathValue, _right: &FhirPathValue) -> Result<FhirPathValue> {
        match operand.to_boolean() {
            Some(b) => Ok(FhirPathValue::Boolean(!b)),
            None => Ok(FhirPathValue::Empty),
        }
    }
}

struct UnaryMinusOperator;
impl FhirPathOperator for UnaryMinusOperator {
    fn symbol(&self) -> &str { "-" }
    fn precedence(&self) -> u8 { 10 }
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn return_type(&self, operand_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        operand_type.clone()
    }
    
    fn evaluate(&self, operand: &FhirPathValue, _right: &FhirPathValue) -> Result<FhirPathValue> {
        match operand {
            FhirPathValue::Integer(i) => {
                i.checked_neg()
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::arithmetic_overflow("unary negation"))
            }
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
            FhirPathValue::Quantity { value, unit, .. } => {
                Ok(FhirPathValue::quantity(-value, unit.clone()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::invalid_operand_types("-", operand.type_name(), ""))
        }
    }
}

struct UnaryPlusOperator;
impl FhirPathOperator for UnaryPlusOperator {
    fn symbol(&self) -> &str { "+" }
    fn precedence(&self) -> u8 { 10 }
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn return_type(&self, operand_type: &TypeInfo, _right_type: &TypeInfo) -> TypeInfo {
        operand_type.clone()
    }
    
    fn evaluate(&self, operand: &FhirPathValue, _right: &FhirPathValue) -> Result<FhirPathValue> {
        match operand {
            FhirPathValue::Integer(_) | 
            FhirPathValue::Decimal(_) | 
            FhirPathValue::Quantity { .. } => Ok(operand.clone()),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::invalid_operand_types("+", operand.type_name(), ""))
        }
    }
}