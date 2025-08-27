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

//! Context management for FHIRPath expression evaluation
//!
//! This module provides comprehensive context management for FHIRPath expressions,
//! including variable scoping with Copy-on-Write semantics, lambda expression
//! support, and efficient resource sharing.
//!
//! # Architecture Overview
//!
//! The context system is built around several key components:
//!
//! - **EvaluationContext**: Main context structure holding input, variables, registry, and model provider
//! - **VariableScope**: COW-optimized variable scoping with parent chain traversal
//! - **LambdaMetadata**: Support for implicit lambda variables (`$this`, `$index`, `$total`)
//! - **Helpers**: Builder patterns and utilities for context creation
//!
//! # Memory Efficiency
//!
//! The context system is designed for optimal memory usage:
//! - **Arc-based sharing**: Large resources (root, registry, model provider) shared via Arc
//! - **COW variable scopes**: Variables inherited with zero-copy until modification
//! - **Shared type cache**: Type annotations cached across context hierarchies
//!
//! # Variable Resolution
//!
//! Variables are resolved in the following order:
//! 1. **Local variables**: Variables in current scope
//! 2. **Lambda metadata**: Implicit variables (`$this`, `$index`, `$total`)
//! 3. **Parent scopes**: Recursive search up scope chain
//! 4. **Environment variables**: System variables (`%context`, `%resource`, etc.)
//!
//! # Lambda Support
//!
//! The context system provides comprehensive support for lambda expressions:
//! - Automatic implicit variable creation
//! - Environment variable inheritance from parent scopes
//! - Parameter mapping for complex lambda expressions
//! - Index preservation for nested lambda contexts

// Core modules
pub mod evaluation_context;
pub mod helpers;
pub mod lambda_metadata;
pub mod variable_scope;

// Re-export main types for convenient access
pub use evaluation_context::EvaluationContext;
pub use helpers::LambdaContextBuilder;
pub use lambda_metadata::LambdaMetadata;
pub use variable_scope::VariableScope;
