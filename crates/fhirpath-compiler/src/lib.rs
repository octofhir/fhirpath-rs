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

//! Bytecode compiler and virtual machine for FHIRPath expressions
//!
//! This crate provides bytecode compilation and virtual machine execution
//! for high-performance FHIRPath expression evaluation.

pub mod bytecode;
pub mod bytecode_cache;
pub mod compiler;
pub mod optimizer;
pub mod vm;

// Re-export main types
pub use bytecode::{Bytecode, Instruction, OptimizationLevel};
pub use compiler::{CompilationError, CompilationResult, CompilerConfig, ExpressionCompiler};
pub use vm::{VirtualMachine, VmConfig};
// Note: Additional exports will be added as more implementations are completed
// pub use optimizer::{Optimizer}; // Not yet implemented
// pub use bytecode_cache::BytecodeCache; // Not yet implemented

// Re-export from workspace crates for convenience
pub use fhirpath_ast::{BinaryOperator, ExpressionNode, UnaryOperator};
pub use fhirpath_core::{FhirPathError, Result};
pub use fhirpath_registry::FunctionRegistry;
