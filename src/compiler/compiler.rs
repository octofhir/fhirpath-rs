//! Expression compiler for FHIRPath expressions
//!
//! This module provides compilation from AST expressions to bytecode for optimized execution.
//! The compiler performs various optimizations including constant folding, dead code elimination,
//! and function inlining where appropriate.

use crate::ast::{
    BinaryOpData, BinaryOperator, ConditionalData, ExpressionNode, FunctionCallData, LambdaData,
    LiteralValue, MethodCallData, UnaryOperator,
};
use crate::compiler::bytecode::{
    Bytecode, BytecodeBuilder, BytecodeMetadata, Instruction, OptimizationLevel,
};
use crate::compiler::optimizer::{ExpressionOptimizer, OptimizationConfig};
use crate::model::{FhirPathValue, quantity::Quantity};
use crate::registry::FunctionRegistry;
use chrono::DateTime;
use rust_decimal::Decimal;
use std::sync::Arc;

/// Compilation error types
#[derive(Debug, Clone)]
pub enum CompilationError {
    /// Unknown function name
    UnknownFunction(String),
    /// Invalid function arity
    InvalidArity {
        function: String,
        expected: usize,
        got: usize,
    },
    /// Unsupported expression type
    UnsupportedExpression(String),
    /// Internal compiler error
    InternalError(String),
    /// Jump target out of range
    JumpTargetOutOfRange(i32),
    /// Maximum recursion depth exceeded
    MaxRecursionDepthExceeded,
}

impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFunction(name) => write!(f, "Unknown function: {name}"),
            Self::InvalidArity {
                function,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Function {function} expects {expected} arguments, got {got}"
                )
            }
            Self::UnsupportedExpression(desc) => write!(f, "Unsupported expression: {desc}"),
            Self::InternalError(msg) => write!(f, "Internal compiler error: {msg}"),
            Self::JumpTargetOutOfRange(offset) => write!(f, "Jump target out of range: {offset}"),
            Self::MaxRecursionDepthExceeded => write!(f, "Maximum recursion depth exceeded"),
        }
    }
}

impl std::error::Error for CompilationError {}

/// Result type for compilation operations
pub type CompilationResult<T> = Result<T, CompilationError>;

/// Configuration for the expression compiler
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Optimization level to apply during compilation
    pub optimization_level: OptimizationLevel,
    /// Maximum recursion depth to prevent stack overflow
    pub max_recursion_depth: usize,
    /// Whether to enable constant folding
    pub constant_folding: bool,
    /// Whether to inline simple function calls
    pub function_inlining: bool,
    /// Whether to eliminate dead code
    pub dead_code_elimination: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Basic,
            max_recursion_depth: 100,
            constant_folding: true,
            function_inlining: true,
            dead_code_elimination: true,
        }
    }
}

/// Expression compiler that converts AST to bytecode
pub struct ExpressionCompiler {
    /// Compiler configuration
    config: CompilerConfig,
    /// Function registry for looking up function signatures
    #[allow(dead_code)]
    functions: Arc<FunctionRegistry>,
    /// Expression optimizer for constant folding and other optimizations
    optimizer: ExpressionOptimizer,
    /// Current recursion depth
    recursion_depth: usize,
    /// Label counter for generating unique labels
    label_counter: usize,
}

impl ExpressionCompiler {
    /// Create a new expression compiler
    pub fn new(functions: Arc<FunctionRegistry>) -> Self {
        let config = CompilerConfig::default();
        let opt_config = OptimizationConfig {
            constant_folding: config.constant_folding,
            dead_code_elimination: config.dead_code_elimination,
            strength_reduction: true,
            max_depth: 32,
        };

        Self {
            config,
            functions,
            optimizer: ExpressionOptimizer::with_config(opt_config),
            recursion_depth: 0,
            label_counter: 0,
        }
    }

    /// Create a new expression compiler with custom configuration
    pub fn with_config(functions: Arc<FunctionRegistry>, config: CompilerConfig) -> Self {
        let opt_config = OptimizationConfig {
            constant_folding: config.constant_folding,
            dead_code_elimination: config.dead_code_elimination,
            strength_reduction: true,
            max_depth: 32,
        };

        Self {
            config,
            functions,
            optimizer: ExpressionOptimizer::with_config(opt_config),
            recursion_depth: 0,
            label_counter: 0,
        }
    }

    /// Compile an expression to bytecode
    pub fn compile(&mut self, expression: &ExpressionNode) -> CompilationResult<Bytecode> {
        // Reset compiler state
        self.recursion_depth = 0;
        self.label_counter = 0;

        // Apply optimizations if enabled
        let optimized_expression =
            if self.config.constant_folding || self.config.dead_code_elimination {
                self.optimizer.optimize(expression.clone())
            } else {
                expression.clone()
            };

        let mut builder = BytecodeBuilder::new();

        // Compile the (possibly optimized) expression
        self.compile_expression(&optimized_expression, &mut builder)?;

        // Add return instruction
        builder.emit(Instruction::Return);

        // Finalize bytecode
        let mut bytecode = builder
            .finalize()
            .map_err(CompilationError::InternalError)?;

        // Set metadata
        bytecode.metadata = BytecodeMetadata {
            source: None, // Could be set by caller
            optimization_level: self.config.optimization_level,
            uses_lambdas: self.expression_uses_lambdas(expression),
            modifies_variables: self.expression_modifies_variables(expression),
            complexity_score: expression.complexity() as u32,
        };

        // Apply optimizations if enabled
        if self.config.optimization_level != OptimizationLevel::None {
            self.optimize_bytecode(&mut bytecode)?;
        }

        Ok(bytecode)
    }

    /// Compile an expression with source information
    pub fn compile_with_source(
        &mut self,
        expression: &ExpressionNode,
        source: String,
    ) -> CompilationResult<Bytecode> {
        let mut bytecode = self.compile(expression)?;
        bytecode.metadata.source = Some(source);
        Ok(bytecode)
    }

    /// Recursively compile an expression node
    fn compile_expression(
        &mut self,
        expression: &ExpressionNode,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Check recursion depth
        if self.recursion_depth >= self.config.max_recursion_depth {
            return Err(CompilationError::MaxRecursionDepthExceeded);
        }
        self.recursion_depth += 1;

        let result = match expression {
            ExpressionNode::Literal(literal) => self.compile_literal(literal, builder),
            ExpressionNode::Identifier(name) => self.compile_identifier(name, builder),
            ExpressionNode::Variable(name) => self.compile_variable(name, builder),
            ExpressionNode::Path { base, path } => self.compile_path(base, path, builder),
            ExpressionNode::BinaryOp(data) => self.compile_binary_op(data, builder),
            ExpressionNode::UnaryOp { op, operand } => self.compile_unary_op(*op, operand, builder),
            ExpressionNode::FunctionCall(data) => self.compile_function_call(data, builder),
            ExpressionNode::MethodCall(data) => self.compile_method_call(data, builder),
            ExpressionNode::Index { base, index } => self.compile_index(base, index, builder),
            ExpressionNode::Filter { base, condition } => {
                self.compile_filter(base, condition, builder)
            }
            ExpressionNode::Union { left, right } => self.compile_union(left, right, builder),
            ExpressionNode::TypeCheck {
                expression,
                type_name,
            } => self.compile_type_check(expression, type_name, builder),
            ExpressionNode::TypeCast {
                expression,
                type_name,
            } => self.compile_type_cast(expression, type_name, builder),
            ExpressionNode::Lambda(data) => self.compile_lambda(data, builder),
            ExpressionNode::Conditional(data) => self.compile_conditional(data, builder),
        };

        self.recursion_depth -= 1;
        result
    }

    /// Compile a literal value
    fn compile_literal(
        &self,
        literal: &LiteralValue,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        let value = self.literal_to_value(literal);

        // Check if constant folding is enabled and this is a simple constant
        if self.config.constant_folding && self.is_simple_constant(&value) {
            let const_idx = builder.add_constant(value);
            builder.emit(Instruction::FastConstant(const_idx));
        } else {
            builder.push_constant(value);
        }

        Ok(())
    }

    /// Compile an identifier (property access)
    fn compile_identifier(
        &self,
        name: &str,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Use fast property access for simple identifiers
        let string_idx = builder.add_string(name.to_string());
        builder.emit(Instruction::FastProperty(string_idx));
        Ok(())
    }

    /// Compile a variable reference
    fn compile_variable(&self, name: &str, builder: &mut BytecodeBuilder) -> CompilationResult<()> {
        let string_idx = builder.add_string(name.to_string());
        builder.emit(Instruction::LoadVariable(string_idx));
        Ok(())
    }

    /// Compile a path expression (base.property)
    fn compile_path(
        &mut self,
        base: &ExpressionNode,
        path: &str,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile base expression first
        self.compile_expression(base, builder)?;

        // Then load the property
        self.compile_identifier(path, builder)?;

        Ok(())
    }

    /// Compile a binary operation
    fn compile_binary_op(
        &mut self,
        data: &BinaryOpData,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Try constant folding if both operands are literals
        if self.config.constant_folding {
            if let (Some(left_lit), Some(right_lit)) =
                (data.left.as_literal(), data.right.as_literal())
            {
                if let Some(result) = self.try_constant_fold_binary(data.op, left_lit, right_lit) {
                    builder.push_constant(result);
                    return Ok(());
                }
            }
        }

        // Compile operands
        self.compile_expression(&data.left, builder)?;
        self.compile_expression(&data.right, builder)?;

        // Emit operation instruction
        let instruction = match data.op {
            BinaryOperator::Equal => Instruction::Equal,
            BinaryOperator::NotEqual => Instruction::NotEqual,
            BinaryOperator::LessThan => Instruction::LessThan,
            BinaryOperator::LessThanOrEqual => Instruction::LessThanOrEqual,
            BinaryOperator::GreaterThan => Instruction::GreaterThan,
            BinaryOperator::GreaterThanOrEqual => Instruction::GreaterThanOrEqual,
            BinaryOperator::Add => Instruction::Add,
            BinaryOperator::Subtract => Instruction::Subtract,
            BinaryOperator::Multiply => Instruction::Multiply,
            BinaryOperator::Divide => Instruction::Divide,
            BinaryOperator::IntegerDivide => Instruction::Divide, // Use same instruction for now
            BinaryOperator::Modulo => Instruction::Modulo,
            BinaryOperator::And => Instruction::And,
            BinaryOperator::Or => Instruction::Or,
            BinaryOperator::Union => Instruction::Union,
            BinaryOperator::Equivalent => Instruction::Equal, // Similar semantics for now
            BinaryOperator::NotEquivalent => Instruction::NotEqual,
            BinaryOperator::Xor => {
                return Err(CompilationError::UnsupportedExpression(
                    "XOR operator not implemented in bytecode".to_string(),
                ));
            }
            BinaryOperator::Implies => {
                return Err(CompilationError::UnsupportedExpression(
                    "Implies operator not implemented in bytecode".to_string(),
                ));
            }
            BinaryOperator::In => {
                // 'in' operator is implemented as a special case
                return Err(CompilationError::UnsupportedExpression(
                    "'in' operator requires special handling".to_string(),
                ));
            }
            BinaryOperator::Contains => {
                // 'contains' operator needs function call
                return Err(CompilationError::UnsupportedExpression(
                    "'contains' operator requires function call".to_string(),
                ));
            }
            BinaryOperator::Concatenate => {
                return Err(CompilationError::UnsupportedExpression(
                    "Concatenate operator not implemented in bytecode".to_string(),
                ));
            }
            BinaryOperator::Is => {
                return Err(CompilationError::UnsupportedExpression(
                    "Is operator should use TypeCheck node".to_string(),
                ));
            }
        };

        builder.emit(instruction);
        Ok(())
    }

    /// Compile a unary operation
    fn compile_unary_op(
        &mut self,
        op: UnaryOperator,
        operand: &ExpressionNode,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Try constant folding if operand is a literal
        if self.config.constant_folding {
            if let Some(operand_lit) = operand.as_literal() {
                if let Some(result) = self.try_constant_fold_unary(op, operand_lit) {
                    builder.push_constant(result);
                    return Ok(());
                }
            }
        }

        // Compile operand
        self.compile_expression(operand, builder)?;

        // Emit operation instruction
        let instruction = match op {
            UnaryOperator::Minus => Instruction::Negate,
            UnaryOperator::Not => Instruction::Not,
            UnaryOperator::Plus => {
                // Unary plus is a no-op
                return Ok(());
            }
        };

        builder.emit(instruction);
        Ok(())
    }

    /// Compile a function call
    fn compile_function_call(
        &mut self,
        data: &FunctionCallData,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Check if this is a built-in function that can be inlined
        if self.config.function_inlining {
            if let Some(inlined) = self.try_inline_function(&data.name, &data.args) {
                return self.compile_expression(&inlined, builder);
            }
        }

        // Compile arguments in order
        for arg in &data.args {
            self.compile_expression(arg, builder)?;
        }

        // Look up function in registry (simplified - real implementation would be more complex)
        let function_idx = self.get_function_index(&data.name)?;

        // Emit function call
        builder.emit(Instruction::CallFunction(
            function_idx,
            data.args.len() as u8,
        ));

        Ok(())
    }

    /// Compile a method call
    fn compile_method_call(
        &mut self,
        data: &MethodCallData,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile base expression
        self.compile_expression(&data.base, builder)?;

        // Compile arguments
        for arg in &data.args {
            self.compile_expression(arg, builder)?;
        }

        // Emit method call
        let method_idx = builder.add_string(data.method.clone());
        builder.emit(Instruction::CallMethod(method_idx, data.args.len() as u8));

        Ok(())
    }

    /// Compile an index access expression
    fn compile_index(
        &mut self,
        base: &ExpressionNode,
        index: &ExpressionNode,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile base expression
        self.compile_expression(base, builder)?;

        // Compile index expression
        self.compile_expression(index, builder)?;

        // Emit index access
        builder.emit(Instruction::IndexAccess);

        Ok(())
    }

    /// Compile a filter expression
    fn compile_filter(
        &mut self,
        base: &ExpressionNode,
        condition: &ExpressionNode,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile base expression
        self.compile_expression(base, builder)?;

        // Emit filter instruction
        builder.emit(Instruction::Filter);

        // Compile condition (this would need special handling in a real VM)
        self.compile_expression(condition, builder)?;

        Ok(())
    }

    /// Compile a union expression
    fn compile_union(
        &mut self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile both expressions
        self.compile_expression(left, builder)?;
        self.compile_expression(right, builder)?;

        // Emit union instruction
        builder.emit(Instruction::Union);

        Ok(())
    }

    /// Compile a type check expression
    fn compile_type_check(
        &mut self,
        expression: &ExpressionNode,
        type_name: &str,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile expression
        self.compile_expression(expression, builder)?;

        // Emit type check
        let type_idx = builder.add_string(type_name.to_string());
        builder.emit(Instruction::IsType(type_idx));

        Ok(())
    }

    /// Compile a type cast expression
    fn compile_type_cast(
        &mut self,
        expression: &ExpressionNode,
        type_name: &str,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile expression
        self.compile_expression(expression, builder)?;

        // Emit type cast
        let type_idx = builder.add_string(type_name.to_string());
        builder.emit(Instruction::AsType(type_idx));

        Ok(())
    }

    /// Compile a lambda expression
    fn compile_lambda(
        &mut self,
        data: &LambdaData,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Begin lambda scope
        builder.emit(Instruction::BeginLambda);

        // Bind parameters
        for param in &data.params {
            let param_idx = builder.add_string(param.clone());
            builder.emit(Instruction::BindParameter(param_idx));
        }

        // Compile lambda body
        self.compile_expression(&data.body, builder)?;

        // End lambda scope
        builder.emit(Instruction::EndLambda);

        Ok(())
    }

    /// Compile a conditional expression
    fn compile_conditional(
        &mut self,
        data: &ConditionalData,
        builder: &mut BytecodeBuilder,
    ) -> CompilationResult<()> {
        // Compile condition
        self.compile_expression(&data.condition, builder)?;

        // Generate unique labels
        let else_label = self.generate_label("else");
        let end_label = self.generate_label("end");

        // Jump to else if condition is false
        builder.jump_if_false_to(else_label.clone());

        // Compile then expression
        self.compile_expression(&data.then_expr, builder)?;

        // Jump to end (skip else)
        builder.jump_to(end_label.clone());

        // Else label
        builder.label(else_label);

        // Compile else expression if present
        if let Some(else_expr) = &data.else_expr {
            self.compile_expression(else_expr, builder)?;
        } else {
            // Push empty if no else clause
            builder.push_constant(FhirPathValue::Empty);
        }

        // End label
        builder.label(end_label);

        Ok(())
    }

    /// Convert a literal AST value to a FhirPathValue
    fn literal_to_value(&self, literal: &LiteralValue) -> FhirPathValue {
        match literal {
            LiteralValue::Boolean(b) => FhirPathValue::Boolean(*b),
            LiteralValue::Integer(i) => FhirPathValue::Integer(*i),
            LiteralValue::Decimal(d) => FhirPathValue::Decimal(d.parse().unwrap_or_default()),
            LiteralValue::String(s) => FhirPathValue::interned_string(s),
            LiteralValue::Date(d) => {
                // Parse date string to NaiveDate
                match d.parse() {
                    Ok(date) => FhirPathValue::Date(date),
                    Err(_) => FhirPathValue::Empty, // Invalid date becomes empty
                }
            }
            LiteralValue::DateTime(dt) => {
                // Parse datetime string to DateTime<FixedOffset>
                match DateTime::parse_from_rfc3339(dt) {
                    Ok(datetime) => FhirPathValue::DateTime(datetime),
                    Err(_) => FhirPathValue::Empty, // Invalid datetime becomes empty
                }
            }
            LiteralValue::Time(t) => {
                // Parse time string to NaiveTime
                match t.parse() {
                    Ok(time) => FhirPathValue::Time(time),
                    Err(_) => FhirPathValue::Empty, // Invalid time becomes empty
                }
            }
            LiteralValue::Quantity { value, unit } => {
                // Create a Quantity object
                let decimal_value: Decimal = value.parse().unwrap_or_default();
                let quantity = Quantity::new(decimal_value, Some(unit.clone()));
                FhirPathValue::Quantity(quantity.into())
            }
            LiteralValue::Null => FhirPathValue::Empty,
        }
    }

    /// Check if a value is a simple constant suitable for fast constant instruction
    fn is_simple_constant(&self, value: &FhirPathValue) -> bool {
        matches!(
            value,
            FhirPathValue::Boolean(_) | FhirPathValue::Integer(_) | FhirPathValue::Empty
        )
    }

    /// Try to fold a binary operation at compile time
    fn try_constant_fold_binary(
        &self,
        op: BinaryOperator,
        left: &LiteralValue,
        right: &LiteralValue,
    ) -> Option<FhirPathValue> {
        match (left, right) {
            (LiteralValue::Integer(l), LiteralValue::Integer(r)) => match op {
                BinaryOperator::Add => Some(FhirPathValue::Integer(l + r)),
                BinaryOperator::Subtract => Some(FhirPathValue::Integer(l - r)),
                BinaryOperator::Multiply => Some(FhirPathValue::Integer(l * r)),
                BinaryOperator::Divide => {
                    if *r != 0 {
                        let result = *l as f64 / *r as f64;
                        Some(FhirPathValue::Decimal(
                            Decimal::try_from(result).unwrap_or_default(),
                        ))
                    } else {
                        None
                    }
                }
                BinaryOperator::Equal => Some(FhirPathValue::Boolean(l == r)),
                BinaryOperator::NotEqual => Some(FhirPathValue::Boolean(l != r)),
                BinaryOperator::LessThan => Some(FhirPathValue::Boolean(l < r)),
                BinaryOperator::LessThanOrEqual => Some(FhirPathValue::Boolean(l <= r)),
                BinaryOperator::GreaterThan => Some(FhirPathValue::Boolean(l > r)),
                BinaryOperator::GreaterThanOrEqual => Some(FhirPathValue::Boolean(l >= r)),
                _ => None,
            },
            (LiteralValue::Boolean(l), LiteralValue::Boolean(r)) => match op {
                BinaryOperator::And => Some(FhirPathValue::Boolean(*l && *r)),
                BinaryOperator::Or => Some(FhirPathValue::Boolean(*l || *r)),
                BinaryOperator::Equal => Some(FhirPathValue::Boolean(l == r)),
                BinaryOperator::NotEqual => Some(FhirPathValue::Boolean(l != r)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Try to fold a unary operation at compile time
    fn try_constant_fold_unary(
        &self,
        op: UnaryOperator,
        operand: &LiteralValue,
    ) -> Option<FhirPathValue> {
        match operand {
            LiteralValue::Integer(i) => match op {
                UnaryOperator::Minus => Some(FhirPathValue::Integer(-i)),
                UnaryOperator::Plus => Some(FhirPathValue::Integer(*i)),
                _ => None,
            },
            LiteralValue::Boolean(b) => match op {
                UnaryOperator::Not => Some(FhirPathValue::Boolean(!b)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Try to inline a simple function call
    fn try_inline_function(&self, name: &str, args: &[ExpressionNode]) -> Option<ExpressionNode> {
        // Only inline very simple functions for now
        match name {
            "empty" if args.is_empty() => {
                // empty() -> Collection(empty)
                Some(ExpressionNode::literal(LiteralValue::Null))
            }
            "true" if args.is_empty() => Some(ExpressionNode::literal(LiteralValue::Boolean(true))),
            "false" if args.is_empty() => {
                Some(ExpressionNode::literal(LiteralValue::Boolean(false)))
            }
            _ => None,
        }
    }

    /// Get function index from registry (simplified)
    fn get_function_index(&self, name: &str) -> CompilationResult<u16> {
        // This is a simplified implementation
        // Real implementation would look up function in registry
        let functions = [
            "count",
            "exists",
            "empty",
            "first",
            "last",
            "where",
            "select",
            "defineVariable",
        ];

        functions
            .iter()
            .position(|&f| f == name)
            .map(|pos| pos as u16)
            .ok_or_else(|| CompilationError::UnknownFunction(name.to_string()))
    }

    /// Generate a unique label
    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Check if expression uses lambdas
    fn expression_uses_lambdas(&self, expression: &ExpressionNode) -> bool {
        match expression {
            ExpressionNode::Lambda(_) => true,
            ExpressionNode::Path { base, .. } => self.expression_uses_lambdas(base),
            ExpressionNode::BinaryOp(data) => {
                self.expression_uses_lambdas(&data.left)
                    || self.expression_uses_lambdas(&data.right)
            }
            ExpressionNode::UnaryOp { operand, .. } => self.expression_uses_lambdas(operand),
            ExpressionNode::FunctionCall(data) => data
                .args
                .iter()
                .any(|arg| self.expression_uses_lambdas(arg)),
            ExpressionNode::MethodCall(data) => {
                self.expression_uses_lambdas(&data.base)
                    || data
                        .args
                        .iter()
                        .any(|arg| self.expression_uses_lambdas(arg))
            }
            _ => false,
        }
    }

    /// Check if expression modifies variables
    fn expression_modifies_variables(&self, expression: &ExpressionNode) -> bool {
        match expression {
            ExpressionNode::FunctionCall(data) => {
                // defineVariable modifies variables
                data.name == "defineVariable"
            }
            ExpressionNode::Path { base, .. } => self.expression_modifies_variables(base),
            ExpressionNode::BinaryOp(data) => {
                self.expression_modifies_variables(&data.left)
                    || self.expression_modifies_variables(&data.right)
            }
            ExpressionNode::UnaryOp { operand, .. } => self.expression_modifies_variables(operand),
            ExpressionNode::MethodCall(data) => {
                self.expression_modifies_variables(&data.base)
                    || data
                        .args
                        .iter()
                        .any(|arg| self.expression_modifies_variables(arg))
            }
            _ => false,
        }
    }

    /// Apply optimizations to compiled bytecode
    fn optimize_bytecode(&self, bytecode: &mut Bytecode) -> CompilationResult<()> {
        if self.config.dead_code_elimination {
            self.eliminate_dead_code(bytecode);
        }

        // Recalculate stack depth after optimizations
        bytecode.calculate_max_stack_depth();

        Ok(())
    }

    /// Eliminate dead code (simplified implementation)
    fn eliminate_dead_code(&self, bytecode: &mut Bytecode) {
        // Remove consecutive POP instructions
        let mut i = 0;
        while i < bytecode.instructions.len().saturating_sub(1) {
            if matches!(bytecode.instructions[i], Instruction::Pop)
                && matches!(bytecode.instructions[i + 1], Instruction::Pop)
            {
                bytecode.instructions.remove(i);
            } else {
                i += 1;
            }
        }

        // Remove NOPs
        bytecode
            .instructions
            .retain(|inst| !matches!(inst, Instruction::Nop));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, ExpressionNode, LiteralValue};
    use crate::registry::FunctionRegistry;

    fn create_test_compiler() -> ExpressionCompiler {
        let functions = Arc::new(FunctionRegistry::new());
        ExpressionCompiler::new(functions)
    }

    #[test]
    fn test_compile_literal() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::literal(LiteralValue::Integer(42));

        let bytecode = compiler.compile(&expr).unwrap();

        assert!(!bytecode.instructions.is_empty());
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], FhirPathValue::Integer(42));
    }

    #[test]
    fn test_compile_identifier() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::identifier("name");

        let bytecode = compiler.compile(&expr).unwrap();

        assert!(!bytecode.instructions.is_empty());
        assert_eq!(bytecode.strings.len(), 1);
        assert_eq!(bytecode.strings[0], "name");
    }

    #[test]
    fn test_compile_binary_op() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::binary_op(
            BinaryOperator::Add,
            ExpressionNode::literal(LiteralValue::Integer(1)),
            ExpressionNode::literal(LiteralValue::Integer(2)),
        );

        let bytecode = compiler.compile(&expr).unwrap();

        assert!(!bytecode.instructions.is_empty());
        // With constant folding enabled, 1 + 2 becomes a single constant: 3
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], FhirPathValue::Integer(3));
    }

    #[test]
    fn test_compile_path() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::path(ExpressionNode::identifier("Patient"), "name");

        let bytecode = compiler.compile(&expr).unwrap();

        assert!(!bytecode.instructions.is_empty());
        assert!(bytecode.strings.len() >= 2);
    }

    #[test]
    fn test_constant_folding() {
        let mut compiler = create_test_compiler();
        compiler.config.constant_folding = true;

        // This should be folded to a single constant
        let expr = ExpressionNode::binary_op(
            BinaryOperator::Add,
            ExpressionNode::literal(LiteralValue::Integer(1)),
            ExpressionNode::literal(LiteralValue::Integer(2)),
        );

        let bytecode = compiler.compile(&expr).unwrap();

        // Should have folded to a single constant: 3
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], FhirPathValue::Integer(3));
    }

    #[test]
    fn test_conditional_compilation() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::conditional(
            ExpressionNode::literal(LiteralValue::Boolean(true)),
            ExpressionNode::literal(LiteralValue::Integer(1)),
            Some(ExpressionNode::literal(LiteralValue::Integer(2))),
        );

        let bytecode = compiler.compile(&expr).unwrap();

        assert!(!bytecode.instructions.is_empty());
        assert!(bytecode.constants.len() >= 2);
    }

    #[test]
    fn test_function_inlining() {
        let mut compiler = create_test_compiler();
        compiler.config.function_inlining = true;

        let expr = ExpressionNode::function_call("true", vec![]);

        let bytecode = compiler.compile(&expr).unwrap();

        // Should be inlined to a boolean constant
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_recursion_depth_limit() {
        let mut compiler = create_test_compiler();
        compiler.config.max_recursion_depth = 2;

        // Create deeply nested expression
        let mut expr = ExpressionNode::identifier("x");
        for _ in 0..5 {
            expr = ExpressionNode::path(expr, "child");
        }

        let result = compiler.compile(&expr);
        assert!(matches!(
            result,
            Err(CompilationError::MaxRecursionDepthExceeded)
        ));
    }

    #[test]
    fn test_metadata_generation() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::function_call(
            "defineVariable",
            vec![
                ExpressionNode::literal(LiteralValue::String("x".to_string())),
                ExpressionNode::literal(LiteralValue::Integer(42)),
            ],
        );

        let bytecode = compiler.compile(&expr).unwrap();

        assert!(bytecode.metadata.modifies_variables);
        assert_eq!(
            bytecode.metadata.optimization_level,
            OptimizationLevel::Basic
        );
        assert!(bytecode.metadata.complexity_score > 0);
    }

    #[test]
    fn test_compile_with_source() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::identifier("name");
        let source = "Patient.name".to_string();

        let bytecode = compiler.compile_with_source(&expr, source.clone()).unwrap();

        assert_eq!(bytecode.metadata.source, Some(source));
    }

    #[test]
    fn test_bytecode_disassembly() {
        let mut compiler = create_test_compiler();
        let expr = ExpressionNode::binary_op(
            BinaryOperator::Equal,
            ExpressionNode::identifier("active"),
            ExpressionNode::literal(LiteralValue::Boolean(true)),
        );

        let bytecode = compiler.compile(&expr).unwrap();
        let disassembly = bytecode.disassemble();

        assert!(disassembly.contains("FAST_PROP"));
        // Could be PUSH_CONST or FAST_CONST depending on optimization
        assert!(disassembly.contains("CONST") || disassembly.contains("PUSH"));
        assert!(disassembly.contains("EQ"));
    }
}
