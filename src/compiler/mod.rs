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

//! Compiler module for FHIRPath expressions
//!
//! This module provides compilation from AST to bytecode for optimized execution.

pub mod bytecode;
/// Bytecode caching for compiled FHIRPath expressions
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
