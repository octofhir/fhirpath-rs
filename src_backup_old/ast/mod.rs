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

//! Abstract Syntax Tree (AST) definitions for FHIRPath expressions
//!
//! This crate provides the core AST types used to represent parsed FHIRPath expressions.
//! It is designed to be lightweight with minimal dependencies.

#![warn(missing_docs)]

mod expression;
mod intern;
mod operator;
mod visitor;

pub use expression::*;
pub use intern::*;
pub use operator::*;
pub use visitor::*;
