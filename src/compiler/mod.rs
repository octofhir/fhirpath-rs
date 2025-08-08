//! Compiler module for FHIRPath expressions
//!
//! This module provides compilation from AST to bytecode for optimized execution.

pub mod bytecode;
pub mod bytecode_cache;
pub mod compiler;
pub mod optimizer;
pub mod vm;

pub use bytecode::*;
pub use compiler::*;
pub use optimizer::*;
pub use vm::*;

// Explicit re-exports from bytecode_cache
pub use bytecode_cache::{
    CacheConfig, CacheMetadata, CacheStats, CompilationMetadata, CompressedBytecode,
    CompressionError, GlobalBytecodeCache, SharedBytecode, global_cache,
};
