// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Core arithmetic operations for the unified registry
//!
//! This module provides high-performance implementations of arithmetic operators
//! with both sync and async evaluation paths.

use crate::fhirpath_registry::FhirPathRegistry;
use crate::metadata::{
    MetadataBuilder, OperationType, Associativity, TypeConstraint, FhirPathType,
    OperationMetadata,
};
use crate::operation::FhirPathOperation;
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use std::sync::OnceLock;

/// Unified arithmetic operation implementations
pub struct ArithmeticOperations;

impl ArithmeticOperations {
    /// Register all arithmetic operations
    pub async fn register_all(registry: &mut FhirPathRegistry) -> Result<()> {
        registry.register(AdditionOperation).await?;
        registry.register(SubtractionOperation).await?;
        registry.register(MultiplicationOperation).await?;
        registry.register(DivisionOperation).await?;
        registry.register(ModuloOperation).await?;
        registry.register(IntegerDivisionOperation).await?;
        registry.register(UnaryMinusOperation).await?;
        registry.register(UnaryPlusOperation).await?;
        Ok(())
    }
}

/// Addition operation (+) - supports both binary and unary
pub struct AdditionOperation;

impl AdditionOperation {
    fn evaluate_binary_sync(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<Result<FhirPathValue>> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_add(*b)
                    .map(FhirPathValue::Integer)
                    .map(Ok)
                    .or_else(|| Some(Err(FhirPathError::ArithmeticError {
                        message: "Integer overflow in addition".to_string()
                    })))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Some(Ok(FhirPathValue::Decimal(a + b)))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if let Ok(a_decimal) = Decimal::try_from(*a) {
                    Some(Ok(FhirPathValue::Decimal(a_decimal + b)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if let Ok(b_decimal) = Decimal::try_from(*b) {
                    Some(Ok(FhirPathValue::Decimal(a + b_decimal)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            _ => None, // Fallback to async for complex cases
        }
    }

    async fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue, _context: &EvaluationContext) -> Result<FhirPathValue> {
        // Try sync path first
        if let Some(result) = self.evaluate_binary_sync(left, right) {
            return result;
        }

        // Handle string concatenation and other complex cases
        match (left, right) {
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::String(format!("{}{}", a, b).into()))
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot add {} and {}",
                    left.type_name(), right.type_name()
                )
            })
        }
    }

    async fn evaluate_unary(&self, value: &FhirPathValue, _context: &EvaluationContext) -> Result<FhirPathValue> {
        // Unary plus - return the value unchanged for numbers
        match value {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(value.clone()),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: format!("Unary plus not supported for {}", value.type_name())
            })
        }
    }
}

#[async_trait]
impl FhirPathOperation for AdditionOperation {
    fn identifier(&self) -> &str { 
        "+" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            precedence: 12, 
            associativity: Associativity::Left 
        }
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("+", OperationType::BinaryOperator { 
                precedence: 12, 
                associativity: Associativity::Left 
            })
            .description("Addition operator - adds two numbers or concatenates strings")
            .example("1 + 2")
            .example("'hello' + 'world'")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
                FhirPathType::String,
            ]))
            .parameter("left", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
                FhirPathType::String,
            ]))
            .parameter("right", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
                FhirPathType::String,
            ]))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match args.len() {
            2 => self.evaluate_binary(&args[0], &args[1], context).await,
            1 => self.evaluate_unary(&args[0], context).await,
            _ => Err(FhirPathError::InvalidArgumentCount { 
                function_name: "+".to_string(),
                expected: 2,
                actual: args.len()
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        match args.len() {
            2 => self.evaluate_binary_sync(&args[0], &args[1]),
            1 => {
                // Unary plus sync evaluation
                match &args[0] {
                    FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => {
                        Some(Ok(args[0].clone()))
                    }
                    FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
                    _ => None
                }
            }
            _ => Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "+".to_string(),
                expected: 2,
                actual: args.len()
            }))
        }
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() < 1 || args.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "+".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Subtraction operation (-)
pub struct SubtractionOperation;

impl SubtractionOperation {
    fn evaluate_binary_sync(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<Result<FhirPathValue>> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_sub(*b)
                    .map(FhirPathValue::Integer)
                    .map(Ok)
                    .or_else(|| Some(Err(FhirPathError::ArithmeticError {
                        message: "Integer overflow in subtraction".to_string()
                    })))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Some(Ok(FhirPathValue::Decimal(a - b)))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if let Ok(a_decimal) = Decimal::try_from(*a) {
                    Some(Ok(FhirPathValue::Decimal(a_decimal - b)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if let Ok(b_decimal) = Decimal::try_from(*b) {
                    Some(Ok(FhirPathValue::Decimal(a - b_decimal)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            _ => None,
        }
    }

    async fn evaluate_unary(&self, value: &FhirPathValue, _context: &EvaluationContext) -> Result<FhirPathValue> {
        // Unary minus - negate the value
        match value {
            FhirPathValue::Integer(n) => {
                n.checked_neg()
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::ArithmeticError {
                        message: "Integer overflow in negation".to_string()
                    })
            }
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: format!("Unary minus not supported for {}", value.type_name())
            })
        }
    }
}

#[async_trait]
impl FhirPathOperation for SubtractionOperation {
    fn identifier(&self) -> &str { 
        "-" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            precedence: 12, 
            associativity: Associativity::Left 
        }
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("-", OperationType::BinaryOperator { 
                precedence: 12, 
                associativity: Associativity::Left 
            })
            .description("Subtraction operator - subtracts right operand from left operand")
            .example("5 - 2")
            .example("3.5 - 1.2")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .parameter("left", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .parameter("right", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match args.len() {
            2 => {
                if let Some(result) = self.evaluate_binary_sync(&args[0], &args[1]) {
                    result
                } else {
                    match (&args[0], &args[1]) {
                        (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
                        (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
                        _ => Err(FhirPathError::TypeError {
                            message: format!(
                                "Cannot subtract {} from {}",
                                args[1].type_name(), args[0].type_name()
                            )
                        })
                    }
                }
            }
            1 => self.evaluate_unary(&args[0], context).await,
            _ => Err(FhirPathError::InvalidArgumentCount { 
                function_name: "-".to_string(),
                expected: 2,
                actual: args.len()
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        match args.len() {
            2 => self.evaluate_binary_sync(&args[0], &args[1]),
            1 => {
                // Unary minus sync evaluation
                match &args[0] {
                    FhirPathValue::Integer(n) => {
                        n.checked_neg()
                            .map(FhirPathValue::Integer)
                            .map(Ok)
                            .or_else(|| Some(Err(FhirPathError::ArithmeticError {
                                message: "Integer overflow in negation".to_string()
                            })))
                    }
                    FhirPathValue::Decimal(d) => Some(Ok(FhirPathValue::Decimal(-d))),
                    FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
                    _ => None
                }
            }
            _ => Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "-".to_string(),
                expected: 2,
                actual: args.len()
            }))
        }
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() < 1 || args.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "-".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Multiplication operation (*)
pub struct MultiplicationOperation;

impl MultiplicationOperation {
    fn evaluate_sync(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<Result<FhirPathValue>> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_mul(*b)
                    .map(FhirPathValue::Integer)
                    .map(Ok)
                    .or_else(|| Some(Err(FhirPathError::ArithmeticError {
                        message: "Integer overflow in multiplication".to_string()
                    })))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Some(Ok(FhirPathValue::Decimal(a * b)))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if let Ok(a_decimal) = Decimal::try_from(*a) {
                    Some(Ok(FhirPathValue::Decimal(a_decimal * b)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if let Ok(b_decimal) = Decimal::try_from(*b) {
                    Some(Ok(FhirPathValue::Decimal(a * b_decimal)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            _ => None,
        }
    }
}

#[async_trait]
impl FhirPathOperation for MultiplicationOperation {
    fn identifier(&self) -> &str { 
        "*" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            precedence: 13, 
            associativity: Associativity::Left 
        }
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("*", OperationType::BinaryOperator { 
                precedence: 13, 
                associativity: Associativity::Left 
            })
            .description("Multiplication operator")
            .example("3 * 4")
            .example("2.5 * 3")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .parameter("left", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .parameter("right", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len()
            });
        }

        if let Some(result) = self.evaluate_sync(&args[0], &args[1]) {
            result
        } else {
            match (&args[0], &args[1]) {
                (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
                (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
                _ => Err(FhirPathError::TypeError {
                    message: format!(
                        "Cannot multiply {} and {}",
                        args[0].type_name(), args[1].type_name()
                    )
                })
            }
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len()
            }));
        }
        self.evaluate_sync(&args[0], &args[1])
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Division operation (/)
pub struct DivisionOperation;

impl DivisionOperation {
    fn evaluate_sync(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<Result<FhirPathValue>> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Division by zero".to_string()
                    }))
                } else {
                    // Integer division returns decimal
                    if let (Ok(a_decimal), Ok(b_decimal)) = (Decimal::try_from(*a), Decimal::try_from(*b)) {
                        Some(Ok(FhirPathValue::Decimal(a_decimal / b_decimal)))
                    } else {
                        Some(Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integers to decimal for division".to_string()
                        }))
                    }
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Division by zero".to_string()
                    }))
                } else {
                    Some(Ok(FhirPathValue::Decimal(a / b)))
                }
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Division by zero".to_string()
                    }))
                } else if let Ok(a_decimal) = Decimal::try_from(*a) {
                    Some(Ok(FhirPathValue::Decimal(a_decimal / b)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Division by zero".to_string()
                    }))
                } else if let Ok(b_decimal) = Decimal::try_from(*b) {
                    Some(Ok(FhirPathValue::Decimal(a / b_decimal)))
                } else {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    }))
                }
            }
            _ => None,
        }
    }
}

#[async_trait]
impl FhirPathOperation for DivisionOperation {
    fn identifier(&self) -> &str { 
        "/" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            precedence: 13, 
            associativity: Associativity::Left 
        }
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("/", OperationType::BinaryOperator { 
                precedence: 13, 
                associativity: Associativity::Left 
            })
            .description("Division operator - always returns decimal result")
            .example("10 / 3")
            .example("7.5 / 2.5")
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .parameter("left", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .parameter("right", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: "/".to_string(),
                expected: 2,
                actual: args.len()
            });
        }

        if let Some(result) = self.evaluate_sync(&args[0], &args[1]) {
            result
        } else {
            match (&args[0], &args[1]) {
                (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
                (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
                _ => Err(FhirPathError::TypeError {
                    message: format!(
                        "Cannot divide {} by {}",
                        args[0].type_name(), args[1].type_name()
                    )
                })
            }
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "/".to_string(),
                expected: 2,
                actual: args.len()
            }));
        }
        self.evaluate_sync(&args[0], &args[1])
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "/".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Modulo operation (mod)
pub struct ModuloOperation;

#[async_trait]
impl FhirPathOperation for ModuloOperation {
    fn identifier(&self) -> &str { 
        "mod" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            precedence: 13, 
            associativity: Associativity::Left 
        }
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("mod", OperationType::BinaryOperator { 
                precedence: 13, 
                associativity: Associativity::Left 
            })
            .description("Modulo operator - returns remainder of division")
            .example("10 mod 3")
            .example("7 mod 2")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .parameter("left", TypeConstraint::Specific(FhirPathType::Integer))
            .parameter("right", TypeConstraint::Specific(FhirPathType::Integer))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: "mod".to_string(),
                expected: 2,
                actual: args.len()
            });
        }

        match (&args[0], &args[1]) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    Err(FhirPathError::ArithmeticError {
                        message: "Modulo by zero".to_string()
                    })
                } else {
                    Ok(FhirPathValue::Integer(a % b))
                }
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "Modulo operator requires integer operands".to_string()
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "mod".to_string(),
                expected: 2,
                actual: args.len()
            }));
        }

        match (&args[0], &args[1]) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Modulo by zero".to_string()
                    }))
                } else {
                    Some(Ok(FhirPathValue::Integer(a % b)))
                }
            }
            (FhirPathValue::Empty, _) => Some(Ok(FhirPathValue::Empty)),
            (_, FhirPathValue::Empty) => Some(Ok(FhirPathValue::Empty)),
            _ => None
        }
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "mod".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Integer division operation (div)
pub struct IntegerDivisionOperation;

#[async_trait]
impl FhirPathOperation for IntegerDivisionOperation {
    fn identifier(&self) -> &str { 
        "div" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            precedence: 13, 
            associativity: Associativity::Left 
        }
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("div", OperationType::BinaryOperator { 
                precedence: 13, 
                associativity: Associativity::Left 
            })
            .description("Integer division operator - returns integer result")
            .example("10 div 3")
            .example("7 div 2")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .parameter("left", TypeConstraint::Specific(FhirPathType::Integer))
            .parameter("right", TypeConstraint::Specific(FhirPathType::Integer))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: "div".to_string(),
                expected: 2,
                actual: args.len()
            });
        }

        match (&args[0], &args[1]) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    Err(FhirPathError::ArithmeticError {
                        message: "Division by zero".to_string()
                    })
                } else {
                    Ok(FhirPathValue::Integer(a / b))
                }
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "Integer division requires integer operands".to_string()
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "div".to_string(),
                expected: 2,
                actual: args.len()
            }));
        }

        match (&args[0], &args[1]) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    Some(Err(FhirPathError::ArithmeticError {
                        message: "Division by zero".to_string()
                    }))
                } else {
                    Some(Ok(FhirPathValue::Integer(a / b)))
                }
            }
            (FhirPathValue::Empty, _) => Some(Ok(FhirPathValue::Empty)),
            (_, FhirPathValue::Empty) => Some(Ok(FhirPathValue::Empty)),
            _ => None
        }
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "div".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Unary minus operation
pub struct UnaryMinusOperation;

#[async_trait]
impl FhirPathOperation for UnaryMinusOperation {
    fn identifier(&self) -> &str { 
        "-" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::UnaryOperator
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("-", OperationType::UnaryOperator)
            .description("Unary minus operator - negates numeric values")
            .example("-5")
            .example("-3.14")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .parameter("value", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: "-".to_string(),
                expected: 1,
                actual: args.len()
            });
        }

        match &args[0] {
            FhirPathValue::Integer(n) => {
                n.checked_neg()
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::ArithmeticError {
                        message: "Integer overflow in negation".to_string()
                    })
            }
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: format!("Unary minus not supported for {}", args[0].type_name())
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "-".to_string(),
                expected: 1,
                actual: args.len()
            }));
        }

        match &args[0] {
            FhirPathValue::Integer(n) => {
                n.checked_neg()
                    .map(FhirPathValue::Integer)
                    .map(Ok)
                    .or_else(|| Some(Err(FhirPathError::ArithmeticError {
                        message: "Integer overflow in negation".to_string()
                    })))
            }
            FhirPathValue::Decimal(d) => Some(Ok(FhirPathValue::Decimal(-d))),
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            _ => None
        }
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "-".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Unary plus operation
pub struct UnaryPlusOperation;

#[async_trait]
impl FhirPathOperation for UnaryPlusOperation {
    fn identifier(&self) -> &str { 
        "+" 
    }
    
    fn operation_type(&self) -> OperationType {
        OperationType::UnaryOperator
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            MetadataBuilder::new("+", OperationType::UnaryOperator)
            .description("Unary plus operator - returns numeric values unchanged")
            .example("+5")
            .example("+3.14")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .parameter("value", TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .performance(crate::enhanced_metadata::PerformanceComplexity::Constant, true)
            .build()
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: "+".to_string(),
                expected: 1,
                actual: args.len()
            });
        }

        match &args[0] {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(args[0].clone()),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: format!("Unary plus not supported for {}", args[0].type_name())
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: "+".to_string(),
                expected: 1,
                actual: args.len()
            }));
        }

        match &args[0] {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Some(Ok(args[0].clone())),
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            _ => None
        }
    }
    
    fn supports_sync(&self) -> bool { 
        true 
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "+".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_addition_integers() {
        let op = AdditionOperation;
        let args = vec![FhirPathValue::Integer(2), FhirPathValue::Integer(3)];
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        let result = op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(5));
    }

    #[tokio::test]
    async fn test_addition_sync_path() {
        let op = AdditionOperation;
        let args = vec![FhirPathValue::Integer(10), FhirPathValue::Integer(20)];
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        let result = op.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Integer(30));
    }

    #[tokio::test]
    async fn test_string_concatenation() {
        let op = AdditionOperation;
        let args = vec![
            FhirPathValue::String("hello".into()), 
            FhirPathValue::String(" world".into())
        ];
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        let result = op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));
    }

    #[tokio::test]
    async fn test_division_by_zero() {
        let op = DivisionOperation;
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(0)];
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        let result = op.evaluate(&args, &context).await;
        assert!(matches!(result, Err(FhirPathError::ArithmeticError { .. })));
    }

    #[tokio::test]
    async fn test_unary_minus() {
        let op = UnaryMinusOperation;
        let args = vec![FhirPathValue::Integer(5)];
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        let result = op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(-5));
    }

    #[tokio::test]
    async fn test_modulo_operation() {
        let op = ModuloOperation;
        let args = vec![FhirPathValue::Integer(10), FhirPathValue::Integer(3)];
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        let result = op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));
    }

    #[test]
    fn test_all_operations_support_sync() {
        assert!(AdditionOperation.supports_sync());
        assert!(SubtractionOperation.supports_sync());
        assert!(MultiplicationOperation.supports_sync());
        assert!(DivisionOperation.supports_sync());
        assert!(ModuloOperation.supports_sync());
        assert!(IntegerDivisionOperation.supports_sync());
        assert!(UnaryMinusOperation.supports_sync());
        assert!(UnaryPlusOperation.supports_sync());
    }
}