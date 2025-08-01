//! Compiler module for FHIRPath expressions
//!
//! This module provides compilation from AST to bytecode for optimized execution.

pub mod bytecode;
pub mod compiler;
pub mod optimizer;
pub mod vm;

pub use bytecode::*;
pub use compiler::*;
pub use optimizer::*;
pub use vm::*;
