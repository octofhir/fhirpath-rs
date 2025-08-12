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

//! Core VM types and errors
//!
//! This module provides shared types for virtual machine operations
//! that can be used across different crates without circular dependencies.

use crate::evaluation::EvaluationError;

/// Virtual machine execution error types
#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    /// Stack underflow (not enough values on stack)
    StackUnderflow,
    /// Stack overflow (too many values on stack)
    StackOverflow,
    /// Invalid instruction pointer
    InvalidInstructionPointer(usize),
    /// Invalid constant index
    InvalidConstantIndex(u16),
    /// Invalid string index
    InvalidStringIndex(u16),
    /// Invalid function index
    InvalidFunctionIndex(u16),
    /// Function evaluation error
    FunctionError(String),
    /// Type conversion error
    TypeConversionError(String),
    /// Runtime error during execution
    RuntimeError(String),
    /// Jump target out of bounds
    JumpOutOfBounds(i16),
    /// Maximum execution steps exceeded
    ExecutionLimitExceeded,
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "Stack underflow"),
            Self::StackOverflow => write!(f, "Stack overflow"),
            Self::InvalidInstructionPointer(ip) => write!(f, "Invalid instruction pointer: {ip}"),
            Self::InvalidConstantIndex(idx) => write!(f, "Invalid constant index: {idx}"),
            Self::InvalidStringIndex(idx) => write!(f, "Invalid string index: {idx}"),
            Self::InvalidFunctionIndex(idx) => write!(f, "Invalid function index: {idx}"),
            Self::FunctionError(msg) => write!(f, "Function error: {msg}"),
            Self::TypeConversionError(msg) => write!(f, "Type conversion error: {msg}"),
            Self::RuntimeError(msg) => write!(f, "Runtime error: {msg}"),
            Self::JumpOutOfBounds(offset) => write!(f, "Jump target out of bounds: {offset}"),
            Self::ExecutionLimitExceeded => write!(f, "Execution limit exceeded"),
        }
    }
}

impl std::error::Error for VmError {}

impl From<VmError> for EvaluationError {
    fn from(err: VmError) -> Self {
        EvaluationError::RuntimeError {
            message: err.to_string(),
        }
    }
}

/// Result type for VM operations
pub type VmResult<T> = Result<T, VmError>;

/// Configuration for the virtual machine
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Maximum stack size to prevent stack overflow
    pub max_stack_size: usize,
    /// Maximum number of execution steps to prevent infinite loops
    pub max_execution_steps: usize,
    /// Enable debug mode for tracing execution
    pub debug_mode: bool,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_stack_size: 1024,
            max_execution_steps: 1_000_000,
            debug_mode: false,
        }
    }
}
